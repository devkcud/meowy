use super::{accepts, rejects};
use crate::ast::Span;
use crate::borrow::published::unchanged;
use crate::flow::{FALSE, Flow, TRUE};

#[test]
pub(crate) fn fixed_published_results_allow_writes_before_and_after_inner_restarts() {
    for source in [
        "x:=1;y:2;n:=0;r:{->p:=&x;p=&y;'loop{x=3;n=n+1;|n<2|'loop.restart()}};v:*r.p",
        "x:=1;y:2;n:=0;r:{->p:=&x;'loop{x=3;n=n+1;|n<2|'loop.restart()};p=&y};v:*r.p",
        "x:1;y:2;n:=0;r:'out{{'out->p:=&x;p=&y};'loop{n=n+1;|n<2|'loop.restart()}};v:*r.p",
        "x:1;y:2;n:=0;r:{'loop{n=n+1;|n<2|'loop.restart()};->p:=&x;p=&y};v:*r.p",
    ] {
        accepts(source);
    }
}

#[test]
pub(crate) fn fixed_published_headers_keep_nested_fields_and_both_union_domains() {
    for source in [
        "x:1;y:2;n:=0;r:{->c:={->p:=&x;->q:=&y};c.p=&y;'loop{n=n+1;|n<2|'loop.restart()};c.q=&x};copy:r",
        "<R>:<{p<&int32><null><string>:=}>;x:1;y:2;n:=0;r<R>:{->p<&int32><null>:=&x;p=&y;'loop{n=n+1;|n<2|'loop.restart()};p=null};copy:r",
        "x:1;y:2;flag:=true;n:=0;r:'out{|flag|{'out->p:=&x;p=&y};'loop{n=n+1;|n<2|'loop.restart()}};copy:r",
        "x:1;y:2;n:=0;m:=0;r:{->p:=&x;p=&y;'outer{m=0;'inner{m=m+1;|m<2|'inner.restart()};n=n+1;|n<2|'outer.restart()}};v:*r.p",
    ] {
        accepts(source);
    }
}

#[test]
pub(crate) fn fixed_published_headers_preserve_rhs_restarts_and_leave_effects() {
    accepts("x:1;y:2;n:=0;r:{->p:=&x;p={'loop{v:*p;n=n+1;|n<2|'loop.restart()};->&y}};v:*r.p");
    accepts(
        "x:1;y:2;n:=0;r:'out{->p:=&x;p={p=&y;'loop{v:*p;n=n+1;|n<2|'loop.restart()};'out.leave();->&x}};v:*r.p",
    );
    accepts("x:1;y:2;n:=0;r:{->p:=&x;p={'loop{p=&y;n=n+1;|n<2|'loop.restart()};->&x}}");
}

#[test]
pub(crate) fn fixed_published_headers_preserve_owner_cell_and_old_copy_loans() {
    for source in [
        "x:1;y:=2;n:=0;r:{->p:=&x;p=&y;'loop{y=3;n=n+1;|n<2|'loop.restart()}};v:*r.p",
        "x:=1;y:2;n:=0;r:{->p:=&x;old:p;p=&y;'loop{x=3;n=n+1;|n<2|'loop.restart()};v:*old}",
        "x:1;y:2;n:=0;r:{->p:=&x;cell:&p;'loop{n=n+1;|n<2|'loop.restart()};p=&y;v:**cell}",
        "first<&int32>:(p<&int32>,s<&string>){->p};x:1;s:=\"a\";n:=0;r:{->p:=&x;p=first(&x,&s);'loop{s=\"b\";n=n+1;|n<2|'loop.restart()}};v:*r.p",
    ] {
        rejects(source, "E302");
    }
    rejects(
        "x:1;n:=0;r:{->p:=&x;local:2;p=&local;'loop{n=n+1;|n<2|'loop.restart()}};v:*r.p",
        "E303",
    );
}

#[test]
pub(crate) fn fixed_published_planning_uses_the_write_scope_and_validated_ancestry() {
    use crate::borrow::mutable::alias_restarts;
    use std::collections::{BTreeMap, BTreeSet};

    let parents = BTreeMap::from([
        (0, None),
        (1, Some(0)),
        (2, Some(1)),
        (3, Some(2)),
        (4, Some(1)),
    ]);
    let restarts = BTreeSet::from([2]);
    for owner in [1, 4] {
        let writes = BTreeMap::from([((1, owner, 7), Span::default())]);
        let fixed = alias_restarts(&parents, &writes, &restarts, &mut Flow::default()).unwrap();
        assert_eq!(fixed.fixed, BTreeMap::from([(2, BTreeSet::from([1]))]));
    }
    let writes = BTreeMap::from([((1, 99, 7), Span::default())]);
    assert_eq!(
        alias_restarts(&parents, &writes, &restarts, &mut Flow::default())
            .unwrap_err()
            .code,
        "B001"
    );
    let writes = BTreeMap::from([((2, 3, 7), Span::default())]);
    assert!(
        alias_restarts(&parents, &writes, &restarts, &mut Flow::default())
            .unwrap()
            .changing
            .is_empty()
    );
}

