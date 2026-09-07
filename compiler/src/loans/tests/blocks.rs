use super::access::inspect_body;
use super::authority::reads;
use crate::flow::{FALSE, TRUE};
use crate::hir::Program;
use crate::loans::Graph;

#[test]
pub(crate) fn block_results_keep_guarded_acquisitions_without_new_authority() {
    inspect_body(
        "f<int32>:(flag<boolean>){x:=1;r:{|flag|->&!x;|!flag|->&!x};->*r}",
        Some(0),
        |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert_eq!(graph.loans.len(), 2);
            assert!(graph.loans.iter().all(|loan| loan.parent.is_none()));
            let value = *reads(graph, reach).last().unwrap();
            let authority = &graph.authority[value];
            assert_eq!(authority.opaque, FALSE);
            assert_eq!(authority.loans.len(), 2);
            let mut covered = FALSE;
            for guard in authority.loans.values() {
                assert!(!graph.guards.overlap(covered, *guard));
                covered = graph.guards.or(covered, *guard);
            }
            assert_eq!(covered, TRUE);
        },
    );
}

#[test]
pub(crate) fn cancellation_cannot_bypass_missing_completion_evidence() {
    for completion in [false, true] {
        let source = "x:=1;p:&!x;'out{r:{->p;'out.leave()}};x=2";
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
        let facts = crate::borrow::check(&program, &mut checker.flow, &checker.proofs).unwrap();
        if completion {
            checker.proofs.completions.clear();
        } else {
            checker.proofs.emissions.clear();
        }
        let graph = Graph::new(&program, &facts, &checker.proofs, &mut checker.flow);
        let error = graph.check(&program.body, &[]).unwrap_err();
        assert_eq!(error.code, "B001");
        assert!(
            error
                .message
                .contains("missing scalar emission completion proof")
        );
    }
}
