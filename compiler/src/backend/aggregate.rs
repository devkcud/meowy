use super::{Generator, ir_type, union_words};
use crate::hir::{Expr, Place, Type, WriteStep};

impl<'a> Generator<'a> {
    pub(crate) fn place(&mut self, place: &Place) -> Result<(String, Type), String> {
        let mut ty = self
            .program
            .locals
            .get(place.root)
            .ok_or_else(|| format!("missing storage root {}", place.root))?
            .clone();
        self.local(place.root);
        let mut ptr = format!("%local{}", place.root);
        for index in &place.fields {
            let Type::Record { fields, .. } = &ty else {
                return Err("storage projection requires a declared record".into());
            };
            let field = fields
                .get(*index)
                .ok_or_else(|| format!("missing storage field {index}"))?
                .ty
                .clone();
            ptr = self.value(format!(
                "getelementptr {}, ptr {ptr}, i32 0, i32 {}",
                ir_type(&ty),
                index + 1
            ));
            ty = field;
        }
        Ok((ptr, ty))
    }

    pub(crate) fn set_path(
        &mut self,
        id: usize,
        path: &[WriteStep],
        value: &Expr,
    ) -> Result<(), String> {
        if path.is_empty() {
            return Err("storage assignment requires a projection path".into());
        }
        let root = self
            .program
            .locals
            .get(id)
            .ok_or("missing write storage root")?
            .clone();
        self.local(id);
        let mut ptr = format!("%local{id}");
        let mut ty = &root;
        for step in path {
            match step {
                WriteStep::Field(index) => {
                    let Type::Record { fields, .. } = ty else {
                        return Err("field assignment requires concrete record storage".into());
                    };
                    let field = &fields.get(*index).ok_or("missing write field")?.ty;
                    ptr = self.value(format!(
                        "getelementptr {}, ptr {ptr}, i32 0, i32 {}",
                        ir_type(ty),
                        index + 1
                    ));
                    ty = field;
                }
                WriteStep::Index(step) => {
                    let Type::List { element, .. } = ty else {
                        return Err("element assignment requires concrete list storage".into());
                    };
                    let length = self.value(format!("load i64, ptr {ptr}"));
                    let position = self.expression(&step.index)?;
                    if self.ended {
                        return Ok(());
                    }
                    let offset = self.list_offset(&step.index, position, &length, step.span)?;
                    ptr = self.list_item(ty, &ptr, &offset);
                    ty = element;
                }
            }
        }
        let result = self.expression(value)?;
        if self.ended {
            return Ok(());
        }
        let result = self.coerce(&value.ty, ty, &result)?;
        self.store_value(ty, &result, &ptr);
        Ok(())
    }

