use super::access::inspect;
use crate::borrow::{Projection, Source};
use crate::loans::access::{Kind, Target};
use crate::loans::emission_init::Event;

pub(crate) const SOURCE: &str = "<Row>:<{items<int32[2][2]>:=;side<int32>:=}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:=[[7]];->side:=1};row.items[1][1]=9;first=false;'loop.restart()}}}";

#[test]
pub(crate) fn carried_writes_keep_canonical_reservations_and_final_store_regions() {
    inspect(SOURCE, |graph, reach| {
        graph.emission_states().unwrap();
        graph.solve_authority(reach).unwrap();
        assert!(graph.loans.is_empty());
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
        assert_eq!(events[0].1, events[1].1);
        let write = events[1].0;
        let access = graph.nodes[write].access.as_ref().unwrap();
        assert_eq!(access.kind, Kind::Write);
        let Target::Storage { place, view } = &access.target else {
            panic!("store target")
        };
        let alias = &graph.proofs.aliases[view];
        assert_eq!(place.root, alias.root);
        assert_eq!(place.fields, vec![0]);
        assert_eq!(events[0].1, &(alias.target, Some(alias.field.clone())));
        assert_eq!(graph.nodes[write].uses.len(), 1);
        let reservation = graph.nodes[write].uses[0];
        assert!(
            matches!(&graph.values[reservation].origins[0].source, Source::Slot {target, root, view: id, fields} if *target == alias.target && *root == alias.root && id == view && fields == &[Projection::Field(0)])
        );
        assert!(graph.authority[reservation].loans.is_empty());
        let capture = graph
            .nodes
            .iter()
            .position(|node| node.defs.contains(&reservation))
            .unwrap();
        assert!(events[0].0 < capture);
        assert!(graph.nodes[events[0].0].uses.is_empty());
        assert_eq!(
            graph
                .nodes
                .iter()
                .filter(|node| node.uses.contains(&reservation))
                .count(),
            3
        );
        let live = graph.liveness(reach).unwrap();
        assert!(live[write].contains_key(&reservation));
        assert!(
            !graph
                .outgoing(write, &live)
                .unwrap()
                .contains_key(&reservation)
        );
    });
}

#[test]
pub(crate) fn carried_writes_require_initialized_active_storage_at_capture_and_store() {
    for index in 0..2 {
        for change in 0..3 {
            inspect(SOURCE, |graph, _| {
                let event = graph
                    .nodes
                    .iter()
                    .flat_map(|node| &node.emissions)
                    .filter(|event| matches!(event, Event::Acquire(_, _)))
                    .nth(index)
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
pub(crate) fn carried_writes_cancel_without_erasing_completed_phase_demand() {
    for (statement, phases) in [
        ("row.items[{'cancel.leave()}][1]=9", 0),
        ("row.items[1][{'cancel.leave()}]=9", 1),
        ("row.items[1][1]={'cancel.leave()}", 2),
    ] {
        let source = SOURCE.replace("row.items[1][1]=9", &format!("'cancel{{{statement}}}"));
        inspect(&source, |graph, reach| {
            graph.emission_states().unwrap();
            graph.solve_authority(reach).unwrap();
            assert!(graph.loans.is_empty());
            let events = graph
                .nodes
                .iter()
                .flat_map(|node| &node.emissions)
                .filter_map(|event| match event {
                    Event::Acquire(_, span) => Some(*span),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(events.len(), 1);
            let reservation = graph.values.iter().position(|value| value.origins.iter().any(|origin| matches!(&origin.source, Source::Slot { fields, .. } if fields == &[Projection::Field(0)]))).unwrap();
            assert_eq!(
                graph
                    .nodes
                    .iter()
                    .filter(|node| node.uses.contains(&reservation))
                    .count(),
                phases
            );
            for node in &mut graph.nodes {
                node.emissions
                    .retain(|event| !matches!(event, Event::Emit(_)));
            }
            let error = graph.emission_states().unwrap_err();
            assert_eq!(error.code, "B001");
            assert_eq!(error.span, events[0]);
            assert!(error.message.contains("initialized before borrowing"));
        });
    }
}

#[test]
pub(crate) fn carried_writes_leave_static_field_paths_without_index_reservations() {
    let source = SOURCE.replace("row.items[1][1]=9", "row.side=9");
    inspect(&source, |graph, _| {
        graph.emission_states().unwrap();
        assert!(graph.nodes.iter().all(|node| {
            !node
                .emissions
                .iter()
                .any(|event| matches!(event, Event::Acquire(_, _)))
        }));
        assert!(graph.values.is_empty());
        let access = graph
            .nodes
            .iter()
            .filter_map(|node| node.access.as_ref())
            .find(|access| {
                matches!(&access.target, Target::Storage {place, ..} if place.fields == vec![1])
                    && access.kind == Kind::Write
            })
            .unwrap();
        assert!(access.path.is_empty());
    });
}
