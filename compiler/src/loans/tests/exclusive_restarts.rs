use super::access::inspect;
use crate::flow::FALSE;
use crate::hir::ReferenceMode;

pub(crate) const SOURCE: &str = "a:1;q:&a;<R>:<{n<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->n:=7;p:&!n;*p=8;first=false;'loop.restart()}}}";

#[test]
pub(crate) fn exclusive_restart_frontiers_preserve_precise_local_authority() {
    inspect(SOURCE, |graph, reach| {
        graph.emission_states().unwrap();
        graph.exclusive_restart_frontiers(reach).unwrap();
        graph.solve_authority(reach).unwrap();
        let loan = graph
            .loans
            .iter()
            .find(|loan| loan.mode == ReferenceMode::Exclusive)
            .unwrap();
        assert_eq!(graph.authority[loan.value].opaque, FALSE);
        assert_eq!(graph.authority[loan.value].loans.len(), 1);
    });
}

#[test]
pub(crate) fn exclusive_restart_frontiers_reject_opaque_and_unrooted_demand() {
    for opaque in [false, true] {
        inspect(SOURCE, |graph, reach| {
            let value = graph
                .loans
                .iter()
                .find(|loan| loan.mode == ReferenceMode::Shared)
                .unwrap()
                .value;
            let value = if opaque {
                graph.nodes[0].opaque.push(value);
                value
            } else {
                graph.value(graph.values[value].origins.clone()).unwrap()
            };
            let target = graph
                .nodes
                .iter()
                .flat_map(|node| &node.next)
                .find(|edge| edge.reset)
                .unwrap()
                .target;
            graph.nodes[target].uses.push(value);
            let error = graph.exclusive_restart_frontiers(reach).unwrap_err();
            assert_eq!(error.code, "B001");
            assert!(error.message.contains("loan ancestry"));
        });
    }
}

#[test]
pub(crate) fn exclusive_restart_frontiers_bound_work_and_reject_invalid_transfers() {
    for budget in [false, true] {
        inspect(SOURCE, |graph, reach| {
            if budget {
                graph.guards.spend(usize::MAX);
            } else {
                graph.nodes[0]
                    .transfers
                    .push((graph.values.len(), 0, crate::flow::TRUE));
            }
            assert_eq!(
                graph.exclusive_restart_frontiers(reach).unwrap_err().code,
                "B001"
            );
        });
    }
}
