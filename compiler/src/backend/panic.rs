use super::Generator;
use crate::ast::Span;
use crate::hir::{Expr, Type};

impl<'a> Generator<'a> {
    pub(crate) fn panic_slot(&mut self) -> String {
        let size = format!("%{}", self.name("panic_size"));
        let slot = format!("%{}", self.name("panic"));
        self.slots.push(format!(
            "  {size} = call i64 @meowy_cleanup_panic_bytes_v0()"
        ));
        self.slots
            .push(format!("  {slot} = alloca i8, i64 {size}, align 16"));
        slot
    }

    pub(crate) fn failure_exit(&mut self) {
        self.label("panic_exit");
        self.line("ret i1 false".into());
        self.ended = true;
    }

    pub(crate) fn entry(&mut self) {
        self.begin();
        let panic = self.panic_slot();
        self.line(format!(
            "call void @meowy_panic_begin_v0(ptr {panic}, i32 0)"
        ));
        let success = self.value(format!("call i1 @meowy_entry(ptr {panic})"));
        let code = self.value(format!("select i1 {success}, i32 0, i32 1"));
        self.line(format!("ret i32 {code}"));
        self.finish("define i32 @main()".into());
    }

    pub(crate) fn panic(&mut self, parts: &[Expr], span: Span) -> Result<String, String> {
        let pending = self.panic_slot();
        self.line(format!(
            "call void @meowy_panic_begin_v0(ptr {pending}, i32 6)"
        ));
        let prefix = self.string("panic[P006]: ");
        self.print_value(&Type::String, &prefix, 2)?;
        for part in parts {
            let value = self.expression(part)?;
            if self.ended {
                return Ok("undef".into());
            }
            self.output_value(&part.ty, &value, 2, Some(&pending))?;
        }
        self.line(format!(
            "call void @meowy_panic_site_v0(ptr {pending}, i64 {}, i64 {})",
            span.start, span.end
        ));
        self.line(format!(
            "call void @meowy_panic_copy_v0(ptr %panic, ptr {pending})"
        ));
        self.jump("panic_exit");
        Ok("undef".into())
    }
}
