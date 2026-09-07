use super::authority::{inspect, reads};
use crate::flow::{FALSE, TRUE};
use crate::hir::Program;
use crate::loans::Graph;

#[test]
pub(crate) fn guarded_result_parents_match_explicit_input_choices() {
    inspect(
        "pick<&!int32>:(p<&!int32>,q<&!int32>){->p};a:=1;b:=2;r:pick(&!a,&!b);v:*r",
        |graph, reach| {
            let value = *reads(graph, reach).last().unwrap();
            let loan = *graph.authority[value].loans.keys().next().unwrap();
            let parents = graph.loans[loan.0].parents.loans.clone();
            assert_eq!(parents.len(), 2);
            assert_eq!(graph.authority[value].opaque, FALSE);
            let mut covered = FALSE;
            for (parent, guard) in parents {
                assert!(!graph.guards.overlap(covered, guard));
                covered = graph.guards.or(covered, guard);
                let source = &graph.values[graph.loans[parent.0].value].origins[0].source;
                assert!(
                    graph.values[value]
                        .origins
                        .iter()
                        .filter(|origin| graph.guards.overlap(guard, origin.guard))
                        .all(|origin| &origin.source == source)
                );
            }
            assert_eq!(covered, TRUE);
        },
    );
}

#[test]
pub(crate) fn equal_addresses_do_not_merge_distinct_argument_authority() {
    inspect(
        "pick<&int32>:(p<&int32>,q<&int32>){->p};a:1;r:pick(&a,&a);v:*r",
        |graph, reach| {
            let value = *reads(graph, reach).last().unwrap();
            let loan = *graph.authority[value].loans.keys().next().unwrap();
            let parents: Vec<_> = graph.loans[loan.0].parents.loans.iter().collect();
            assert_eq!(parents.len(), 2);
            assert_ne!(parents[0].0, parents[1].0);
            assert!(!graph.guards.overlap(*parents[0].1, *parents[1].1));
        },
    );
}

#[test]
pub(crate) fn missing_or_incomplete_return_evidence_cannot_authorize_access() {
    for missing in [0, 1, 2] {
        let source = "pick<&!int32>:(p<&!int32>,q<&!int32>){->p};a:=1;b:=2;r:pick(&!a,&!b);*r=3";
        let tree = crate::parser::parse(source).unwrap();
        let mut checker = crate::check::Checker::new();
        let body = checker.block(&tree, None, None).unwrap();
        let program = Program {
            body,
            functions: checker.functions.into_iter().flatten().collect(),
            locals: checker.locals,
        };
        checker.proofs.conditions = checker.guards;
        checker.proofs.tags = checker.tags;
        let mut facts = crate::borrow::check(&program, &mut checker.flow, &checker.proofs).unwrap();
        if missing == 0 {
            facts.returns.clear();
        } else if missing == 1 {
            facts.returns.values_mut().next().unwrap().pop();
        } else {
            facts.returns.values_mut().next().unwrap()[0].input = usize::MAX;
        }
        let graph = Graph::new(&program, &facts, &checker.proofs, &mut checker.flow);
        let error = graph.check(&program.body, &[]).unwrap_err();
        assert_eq!(error.code, "B001");
        assert!(
            error
                .message
                .contains("missing function result borrow proof")
        );
    }
}
