use super::{Generator, ir_type};
use crate::ast::Span;
use crate::hir::{Expr, Type};

impl<'a> Generator<'a> {
    pub(crate) fn checked(
        &mut self,
        op: &str,
        ty: &Type,
        left: &str,
        right: Option<&str>,
        span: Span,
    ) -> Result<String, String> {
        let Type::Int { bits, signed } = ty else {
            unreachable!()
        };
        let prefix = if *signed { "s" } else { "u" };
        let operation = match op {
            "+" => "add",
            "-" => "sub",
            "*" => "mul",
            _ => unreachable!(),
        };
        let name = format!("llvm.{prefix}{operation}.with.overflow.i{bits}");
        self.declarations.insert(format!(
            "declare {{ i{bits}, i1 }} @{name}(i{bits}, i{bits})"
        ));
        let (first, second) = right.map(|right| (left, right)).unwrap_or(("0", left));
        let pair = self.value(format!(
            "call {{ i{bits}, i1 }} @{name}(i{bits} {first}, i{bits} {second})"
        ));
        let overflow = self.value(format!("extractvalue {{ i{bits}, i1 }} {pair}, 1"));
        self.arithmetic_guard(&overflow, op, ty, (left, right), span)?;
        Ok(self.value(format!("extractvalue {{ i{bits}, i1 }} {pair}, 0")))
    }

    pub(crate) fn arithmetic_guard(
        &mut self,
        failure: &str,
        op: &str,
        ty: &Type,
        operands: (&str, Option<&str>),
        span: Span,
    ) -> Result<(), String> {
        let Type::Int { bits, signed } = ty else {
            unreachable!()
        };
        let fail = self.name("fail");
        let next = self.name("ok");
        self.branch(failure, &fail, &next);
        self.label(&fail);
        let extend = if *signed { "sext" } else { "zext" };
        let mut left = operands.0.to_owned();
        let mut right = operands.1.unwrap_or("0").to_owned();
        if *bits < 64 {
            left = self.value(format!("{extend} i{bits} {left} to i64"));
            right = self.value(format!("{extend} i{bits} {right} to i64"));
        }
        let operation = if operands.1.is_none() {
            0
        } else {
            i32::from(op.as_bytes()[0])
        };
        let call = self.capture(
            "arithmetic_capture",
            "%panic",
            &format!(
                "i32 {operation}, i32 {bits}, i32 {}, i64 {left}, i64 {right}",
                i32::from(*signed)
            ),
            span,
        )?;
        self.line(call);
        self.jump("panic_exit");
        self.label(&next);
        Ok(())
    }

    pub(crate) fn binary(
        &mut self,
        op: &str,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> Result<String, String> {
        let first = self.expression(left)?;
        if self.ended {
            return Ok("undef".into());
        }
        if op == "&&" || op == "||" {
            let slot = self.slot(&Type::Bool);
            self.line(format!("store i1 {first}, ptr {slot}"));
            let rhs = self.name("rhs");
            let end = self.name("bool_end");
            if op == "&&" {
                self.branch(&first, &rhs, &end);
            } else {
                self.branch(&first, &end, &rhs);
            }
            self.label(&rhs);
            let second = self.expression(right)?;
            if !self.ended {
                self.line(format!("store i1 {second}, ptr {slot}"));
                self.jump(&end);
            }
            self.label(&end);
            return Ok(self.value(format!("load i1, ptr {slot}")));
        }
        let second = self.expression(right)?;
        if self.ended {
            return Ok("undef".into());
        }
        let ty = ir_type(&left.ty);
        if op == "==" || op == "!=" {
            let equal = self.equal(&left.ty, &first, &second)?;
            return if op == "==" {
                Ok(equal)
            } else {
                Ok(self.value(format!("xor i1 {equal}, true")))
            };
        }
        match &left.ty {
            Type::Int { bits, signed } => {
                let mut divisor = second.clone();
                let operation = match op {
                    "+" | "-" | "*" => {
                        return self.checked(op, &left.ty, &first, Some(&second), span);
                    }
                    "/" | "%" => {
                        let zero = self.value(format!("icmp eq {ty} {second}, 0"));
                        let failure = if *signed {
                            let minimum = -(1i128 << (bits - 1));
                            let minimum = self.value(format!("icmp eq {ty} {first}, {minimum}"));
                            let negative = self.value(format!("icmp eq {ty} {second}, -1"));
                            let overflow = self.value(format!("and i1 {minimum}, {negative}"));
                            if op == "%" {
                                divisor = self
                                    .value(format!("select i1 {overflow}, {ty} 1, {ty} {second}"));
                                zero
                            } else {
                                self.value(format!("or i1 {zero}, {overflow}"))
                            }
                        } else {
                            zero
                        };
                        self.arithmetic_guard(
                            &failure,
                            op,
                            &left.ty,
                            (&first, Some(&second)),
                            span,
                        )?;
                        match (op, signed) {
                            ("/", true) => "sdiv",
                            ("/", false) => "udiv",
                            (_, true) => "srem",
                            (_, false) => "urem",
                        }
                    }
                    "&" => "and",
                    "|" => "or",
                    "^" => "xor",
                    "<" | "<=" | ">" | ">=" => {
                        let prefix = if *signed { "s" } else { "u" };
                        let comparison = match op {
                            "<" => "lt",
                            "<=" => "le",
                            ">" => "gt",
                            _ => "ge",
                        };
                        return Ok(
                            self.value(format!("icmp {prefix}{comparison} {ty} {first}, {second}"))
                        );
                    }
                    _ => return Err(format!("unsupported checked integer operator {op}")),
                };
                Ok(self.value(format!("{operation} {ty} {first}, {divisor}")))
            }
            Type::Float { .. } => {
                let operation = match op {
                    "+" => "fadd",
                    "-" => "fsub",
                    "*" => "fmul",
                    "/" => "fdiv",
                    "%" => "frem",
                    "<" | "<=" | ">" | ">=" => {
                        let comparison = match op {
                            "<" => "olt",
                            "<=" => "ole",
                            ">" => "ogt",
                            _ => "oge",
                        };
                        return Ok(self.value(format!("fcmp {comparison} {ty} {first}, {second}")));
                    }
                    _ => return Err(format!("unsupported checked float operator {op}")),
                };
                Ok(self.value(format!("{operation} {ty} {first}, {second}")))
            }
            Type::String if matches!(op, "<" | "<=" | ">" | ">=") => {
                let a = self.value(format!("extractvalue {{ ptr, i64 }} {first}, 0"));
                let size_a = self.value(format!("extractvalue {{ ptr, i64 }} {first}, 1"));
                let b = self.value(format!("extractvalue {{ ptr, i64 }} {second}, 0"));
                let size_b = self.value(format!("extractvalue {{ ptr, i64 }} {second}, 1"));
                let order = self.value(format!("call i32 @meowy_string_compare_v1(ptr {a}, i64 {size_a}, ptr {b}, i64 {size_b})"));
                let comparison = match op {
                    "<" => "slt",
                    "<=" => "sle",
                    ">" => "sgt",
                    _ => "sge",
                };
                Ok(self.value(format!("icmp {comparison} i32 {order}, 0")))
            }
            _ => Err(format!(
                "unsupported checked binary operator {op} on {:?}",
                left.ty
            )),
        }
    }
}
