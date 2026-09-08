use super::{accepts, rejects};

#[test]
pub(crate) fn borrowed_alias_writes_publish_current_sources() {
    accepts("outer:1;result:{local:2;r:{->p:=&local;p=&outer};->r};v:*result.p");
    rejects(
        "outer:1;result:{local:2;r:{->p:=&outer;p=&local};->r}",
        "E303",
    );
    accepts("x:=1;y:2;r:{->p:=&x;p=&y;x=3};v:*r.p");
    rejects("x:=1;y:2;r:{->p:=&x;old:p;p=&y;x=3;v:*old}", "E302");
    rejects("x:=1;y:=2;r:{->p:=&x;p=&y;y=3};v:*r.p", "E302");
    rejects("x:1;r:{->p:=&x;y:2;p=&y};v:*r.p", "E303");
    accepts("x:1;y:2;r:{->p:=&x;p=&y;v:*p};v:*r.p");
}

#[test]
pub(crate) fn borrowed_alias_field_writes_keep_siblings_and_rhs_effects() {
    accepts("x:=1;y:2;r:{->c:={->p:=&x;->q:=&y;->n:=0};c.p=&y;x=3};v:*r.c.p");
    rejects(
        "x:=1;y:=2;r:{->c:={->p:=&x;->q:=&y};c.p=&y;y=3};v:*r.c.q",
        "E302",
    );
    accepts("x:=1;y:2;r:{->c:={->p:=&x;->q:=&y};c.p={c.q=&x;->&y};c.q=&y;x=3};v:r.c");
    rejects(
        "x:=1;y:2;r:{->c:={->p:=&x;->q:=&y};c.p={c.q=&x;->&y};x=3};v:*r.c.q",
        "E302",
    );
    accepts("x:=1;y:2;r:{->c:={->p:=&x;->n:=0};c.n={c={->p:=&y;->n:=1};->2};x=3};v:r.c");
}

#[test]
pub(crate) fn borrowed_alias_branches_and_leave_keep_result_versions() {
    accepts("x:1;y:2;flag:=true;r:{->p:=&x;|flag|p=&y};v:*r.p");
    rejects(
        "x:=1;y:=2;flag:=true;r:{->p:=&x;|flag|p=&y;y=3};v:*r.p",
        "E302",
    );
    accepts("x:=1;y:2;r:'out{->p:=&x;p=&y;'out.leave()};x=3;v:*r.p");
    accepts("x:1;y:2;r:'out{->p:=&x;p={p=&y;'out.leave();->&x}};v:*r.p");
    rejects(
        "x:1;y:=2;r:'out{->p:=&x;p={p=&y;'out.leave();->&x}};y=3;v:*r.p",
        "E302",
    );
    accepts("x:1;y:2;flag:=true;r:'out{->p:=&x;|flag|{p=&y;'out.leave()};p=&x};v:*r.p");
}

#[test]
pub(crate) fn borrowed_alias_tags_follow_replacement_and_null_results() {
    accepts("x:=1;r:{->p<&int32><null>:=&x;p=null;x=2};copy:r");
    accepts(
        "x:1;r:{->p<&int32><null>:=null;|p<null>|{p=&x;|p<&int32>|{v:*p}}};|r.p<&int32>|{v:*r.p}",
    );
    accepts("x:1;flag:=true;r:{->p<&int32><null>:=&x;|flag|p=null};|r.p<null>|{copy:r.p}");
    rejects(
        "x:=1;flag:=true;r:{->p<&int32><null>:=null;|flag|p=&x;x=2};|r.p<&int32>|{v:*r.p}",
        "E302",
    );
    accepts("x:=1;r:{->c:={->p<&int32><null>:=&x;->n:=0};c.p=null;x=2};copy:r");
}

#[test]
pub(crate) fn borrowed_alias_writes_preserve_cell_and_transitive_loans() {
    accepts("x:1;y:2;r:{->p:=&x;cell:&p;p=*cell;p=&y};v:*r.p");
    rejects("x:1;y:2;r:{->p:=&x;cell:&p;p=&y;v:**cell}", "E302");
    accepts("x:=1;y:2;p:&x;q:&y;r:{->v:=&p;v=&q;x=3};copy:**r.v");
    rejects(
        "x:=1;y:2;p:&x;q:&y;r:{->v:=&p;old:v;v=&q;x=3;copy:**old}",
        "E302",
    );
    accepts(
        "m:@\"memory\";f<m.Allocator>:(p<&int32>){->m.heap};x:=1;y:2;r:{->c:={->p:=&x;->h:=m.heap};c.p=&y;c.h=f(&x);x=3};copy:r",
    );
}

#[test]
pub(crate) fn borrowed_alias_views_keep_copies_and_discarded_effects() {
    accepts("x:1;y:2;r:{->c:={->p:=&x};old:c;c.p=&y;v:*old.p};v:*r.c.p");
    accepts("x:1;y:2;'out{r:{->p:=&x;p={'out.leave();->&y}}}");
    rejects("x:1;y:2;'out{r:{->p:=&x;p=&y;'out.leave()}}", "B001");
    accepts("x:1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;p=&y};|!flag|{'out->p:=&y;p=&x}};v:*r.p");
}

#[test]
pub(crate) fn borrowed_alias_writes_keep_unsupported_backing_and_restart_gates() {
    rejects("x:1;y:2;r:{->p:=&x;p=&y;'loop{'loop.restart()}}", "B001");
    rejects("x:1;flag:=true;r:'out{|flag|{'out->p:=&x;p=&x}}", "B001");
    rejects(
        "x:1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=null;p=&x};|!flag|{'out->p:=\"x\"}}",
        "B001",
    );
    rejects(
        "m:@\"memory\";f<m.Allocator>:(p<&int32>){->m.heap};x:1;r:{->h:=f(&x)}",
        "B001",
    );
    rejects("x:1;r:{->c:{->p:=&x};c.p=&x}", "E305");
}
