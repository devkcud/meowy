use super::{accepts, rejects};

#[test]
pub(crate) fn changing_published_assignments_keep_current_sources_and_old_copies() {
    accepts("x:=1;y:2;n:=0;r:{->p:=&x;'loop{p=&y;n=n+1;|n<2|'loop.restart()}};x=3;v:*r.p");
    accepts("x:1;y:2;n:=0;r:{->p:=&x;old:p;'loop{p=&y;n=n+1;|n<2|'loop.restart()};v:*old};v:*r.p");
    rejects(
        "x:=1;y:2;n:=0;r:{->p:=&x;old:p;'loop{p=&y;n=n+1;|n<2|'loop.restart()};x=3;v:*old}",
        "E302",
    );
    rejects(
        "x:1;y:=2;n:=0;r:{->p:=&x;'loop{p=&y;n=n+1;|n<2|'loop.restart()}};y=3;v:*r.p",
        "E302",
    );
}

#[test]
pub(crate) fn changing_published_unwritten_final_iterations_retain_backedge_sources() {
    accepts("x:1;y:2;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{p=&y;'loop.restart()}}};v:*r.p");
    rejects(
        "x:1;y:=2;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{p=&y;'loop.restart()}}};y=3;v:*r.p",
        "E302",
    );
    accepts(
        "x:1;y:2;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{p=&y;'loop.restart()};'loop.leave()}};v:*r.p",
    );
}

#[test]
pub(crate) fn changing_published_nullable_and_wider_views_keep_both_tag_domains() {
    accepts(
        "<R>:<{p<&int32><null><string>:=}>;x:=1;n:=0;r<R>:{->p<&int32><null>:=null;'loop{p=&x;n=n+1;|n<2|'loop.restart();p=null}};x=2;copy:r",
    );
    rejects(
        "<R>:<{p<&int32><null><string>:=}>;x:=1;n:=0;r<R>:{->p<&int32><null>:=null;'loop{n=n+1;|n<2|{p=&x;'loop.restart()}}};x=2;copy:r",
        "E302",
    );
    accepts(
        "x:1;y:2;flag:=true;n:=0;r:'out{|flag|{'out->p:=&x;'loop{p=&y;n=n+1;|n<2|'loop.restart()}}};copy:r",
    );
    accepts(
        "<A>:<{p<&int32><null>}>;<B>:<{n<int32>}>;x:1;n:=0;r:{->c<A><B>:={->n:0};'loop{c={->p:&x};n=n+1;|n<2|'loop.restart()}};copy:r",
    );
}

#[test]
pub(crate) fn changing_published_fields_preserve_siblings_and_rhs_effects() {
    accepts(
        "x:1;y:2;n:=0;r:{->c:={->p:=&x;->q:=&y;->n:=0};'loop{c.p={c.q=&x;->&y};c.n=c.n+1;n=n+1;|n<2|'loop.restart()}};copy:r",
    );
    accepts(
        "x:=1;y:2;n:=0;r:{->c:={->p:=&x;->q:=&y};'loop{c.p=&y;c.q=&y;n=n+1;|n<2|'loop.restart()}};x=3;copy:r",
    );
    rejects(
        "x:1;y:=2;n:=0;r:{->c:={->p:=&x;->q:=&y};'loop{c.p=&y;n=n+1;|n<2|'loop.restart()}};y=3;copy:r",
        "E302",
    );
}

#[test]
pub(crate) fn changing_published_restarts_keep_the_original_binding_guard() {
    accepts(
        "make:(flag<boolean>,x<&int32>,y<&int32>)'out{|flag|{'out->p<&int32><null>:=null;n:=0;'loop{p=x;n=n+1;|n<2|'loop.restart();p=y}};|!flag|{'out->p:=\"none\"}};x:7;y:9;r:make(true,&x,&y);copy:r",
    );
    accepts(
        "x:1;y:2;flag:=true;n:=0;r:'out{|flag|{'out->p<&int32><null>:=null;'loop{flag=false;p=&y;n=n+1;|n<2|'loop.restart()}}};copy:r",
    );
}

#[test]
pub(crate) fn changing_published_leave_refreshes_precede_environment_restoration() {
    accepts(
        "x:1;y:2;n:=0;r:'out{->p:=&x;'loop{p={p=&y;n=n+1;|n<2|'loop.restart();'out.leave();->&x}}};v:*r.p",
    );
    rejects(
        "x:1;y:=2;n:=0;r:'out{->p:=&x;'loop{p={p=&y;n=n+1;|n<2|'loop.restart();'out.leave();->&x}}};y=3;v:*r.p",
        "E302",
    );
    accepts(
        "x:1;y:2;n:=0;r:{->p:=&x;'middle{'loop{n=n+1;|n<2|{p=&y;'loop.restart()};'middle.leave()}}};v:*r.p",
    );
    rejects(
        "x:1;y:=2;n:=0;r:{->p:=&x;'middle{'loop{n=n+1;|n<2|{p=&y;'loop.restart()};'middle.leave()}}};y=3;v:*r.p",
        "E302",
    );
}

#[test]
pub(crate) fn changing_published_nested_loops_and_transitive_bounds_preserve_versions() {
    accepts(
        "x:=1;y:=2;z:3;n:=0;m:=0;r:{->p:=&x;'outer{m=0;'inner{p=&y;m=m+1;|m<2|'inner.restart()};p=&z;n=n+1;|n<2|'outer.restart()}};x=4;y=5;v:*r.p",
    );
    accepts(
        "x:=1;y:2;a:&x;b:&y;n:=0;r:{->p:=&a;'loop{p=&b;n=n+1;|n<2|'loop.restart()}};x=3;v:**r.p",
    );
    rejects(
        "first<&int32>:(p<&int32>,s<&string>){->p};x:1;s:=\"a\";n:=0;r:{->p:=&x;'loop{p=first(&x,&s);n=n+1;|n<2|'loop.restart()}};s=\"b\";v:*r.p",
        "E302",
    );
}

