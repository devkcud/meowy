use super::{Checker, Input};
use crate::hir::{self, ExprKind, Type};
use std::collections::BTreeMap;

pub(crate) const MAX_FIELDS: usize = 256;

#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub(crate) input: Input,
    pub(crate) values: Vec<Option<i128>>,
}

impl Checker {
    pub(crate) fn field_input(&self, id: usize, index: usize) -> Option<Input> {
        let record = self.record_inputs.get(&id)?;
        let mut input = record.input.clone();
        input.work = input.work.saturating_add(1);
        input.value = if input.error.is_none() {
            *record.values.get(index)?
        } else {
            None
        };
        Some(input)
    }

    pub(crate) fn record_input(&mut self, expr: &hir::Expr, ty: &Type) -> Option<Record> {
        let Type::Record { primary, fields } = ty else {
            return None;
        };
        if **primary != Type::Null
            || fields.is_empty()
            || fields.len() > MAX_FIELDS
            || fields
                .iter()
                .any(|field| field.mutable || !matches!(field.ty, Type::Int { .. }))
            || !self.flow.spend(fields.len() + 1)
        {
            return None;
        }
        if let ExprKind::Local(id) = &expr.kind {
            if self.locals.get(*id) != Some(ty) {
                return None;
            }
            let mut record = self.record_inputs.get(id)?.clone();
            record.input.work = record.input.work.saturating_add(1);
            return Some(record);
        }
        let ExprKind::Block(block) = &expr.kind else {
            return None;
        };
        let mut record = Record {
            input: Input {
                work: 1,
                error: None,
                value: None,
            },
            values: vec![None; fields.len()],
        };
        let mut locals = BTreeMap::new();
        let mut emitted = BTreeMap::new();
        let mut count = 0;
        for stmt in &block.stmts {
            count += 1;
            if count > super::super::type_values::MAX_WORK || !self.flow.spend(1) {
                return None;
            }
            record.input.work = record.input.work.saturating_add(1);
            match stmt {
                hir::Stmt::Bind { id, value } if !self.proofs.mutable.contains(id) => {
                    let input = self.input_expr(value, 1, &mut count, &locals)?;
                    record.input.add(&input);
                    locals.insert(*id, input);
                }
                hir::Stmt::Emit {
                    target,
                    field: Some(name),
                    value,
                    ..
                } if *target == block.id => {
                    if !self.flow.spend(fields.len()) {
                        return None;
                    }
                    let index = fields.iter().position(|field| field.name == *name)?;
                    let ExprKind::Local(id) = value.kind else {
                        return None;
                    };
                    if emitted.insert(name.clone(), id).is_some() {
                        return None;
                    }
                    let input = self.input_expr(value, 1, &mut count, &locals)?;
                    record.input.add(&input);
                    record.values[index] = input.value;
                }
                hir::Stmt::SlotAlias {
                    id,
                    target,
                    field,
                    mutable: false,
                } if *target == block.id && emitted.get(field) == Some(id) => {}
                _ => return None,
            }
        }
        if emitted.len() != fields.len() || expr.ty == Type::Never && record.input.error.is_none() {
            return None;
        }
        Some(record)
    }
}

#[cfg(test)]
mod tests;
