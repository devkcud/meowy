use super::{accepts, rejects};

#[test]
pub(crate) fn optional_alias_writes_replace_only_the_selected_member_sources() {
    accepts(
        "<R>:<{p<&int32><null>:=}>;x:1;y:2;r<R>:{->p<&int32>:=&x;p=&y};|r.p<&int32>|{v:*(r.p)}",
    );
    accepts("x:=1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;p=&y;x=3}};|r.p<&int32>|{v:*(r.p)}");
    accepts("x:=1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;p=&y}};x=3;|r.p<&int32>|{v:*(r.p)}");
    rejects(
        "x:=1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;old:p;p=&y;x=3;v:*old}}",
        "E302",
    );
    rejects(
        "x:1;y:=2;flag:=true;r:'out{|flag|{'out->p:=&x;p=&y}};y=3;|r.p<&int32>|{v:*(r.p)}",
        "E302",
    );
}

#[test]
pub(crate) fn widened_alias_branches_preserve_distinct_outer_members() {
    let prefix = "x:=1;y:2;text:\"a\";other:\"b\";flag:=true;";
    accepts(&format!(
        "{prefix}r:'out{{|flag|{{'out->p:=&x;p=&y}};|!flag|{{'out->p:=&text;p=&other}}}};x=3;|r.p<&int32>|{{v:*(r.p)}};|r.p<&string>|{{v:*(r.p)}}"
    ));
    rejects(
        "x:1;y:=2;flag:=true;r:'out{|flag|{'out->p:=&x;p=&y};|!flag|{'out->p:=\"x\"}};y=3;|r.p<&int32>|{v:*(r.p)}",
        "E302",
    );
    accepts(
        "x:1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;p=&y;'out.leave()};->p:=\"x\"};|r.p<&int32>|{v:*(r.p)}",
    );
}

#[test]
pub(crate) fn widened_record_alias_fields_keep_siblings_and_rhs_versions() {
    let prefix = "x:=1;y:=2;flag:=true;";
    accepts(&format!(
        "{prefix}r:'out{{|flag|{{'out->c:={{->p:=&x;->q:=&y;->n:=0}};c.p=&y;c.n=3}}}};x=4"
    ));
    rejects(
        &format!(
            "{prefix}r:'out{{|flag|{{'out->c:={{->p:=&x;->q:=&y;->n:=0}};c.p=&y;c.n=3}}}};y=4;copy:r"
        ),
        "E302",
    );
    accepts(&format!(
        "{prefix}r:'out{{|flag|{{'out->c:={{->p:=&x;->q:=&y;->n:=0}};c.p={{c.q=&x;->&y}};c.q=&y}}}};x=3;copy:r"
    ));
    rejects(
        &format!(
            "{prefix}r:'out{{|flag|{{'out->c:={{->p:=&x;->q:=&y;->n:=0}};c.p={{c.q=&x;->&y}};x=3;v:*(c.q)}}}}"
        ),
        "E302",
    );
    accepts(&format!(
        "{prefix}r:'out{{|flag|{{'out->c:={{->p:=&x;->n:=0}};c.n={{c={{->p:=&y;->n:=1}};->2}}}}}};x=3;copy:r"
    ));
}

#[test]
pub(crate) fn widened_alias_updates_preserve_leave_and_cell_loans() {
    accepts(
        "x:1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;p={p=&y;'out.leave();->&x}}};|r.p<&int32>|{v:*(r.p)}",
    );
    accepts("x:1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;cell:&p;p=*cell;p=&y}};copy:r");
    rejects(
        "x:1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;cell:&p;p=&y;v:**cell}}",
        "E302",
    );
    rejects(
        "x:1;y:=2;flag:=true;r:'out{|flag|{'out->p:=&x;p={p=&y;'out.leave();->&x}}};y=3;copy:r",
        "E302",
    );
}

#[test]
pub(crate) fn widened_alias_payload_tags_do_not_replace_parent_activity() {
    let prefix = "x:1;flag:=true;";
    accepts(&format!(
        "{prefix}r:'out{{|flag|{{'out->c:={{->p<&int32><null>:=null}};|c.p<null>|{{c.p=&x;|c.p<&int32>|{{v:*(c.p)}}}}}}}};copy:r"
    ));
    accepts("x:=1;flag:=true;r:'out{|flag|{'out->c:={->p<&int32><null>:=&x};c.p=null}};x=2;copy:r");
    rejects(
        "x:=1;flag:=true;r:'out{|flag|{'out->c:={->p<&int32><null>:=null};c.p=&x;x=2;|c.p<&int32>|{v:*(c.p)}}}",
        "E302",
    );
}

#[test]
pub(crate) fn widened_alias_lifetimes_and_reset_iterations_remain_checked() {
    accepts(
        "outer:1;flag:=true;result:{local:2;r:'out{|flag|{'out->p:=&local;p=&outer}};->r};copy:result",
    );
    rejects(
        "outer:1;flag:=true;result:{local:2;r:'out{|flag|{'out->p:=&outer;p=&local}};->r}",
        "E303",
    );
    accepts(
        "x:1;y:2;flag:=true;n:=0;r:'loop{|flag|{'loop->p:=&x;p=&y};n=n+1;|n<2|'loop.restart()};|r.p<&int32>|{v:*(r.p)}",
    );
    accepts(
        "x:1;y:2;flag:=true;n:=2;r:'out{|flag|{'out->p:=&x;'loop{p=&y;n=n-1;|n>0|'loop.restart()}}};copy:r",
    );
}

#[test]
pub(crate) fn widened_union_views_and_allocator_only_aliases_stay_gated() {
    accepts(
        "x:1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=null;p=&x};|!flag|{'out->p:=\"x\"}}",
    );
    rejects(
        "m:@\"memory\";f<m.Allocator>:(p<&int32>){->m.heap};x:1;flag:=true;r:'out{|flag|{'out->h:=f(&x)}}",
        "B001",
    );
    rejects("x:1;flag:=true;r:'out{|flag|{'out->p:&x;p=&x}}", "E305");
    rejects("x:1;flag:=true;r:'out{|flag|{'out->p:=&x;p=\"x\"}}", "E207");
}