    pub(crate) fn store_value(&mut self, ty: &Type, value: &str, ptr: &str) {
        let fields = match ty {
            Type::Record { primary, fields } => std::iter::once(primary.as_ref())
                .chain(fields.iter().map(|field| &field.ty))
                .cloned()
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        if !fields.is_empty() {
            for (index, field) in fields.iter().enumerate() {
                let part = self.value(format!("extractvalue {} {value}, {index}", ir_type(ty)));
                let dest = self.value(format!(
                    "getelementptr {}, ptr {ptr}, i32 0, i32 {index}",
                    ir_type(ty)
                ));
                self.store_value(field, &part, &dest);
            }
        } else if let Type::List { element, .. } = ty {
            let source = self.slot(ty);
            self.line(format!("store {} {value}, ptr {source}", ir_type(ty)));
            let length = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
            self.line(format!("store i64 {length}, ptr {ptr}"));
            let position = self.slot(&Type::Int {
                bits: 64,
                signed: false,
            });
            self.line(format!("store i64 0, ptr {position}"));
            let head = self.name("list_copy");
            let body = self.name("list_copy_item");
            let end = self.name("list_copy_end");
            self.jump(&head);
            self.label(&head);
            let index = self.value(format!("load i64, ptr {position}"));
            let active = self.value(format!("icmp ult i64 {index}, {length}"));
            self.branch(&active, &body, &end);
            self.label(&body);
            let item = self.list_item(ty, &source, &index);
            let value = self.value(format!("load {}, ptr {item}", ir_type(element)));
            let dest = self.list_item(ty, ptr, &index);
            self.store_value(element, &value, &dest);
            let next = self.value(format!("add i64 {index}, 1"));
            self.line(format!("store i64 {next}, ptr {position}"));
            self.jump(&head);
            self.label(&end);
        } else if let Type::Union(types) = ty {
            let tag = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
            let payload = self.value(format!("extractvalue {} {value}, 1", ir_type(ty)));
            let dest = self.value(format!(
                "getelementptr {}, ptr {ptr}, i32 0, i32 1",
                ir_type(ty)
            ));
            self.line(format!("store i32 {tag}, ptr {ptr}"));
            self.line(format!(
                "store [{} x i64] {payload}, ptr {dest}",
                union_words(types)
            ));
        } else if *ty == Type::Bool {
            let byte = self.value(format!("zext i1 {value} to i8"));
            self.line(format!("store i8 {byte}, ptr {ptr}"));
        } else {
            self.line(format!("store {} {value}, ptr {ptr}", ir_type(ty)));
        }
    }

    pub(crate) fn union_cases<F>(
        &mut self,
        ty: &Type,
        value: &str,
        mut apply: F,
    ) -> Result<(), String>
    where
        F: FnMut(&mut Self, &Type, &str) -> Result<(), String>,
    {
        let Type::Union(types) = ty else {
            return Err("variant dispatch requires a union".into());
        };
        let slot = self.slot(ty);
        self.line(format!("store {} {value}, ptr {slot}", ir_type(ty)));
        let ptr = self.value(format!(
            "getelementptr {}, ptr {slot}, i32 0, i32 1",
            ir_type(ty)
        ));
        let tag = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
        let labels = types
            .iter()
            .map(|_| self.name("variant"))
            .collect::<Vec<_>>();
        let invalid = self.name("invalid_tag");
        let end = self.name("union_end");
        let cases = labels
            .iter()
            .enumerate()
            .map(|(index, label)| format!("i32 {index}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        self.line(format!("switch i32 {tag}, label %{invalid} [ {cases} ]"));
        self.ended = true;
        for (label, member) in labels.iter().zip(types) {
            self.label(label);
            apply(self, member, &ptr)?;
            if !self.ended {
                self.jump(&end);
            }
        }
        self.label(&invalid);
        self.line("unreachable".into());
        self.ended = true;
        self.label(&end);
        Ok(())
    }

    pub(crate) fn coerce(&mut self, from: &Type, to: &Type, value: &str) -> Result<String, String> {
        if from == to {
            return Ok(value.into());
        }
        if *from == Type::Never {
            return Ok("undef".into());
        }
        if let Type::Union(_) = from {
            if from.intersection(to) == Type::Never {
                return Err(format!("cannot convert {from:?} to {to:?}"));
            }
            let slot = self.slot(to);
            self.union_cases(from, value, |this, member, ptr| {
                if to.accepts(member) {
                    let value = this.value(format!("load {}, ptr {ptr}", ir_type(member)));
                    let value = this.coerce(member, to, &value)?;
                    this.line(format!("store {} {value}, ptr {slot}", ir_type(to)));
                } else {
                    this.line("unreachable".into());
                    this.ended = true;
                }
                Ok(())
            })?;
            return Ok(self.value(format!("load {}, ptr {slot}", ir_type(to))));
        }
        if let Type::Union(types) = to
            && let Some(tag) = types.iter().position(|ty| ty == from)
        {
            let slot = self.slot(to);
            self.line(format!("store {} zeroinitializer, ptr {slot}", ir_type(to)));
            let ptr = self.value(format!(
                "getelementptr {}, ptr {slot}, i32 0, i32 1",
                ir_type(to)
            ));
            self.store_value(from, value, &ptr);
            let value = self.value(format!("load {}, ptr {slot}", ir_type(to)));
            return Ok(self.value(format!("insertvalue {} {value}, i32 {tag}, 0", ir_type(to))));
        }
        Err(format!("cannot convert {from:?} to {to:?}"))
    }

    pub(crate) fn type_test(&mut self, from: &Type, to: &Type, value: &str) -> String {
        if to.accepts(from) {
            return "true".into();
        }
        let Type::Union(types) = from else {
            return "false".into();
        };
        let tag = self.value(format!("extractvalue {} {value}, 0", ir_type(from)));
        let mut result = "false".to_owned();
        for (index, member) in types.iter().enumerate() {
            if to.accepts(member) {
                let matched = self.value(format!("icmp eq i32 {tag}, {index}"));
                result = self.value(format!("or i1 {result}, {matched}"));
            }
        }
        result
    }

    pub(crate) fn equal(&mut self, ty: &Type, left: &str, right: &str) -> Result<String, String> {
        match ty {
            Type::Null => Ok("true".into()),
            Type::Never => Err("cannot compare a never value".into()),
            Type::Bool | Type::Int { .. } | Type::Reference(_) => {
                Ok(self.value(format!("icmp eq {} {left}, {right}", ir_type(ty))))
            }
            Type::Float { .. } => {
                Ok(self.value(format!("fcmp oeq {} {left}, {right}", ir_type(ty))))
            }
            Type::String => {
                let a = self.value(format!("extractvalue {{ ptr, i64 }} {left}, 0"));
                let size_a = self.value(format!("extractvalue {{ ptr, i64 }} {left}, 1"));
                let b = self.value(format!("extractvalue {{ ptr, i64 }} {right}, 0"));
                let size_b = self.value(format!("extractvalue {{ ptr, i64 }} {right}, 1"));
                Ok(self.value(format!("call zeroext i1 @meowy_string_equal_v1(ptr {a}, i64 {size_a}, ptr {b}, i64 {size_b})")))
            }
            Type::List { element, .. } => self.list_equal(ty, element, left, right),
            Type::Record { primary, fields } => {
                let types =
                    std::iter::once(primary.as_ref()).chain(fields.iter().map(|field| &field.ty));
                let mut result = "true".to_owned();
                for (index, field) in types.enumerate() {
                    let a = self.value(format!("extractvalue {} {left}, {index}", ir_type(ty)));
                    let b = self.value(format!("extractvalue {} {right}, {index}", ir_type(ty)));
                    let equal = self.equal(field, &a, &b)?;
                    result = self.value(format!("and i1 {result}, {equal}"));
                }
                Ok(result)
            }
            Type::Union(_) => {
                let slot = self.slot(&Type::Bool);
                self.line(format!("store i1 false, ptr {slot}"));
                let a = self.value(format!("extractvalue {} {left}, 0", ir_type(ty)));
                let b = self.value(format!("extractvalue {} {right}, 0", ir_type(ty)));
                let same = self.value(format!("icmp eq i32 {a}, {b}"));
                let compare = self.name("union_equal");
                let end = self.name("equal_end");
                self.branch(&same, &compare, &end);
                self.label(&compare);
                let other = self.slot(ty);
                self.line(format!("store {} {right}, ptr {other}", ir_type(ty)));
                let other = self.value(format!(
                    "getelementptr {}, ptr {other}, i32 0, i32 1",
                    ir_type(ty)
                ));
                self.union_cases(ty, left, |this, member, ptr| {
                    let a = this.value(format!("load {}, ptr {ptr}", ir_type(member)));
                    let b = this.value(format!("load {}, ptr {other}", ir_type(member)));
                    let result = this.equal(member, &a, &b)?;
                    this.line(format!("store i1 {result}, ptr {slot}"));
                    Ok(())
                })?;
                self.jump(&end);
                self.label(&end);
                Ok(self.value(format!("load i1, ptr {slot}")))
            }
        }
    }
}
