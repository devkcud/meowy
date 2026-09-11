mod blocks;
mod records;

pub(crate) use records::Record;

use super::{Checker, Constant};
use crate::diagnostic::Diagnostic;
use crate::hir::{self, ExprKind, Type};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub(crate) struct Sources {
    pub(crate) integers: BTreeMap<usize, Input>,
    pub(crate) records: BTreeMap<usize, Record>,
}

#[derive(Clone, Debug)]
pub(crate) struct Input {
    pub(crate) work: usize,
    pub(crate) error: Option<Diagnostic>,
    pub(crate) value: Option<i128>,
}

impl Input {
    pub(crate) fn add(&mut self, source: &Self) {
        self.work = self.work.saturating_add(source.work);
        self.error = self.error.take().or_else(|| source.error.clone());
    }

    pub(crate) fn literal(&self, source: &hir::Expr) -> hir::Expr {
        hir::Expr {
            kind: ExprKind::Int(self.value.unwrap_or(0)),
            ty: source.ty.clone(),
            span: source.span,
        }
    }
}

impl Checker {
    pub(crate) fn integer_input(&mut self, expr: &hir::Expr) -> Option<Input> {
        self.input_expr(expr, 0, &mut 0, &Sources::default())
    }

    pub(crate) fn input_expr(
        &mut self,
        expr: &hir::Expr,
        depth: usize,
        count: &mut usize,
        locals: &Sources,
    ) -> Option<Input> {
        *count += 1;
        if *count > super::type_values::MAX_WORK
            || depth >= super::type_values::MAX_DEPTH
            || !self.flow.spend(1)
            || !(matches!(expr.ty, Type::Int { .. })
                || expr.ty == Type::Never && matches!(expr.kind, ExprKind::Block(_)))
        {
            return None;
        }
        let mut input = Input {
            work: 1,
            error: None,
            value: None,
        };
        let kind = match &expr.kind {
            ExprKind::Int(value) => ExprKind::Int(*value),
            ExprKind::Local(id) => {
                let source = locals.integers.get(id).or_else(|| self.inputs.get(id))?;
                input.add(source);
                source.literal(expr).kind
            }
            ExprKind::Unary { op, value } if matches!(op.as_str(), "-" | "~") => {
                let source = self.input_expr(value, depth + 1, count, locals)?;
                input.add(&source);
                ExprKind::Unary {
                    op: op.clone(),
                    value: Box::new(source.literal(value)),
                }
            }
            ExprKind::Binary { op, left, right }
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^") =>
            {
                let a = self.input_expr(left, depth + 1, count, locals)?;
                let b = self.input_expr(right, depth + 1, count, locals)?;
                input.add(&a);
                input.add(&b);
                ExprKind::Binary {
                    op: op.clone(),
                    left: Box::new(a.literal(left)),
                    right: Box::new(b.literal(right)),
                }
            }
            ExprKind::Field { value, index } => {
                let ExprKind::Local(id) = value.kind else {
                    return None;
                };
                let source = self.field_input(id, *index)?;
                input.add(&source);
                source.literal(expr).kind
            }
            ExprKind::Block(block) => {
                let source = self.input_block(block, depth + 1, count, locals)?;
                input.add(&source);
                if expr.ty == Type::Never {
                    return input.error.is_some().then_some(input);
                }
                source.literal(expr).kind
            }
            _ => return None,
        };
        if input.error.is_none() {
            let value = hir::Expr {
                kind,
                ty: expr.ty.clone(),
                span: expr.span,
            };
            match self.constant(&value) {
                Some(Constant::Int(value)) if Self::in_range(value, &expr.ty) => {
                    input.value = Some(value)
                }
                _ => {
                    input.error = Some(Self::error(
                        "E107",
                        "invalid integer initializer during required evaluation",
                        expr.span,
                    ))
                }
            }
        }
        Some(input)
    }
}

#[cfg(test)]
mod tests;
