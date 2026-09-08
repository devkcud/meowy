use crate::ast::Span;
use crate::borrow::published::{Snapshot, capture};
use crate::borrow::{Facts, Source, Step};
use crate::flow::{FALSE, Flow, TRUE};
use crate::hir::{Program, Type};

pub(crate) fn analyze(source: &str) -> (Facts, Flow) {
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
    let facts = crate::borrow::check(&program, &mut checker.flow, &checker.proofs)
        .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
    crate::loans::check(&program, &facts, &checker.proofs, &mut checker.flow)
        .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
    (facts, checker.flow)
}

pub(crate) fn last(facts: &Facts) -> &Snapshot {
    facts.published_inputs.values().next_back().unwrap()
}

#[test]
pub(crate) fn published_snapshots_outlive_the_alias_lexical_scope() {
    let (facts, _) = analyze("x:1;y:2;r:'out{{'out->p:=&x;p=&y};probe:{}};v:*r.p");
    let input = last(&facts);
    assert_ne!(input.entered, FALSE);
    assert_eq!(input.slots.len(), 1);
    let slot = input.slots.values().next().unwrap();
    assert_eq!(slot.state.origins.len(), 1);
    assert_eq!(
        slot.state.origins[0].source,
        Source::Local {
            id: 1,
            fields: Vec::new()
        }
    );
    assert!(slot.state.origins[0].component.is_empty());
}

#[test]
pub(crate) fn published_snapshots_use_backing_union_tags_and_keep_old_copies() {
    let (facts, mut flow) = analyze(
        "<R>:<{p<&int32><null><string>:=}>;x:1;y:2;r<R>:{->p<&int32><null>:=&x;old:p;p=&y;probe:{};copy:old};copy:r",
    );
    let slot = last(&facts).slots.values().next().unwrap();
    let Type::Union(members) = &slot.ty else {
        panic!("backing union")
    };
    assert_eq!(members.len(), 3);
    let member = members
        .iter()
        .position(|ty| matches!(ty, Type::Reference(_)))
        .unwrap();
    assert_eq!(slot.state.origins.len(), 1);
    assert_eq!(slot.state.origins[0].component, vec![Step::Variant(member)]);
    assert_eq!(
        slot.state.origins[0].source,
        Source::Local {
            id: 1,
            fields: Vec::new()
        }
    );
    let active = slot.state.member(&[], member, &mut flow);
    assert!(flow.implies(slot.state.present, active));
    assert!(
        facts
            .locals
            .values()
            .any(|state| state.origins.iter().any(|origin| {
                origin.source
                    == Source::Local {
                        id: 0,
                        fields: Vec::new(),
                    }
            }))
    );
}

#[test]
pub(crate) fn published_field_snapshots_keep_rhs_writes_when_the_outer_store_leaves() {
    let (facts, _) = analyze(
        "x:1;y:2;z:3;r:{->c:={->p:=&x;->q:=&y};'work{c.p={c.q=&z;'work.leave();->&y}};probe:{}};copy:r",
    );
    let slot = last(&facts).slots.values().next().unwrap();
    assert_eq!(slot.state.origins.len(), 2);
    for (path, id) in [(1, 0), (2, 2)] {
        assert!(slot.state.origins.iter().any(|origin| {
            origin.component == vec![Step::Slot(path)]
                && origin.source
                    == Source::Local {
                        id,
                        fields: Vec::new(),
                    }
        }));
    }
}

#[test]
pub(crate) fn published_snapshot_capture_preserves_conditional_initialization() {
    let (facts, mut flow) =
        analyze("x:1;flag:=true;q:=&x;r:'out{|flag|{'out->p:=&x};q=&x;probe:{}};copy:r");
    let input = last(&facts);
    let slot = input.slots.values().next().unwrap();
    assert_ne!(slot.state.present, FALSE);
    assert!(!flow.implies(input.entered, slot.state.present));
    let Type::Union(members) = &slot.ty else {
        panic!("optional backing")
    };
    let null = members.iter().position(|ty| ty == &Type::Null).unwrap();
    assert_eq!(slot.state.member(&[], null, &mut flow), FALSE);
}

#[test]
pub(crate) fn published_restart_inputs_preserve_public_bounds_and_owner_identity() {
    let (facts, _) = analyze(
        "first<&int32>:(p<&int32>,s<&string>){->p};x:1;s:\"s\";q:=&x;n:=0;r:{->p:=first(&x,&s);'inner{q=&x;n=n+1;|n<2|'inner.restart()}};v:*r.p",
    );
    let input = facts.published_restarts.values().next().unwrap();
    assert_eq!(facts.published_restarts.len(), 1);
    assert_eq!(input.slots.len(), 1);
    let (key, slot) = input.slots.iter().next().unwrap();
    assert_ne!(input.entered, FALSE);
    assert!(!slot.state.bounds.is_empty());
    assert!(slot.state.bounds.iter().any(|bound| {
        !slot
            .state
            .origins
            .iter()
            .any(|origin| origin.source == bound.source)
    }));
    assert!(
        facts
            .published_inputs
            .values()
            .any(|entry| entry.slots.contains_key(key))
    );
}

#[test]
pub(crate) fn published_restart_inputs_exclude_reset_targets_and_nested_results() {
    let (facts, _) = analyze(
        "x:1;q:=&x;n:=0;r:'outer{->p:=&x;'inner{->p:=&x;q=&x;n=n+1;|n<2|'inner.restart()}};v:*r.p",
    );
    assert_eq!(facts.published_restarts.len(), 1);
    assert_eq!(
        facts
            .published_restarts
            .values()
            .next()
            .unwrap()
            .slots
            .len(),
        1
    );
    let (facts, _) = analyze(
        "x:1;q:=&x;n:=0;r:'outer{->p:=&x;'inner{->p:=&x;q=&x;n=n+1;|n<2|'outer.restart()}};v:*r.p",
    );
    assert!(facts.published_restarts.is_empty());
}

#[test]
pub(crate) fn published_snapshot_capture_is_bounded_and_validates_reference_coverage() {
    let (facts, _) = analyze("x:1;y:2;r:{->p:=&x;p=&y;probe:{}};v:*r.p");
    let mut input = facts.published_inputs.into_values().next_back().unwrap();
    let owner = input.slots.keys().next().unwrap().0;
    let mut flow = Flow::default();
    flow.spend(usize::MAX);
    assert_eq!(
        capture(&input.slots, &[owner], TRUE, &mut flow, Span::default())
            .err()
            .unwrap()
            .code,
        "B001"
    );
    let (empty, _) = capture(
        &input.slots,
        &[owner],
        FALSE,
        &mut Flow::default(),
        Span::default(),
    )
    .unwrap();
    assert!(empty.slots.is_empty());
    input
        .slots
        .values_mut()
        .next()
        .unwrap()
        .state
        .origins
        .clear();
    assert_eq!(
        capture(
            &input.slots,
            &[owner],
            TRUE,
            &mut Flow::default(),
            Span::default()
        )
        .err()
        .unwrap()
        .code,
        "B001"
    );
}
