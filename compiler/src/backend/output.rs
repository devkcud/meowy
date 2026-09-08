use super::{Generator, ir_type};
use crate::hir::{Expr, Type};

impl<'a> Generator<'a> {
    pub(crate) fn string(&mut self, value: &str) -> String {
        let name = self.name("str");
        let bytes = value
            .as_bytes()
            .iter()
            .map(|byte| format!("\\{byte:02X}"))
            .collect::<String>();
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{bytes}\", align 1",
            value.len()
        ));
        format!("{{ ptr @{name}, i64 {} }}", value.len())
    }

    pub(crate) fn print(&mut self, parts: &[Expr], fd: i32) -> Result<(), String> {
        for part in parts {
            let value = self.expression(part)?;
            if self.ended {
                break;
            }
            self.print_value(&part.ty, &value, fd)?;
        }
        Ok(())
    }

    pub(crate) fn print_value(&mut self, ty: &Type, value: &str, fd: i32) -> Result<(), String> {
        self.output_value(ty, value, fd, None)
    }

    pub(crate) fn output_value(
        &mut self,
        ty: &Type,
        value: &str,
        fd: i32,
        panic: Option<&str>,
    ) -> Result<(), String> {
        let sink = panic
            .map(|ptr| format!("ptr {ptr}"))
            .unwrap_or_else(|| format!("i32 {fd}"));
        match ty {
            Type::Null => {
                let value = self.string("null");
                self.output_value(&Type::String, &value, fd, panic)?;
            }
            Type::Never => return Err("cannot print a never value".into()),
            Type::Reference(_) | Type::Exclusive(_) => {
                return Err("shared-reference formatting is unavailable".into());
            }
            Type::List { .. } => return Err("list formatting is unavailable".into()),
            Type::Bool => {
                let name = if panic.is_some() {
                    "meowy_panic_bool_v0"
                } else {
                    "meowy_bool_v1"
                };
                self.line(format!("call void @{name}({sink}, i1 zeroext {value})"));
            }
            Type::Int { bits, signed } => {
                let value = if *bits < 64 {
                    self.value(format!(
                        "{} i{bits} {value} to i64",
                        if *signed { "sext" } else { "zext" }
                    ))
                } else {
                    value.to_owned()
                };
                let name = if *signed { "int" } else { "uint" };
                let name = if panic.is_some() {
                    format!("meowy_panic_{name}_v0")
                } else {
                    format!("meowy_{name}_v1")
                };
                self.line(format!("call void @{name}({sink}, i64 {value})"));
            }
            Type::Float { bits } => {
                let value = if *bits == 32 {
                    self.value(format!("fpext float {value} to double"))
                } else {
                    value.to_owned()
                };
                let name = if panic.is_some() {
                    "meowy_panic_float_v0"
                } else {
                    "meowy_float_v1"
                };
                self.line(format!(
                    "call void @{name}({sink}, double {value}, i32 {bits})"
                ));
            }
            Type::String => {
                let pointer = self.value(format!("extractvalue {{ ptr, i64 }} {value}, 0"));
                let size = self.value(format!("extractvalue {{ ptr, i64 }} {value}, 1"));
                let name = if panic.is_some() {
                    "meowy_panic_text_v0"
                } else {
                    "meowy_write_v1"
                };
                self.line(format!(
                    "call void @{name}({sink}, ptr {pointer}, i64 {size})"
                ));
            }
            Type::Record { primary, .. } => {
                let value = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
                self.output_value(primary, &value, fd, panic)?;
            }
            Type::Union(_) => {
                self.union_cases(ty, value, |this, member, ptr| {
                    let value = this.value(format!("load {}, ptr {ptr}", ir_type(member)));
                    this.output_value(member, &value, fd, panic)
                })?;
            }
        }
        Ok(())
    }
}
