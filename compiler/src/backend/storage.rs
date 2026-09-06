use super::{Generator, ir_type};
use crate::hir::{BlockId, Type};

#[derive(Clone)]
pub(crate) struct Alias {
    pub(crate) target: BlockId,
    pub(crate) field: String,
    pub(crate) mutable: bool,
}

impl<'a> Generator<'a> {
    pub(crate) fn result_alias(
        &mut self,
        id: usize,
        target: BlockId,
        field: &str,
        mutable: bool,
    ) -> Result<(), String> {
        let local = self.program.locals.get(id).ok_or("missing alias local")?;
        let destination = self.blocks.get(&target).ok_or("missing alias target")?;
        let shared = match &destination.ty {
            Type::Record { fields, .. } => fields.iter().any(|slot| {
                slot.name == field && slot.mutable == mutable && slot.ty.accepts(local)
            }),
            _ => false,
        };
        if shared {
            self.aliases.insert(
                id,
                Alias {
                    target,
                    field: field.into(),
                    mutable,
                },
            );
        } else {
            self.aliases.remove(&id);
        }
        Ok(())
    }

    pub(crate) fn local_cell(&mut self, id: usize) -> Result<(String, Type), String> {
        let local = self
            .program
            .locals
            .get(id)
            .ok_or("missing local storage")?
            .clone();
        let Some(alias) = self.aliases.get(&id).cloned() else {
            self.local(id);
            return Ok((format!("%local{id}"), local));
        };
        let destination = self
            .blocks
            .get(&alias.target)
            .ok_or("alias target is no longer active")?
            .clone();
        let Type::Record { fields, .. } = &destination.ty else {
            return Err("alias target has no concrete record storage".into());
        };
        let (index, field) = fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == alias.field)
            .ok_or("missing alias result field")?;
        if field.mutable != alias.mutable || !field.ty.accepts(&local) {
            return Err("alias local type differs from result storage".into());
        }
        let ptr = self.value(format!(
            "getelementptr {}, ptr {}, i32 0, i32 {}",
            ir_type(&destination.ty),
            destination.slot,
            index + 1
        ));
        Ok((ptr, field.ty.clone()))
    }

    pub(crate) fn local_place(&mut self, id: usize) -> Result<(String, Type), String> {
        let local = self
            .program
            .locals
            .get(id)
            .ok_or("missing local place")?
            .clone();
        let (ptr, stored) = self.local_cell(id)?;
        if stored == local {
            return Ok((ptr, local));
        }
        let Type::Union(members) = &stored else {
            return Err("alias storage cannot represent its declared place".into());
        };
        let index = members
            .iter()
            .position(|member| member == &local)
            .ok_or("alias place needs an exact union member")?;
        let tag = self.value(format!("load i32, ptr {ptr}"));
        let active = self.value(format!("icmp eq i32 {tag}, {index}"));
        let ready = self.name("alias_member");
        let invalid = self.name("alias_invalid");
        self.branch(&active, &ready, &invalid);
        self.label(&invalid);
        self.line("unreachable".into());
        self.label(&ready);
        let ptr = self.value(format!(
            "getelementptr {}, ptr {ptr}, i32 0, i32 1",
            ir_type(&stored)
        ));
        Ok((ptr, local))
    }
}
