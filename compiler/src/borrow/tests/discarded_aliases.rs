use super::{accepts, rejects};

#[test]
pub(crate) fn discarded_alias_writes_preserve_local_sources_and_old_copies() {
    accepts("x:=1;y:2;'out{r:{->p:=&x;p=&y;x=3;v:*p;'out.leave()}}");
    rejects(
        "x:=1;y:2;'out{r:{->p:=&x;old:p;p=&y;x=3;v:*old;'out.leave()}}",
        "E302",
    );
    rejects(
        "x:1;y:=2;'out{r:{->p:=&x;p=&y;y=3;v:*p;'out.leave()}}",
        "E302",
    );
    accepts("x:1;'out{r:{->p:=&x;y:2;p=&y;v:*p;'out.leave()}}");
}

#[test]
pub(crate) fn discarded_alias_fields_keep_rhs_siblings_and_leave_effects() {
    accepts(
        "x:=1;y:2;'out{r:{->c:={->p:=&x;->q:=&y;->n:=0};c.p={c.q=&x;->&y};c.q=&y;x=3;copy:c;'out.leave()}}",
    );
    rejects(
        "x:=1;y:2;'out{r:{->c:={->p:=&x;->q:=&y};c.p={c.q=&x;->&y};x=3;v:*(c.q);'out.leave()}}",
        "E302",
    );
    accepts(
        "x:1;y:2;'out{r:{->c:={->p:=&x;->n:=0};'inner{c.p={c={->p:=&y;->n:=3};'inner.leave();->&x}};v:*(c.p);'out.leave()}}",
    );
}

#[test]
pub(crate) fn discarded_aliases_keep_expiry_and_allow_overwrite_before_read() {
    accepts("x:1;'out{r:{->p:=&x;{y:2;p=&y};p=&x;v:*p;'out.leave()}}");
    rejects("x:1;'out{r:{->p:=&x;{y:2;p=&y};v:*p;'out.leave()}}", "E303");
    accepts(
        "'out{r:{->p<&int32><null>:=null;{x:1;p=&x};|p<null>|{copy:p};p=null;copy:p;'out.leave()}}",
    );
    rejects(
        "'out{r:{->p<&int32><null>:=null;{x:1;p=&x};|p<&int32>|{v:*p};'out.leave()}}",
        "E303",
    );
}

#[test]
pub(crate) fn discarded_alias_cells_follow_their_target_beyond_lexical_scope() {
    accepts("x:1;y:2;'out{r:'target{cell:{'target->p:=&x;p=&y;->&p};v:**cell;'out.leave()}}");
    rejects(
        "x:1;y:2;first:&x;cell:=&first;'out{r:'target{{'target->p:=&x;p=&y;cell=&p};'out.leave()}};v:**cell",
        "E303",
    );
    rejects(
        "x:1;y:2;'out{r:{->p:=&x;cell:&p;p=&y;v:**cell;'out.leave()}}",
        "E302",
    );
    accepts("x:1;y:2;'out{r:{->p:=&x;cell:&p;p=*cell;p=&y;'out.leave()}}");
}

#[test]
pub(crate) fn discarded_alias_versions_survive_inner_restarts() {
    accepts(
        "x:=1;y:2;n:=2;'out{r:{->p:=&x;'loop{p=&y;n=n-1;|n>0|'loop.restart()};x=3;v:*p;'out.leave()}}",
    );
    rejects(
        "x:1;'out{r:{->p:=&x;'loop{v:*p;y:2;p=&y;'loop.restart()};'out.leave()}}",
        "E303",
    );
    accepts(
        "x:1;n:=2;'out{r:{->p:=&x;'loop{p=&x;v:*p;y:2;p=&y;n=n-1;|n>0|'loop.restart()};p=&x;v:*p;'out.leave()}}",
    );
    accepts(
        "x:1;y:2;n:=2;'out{r:'loop{->p:=&x;p=&y;n=n-1;|n>0|'loop.restart();v:*p;'out.leave()}}",
    );
}

#[test]
pub(crate) fn discarded_aliases_preserve_nested_bounds_and_real_call_entry() {
    let prefix = "m:@\"memory\";f<m.Allocator>:(p<&int32>){->m.heap};g<null>:(p<&m.Allocator>,n<int32>){};x:1;";
    accepts(&format!(
        "{prefix}'out{{r:{{->c:={{->p:=&x;->h:=m.heap}};c.h=f(&1);g(&(c.h),{{'out.leave();->0}})}}}}"
    ));
    rejects(
        &format!(
            "{prefix}'out{{r:{{->c:={{->p:=&x;->h:=m.heap}};c.h=f(&1);g(&(c.h),0);'out.leave()}}}}"
        ),
        "E303",
    );
    accepts("x:=1;y:2;a:&x;b:&y;'out{r:{->p:=&a;p=&b;x=3;v:**p;'out.leave()}}");
}

#[test]
pub(crate) fn published_result_and_backing_boundaries_remain_explicit() {
    accepts("x:1;y:2;n:=2;r:{->p:=&x;'loop{p=&y;n=n-1;|n>0|'loop.restart()}};v:*(r.p)");
    rejects("x:1;r:{->p:=&x;y:2;p=&y};v:*(r.p)", "E303");
    accepts("x:1;flag:=true;r:'target{|flag|{'target->p:=&x;p=&x}}");
    rejects("x:=1;'out{r:{->p:=&!x;'out.leave()}}", "B001");
    rejects(
        "m:@\"memory\";f<m.Allocator>:(p<&int32>){->m.heap};x:1;'out{r:{->h:=f(&x);'out.leave()}}",
        "B001",
    );
    rejects("x:1;'out{r:{->p:&x;p=&x;'out.leave()}}", "E305");
}