#[test]
pub(crate) fn changing_published_expired_headers_require_repair_before_retention() {
    rejects(
        "x:1;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{local:2;p=&local;'loop.restart()}}};v:*r.p",
        "E303",
    );
    accepts(
        "x:1;n:=0;r:{->p:=&x;'loop{p=&x;n=n+1;|n<2|{local:2;p=&local;'loop.restart()}}};v:*r.p",
    );
    rejects(
        "x:1;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{p=&2;'loop.restart()}}};v:*r.p",
        "E303",
    );
    rejects(
        "first<&int32>:(p<&int32>,s<&string>){->p};x:1;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{p=first(&x,&\"short\");'loop.restart()}}};v:*r.p",
        "E303",
    );
}

#[test]
pub(crate) fn changing_published_requires_safe_initialization_and_preserves_cell_loans() {
    accepts("x:1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y}}");
    rejects(
        "x:1;y:2;n:=0;r:{->p:=&x;cell:&p;'loop{p=&y;n=n+1;|n<2|'loop.restart()};v:**cell}",
        "E302",
    );
    rejects(
        "<R>:<{p<&int32><null><string>:=}>;x:1;n:=0;r<R>:{->p<&int32><null>:=null;cell:&p;'loop{p=&x;n=n+1;|n<2|'loop.restart()}}",
        "B001",
    );
}

#[test]
pub(crate) fn changing_published_mixed_allocator_bounds_expire_and_can_be_repaired() {
    let prefix = "m:@\"memory\";f<m.Allocator>:(p<&int32>){->m.heap};x:1;n:=0;";
    accepts(&format!(
        "{prefix}r:{{->c:={{->p<&int32><null>:=null;->h:=m.heap}};'loop{{c.h=f(&x);n=n+1;|n<2|'loop.restart()}}}};copy:r"
    ));
    rejects(
        &format!(
            "{prefix}r:{{->c:={{->p<&int32><null>:=null;->h:=m.heap}};'loop{{n=n+1;|n<2|{{c.h=f(&1);'loop.restart()}}}}}};copy:r"
        ),
        "E303",
    );
    accepts(&format!(
        "{prefix}r:{{->c:={{->p<&int32><null>:=null;->h:=m.heap}};'loop{{c.h=m.heap;n=n+1;|n<2|{{c.h=f(&1);'loop.restart()}}}}}};copy:r"
    ));
}

#[test]
pub(crate) fn changing_published_plan_keeps_alias_identity_and_exit_coverage() {
    use crate::ast::Span;
    use crate::borrow::mutable::alias_restarts;
    use crate::flow::Flow;
    use std::collections::{BTreeMap, BTreeSet};

    let parents = BTreeMap::from([
        (0, None),
        (1, Some(0)),
        (2, Some(1)),
        (3, Some(2)),
        (4, Some(3)),
    ]);
    let writes = BTreeMap::from([((1, 1, 7), Span::default()), ((1, 4, 7), Span::default())]);
    let plan = alias_restarts(
        &parents,
        &writes,
        &BTreeSet::from([3]),
        &mut Flow::default(),
    )
    .unwrap();
    assert!(plan.fixed.is_empty());
    assert_eq!(plan.changing, BTreeMap::from([(3, BTreeSet::from([7]))]));
    assert_eq!(
        plan.refresh,
        BTreeMap::from([
            (1, BTreeSet::from([7])),
            (2, BTreeSet::from([7])),
            (3, BTreeSet::from([7]))
        ])
    );
}

#[test]
pub(crate) fn changing_published_summary_transfers_add_demand_without_payload_reads() {
    use crate::borrow::{Facts, Origin, Proofs, Source};
    use crate::flow::{Flow, TRUE};
    use crate::hir::{Block, Program, Type};
    use crate::loans::{Bundle, Graph};
    use std::collections::{BTreeMap, BTreeSet};

    let program = Program {
        body: Block {
            id: 0,
            stmts: Vec::new(),
            ty: Type::Null,
        },
        functions: Vec::new(),
        locals: Vec::new(),
    };
    let facts = Facts {
        changing_published: BTreeMap::from([(1, BTreeSet::from([7]))]),
        ..Facts::default()
    };
    let proofs = Proofs::default();
    let mut flow = Flow::default();
    let mut graph = Graph::new(&program, &facts, &proofs, &mut flow);
    let origin = Origin {
        component: Vec::new(),
        source: Source::Expired { id: 7 },
        guard: TRUE,
    };
    let source = graph.value(vec![origin.clone()]).unwrap();
    let target = graph.value(vec![origin]).unwrap();
    let source = Bundle::from([(Vec::new(), source)]);
    let target = Bundle::from([(Vec::new(), target)]);
    let node = graph.retained_alias(&source, &target).unwrap();
    assert!(node.uses.is_empty());
    assert!(node.access.is_none());
    assert!(node.events.is_empty());
    assert_eq!(node.defs, target.values().copied().collect::<Vec<_>>());
    assert_eq!(
        node.transfers,
        vec![(target[&Vec::new()], source[&Vec::new()], TRUE)]
    );
    assert_eq!(
        graph
            .changing_header(1, &BTreeMap::new(), &BTreeMap::new())
            .unwrap(),
        TRUE
    );
    graph.guards.spend(usize::MAX);
    assert_eq!(
        graph.retained_alias(&source, &target).err().unwrap().code,
        "B001"
    );
}
