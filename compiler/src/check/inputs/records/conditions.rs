use crate::check::{Checker, type_values};
use crate::hir::{Expr, ExprKind, Type};

impl Checker {
    pub(crate) fn literal_condition(
        &mut self,
        expr: &Expr,
        depth: usize,
        count: &mut usize,
    ) -> Option<(bool, usize)> {
        *count += 1;
        if *count > type_values::MAX_WORK
            || depth >= type_values::MAX_DEPTH
            || !self.flow.spend(1)
            || expr.ty != Type::Bool
        {
            return None;
        }
        match &expr.kind {
            ExprKind::Bool(value) => Some((*value, 1)),
            ExprKind::Unary { op, value } if op == "!" => {
                let (value, work) = self.literal_condition(value, depth + 1, count)?;
                Some((!value, work.saturating_add(1)))
            }
            ExprKind::Binary { op, left, right } if matches!(op.as_str(), "&&" | "||") => {
                let (value, work) = self.literal_condition(left, depth + 1, count)?;
                let work = work.saturating_add(1);
                if value == (op == "||") {
                    return Some((value, work));
                }
                let (value, rest) = self.literal_condition(right, depth + 1, count)?;
                Some((value, work.saturating_add(rest)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    #[test]
    pub(crate) fn literal_conditions_bound_visited_nodes_and_active_depth() {
        let expr = Expr {
            kind: ExprKind::Bool(true),
            ty: Type::Bool,
            span: Span { start: 0, end: 4 },
        };
        let mut checker = Checker::new();
        let mut count = type_values::MAX_WORK - 1;
        assert_eq!(
            checker.literal_condition(&expr, 0, &mut count),
            Some((true, 1))
        );
        assert!(checker.literal_condition(&expr, 0, &mut count).is_none());
        assert_eq!(
            checker.literal_condition(&expr, type_values::MAX_DEPTH - 1, &mut 0),
            Some((true, 1))
        );
        assert!(
            checker
                .literal_condition(&expr, type_values::MAX_DEPTH, &mut 0)
                .is_none()
        );
    }
}
