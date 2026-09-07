use super::access::inspect_body;
use crate::borrow::Source;
use crate::flow::FALSE;
use crate::loans::storage::ScopeKind;

#[test]
pub(crate) fn guarded_alias_views_share_target_storage_and_keep_distinct_loans() {
    inspect_body(
        "f<null>:(flag<boolean>){r:'out{|flag|{'out->n:=1;p:&!n;v:*p};|!flag|{'out->n:=2;p:&!n;v:*p}}}",
        Some(0),
        |graph, reach| {
            graph.solve_authority(reach).unwrap();
            assert_eq!(graph.loans.len(), 2);
            let first = &graph.loans[0];
            let second = &graph.loans[1];
            let Source::Slot {
                target,
                root,
                view: a,
                ..
            } = &graph.values[first.value].origins[0].source
            else {
                panic!("slot source");
            };
            let Source::Slot {
                target: other,
                root: same,
                view: b,
                ..
            } = &graph.values[second.value].origins[0].source
            else {
                panic!("slot source");
            };
            assert_eq!((target, root), (other, same));
            assert_ne!(a, b);
            assert!(graph.proofs.aliases[a].exclusive.is_some());
            assert!(graph.proofs.aliases[b].exclusive.is_some());
            let scope = graph.stores[root].scope;
            assert_eq!(graph.scopes[scope.0].kind, ScopeKind::Block(*target));
            assert!(!graph.guards.overlap(reach[first.node], reach[second.node]));
            assert_ne!(reach[first.node], FALSE);
            assert_ne!(reach[second.node], FALSE);
        },
    );
}
