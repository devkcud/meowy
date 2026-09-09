use super::access::inspect;
use crate::loans::emission_init::Event;

#[test]
pub(crate) fn carried_lists_empty_values_still_require_whole_slot_initialization() {
    let source = "<R>:<{items<int32[0]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[];first=false;'loop.restart()}}};n:r.items.size()";
    for duplicate in [false, true] {
        inspect(source, |graph, _| {
            graph.emission_states().unwrap();
            assert!(graph.loans.is_empty());
            assert!(graph.nodes.iter().all(|node| node.uses.is_empty()));
            assert!(graph.nodes.iter().all(|node| {
                !node
                    .emissions
                    .iter()
                    .any(|event| matches!(event, Event::Acquire(_, _)))
            }));
            let node = graph
                .nodes
                .iter_mut()
                .find(|node| {
                    node.emissions
                        .iter()
                        .any(|event| matches!(event, Event::Emit(_)))
                })
                .unwrap();
            if duplicate {
                let event = node
                    .emissions
                    .iter()
                    .find(|event| matches!(event, Event::Emit(_)))
                    .unwrap()
                    .clone();
                node.emissions.push(event);
            } else {
                node.emissions
                    .retain(|event| !matches!(event, Event::Emit(_)));
            }
            let error = graph.emission_states().unwrap_err();
            assert_eq!(error.code, "B001");
            assert!(error.message.contains(if duplicate {
                "more than once"
            } else {
                "every completing path"
            }));
        });
    }
}