#[test]
pub(crate) fn fixed_published_snapshots_detect_changed_sources_roles_and_missing_slots() {
    let (facts, mut flow) = super::published::analyze(
        "x:1;y:2;n:=0;r:{->p:=&x;p=&y;'loop{n=n+1;|n<2|'loop.restart()}};v:*r.p",
    );
    let (id, targets) = facts.fixed_published.iter().next().unwrap();
    let initial = &facts.published_inputs[id];
    let mut current = facts.published_restarts.into_values().next().unwrap();
    assert_eq!(
        unchanged(initial, &current, targets, &mut flow, Span::default()).unwrap(),
        FALSE
    );
    let slot = current.slots.values_mut().next().unwrap();
    let origin = slot.state.origins[0].clone();
    slot.state.bounds.push(origin);
    assert_ne!(
        unchanged(initial, &current, targets, &mut flow, Span::default()).unwrap(),
        FALSE
    );
    let slot = current.slots.values_mut().next().unwrap();
    slot.state.bounds.clear();
    slot.state.origins[0].source = crate::borrow::Source::Local {
        id: 0,
        fields: Vec::new(),
    };
    assert_ne!(
        unchanged(initial, &current, targets, &mut flow, Span::default()).unwrap(),
        FALSE
    );
    current.slots.clear();
    assert_ne!(
        unchanged(initial, &current, targets, &mut flow, Span::default()).unwrap(),
        FALSE
    );
    current.entered = FALSE;
    assert_eq!(
        unchanged(initial, &current, targets, &mut flow, Span::default()).unwrap(),
        FALSE
    );
    flow.spend(usize::MAX);
    assert_eq!(
        unchanged(initial, &current, targets, &mut flow, Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
}

#[test]
pub(crate) fn fixed_published_loan_proof_requires_snapshots_without_adding_reads() {
    use crate::borrow::published::{Slot, Slots, Snapshot};
    use crate::borrow::{Facts, Origin, Proofs, Source, State};
    use crate::hir::{Block, Field, Program, Type};
    use crate::loans::{Graph, Scope};
    use std::collections::{BTreeMap, BTreeSet};

    let scalar = Type::Int {
        bits: 32,
        signed: true,
    };
    let reference = Type::Reference(Box::new(scalar.clone()));
    let ty = Type::Record {
        primary: Box::new(Type::Null),
        fields: vec![Field {
            name: "p".into(),
            ty: reference.clone(),
            mutable: true,
        }],
    };
    let program = Program {
        body: Block {
            id: 0,
            stmts: Vec::new(),
            ty: ty.clone(),
        },
        functions: Vec::new(),
        locals: vec![scalar],
    };
    let input = Snapshot {
        slots: Slots::from([(
            (0, 1),
            Slot {
                ty: reference,
                state: State {
                    origins: vec![Origin {
                        component: Vec::new(),
                        source: Source::Local {
                            id: 0,
                            fields: Vec::new(),
                        },
                        guard: TRUE,
                    }],
                    ..State::default()
                },
            },
        )]),
        entered: TRUE,
    };
    let facts = Facts {
        fixed_published: BTreeMap::from([(1, BTreeSet::from([0]))]),
        published_inputs: BTreeMap::from([(1, input)]),
        ..Facts::default()
    };
    let proofs = Proofs::default();
    let mut flow = Flow::default();
    let mut graph = Graph::new(&program, &facts, &proofs, &mut flow);
    let life = graph
        .new_scope(crate::loans::storage::ScopeKind::Block(0))
        .unwrap();
    graph.blocks.insert(
        0,
        Scope {
            life,
            start: 0,
            end: 0,
            result: BTreeMap::new(),
            ty,
            incoming: BTreeMap::new(),
            leaves: Vec::new(),
            restarted: false,
        },
    );
    let before = (graph.nodes.len(), graph.values.len());
    assert_eq!(
        graph
            .fixed_published(1, Some(&facts.published_inputs[&1]))
            .unwrap(),
        FALSE
    );
    assert_eq!(graph.fixed_published(1, None).unwrap(), TRUE);
    assert_eq!((graph.nodes.len(), graph.values.len()), before);
    graph.blocks.clear();
    assert_eq!(
        graph
            .fixed_published(1, Some(&facts.published_inputs[&1]))
            .unwrap(),
        TRUE
    );
}
