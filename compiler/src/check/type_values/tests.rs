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

#[test]
pub(crate) fn computed_types_blocks_construct_scoped_records_lists_and_nested_aliases() {
    for source in [
        "<T>:{element:<int32>;-><(element)[4]>};v<T>:[1,2]",
        "<T>:{<Local>:<{n<int32>}>;token:<Local>;->token};v<T>:{->n:7}",
        "<T>:{t:{-><int32>};-><(t)>};v<T>:7",
        "core:@\"core\";<T>:{t:core.int32;->t};v<T>:7",
        "t:<int32>;<T>:{t:<string>;->t};v<T>:\"yes\";n<(t)>:7",
        "<T>:{-><int32>;unused:<string>};v<T>:7",
    ] {
        crate::compile(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
}

#[test]
pub(crate) fn computed_types_blocks_reject_leaks_duplicate_emissions_and_wrong_results() {
    for (source, code) in [
        ("<T>:{inner:<int32>;->inner};v<(inner)>:1", "E201"),
        ("<T>:{<Inner>:<int32>;-><Inner>};v<Inner>:1", "E202"),
        ("<T>:{-><int32>;-><string>}", "E205"),
        ("<T>:{t:<int32>;t:<string>;->t}", "E203"),
        ("<T>:{<T>:<int32>;<T>:<string>;-><T>}", "E203"),
        ("<T>:{t:<int32>}", "E211"),
        ("x:1;<T>:{->x}", "E211"),
        ("<T>:{-><int32>};v<T>:true", "E207"),
    ] {
        assert_eq!(
            crate::compile(source).unwrap_err()[0].code,
            code,
            "{source}"
        );
    }
}

#[test]
pub(crate) fn computed_types_blocks_reject_effects_after_emissions_and_unsupported_control() {
    for source in [
        "d:@\"debug\";<T>:{d.print(1);-><int32>}",
        "d:@\"debug\";<T>:{(d.print(1));-><int32>}",
        "d:@\"debug\";fail:d.panic;<T>:{-><int32>;fail(\"no\")}",
        "d:@\"debug\";<T>:{t:d.print(1);-><int32>}",
    ] {
        assert_eq!(
            crate::compile(source).unwrap_err()[0].code,
            "E219",
            "{source}"
        );
    }
    for source in [
        "<T>:{t:=<int32>;->t}",
        "<T>:{|true|-><int32>;|_|-><string>}",
        "f<int32>:(){->1};<T>:{->f()}",
        "d:@\"debug\";print<int32>:(){->1};<T>:{->print()}",
        "<T>:{->named:<int32>}",
    ] {
        assert_eq!(
            crate::compile(source).unwrap_err()[0].code,
            "B001",
            "{source}"
        );
    }
}

#[test]
pub(crate) fn computed_types_blocks_share_work_limits_across_sibling_blocks() {
    let inner = (0..40)
        .map(|id| format!("t{id}:<int32>;"))
        .collect::<String>();
    let outer = (0..60)
        .map(|id| format!("t{id}:{{{inner}-><int32>}};"))
        .collect::<String>();
    let source = format!("<T>:{{{outer}-><int32>}}");
    let error = crate::compile(&source).unwrap_err().remove(0);
    assert_eq!(error.code, "B001");
    assert!(
        error.message.contains("computed type bootstrap budget"),
        "{error:?}"
    );
}

#[test]
pub(crate) fn computed_types_blocks_leave_no_runtime_statements_or_storage() {
    let program =
        crate::compile("<T>:{element:<int32>;row:{-><{n<int32>}>};-><(element)[4]>}").unwrap();
    assert!(program.body.stmts.is_empty());
    assert!(program.locals.is_empty());
    assert!(program.functions.is_empty());
}

#[test]
pub(crate) fn computed_types_documentation_derives_constructed_signatures() {
    let source = include_str!("../../../examples/computed-types.mwy");
    let (_, model) = crate::documentation::checked(source, true).unwrap();
    let model = model.unwrap();
    let counts = model
        .entries
        .iter()
        .find(|entry| entry.name == "Counts")
        .unwrap();
    assert_eq!(counts.signature, "int32[4]");
    let row = model
        .entries
        .iter()
        .find(|entry| entry.name == "Row")
        .unwrap();
    assert_eq!(row.signature, "{count<int32>;label<string>}");
}

#[test]
pub(crate) fn computed_integers_preserve_literal_widths_aliases_and_local_scope() {
    for source in [
        "<T>:{n:4;-><int32[n]>};v<T>:[1,2]",
        "<T>:{n<uint8>:4;copy:n;-><int32[copy]>};v<T>:[1]",
        "<T>:{n<uint64>:4294967296;copy:n;->copy<>};v<T>:4294967296",
    ] {
        crate::compile(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
    assert_eq!(
        crate::compile("<T>:{n<uint8>:256;-><int32>}").unwrap_err()[0].code,
        "E216"
    );
    assert_eq!(
        crate::compile("<T>:{n:4;-><int32[n]>};v:n").unwrap_err()[0].code,
        "E201"
    );
    let program = crate::compile("<T>:{n<uint8>:4;copy:n;-><int32[copy]>}").unwrap();
    assert!(program.locals.is_empty());
    assert!(program.body.stmts.is_empty());
}
