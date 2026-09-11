use super::{Checker, Constant};
use crate::diagnostic::Diagnostic;
use crate::hir::{self, ExprKind, Type};

#[derive(Clone, Debug)]
pub(crate) struct Input {
    pub(crate) work: usize,
    pub(crate) error: Option<Diagnostic>,
}

impl Checker {
    pub(crate) fn integer_input(&mut self, expr: &hir::Expr) -> Option<Input> {
        let mut pending = vec![(expr, 0)];
        let mut count = 0;
        let mut input = Input {
            work: 0,
            error: None,
        };
        while let Some((expr, depth)) = pending.pop() {
            count += 1;
            if count > super::type_values::MAX_WORK
                || depth >= super::type_values::MAX_DEPTH
                || !self.flow.spend(1)
                || !matches!(expr.ty, Type::Int { .. })
            {
                return None;
            }
            input.work = input.work.saturating_add(1);
            match &expr.kind {
                ExprKind::Int(_) => {}
                ExprKind::Local(id) => {
                    let source = self.inputs.get(id)?;
                    input.work = input.work.saturating_add(source.work);
                    input.error = input.error.or_else(|| source.error.clone());
                }
                ExprKind::Unary { op, value } if matches!(op.as_str(), "-" | "~") => {
                    pending.push((value, depth + 1));
                }
                ExprKind::Binary { op, left, right }
                    if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^") =>
                {
                    pending.push((right, depth + 1));
                    pending.push((left, depth + 1));
                }
                _ => return None,
            }
            if input.error.is_none()
                && !matches!(self.constant(expr), Some(Constant::Int(value)) if Self::in_range(value, &expr.ty))
            {
                input.error = Some(Self::error(
                    "E107",
                    "invalid integer initializer during required evaluation",
                    expr.span,
                ));
            }
        }
        Some(input)
    }
}

#[cfg(test)]
mod tests;
