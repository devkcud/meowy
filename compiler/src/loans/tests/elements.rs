use super::access::inspect_body;
use crate::borrow::Source;
use crate::flow::FALSE;
use crate::hir::Program;
use crate::loans::{Graph, Projection};

#[test]
pub(crate) fn reservations_have_no_authority_and_end_at_acquisition() {
    inspect_body("xs:=[1,2];p:&!xs[xs[1]];v:*p", None, |graph, reach| {
        graph.solve_authority(reach).unwrap();
        assert_eq!(graph.loans.len(), 1);
        let node = graph.loans[0].node;
        assert!(graph.loans[0].parent.is_none());
        let source = &graph.values[graph.loans[0].value].origins[0].source;
        assert!(matches!(source,Source::Local {fields,..} if fields == &[Projection::Element]));
        let reservation=graph.values.iter().position(|value| value.origins.iter().any(|origin| matches!(&origin.source,Source::Local {id:0,fields} if fields.is_empty()))).unwrap();
        assert!(graph.authority[reservation].loans.is_empty());
        assert_eq!(graph.authority[reservation].opaque, FALSE);
        assert!(graph.nodes[node].uses.contains(&reservation));
        assert!(graph.nodes[node].copies.is_empty());
        let live = graph.liveness(reach).unwrap();
        assert!(live[node].contains_key(&reservation));
        assert!(
            !graph
                .outgoing(node, &live)
                .unwrap()
                .contains_key(&reservation)
        );
    });
}

#[test]
pub(crate) fn nonreturning_index_has_no_exclusive_acquisition_or_future_reservation() {
    inspect_body(
        "xs:=[1];'out{p:&!xs[{'out.leave()}]}",
        None,
        |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert!(graph.loans.is_empty());
            let live = graph.liveness(reach).unwrap();
            assert!(live.iter().all(|values| values.is_empty()));
        },
    );
}

#[test]
pub(crate) fn mutable_owner_proof_is_required_independently_of_storage_shape() {
    let tree = crate::parser::parse("xs:=[1];p:&!xs[1];v:*p").unwrap();
    let mut checker = crate::check::Checker::new();
    let body = checker.block(&tree, None, None).unwrap();
    let program = Program {
        body,
        functions: checker.functions.into_iter().flatten().collect(),
        locals: checker.locals,
    };
    checker.proofs.conditions = checker.guards;
    checker.proofs.tags = checker.tags;
    let facts = crate::borrow::check(&program, &mut checker.flow, &checker.proofs).unwrap();
    checker.proofs.mutable.clear();
    let graph = Graph::new(&program, &facts, &checker.proofs, &mut checker.flow);
    let error = graph.check(&program.body, &[]).unwrap_err();
    assert_eq!(error.code, "B001");
    assert!(error.message.contains("mutable scalar list owner proof"));
    let errors = crate::borrow::check(&program, &mut checker.flow, &checker.proofs)
        .err()
        .unwrap();
    assert_eq!(errors[0].code, "B001");
}

#[test]
pub(crate) fn projected_reservations_retain_the_selected_local_or_slot_path() {
    for (source, slot) in [
        ("r:={->xs:=[1];->ys:=[2]};p:&!r.xs[1];v:*p", false),
        ("r:{->row:={->xs:=[1];->ys:=[2]};p:&!row.xs[1];v:*p}", true),
    ] {
        inspect_body(source, None, |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert_eq!(graph.loans.len(), 1);
            let loan = &graph.loans[0];
            assert!(loan.parent.is_none());
            let element = &graph.values[loan.value].origins[0].source;
            let owner = match element {
                Source::Local { id, fields } if !slot => {
                    assert_eq!(fields, &[Projection::Field(0), Projection::Element]);
                    Source::Local {
                        id: *id,
                        fields: vec![Projection::Field(0)],
                    }
                }
                Source::Slot {
                    target,
                    root,
                    view,
                    fields,
                } if slot => {
                    assert_eq!(fields, &[Projection::Field(0), Projection::Element]);
                    Source::Slot {
                        target: *target,
                        root: *root,
                        view: *view,
                        fields: vec![Projection::Field(0)],
                    }
                }
                _ => panic!("missing canonical owned element source"),
            };
            let reservation = graph.nodes[loan.node]
                .uses
                .iter()
                .copied()
                .find(|id| {
                    graph.values[*id]
                        .origins
                        .iter()
                        .any(|origin| origin.source == owner)
                })
                .unwrap();
            assert!(graph.authority[reservation].loans.is_empty());
            assert_eq!(graph.authority[reservation].opaque, FALSE);
            let node = loan.node;
            let live = graph.liveness(reach).unwrap();
            assert!(live[node].contains_key(&reservation));
            assert!(
                !graph
                    .outgoing(node, &live)
                    .unwrap()
                    .contains_key(&reservation)
            );
        });
    }
}

#[test]
pub(crate) fn projected_owner_proof_rejects_missing_alias_and_field_evidence() {
    for change in 0..4 {
        let tree = crate::parser::parse("r:{->row:={->xs:=[1]};p:&!row.xs[1];v:*p}").unwrap();
        let mut checker = crate::check::Checker::new();
        let body = checker.block(&tree, None, None).unwrap();
        let mut program = Program {
            body,
            functions: checker.functions.into_iter().flatten().collect(),
            locals: checker.locals,
        };
        checker.proofs.conditions = checker.guards;
        checker.proofs.tags = checker.tags;
        let facts = crate::borrow::check(&program, &mut checker.flow, &checker.proofs).unwrap();
        let id = checker
            .proofs
            .aliases
            .iter()
            .find(|(_, alias)| alias.field == "row")
            .map(|(id, _)| *id)
            .unwrap();
        let alias = checker.proofs.aliases.get_mut(&id).unwrap();
        match change {
            0 => alias.exclusive = None,
            1 => alias.backing = None,
            2 => alias.mutable = false,
            _ => {
                let crate::hir::Type::Record { fields, .. } = &mut program.locals[id] else {
                    unreachable!()
                };
                fields[0].mutable = false;
            }
        }
        assert!(
            checker
                .proofs
                .exclusive_element_type(
                    &program,
                    &crate::hir::Place {
                        root: id,
                        fields: vec![0]
                    },
                    &mut checker.flow,
                    crate::ast::Span { start: 0, end: 0 },
                )
                .is_none()
        );
        let graph = Graph::new(&program, &facts, &checker.proofs, &mut checker.flow);
        let error = graph.check(&program.body, &[]).unwrap_err();
        assert_eq!(error.code, "B001");
        let errors = crate::borrow::check(&program, &mut checker.flow, &checker.proofs)
            .err()
            .unwrap();
        assert_eq!(errors[0].code, "B001");
    }
}

#[test]
pub(crate) fn cancelled_projected_indices_have_no_acquisition_demand() {
    for source in [
        "r:={->xs:=[1]};'out{p:&!r.xs[{r.xs=[2];'out.leave()}]}",
        "r:'out{->row:={->xs:=[1]};p:&!row.xs[{row.xs=[2];'out.leave()}]}",
    ] {
        inspect_body(source, None, |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert!(graph.loans.is_empty());
            let live = graph.liveness(reach).unwrap();
            assert!(live.iter().all(|values| values.is_empty()));
        });
    }
}
