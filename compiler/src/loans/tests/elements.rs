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
