use super::{Checker, Input, MAX_DEPTH, Record, Sources};
use crate::check::exports;
use crate::hir::{self, ExprKind, Type};

impl Record {
    pub(crate) fn project(&self, path: &[usize]) -> Option<Self> {
        let values = self
            .values
            .iter()
            .filter_map(|(key, value)| {
                let suffix = key.strip_prefix(path)?;
                (!suffix.is_empty()).then(|| (suffix.to_vec(), *value))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        if values.is_empty() {
            return None;
        }
        let mut input = self.input.clone();
        input.work = input.work.saturating_add(path.len());
        Some(Self { input, values })
    }
}

impl Checker {
    pub(crate) fn input_path<'a>(
        &mut self,
        id: usize,
        path: &'a [usize],
    ) -> Option<(exports::Input, &'a [usize])> {
        let Some(module) = self.exports.get(&id) else {
            return Some((exports::Input { id, work: 0 }, path));
        };
        let (index, path) = path.split_first()?;
        let Type::Record { fields, .. } = self.locals.get(id)? else {
            return None;
        };
        let field = fields.get(*index)?;
        if !self.flow.spend(field.name.len() + module.inputs.len() + 1) {
            return None;
        }
        let mut input = *module.inputs.get(&field.name)?;
        input.work = input.work.saturating_add(1);
        Some((input, path))
    }

    pub(crate) fn field_input(
        &mut self,
        id: usize,
        path: &[usize],
        locals: &Sources,
    ) -> Option<Input> {
        let (source, tail) = self.input_path(id, path)?;
        let mut input = if tail.is_empty() {
            locals
                .integers
                .get(&source.id)
                .or_else(|| self.inputs.get(&source.id))?
                .clone()
        } else {
            self.source_record(source.id, locals)?.field(tail)?
        };
        input.work = input.work.saturating_add(source.work);
        Some(input)
    }

    pub(crate) fn record_path(&self, expr: &hir::Expr) -> Option<(usize, Vec<usize>)> {
        let mut root = expr;
        let mut path = Vec::new();
        while let ExprKind::Field { value, index } = &root.kind {
            if path.len() == MAX_DEPTH + 1 {
                return None;
            }
            path.push(*index);
            root = value;
        }
        let ExprKind::Local(id) = root.kind else {
            return None;
        };
        if path.len() > MAX_DEPTH && !self.exports.contains_key(&id) {
            return None;
        }
        path.reverse();
        Some((id, path))
    }
}
