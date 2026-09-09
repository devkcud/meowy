use super::access::inspect;
use crate::borrow::{Projection, Source};
use crate::hir::ReferenceMode;
use crate::loans::emission_init::Event;
use crate::loans::storage::ScopeKind;

pub(crate) const SOURCE: &str = "<Row>:<{items<int32[2]>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:[7]};p:row.&items;q:&(p[1]);s:&(row.items[1]);same:q==s;first=false;'loop.restart()}}}";

#[test]
pub(crate) fn carried_list_borrows_keep_containing_slots_and_element_parent_identity() {
    inspect(SOURCE, |graph, _| {
        graph.emission_states().unwrap();
        let mut roots = 0;
        let mut children = 0;
        for loan in &graph.loans {
            assert_eq!(loan.mode, ReferenceMode::Shared);
            let Source::Slot {
                target,
                root,
                view,
                fields,
            } = &graph.values[loan.value].origins[0].source
            else {
                panic!("carried slot source");
            };
            let alias = &graph.proofs.aliases[view];
            assert_eq!(alias.target, *target);
            assert_eq!(alias.root, *root);
            assert_eq!(
                graph.scopes[graph.stores[root].scope.0].kind,
                ScopeKind::Block(*target)
            );
            if let Some(parent) = loan.parent {
                children += 1;
                assert_eq!(fields, &[Projection::Field(0), Projection::Element]);
                assert!(graph.nodes[loan.node].uses.contains(&parent));
                assert!(graph.values[parent].origins.iter().any(|origin| {
                    origin.source.project(&[Projection::Element])
                        == graph.values[loan.value].origins[0].source
                }));
            } else {
                roots += 1;
                assert_eq!(fields, &[Projection::Field(0)]);
                let node = &graph.nodes[loan.node];
                assert!(node.uses.is_empty());
                assert!(node.emissions.iter().any(|event| matches!(event, Event::Acquire(key, _) if key == &(*target, Some(alias.field.clone())))));
            }
        }
        assert_eq!((roots, children), (2, 2));
    });
}

#[test]
pub(crate) fn carried_list_borrows_require_initialized_active_storage_at_acquisition() {
    for change in 0..3 {
        inspect(SOURCE, |graph, _| {
            graph.emission_states().unwrap();
            let event = graph
                .nodes
                .iter()
                .flat_map(|node| &node.emissions)
                .find(|event| matches!(event, Event::Acquire(_, _)))
                .unwrap()
                .clone();
            let Event::Acquire((owner, _), span) = &event else {
                unreachable!()
            };
            let span = *span;
            match change {
                0 => {
                    for node in &mut graph.nodes {
                        node.emissions
                            .retain(|event| !matches!(event, Event::Emit(_)));
                    }
                }
                1 => graph.nodes[0].emissions.push(event),
                _ => {
                    let node = graph
                        .nodes
                        .iter_mut()
                        .find(|node| {
                            node.emissions.iter().any(
                                |event| matches!(event, Event::Complete(target) if target == owner),
                            )
                        })
                        .unwrap();
                    node.emissions.push(event);
                }
            }
            let error = graph.emission_states().unwrap_err();
            assert_eq!(error.code, "B001");
            assert_eq!(error.span, span);
            assert!(error.message.contains("initialized before borrowing"));
        });
    }
}
