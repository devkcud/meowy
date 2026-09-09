use super::access::inspect_body;
use crate::borrow::Source;
use crate::flow::FALSE;
use crate::hir::{Expr, ExprKind, IndexStep, Place, Program, Type, WriteStep};
use crate::loans::{Graph, Projection};

#[test]
pub(crate) fn indexed_field_acquisition_keeps_the_leaf_region_and_owner_reservation() {
    inspect_body(
        "r:{->rows:=[{->n:=1;->b:=2}];p:&!(rows[1].n);v:*p}",
        None,
        |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert_eq!(graph.loans.len(), 1);
            let loan = &graph.loans[0];
            assert!(loan.parent.is_none());
            assert!(matches!(&graph.values[loan.value].origins[0].source,
            Source::Slot { fields, .. } if fields == &[Projection::Element, Projection::Field(1)]));
            let node = loan.node;
            assert_eq!(graph.nodes[node].uses.len(), 1);
            let reservation = graph.nodes[node].uses[0];
            assert!(
                matches!(&graph.values[reservation].origins[0].source, Source::Slot { fields, .. } if fields.is_empty())
            );
            assert!(graph.authority[reservation].loans.is_empty());
            assert_eq!(graph.authority[reservation].opaque, FALSE);
            let live = graph.liveness(reach).unwrap();
            assert!(live[node].contains_key(&reservation));
            assert!(
                !graph
                    .outgoing(node, &live)
                    .unwrap()
                    .contains_key(&reservation)
            );
        },
    );
}

#[test]
pub(crate) fn cancelled_field_acquisition_retains_only_completed_index_demand() {
    for (source, completed) in [
        (
            "r:=[{->n:=1}];'out{p:&!(r[{r=[{->n:=2}];'out.leave()}].n)}",
            false,
        ),
        ("r:=[[{->n:=1}]];'out{p:&!(r[1][{'out.leave()}].n)}", true),
    ] {
        inspect_body(source, None, |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert!(graph.loans.is_empty());
            let live = graph.liveness(reach).unwrap();
            let demanded = graph
                .values
                .iter()
                .enumerate()
                .filter(|(id, _)| live.iter().any(|values| values.contains_key(id)))
                .collect::<Vec<_>>();
            assert_eq!(demanded.len(), usize::from(completed));
            for (id, value) in demanded {
                assert!(
                    matches!(&value.origins[0].source, Source::Local { fields, .. } if fields.is_empty())
                );
                assert!(graph.authority[id].loans.is_empty());
            }
        });
    }
}

#[test]
pub(crate) fn mutable_scalar_field_leaf_is_required_in_both_analysis_passes() {
    let tree = crate::parser::parse("rows:=[{->n:=1}];p:&!(rows[1].n);v:*p").unwrap();
    let mut checker = crate::check::Checker::new();
    let body = checker.block(&tree, None, None).unwrap();
    let mut program = Program {
        body,
        functions: Vec::new(),
        locals: checker.locals,
    };
    checker.proofs.conditions = checker.guards;
    checker.proofs.tags = checker.tags;
    let facts = crate::borrow::check(&program, &mut checker.flow, &checker.proofs).unwrap();
    let root = program.locals.iter().position(|ty| matches!(ty, Type::List { element, .. } if matches!(element.as_ref(), Type::Record { .. }))).unwrap();
    let place = Place {
        root,
        fields: Vec::new(),
    };
    let span = crate::ast::Span { start: 0, end: 0 };
    let path = [
        WriteStep::Index(IndexStep {
            index: Expr {
                kind: ExprKind::Int(1),
                ty: Type::Int {
                    signed: true,
                    bits: 32,
                },
                span,
            },
            span,
        }),
        WriteStep::Field(0),
    ];
    assert!(
        checker
            .proofs
            .exclusive_path_type(&program, &place, &path, &mut checker.flow, span)
            .is_some()
    );
    let Type::List { element, .. } = &mut program.locals[root] else {
        unreachable!()
    };
    let Type::Record { fields, .. } = element.as_mut() else {
        unreachable!()
    };
    fields[0].mutable = false;
    assert!(
        checker
            .proofs
            .exclusive_path_type(&program, &place, &path, &mut checker.flow, span)
            .is_none()
    );
    let graph = Graph::new(&program, &facts, &checker.proofs, &mut checker.flow);
    assert_eq!(graph.check(&program.body, &[]).unwrap_err().code, "B001");
    assert_eq!(
        crate::borrow::check(&program, &mut checker.flow, &checker.proofs)
            .err()
            .unwrap()[0]
            .code,
        "B001"
    );
}
