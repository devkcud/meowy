mod paths;

use super::{Checker, Input, Sources};
use crate::hir::{self, ExprKind, Type};
use std::collections::BTreeMap;

pub(crate) const MAX_FIELDS: usize = 256;
pub(crate) const MAX_DEPTH: usize = 32;

#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub(crate) input: Input,
    pub(crate) values: BTreeMap<Vec<usize>, Option<i128>>,
}

impl Record {
    pub(crate) fn field(&self, path: &[usize]) -> Option<Input> {
        let value = *self.values.get(path)?;
        let mut input = self.input.clone();
        input.work = input.work.saturating_add(path.len());
        input.value = if input.error.is_none() { value } else { None };
        Some(input)
    }
}

impl Checker {
    pub(crate) fn source_record<'a>(
        &'a self,
        id: usize,
        locals: &'a Sources,
    ) -> Option<&'a Record> {
        locals
            .records
            .get(&id)
            .or_else(|| self.record_inputs.get(&id))
    }

    pub(crate) fn record_shape(&mut self, ty: &Type) -> bool {
        let mut pending = vec![(ty, 1)];
        let mut count = 0;
        while let Some((ty, depth)) = pending.pop() {
            if !self.flow.spend(1) {
                return false;
            }
            match ty {
                Type::Int { .. } => {}
                Type::Record { primary, fields } => {
                    count += fields.len();
                    if **primary != Type::Null
                        || fields.is_empty()
                        || count > MAX_FIELDS
                        || depth > MAX_DEPTH
                    {
                        return false;
                    }
                    for field in fields {
                        if field.mutable {
                            return false;
                        }
                        pending.push((&field.ty, depth + 1));
                    }
                }
                _ => return false,
            }
        }
        true
    }

    pub(crate) fn record_input(&mut self, expr: &hir::Expr, ty: &Type) -> Option<Record> {
        if !matches!(ty, Type::Record { .. }) {
            return None;
        }
        self.record_expr(expr, ty, 0, &mut 0, &Sources::default())
    }

    pub(crate) fn record_expr(
        &mut self,
        expr: &hir::Expr,
        ty: &Type,
        depth: usize,
        count: &mut usize,
        locals: &Sources,
    ) -> Option<Record> {
        *count += 1;
        if *count > super::super::type_values::MAX_WORK
            || depth >= MAX_DEPTH
            || !self.flow.spend(1)
            || !self.record_shape(ty)
        {
            return None;
        }
        let Type::Record { fields, .. } = ty else {
            return None;
        };
        if let ExprKind::Local(id) = &expr.kind {
            if self.locals.get(*id) != Some(ty) {
                return None;
            }
            let mut record = self.source_record(*id, locals)?.clone();
            record.input.work = record.input.work.saturating_add(1);
            return Some(record);
        }
        if matches!(expr.kind, ExprKind::Field { .. }) {
            let (id, path) = self.record_path(expr)?;
            let source = self.input_path(id, &path)?;
            let mut record = self
                .source_record(source.id, locals)?
                .project(&source.path)?;
            record.input.work = record.input.work.saturating_add(source.work);
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
            values: BTreeMap::new(),
        };
        let mut locals = locals.clone();
        let mut emitted = BTreeMap::new();
        let mut types = BTreeMap::new();
        for stmt in &block.stmts {
            if !self.flow.spend(1) {
                return None;
            }
            if let hir::Stmt::Emit {
                field: Some(name),
                value:
                    hir::Expr {
                        kind: ExprKind::Local(id),
                        ..
                    },
                ..
            } = stmt
            {
                if !self.flow.spend(fields.len()) {
                    return None;
                }
                types.insert(*id, &fields.iter().find(|field| field.name == *name)?.ty);
            }
        }
        for stmt in &block.stmts {
            *count += 1;
            if *count > super::super::type_values::MAX_WORK || !self.flow.spend(1) {
                return None;
            }
            record.input.work = record.input.work.saturating_add(1);
            match stmt {
                hir::Stmt::Bind { id, value } if !self.proofs.mutable.contains(id) => {
                    let ty = types
                        .get(id)
                        .copied()
                        .or_else(|| self.locals.get(*id))
                        .unwrap_or(&value.ty)
                        .clone();
                    if matches!(ty, Type::Record { .. }) {
                        let child = self.record_expr(value, &ty, depth + 1, count, &locals)?;
                        record.input.add(&child.input);
                        locals.records.insert(*id, child);
                    } else {
                        let input = self.input_expr(value, depth + 1, count, &locals)?;
                        record.input.add(&input);
                        locals.integers.insert(*id, input);
                    }
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
                    if matches!(fields[index].ty, Type::Record { .. }) {
                        let child = self.source_record(id, &locals)?;
                        record.input.add(&child.input);
                        for (path, value) in &child.values {
                            let mut target = vec![index];
                            target.extend(path);
                            record.values.insert(target, *value);
                        }
                    } else {
                        let input = self.input_expr(value, depth + 1, count, &locals)?;
                        record.input.add(&input);
                        record.values.insert(vec![index], input.value);
                    }
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
        if emitted.len() != fields.len()
            || depth == 0 && expr.ty == Type::Never && record.input.error.is_none()
        {
            return None;
        }
        Some(record)
    }
}

#[cfg(test)]
mod tests;
