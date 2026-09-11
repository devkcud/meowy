use super::*;
use crate::ast::{Expr, TypeExpr, TypeKind};

pub(crate) fn literal() -> Expr {
    Expr {
        kind: ExprKind::TypeValue(TypeExpr {
            kind: TypeKind::Name("int32".into()),
            span: Span::new(3, 10),
        }),
        span: Span::new(3, 10),
    }
}

#[test]
pub(crate) fn computed_types_bound_nested_depth_and_reset_failed_roots() {
    let mut expr = literal();
    for _ in 0..MAX_DEPTH {
        expr = Expr {
            kind: ExprKind::Group(Box::new(expr)),
            span: Span::new(1, 12),
        };
    }
    let mut checker = Checker::new();
    let error = checker.type_value(&expr).unwrap_err();
    assert_eq!(error.code, "B001");
    assert!(error.message.contains("computed type bootstrap budget"));
    assert!(checker.type_work.is_none());
    for _ in 0..MAX_WORK + 1 {
        assert_eq!(
            checker.type_value(&literal()).unwrap(),
            Type::Int {
                bits: 32,
                signed: true
            }
        );
    }
}

#[test]
pub(crate) fn computed_types_bound_wide_nested_roots_and_materialized_types() {
    for (count, computed) in [(MAX_WORK + 1, true), (MAX_NODES + 1, false)] {
        let fields = (0..count)
            .map(|id| {
                let kind = if computed {
                    TypeKind::Computed(Box::new(literal()))
                } else {
                    TypeKind::Name("int32".into())
                };
                (
                    format!("n{id}"),
                    TypeExpr {
                        kind,
                        span: Span::new(5, 9),
                    },
                    false,
                )
            })
            .collect();
        let expr = Expr {
            kind: ExprKind::TypeValue(TypeExpr {
                kind: TypeKind::Record {
                    primary: None,
                    fields,
                },
                span: Span::new(2, 20),
            }),
            span: Span::new(2, 20),
        };
        let error = Checker::new().type_value(&expr).unwrap_err();
        assert_eq!(error.code, "B001");
        assert!(error.message.contains("computed type bootstrap budget"));
    }
}
