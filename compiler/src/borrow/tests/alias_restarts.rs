use super::{accepts, rejects};

#[test]
pub(crate) fn restarted_alias_slots_publish_only_the_completing_iteration() {
    accepts("x:1;n:=0;r:'loop{->p:=&x;n=n+1;|n<2|{local:2;p=&local;'loop.restart()}};v:*(r.p)");
    accepts("x:=1;y:2;n:=0;r:'loop{->p:=&x;p=&y;n=n+1;|n<2|'loop.restart()};x=3;v:*(r.p)");
    accepts("x:1;y:2;n:=0;r:'loop{->p:=&x;n=n+1;|n<2|{p=&y;'loop.restart()}};v:*(r.p)");
    rejects(
        "x:1;y:=2;n:=0;r:'loop{->p:=&x;p=&y;n=n+1;|n<2|'loop.restart()};y=3;v:*(r.p)",
        "E302",
    );
    accepts(
        "x:1;y:2;n:=0;r:'loop{->p<&int32><null>:=null;p=&y;p=null;n=n+1;|n<2|'loop.restart()};copy:r",
    );
}

#[test]
pub(crate) fn alias_restart_ownership_uses_emission_targets() {
    accepts("x:1;y:2;n:=0;r:'loop{{'loop->p:=&x;p=&y};n=n+1;|n<2|'loop.restart()};v:*(r.p)");
    accepts(
        "x:1;y:2;n:=0;r:'loop{inner:{->p:=&x;p=&y};n=n+1;|n<2|'loop.restart();->inner};v:*(r.p)",
    );
    rejects(
        "x:1;y:2;r:'outer{'inner{'outer->p:=&x;p=&y;'inner.restart()}}",
        "B001",
    );
    accepts("x:1;y:2;n:=2;r:{->p:=&x;'inner{p=&y;n=n-1;|n>0|'inner.restart()}};v:*(r.p)");
    accepts("x:1;y:2;n:=2;r:{->p:=&x;p=&y;'inner{n=n-1;|n>0|'inner.restart()}};v:*(r.p)");
}

#[test]
pub(crate) fn independent_restarts_do_not_block_borrowed_alias_writes() {
    accepts("x:1;y:2;r:{->p:=&x;p=&y};n:=2;'loop{n=n-1;|n>0|'loop.restart()};v:*(r.p)");
    accepts("n:=2;'loop{n=n-1;|n>0|'loop.restart()};x:1;y:2;r:{->p:=&x;p=&y};v:*(r.p)");
    accepts("x:1;y:2;{r:{->p:=&x;p=&y}};n:=2;'loop{n=n-1;|n>0|'loop.restart()}");
}

#[test]
pub(crate) fn restarted_alias_fields_preserve_rhs_leave_and_old_copies() {
    accepts(
        "x:=1;y:2;n:=0;r:'loop{->c:={->p:=&x;->q:=&y;->n:=0};c.p={c.q=&x;->&y};c.q=&y;n=n+1;|n<2|'loop.restart()};x=3;copy:r.c",
    );
    accepts(
        "x:1;y:2;n:=0;r:'loop{->p:=&x;n=n+1;p={p=&y;|n<2|'loop.restart();'loop.leave();->&x}};v:*(r.p)",
    );
    rejects(
        "x:=1;y:2;n:=0;r:'loop{->p:=&x;old:p;p=&y;x=3;v:*old;n=n+1;|n<2|'loop.restart()}",
        "E302",
    );
    rejects(
        "x:1;y:2;n:=0;r:'loop{->p:=&x;cell:&p;p=&y;v:**cell;n=n+1;|n<2|'loop.restart()}",
        "E302",
    );
}

#[test]
pub(crate) fn restarted_alias_cells_expire_without_reviving_next_iteration() {
    rejects(
        "x:1;first:&x;cell:=&first;n:=0;r:'loop{v:**cell;->p:=&x;p=&x;cell=&p;n=n+1;|n<2|'loop.restart()}",
        "E303",
    );
    accepts(
        "x:1;first:&x;cell:=&first;n:=0;r:'loop{cell=&first;v:**cell;->p:=&x;p=&x;cell=&p;n=n+1;|n<2|'loop.restart()};cell=&first;v:**cell",
    );
    rejects(
        "x:1;n:=0;r:'loop{->p:=&x;local:2;p=&local;n=n+1;|n<2|'loop.restart()}",
        "E303",
    );
}

#[test]
pub(crate) fn restarting_nullable_aliases_keep_fresh_current_tags() {
    accepts(
        "x:1;y:2;n:=0;r:'loop{->p<&int32><null>:=null;|p<null>|{copy:p;p=&x};|p<&int32>|{v:*p;p=&y};n=n+1;|n<2|'loop.restart()};|r.p<&int32>|{v:*(r.p)}",
    );
    rejects(
        "x:=1;y:2;n:=0;r:'loop{->p<&int32><null>:=null;|p<null>|{p=&x;x=3;|p<&int32>|{v:*p}};n=n+1;|n<2|'loop.restart()}",
        "E302",
    );
    accepts(
        "x:1;y:2;n:=0;r:'loop{->c:={->p<&int32><null>:=null;->n:=0};c.p=&x;c.n={c.p=&y;->1};n=n+1;|n<2|'loop.restart()};|r.c.p<&int32>|{v:*(r.c.p)}",
    );
}

#[test]
pub(crate) fn alias_restart_ancestry_rejects_missing_cycles_and_exhaustion() {
    use crate::ast::Span;
    use crate::borrow::mutable::alias_restarts;
    use crate::flow::Flow;
    use std::collections::{BTreeMap, BTreeSet};

    let writes = BTreeMap::from([((1, 1, 0), Span::default())]);
    let restarts = BTreeSet::from([2]);
    for parents in [
        BTreeMap::from([(1, None)]),
        BTreeMap::from([(2, None)]),
        BTreeMap::from([(1, None), (2, Some(3)), (3, Some(2))]),
    ] {
        assert_eq!(
            alias_restarts(&parents, &writes, &restarts, &mut Flow::default())
                .unwrap_err()
                .code,
            "B001"
        );
    }
    let parents = BTreeMap::from([(0, None), (1, Some(0)), (2, Some(0))]);
    alias_restarts(&parents, &writes, &restarts, &mut Flow::default()).unwrap();
    let mut flow = Flow::default();
    flow.spend(usize::MAX);
    assert_eq!(
        alias_restarts(&parents, &writes, &restarts, &mut flow)
            .unwrap_err()
            .code,
        "B001"
    );
}
