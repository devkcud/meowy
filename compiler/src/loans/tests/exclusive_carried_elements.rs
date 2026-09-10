use super::access::inspect;
use crate::borrow::{Projection, Source};
use crate::flow::FALSE;
use crate::hir::ReferenceMode;
use crate::loans::emission_init::Event;

pub(crate) const SOURCE: &str = "<Row>:<{xs<int32[2]>:=;zs<int32[2]>}>;<R>:<{rows<Row[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->rows:[{->xs:=[1];->zs:[2]}];p:&!(rows[1].xs[1]);*p=2;first=false;'loop.restart()}}}";

#[test]
pub(crate) fn exclusive_carried_elements_keep_slot_paths_and_reservation_demand() {
    inspect(SOURCE, |graph, reach| {
        graph.emission_states().unwrap();
        graph.solve_authority(reach).unwrap();
        let loan = graph
            .loans
            .iter()
            .find(|loan| loan.mode == ReferenceMode::Exclusive)
            .unwrap();
        let node = loan.node;
        let Source::Slot {
            target,
            root,
            view,
            fields,
        } = &graph.values[loan.value].origins[0].source
        else {
            panic!("carried element source");
        };
        let alias = &graph.proofs.aliases[view];
        assert_eq!((*target, *root), (alias.target, alias.root));
        assert_eq!(
            fields,
            &[
                Projection::Element,
                Projection::Field(0),
                Projection::Element
            ]
        );
        assert!(loan.parent.is_none());
        assert_eq!(graph.authority[loan.value].loans.len(), 1);
        assert_eq!(graph.authority[loan.value].opaque, FALSE);
        let reservations = graph.nodes[node].uses.clone();
        assert_eq!(reservations.len(), 2);
        for (id, fields) in reservations
            .iter()
            .zip([vec![], vec![Projection::Element, Projection::Field(0)]])
        {
            assert!(
                matches!(&graph.values[*id].origins[0].source, Source::Slot {fields: path, ..} if *path == fields)
            );
            assert!(graph.authority[*id].loans.is_empty());
        }
        let events = graph
            .nodes
            .iter()
            .enumerate()
            .flat_map(|(id, node)| {
                node.emissions.iter().filter_map(move |event| match event {
                    Event::Acquire(key, _) => Some((id, key)),
                    _ => None,
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].1, &(*target, Some(alias.field.clone())));
        assert_eq!(events[0].1, events[1].1);
        assert_eq!(events[1].0, node);
        let reserve = graph
            .nodes
            .iter()
            .position(|node| node.defs.contains(&reservations[0]))
            .unwrap();
        assert!(events[0].0 < reserve);
        assert!(graph.nodes[events[0].0].uses.is_empty());
        let live = graph.liveness(reach).unwrap();
        let out = graph.outgoing(node, &live).unwrap();
        for id in reservations {
            assert!(live[node].contains_key(&id));
            assert!(!out.contains_key(&id));
        }
    });
}

#[test]
pub(crate) fn exclusive_carried_elements_require_initialized_capture_and_acquisition() {
    for event in 0..2 {
        for change in 0..3 {
            inspect(SOURCE, |graph, _| {
                let event = graph
                    .nodes
                    .iter()
                    .flat_map(|node| &node.emissions)
                    .filter(|event| matches!(event, Event::Acquire(_, _)))
                    .nth(event)
                    .unwrap()
                    .clone();
                let Event::Acquire((owner, _), span) = &event else {
                    unreachable!()
                };
                let span = *span;
                if change == 0 {
                    for node in &mut graph.nodes {
                        node.emissions
                            .retain(|event| !matches!(event, Event::Emit(_)));
                    }
                } else if change == 1 {
                    graph.nodes[0].emissions.push(event);
                } else {
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
                let error = graph.emission_states().unwrap_err();
                assert_eq!(error.code, "B001");
                assert_eq!(error.span, span);
                assert!(error.message.contains("initialized before borrowing"));
            });
        }
    }
}

#[test]
pub(crate) fn exclusive_carried_elements_cancel_without_losing_completed_index_demand() {
    for (path, completed) in [
        ("[{'cancel.leave()}].xs[1]", false),
        ("[1].xs[{'cancel.leave()}]", true),
    ] {
        let source = SOURCE.replace(
            "p:&!(rows[1].xs[1]);*p=2",
            &format!("'cancel{{p:&!(rows{path})}}"),
        );
        inspect(&source, |graph, reach| {
            graph.emission_states().unwrap();
            graph.solve_authority(reach).unwrap();
            assert!(graph.loans.is_empty());
            let count = graph
                .nodes
                .iter()
                .flat_map(|node| &node.emissions)
                .filter(|event| matches!(event, Event::Acquire(_, _)))
                .count();
            assert_eq!(count, 1);
            let live = graph.liveness(reach).unwrap();
            let demanded = graph
                .values
                .iter()
                .enumerate()
                .filter(|(id, _)| live.iter().any(|state| state.contains_key(id)))
                .collect::<Vec<_>>();
            assert_eq!(demanded.len(), usize::from(completed));
            for (_, value) in demanded {
                assert!(
                    matches!(&value.origins[0].source, Source::Slot { fields, .. } if fields.is_empty())
                );
            }
        });
    }
}

#[test]
pub(crate) fn exclusive_carried_elements_reject_inexact_non_scalar_and_immutable_sources() {
    for change in 0..8 {
        inspect(SOURCE, |graph, _| {
            let loan = graph
                .loans
                .iter()
                .find(|loan| loan.mode == ReferenceMode::Exclusive)
                .unwrap();
            let value = loan.value;
            let span = graph.nodes[loan.node].access.as_ref().unwrap().span;
            let origin = &mut graph.values[value].origins[0];
            let Source::Slot {
                root,
                view,
                target,
                fields,
            } = &mut origin.source
            else {
                unreachable!()
            };
            match change {
                0 => fields.clear(),
                1 => {
                    fields.pop();
                }
                2 => fields[0] = Projection::Field(0),
                3 => fields[1] = Projection::Element,
                4 => *root = usize::MAX,
                5 => *target = usize::MAX,
                6 => *view = usize::MAX,
                7 => fields[1] = Projection::Field(1),
                _ => unreachable!(),
            }
            assert_eq!(
                graph
                    .exclusive_restart_source(value, span)
                    .unwrap_err()
                    .code,
                "B001"
            );
        });
    }
    let source = SOURCE.replace("[1].xs[1]", "[1]");
    assert_eq!(crate::compile(&source).unwrap_err()[0].code, "B001");
}

#[test]
pub(crate) fn exclusive_carried_elements_keep_reborrow_parent_identity() {
    let source = SOURCE.replace("*p=2", "s:&*p;v:*s;q:&!*p;*q=2;*p=3");
    inspect(&source, |graph, reach| {
        graph.emission_states().unwrap();
        graph.solve_authority(reach).unwrap();
        assert_eq!(graph.loans.len(), 3);
        let root = graph
            .loans
            .iter()
            .find(|loan| loan.parent.is_none())
            .unwrap();
        let source = &graph.values[root.value].origins[0].source;
        for loan in graph.loans.iter().filter(|loan| loan.parent.is_some()) {
            let parent = loan.parent.unwrap();
            assert!(graph.nodes[loan.node].uses.contains(&parent));
            assert_eq!(&graph.values[parent].origins[0].source, source);
            assert_eq!(&graph.values[loan.value].origins[0].source, source);
        }
    });
}

#[test]
pub(crate) fn exclusive_carried_elements_require_initialization_even_when_cancelled() {
    let source = SOURCE.replace(
        "p:&!(rows[1].xs[1]);*p=2",
        "'cancel{p:&!(rows[{'cancel.leave()}].xs[1])}",
    );
    inspect(&source, |graph, _| {
        let span = graph
            .nodes
            .iter()
            .flat_map(|node| &node.emissions)
            .find_map(|event| match event {
                Event::Acquire(_, span) => Some(*span),
                _ => None,
            })
            .unwrap();
        for node in &mut graph.nodes {
            node.emissions
                .retain(|event| !matches!(event, Event::Emit(_)));
        }
        let error = graph.emission_states().unwrap_err();
        assert_eq!(error.code, "B001");
        assert_eq!(error.span, span);
        assert!(error.message.contains("initialized before borrowing"));
    });
}
