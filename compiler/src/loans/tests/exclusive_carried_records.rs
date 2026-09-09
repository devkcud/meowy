use super::access::inspect;
use crate::borrow::{Projection, Source};
use crate::flow::FALSE;
use crate::hir::ReferenceMode;
use crate::loans::emission_init::Event;
use crate::loans::storage::ScopeKind;

pub(crate) const SOURCE: &str = "<Inner>:<{n<int32>:=;other<int32>:=}>;<Row>:<{inner<Inner>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->inner:={->n:=7;->other:=1}};p:&!row.inner.n;q:&!row.inner.other;*p=8;*q=9;first=false;'loop.restart()}}}";

#[test]
pub(crate) fn exclusive_carried_record_sources_keep_paths_initialization_and_authority() {
    inspect(SOURCE, |graph, reach| {
        graph.emission_states().unwrap();
        graph.exclusive_restart_frontiers(reach).unwrap();
        graph.solve_authority(reach).unwrap();
        let loans = graph
            .loans
            .iter()
            .filter(|loan| loan.mode == ReferenceMode::Exclusive)
            .collect::<Vec<_>>();
        assert_eq!(loans.len(), 2);
        let mut paths = Vec::new();
        for loan in loans {
            let Source::Slot {
                target,
                root,
                view,
                fields,
            } = &graph.values[loan.value].origins[0].source
            else {
                panic!("projected slot");
            };
            let alias = &graph.proofs.aliases[view];
            let slot = &graph.proofs.carried[&(*target, Some(alias.field.clone()))];
            let crate::hir::Type::Record { fields: row, .. } = &slot.ty else {
                panic!("record slot");
            };
            let index = row.iter().position(|field| field.name == "inner").unwrap();
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0], Projection::Field(index));
            paths.push(fields.clone());
            assert_eq!(
                graph.scopes[graph.stores[root].scope.0].kind,
                ScopeKind::Block(*target)
            );
            assert_eq!(graph.authority[loan.value].opaque, FALSE);
            assert_eq!(graph.authority[loan.value].loans.len(), 1);
            let node = &graph.nodes[loan.node];
            assert!(node.uses.is_empty());
            assert!(node.emissions.iter().any(|event| matches!(event, Event::Acquire(key, _) if key == &(*target, Some(alias.field.clone())))));
        }
        assert_ne!(paths[0], paths[1]);
    });
}

#[test]
pub(crate) fn exclusive_carried_record_acquisition_requires_active_initialized_storage() {
    for inactive in [false, true] {
        inspect(SOURCE, |graph, _| {
            let loan = graph
                .loans
                .iter()
                .find(|loan| loan.mode == ReferenceMode::Exclusive)
                .unwrap();
            let node = loan.node;
            let event = graph.nodes[node]
                .emissions
                .iter()
                .find(|event| matches!(event, Event::Acquire(_, _)))
                .unwrap()
                .clone();
            if inactive {
                graph.nodes[0].emissions.push(event);
            } else {
                for node in &mut graph.nodes {
                    node.emissions
                        .retain(|event| !matches!(event, Event::Emit(_)));
                }
            }
            let error = graph.emission_states().unwrap_err();
            assert_eq!(error.code, "B001");
            assert!(
                error.message.contains("initialized before borrowing"),
                "{error:?}"
            );
        });
    }
}

#[test]
pub(crate) fn exclusive_carried_record_sources_reject_inexact_and_non_scalar_paths() {
    for change in 0..9 {
        inspect(SOURCE, |graph, reach| {
            let value = graph
                .loans
                .iter()
                .find(|loan| loan.mode == ReferenceMode::Exclusive)
                .unwrap()
                .value;
            let origin = &mut graph.values[value].origins[0];
            let Source::Slot {
                root,
                view,
                target,
                fields,
            } = &mut origin.source
            else {
                panic!("slot source");
            };
            match change {
                0 => fields.clear(),
                1 => {
                    fields.pop();
                }
                2 => fields.push(Projection::Field(0)),
                3 => fields[0] = Projection::Field(usize::MAX),
                4 => fields[0] = Projection::Element,
                5 => *root = usize::MAX,
                6 => *target = usize::MAX,
                7 => *view = usize::MAX,
                8 => origin.component.push(crate::borrow::Step::Deref),
                _ => unreachable!(),
            }
            assert_eq!(
                graph.exclusive_restart_frontiers(reach).unwrap_err().code,
                "B001"
            );
        });
    }
}
