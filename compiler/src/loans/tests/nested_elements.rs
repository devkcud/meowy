use super::access::inspect_body;
use crate::borrow::Source;
use crate::flow::FALSE;
use crate::hir::{Expr, ExprKind, IndexStep, Place, Program, Type, WriteStep};
use crate::loans::Projection;

#[test]
pub(crate) fn every_enclosing_list_reservation_reaches_acquisition_without_authority() {
    inspect_body(
        "r:{->rows:=[{->xs:=[1]}];p:&!(rows[1].xs[1]);v:*p}",
        None,
        |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert_eq!(graph.loans.len(), 1);
            let loan = &graph.loans[0];
            let node = loan.node;
            assert!(loan.parent.is_none());
            assert!(matches!(&graph.values[loan.value].origins[0].source,
            Source::Slot { fields, .. } if fields == &[Projection::Element, Projection::Field(0), Projection::Element]));
            let reservations = graph.nodes[node].uses.clone();
            assert_eq!(reservations.len(), 2);
            for (id, fields) in reservations
                .iter()
                .zip([vec![], vec![Projection::Element, Projection::Field(0)]])
            {
                assert!(
                    matches!(&graph.values[*id].origins[0].source, Source::Slot { fields: found, .. } if *found == fields)
                );
                assert!(graph.authority[*id].loans.is_empty());
                assert_eq!(graph.authority[*id].opaque, FALSE);
            }
            let live = graph.liveness(reach).unwrap();
            let out = graph.outgoing(node, &live).unwrap();
            for id in reservations {
                assert!(live[node].contains_key(&id));
                assert!(!out.contains_key(&id));
            }
        },
    );
}

#[test]
pub(crate) fn later_cancellation_retains_only_completed_index_demand() {
    for (source, completed) in [
        ("xs:=[[1]];'out{p:&!(xs[{'out.leave()}][1])}", false),
        ("xs:=[[1]];'out{p:&!(xs[1][{'out.leave()}])}", true),
    ] {
        inspect_body(source, None, |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert!(graph.loans.is_empty());
            let live = graph.liveness(reach).unwrap();
            let mut demanded = 0;
            for (id, value) in graph.values.iter().enumerate() {
                if live.iter().any(|state| state.contains_key(&id)) {
                    demanded += 1;
                    assert!(
                        matches!(&value.origins[0].source, Source::Local { fields, .. } if fields.is_empty())
                    );
                    assert!(graph.authority[id].loans.is_empty());
                    assert_eq!(graph.authority[id].opaque, FALSE);
                }
            }
            assert_eq!(demanded, usize::from(completed));
        });
    }
}

#[test]
pub(crate) fn nested_owner_proof_checks_intermediate_types_mutability_and_budget() {
    let tree = crate::parser::parse("rows:=[{->xs:=[1]}];p:&!(rows[1].xs[1]);v:*p").unwrap();
    let mut checker = crate::check::Checker::new();
    let body = checker.block(&tree, None, None).unwrap();
    let mut program = Program {
        body,
        functions: Vec::new(),
        locals: checker.locals,
    };
    let root = program.locals.iter().position(|ty| matches!(ty, Type::List { element, .. } if matches!(element.as_ref(), Type::Record { .. }))).unwrap();
    let place = Place {
        root,
        fields: Vec::new(),
    };
    let span = crate::ast::Span { start: 0, end: 0 };
    let mut path = vec![
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
    path.push(path[0].clone());
    assert!(
        checker
            .proofs
            .exclusive_path_type(&program, &place, &path, &mut checker.flow, span)
            .is_some()
    );
    let WriteStep::Index(index) = &mut path[0] else {
        unreachable!()
    };
    index.index.ty = Type::Bool;
    assert!(
        checker
            .proofs
            .exclusive_path_type(&program, &place, &path, &mut checker.flow, span)
            .is_none()
    );
    let WriteStep::Index(index) = &mut path[0] else {
        unreachable!()
    };
    index.index.ty = Type::Int {
        signed: true,
        bits: 32,
    };
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
    let Type::List { element, .. } = &mut program.locals[root] else {
        unreachable!()
    };
    let Type::Record { fields, .. } = element.as_mut() else {
        unreachable!()
    };
    fields[0].mutable = true;
    assert!(
        checker
            .proofs
            .exclusive_path_type(&program, &place, &path, &mut checker.flow, span)
            .is_some()
    );
    assert!(!checker.flow.spend(crate::flow::MAX_PROOF_WORK));
    assert!(
        checker
            .proofs
            .exclusive_path_type(&program, &place, &path, &mut checker.flow, span)
            .is_none()
    );
}
