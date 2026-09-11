use super::{Checker, MAX_DEPTH, Record};
use crate::hir::{self, ExprKind};

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
    pub(crate) fn record_path(expr: &hir::Expr) -> Option<(usize, Vec<usize>)> {
        let mut root = expr;
        let mut path = Vec::new();
        while let ExprKind::Field { value, index } = &root.kind {
            if path.len() == MAX_DEPTH {
                return None;
            }
            path.push(*index);
            root = value;
        }
        let ExprKind::Local(id) = root.kind else {
            return None;
        };
        path.reverse();
        Some((id, path))
    }
}
