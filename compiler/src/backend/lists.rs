use super::{Generator, ir_type};
use crate::ast::Span;
use crate::hir::{Expr, Type};

impl<'a> Generator<'a> {
    pub(crate) fn list_item(&mut self, ty: &Type, ptr: &str, index: &str) -> String {
        self.value(format!(
            "getelementptr {}, ptr {ptr}, i32 0, i32 1, i64 {index}",
            ir_type(ty)
        ))
    }

    pub(crate) fn list(&mut self, values: &[Expr], ty: &Type) -> Result<String, String> {
        let Type::List { element, capacity } = ty else {
            return Err("list literal requires a concrete list type".into());
        };
        if values.len() > *capacity {
            return Err("list literal exceeds its capacity".into());
        }
        let ptr = self.slot(ty);
        self.line(format!("store {} zeroinitializer, ptr {ptr}", ir_type(ty)));
        for (index, value) in values.iter().enumerate() {
            let result = self.expression(value)?;
            if self.ended {
                return Ok("undef".into());
            }
            let result = self.coerce(&value.ty, element, &result)?;
            let dest = self.list_item(ty, &ptr, &index.to_string());
            self.store_value(element, &result, &dest);
        }
        self.line(format!("store i64 {}, ptr {ptr}", values.len()));
        Ok(self.value(format!("load {}, ptr {ptr}", ir_type(ty))))
    }

    pub(crate) fn list_index(
        &mut self,
        value: &Expr,
        index: &Expr,
        span: Span,
    ) -> Result<String, String> {
        let result = self.expression(value)?;
        if self.ended {
            return Ok("undef".into());
        }
        let Type::List { element, .. } = &value.ty else {
            return Err("indexing requires a list value".into());
        };
        let ptr = self.slot(&value.ty);
        self.line(format!("store {} {result}, ptr {ptr}", ir_type(&value.ty)));
        let length = self.value(format!("extractvalue {} {result}, 0", ir_type(&value.ty)));
        let position = self.expression(index)?;
        if self.ended {
            return Ok("undef".into());
        }
        let offset = self.list_offset(index, position, &length, span)?;
        let ptr = self.list_item(&value.ty, &ptr, &offset);
        Ok(self.value(format!("load {}, ptr {ptr}", ir_type(element))))
    }

    pub(crate) fn element_borrow(
        &mut self,
        value: &Expr,
        index: &Expr,
        result: &Type,
        span: Span,
    ) -> Result<String, String> {
        let ptr = self.expression(value)?;
        if self.ended {
            return Ok("undef".into());
        }
        let Type::Reference(list) = &value.ty else {
            return Err("element borrowing requires a shared list reference".into());
        };
        let Type::List { element, .. } = list.as_ref() else {
            return Err("element borrowing requires concrete list storage".into());
        };
        let length = self.value(format!("load i64, ptr {ptr}"));
        let position = self.expression(index)?;
        if self.ended {
            return Ok("undef".into());
        }
        if result != &Type::Reference(element.clone()) {
            return Err("element borrow result type mismatch".into());
        }
        let offset = self.list_offset(index, position, &length, span)?;
        Ok(self.list_item(list, &ptr, &offset))
    }

    pub(crate) fn list_offset(
        &mut self,
        index: &Expr,
        mut position: String,
        length: &str,
        span: Span,
    ) -> Result<String, String> {
        let Type::Int { bits, signed } = index.ty else {
            return Err("list position requires an integer".into());
        };
        if bits < 64 {
            position = self.value(format!(
                "{} i{bits} {position} to i64",
                if signed { "sext" } else { "zext" }
            ));
        } else if bits != 64 {
            return Err("list positions wider than 64 bits are unavailable".into());
        }
        let zero = self.value(format!("icmp eq i64 {position}, 0"));
        let beyond = self.value(format!("icmp ugt i64 {position}, {length}"));
        let invalid = self.value(format!("or i1 {zero}, {beyond}"));
        self.list_guard(
            &invalid,
            format!(
                "call void @meowy_index_fail_v1(i64 {position}, i64 {length}, i32 {}, i64 {}, i64 {})",
                i32::from(signed), span.start, span.end
            ),
        );
        Ok(self.value(format!("sub i64 {position}, 1")))
    }

