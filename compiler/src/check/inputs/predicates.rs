use crate::check::{Checker, inputs::Input, type_values};
use crate::hir::{Expr, ExprKind, Type};

impl Checker {
    pub(crate) fn predicate_expr(
        &mut self,
        expr: &Expr,
        depth: usize,
        count: &mut usize,
    ) -> Option<Input<bool>> {
        *count += 1;
        if *count > type_values::MAX_WORK
            || depth >= type_values::MAX_DEPTH
            || !self.flow.spend(1)
            || expr.ty != Type::Bool
        {
            return None;
        }
        let mut input = Input {
            work: 1,
            error: None,
            value: None,
        };
        match &expr.kind {
            ExprKind::Bool(value) => input.value = Some(*value),
            ExprKind::Unary { op, value } if op == "!" => {
                let source = self.predicate_expr(value, depth + 1, count)?;
                input.add(&source);
                input.value = source.value.map(|value| !value);
            }
            ExprKind::Binary { op, left, right } if matches!(op.as_str(), "&&" | "||") => {
                let left = self.predicate_expr(left, depth + 1, count)?;
                input.add(&left);
                input.value = left.value;
                if input.value == Some(op == "||") {
                    return Some(input);
                }
                let right = self.predicate_expr(right, depth + 1, count)?;
                input.add(&right);
                input.value = right.value;
            }
            _ => return None,
        }
        Some(input)
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
            checker
                .predicate_expr(&expr, 0, &mut count)
                .map(|input| (input.value, input.work)),
            Some((Some(true), 1))
        );
        assert!(checker.predicate_expr(&expr, 0, &mut count).is_none());
        assert_eq!(
            checker
                .predicate_expr(&expr, type_values::MAX_DEPTH - 1, &mut 0)
                .map(|input| (input.value, input.work)),
            Some((Some(true), 1))
        );
        assert!(
            checker
                .predicate_expr(&expr, type_values::MAX_DEPTH, &mut 0)
                .is_none()
        );
    }
}