    pub(crate) fn list_add(
        &mut self,
        value: &Expr,
        item: &Expr,
        span: Span,
    ) -> Result<String, String> {
        let result = self.expression(value)?;
        if self.ended {
            return Ok("undef".into());
        }
        let Type::List { element, capacity } = &value.ty else {
            return Err("append requires a list value".into());
        };
        let ptr = self.slot(&value.ty);
        self.line(format!("store {} {result}, ptr {ptr}", ir_type(&value.ty)));
        let length = self.value(format!("extractvalue {} {result}, 0", ir_type(&value.ty)));
        let result = self.expression(item)?;
        if self.ended {
            return Ok("undef".into());
        }
        let result = self.coerce(&item.ty, element, &result)?;
        let full = self.value(format!("icmp uge i64 {length}, {capacity}"));
        self.list_guard(
            &full,
            format!(
                "call void @meowy_list_full_v1(i64 {length}, i64 {capacity}, i64 {}, i64 {})",
                span.start, span.end
            ),
        );
        let dest = self.list_item(&value.ty, &ptr, &length);
        self.store_value(element, &result, &dest);
        let size = self.value(format!("add i64 {length}, 1"));
        self.line(format!("store i64 {size}, ptr {ptr}"));
        Ok(self.value(format!("load {}, ptr {ptr}", ir_type(&value.ty))))
    }

    pub(crate) fn list_guard(&mut self, invalid: &str, call: String) {
        let fail = self.name("list_fail");
        let next = self.name("list_ok");
        self.branch(invalid, &fail, &next);
        self.label(&fail);
        self.line(call);
        self.line("unreachable".into());
        self.label(&next);
    }

    pub(crate) fn list_equal(
        &mut self,
        ty: &Type,
        element: &Type,
        left: &str,
        right: &str,
    ) -> Result<String, String> {
        let a = self.slot(ty);
        let b = self.slot(ty);
        self.line(format!("store {} {left}, ptr {a}", ir_type(ty)));
        self.line(format!("store {} {right}, ptr {b}", ir_type(ty)));
        let size = self.value(format!("extractvalue {} {left}, 0", ir_type(ty)));
        let other = self.value(format!("extractvalue {} {right}, 0", ir_type(ty)));
        let same = self.value(format!("icmp eq i64 {size}, {other}"));
        let result = self.slot(&Type::Bool);
        self.line(format!("store i1 false, ptr {result}"));
        let position = self.slot(&Type::Int {
            bits: 64,
            signed: false,
        });
        self.line(format!("store i64 0, ptr {position}"));
        let head = self.name("list_equal");
        let body = self.name("list_equal_item");
        let step = self.name("list_equal_next");
        let done = self.name("list_equal_done");
        let end = self.name("list_equal_end");
        self.branch(&same, &head, &end);
        self.label(&head);
        let index = self.value(format!("load i64, ptr {position}"));
        let active = self.value(format!("icmp ult i64 {index}, {size}"));
        self.branch(&active, &body, &done);
        self.label(&body);
        let left = self.list_item(ty, &a, &index);
        let right = self.list_item(ty, &b, &index);
        let left = self.value(format!("load {}, ptr {left}", ir_type(element)));
        let right = self.value(format!("load {}, ptr {right}", ir_type(element)));
        let equal = self.equal(element, &left, &right)?;
        self.branch(&equal, &step, &end);
        self.label(&step);
        let next = self.value(format!("add i64 {index}, 1"));
        self.line(format!("store i64 {next}, ptr {position}"));
        self.jump(&head);
        self.label(&done);
        self.line(format!("store i1 true, ptr {result}"));
        self.jump(&end);
        self.label(&end);
        Ok(self.value(format!("load i1, ptr {result}")))
    }
}
