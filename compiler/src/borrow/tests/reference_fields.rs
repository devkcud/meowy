use super::{accepts, rejects};

pub(crate) const RECORD: &str = "x:=1;y:=2;r:={->p:=&x;->q:=&y;->n:=0};";
pub(crate) const NULLABLE: &str = "r:={->p<&int32><null>:=null;->n:=0};";

#[test]
pub(crate) fn reference_field_constructors_preserve_snapshots_and_mutability() {
    accepts("x:1;r:={->p:=&x;->n:*p};v:*(r.p)");
    accepts("x:1;y:2;r:{->c:={->p:=&x};copy:=c;copy.p=&y;v:*(c.p)};v:*(r.c.p)");
    rejects(
        "x:=1;y:2;r:{->c:={->p:=&x};copy:=c;copy.p=&y;x=3};v:*(r.c.p)",
        "E302",
    );
    accepts("<R>:<{p<&int32>:=}>;x:1;y:2;r<R>:={->p:=&x};r.p=&y;v:*(r.p)");
    accepts("<R>:<{p<&int32><null>:=}>;x:1;r<R>:={};r.p=&x;|r.p<&int32>|{v:*(r.p)}");
    accepts("x:1;flag:=true;r:={|flag|->p:=&x};r.p=null;copy:r");
    rejects("<R>:<{p<&int32>:=}>;x:1;r<R>:{->p:&x}", "E206");
    accepts("x:1;r:{->p:=&x};r.p=&x");
    rejects("x:1;r:={->p:=&x};r.p=&true", "E207");
}

#[test]
pub(crate) fn reference_field_versions_preserve_old_copies_and_sibling_loans() {
    accepts(&format!("{RECORD}r.p=&y;x=3;v:*(r.p)"));
    rejects(&format!("{RECORD}old:r.p;r.p=&y;x=3;v:*old"), "E302");
    accepts(&format!("{RECORD}old:r;r.p=&y;v:*(old.p);x=3"));
    rejects(&format!("{RECORD}r.p=&x;y=3;v:*(r.q)"), "E302");
    accepts(&format!("{RECORD}{{z:3;r.p=&z}};v:*(r.q);r.p=&x;copy:r"));
    rejects(&format!("{RECORD}{{z:3;r.p=&z}};r.q=&x;copy:r"), "E303");
}

#[test]
pub(crate) fn nested_reference_field_updates_require_a_mutable_leaf() {
    let prefix = "x:=1;y:=2;r:={->c:={->p:=&x;->n:=0};->q:=&y};";
    accepts(&format!("{prefix}r.c.p=&y;x=3;copy:r"));
    accepts(&format!("{prefix}r.c={{->p:=&y;->n:=3}};x=4;copy:r"));
    rejects(&format!("{prefix}old:r.c;r.c.p=&y;x=3;v:*(old.p)"), "E302");
    accepts("x:1;r:={->c:{->p:=&x}};r.c.p=&x");
    rejects("x:1;r:={->c:={->p:&x}};r.c.p=&x", "E305");
}

#[test]
pub(crate) fn reference_field_commits_keep_rhs_and_leave_effects() {
    accepts(&format!(
        "{RECORD}r.p={{r.q=&x;->&y}};v:*(r.q);r.q=&y;x=3;copy:r"
    ));
    rejects(&format!("{RECORD}r.p={{r.q=&x;->&y}};x=3;v:*(r.q)"), "E302");
    accepts(&format!(
        "{RECORD}r.p={{r={{->p:=&x;->q:=&x;->n:=3}};->&y}};r.q=&y;x=4;copy:r"
    ));
    rejects(
        &format!("{RECORD}'out{{z:3;r.p={{r.q=&z;'out.leave();->&y}}}};v:r.q"),
        "E303",
    );
    accepts(&format!(
        "{RECORD}'out{{z:3;r.p={{r.q=&z;'out.leave();->&y}}}};v:*(r.p);r.q=&y;copy:r"
    ));
}

#[test]
pub(crate) fn nullable_reference_fields_observe_current_activity() {
    accepts(&format!(
        "{NULLABLE}{{x:1;r.p=&x}};|r.p<null>|{{v:r.p}};r.p=null;copy:r"
    ));
    rejects(
        &format!("{NULLABLE}{{x:1;r.p=&x}};|r.p<&int32>|{{v:*(r.p)}}"),
        "E303",
    );
    rejects(
        &format!("{NULLABLE}|r.p<null>|{{{{x:1;r.n={{r.p=&x;->1}}}};|r.p<&int32>|{{v:*(r.p)}}}}"),
        "E303",
    );
    accepts(&format!(
        "{NULLABLE}{{x:1;r.p=&x}};|r.p<&int32>|{{r.p=null;|r.p<null>|{{copy:r}}}}"
    ));
}

#[test]
pub(crate) fn reference_field_restarts_preserve_origins_and_terminal_expiry() {
    accepts(&format!(
        "{RECORD}n:=2;'loop{{r.p=&y;r.q=&x;v:*(r.p);n=n-1;|n>0|'loop.restart()}};copy:r"
    ));
    accepts(&format!(
        "{NULLABLE}again:=true;'loop{{|r.p<null>|{{v:r.p}};x:1;|again|{{again=false;r.p=&x;'loop.restart()}}}};r.p=null;copy:r"
    ));
    rejects(
        &format!("{NULLABLE}'loop{{|r.p<&int32>|{{v:*(r.p)}};x:1;r.p=&x;'loop.restart()}}"),
        "E303",
    );
    accepts("x:1;r:{->p:=&x;n:=2;'loop{v:*p;n=n-1;|n>0|'loop.restart()}};v:*(r.p)");
}

#[test]
pub(crate) fn reference_fields_keep_cell_borrows_and_public_call_bounds() {
    accepts(&format!("{RECORD}cell:&(r.p);r.q=&x;v:**cell;r.p=&y"));
    rejects(&format!("{RECORD}cell:&(r.p);r.p=&y;v:**cell"), "E302");
    accepts(&format!("{RECORD}cell:&(r.p);r.p=*cell;v:*(r.p)"));
    let prefix = "first<&int32>:(p<&int32>,q<&int32>){->p};x:1;r:={->p:=&x};";
    rejects(
        &format!("{prefix}{{y:2;r.p=first(&x,&y)}};v:*(r.p)"),
        "E303",
    );
    let call = "g<null>:(p<& &int32>,n<int32>){};x:1;r:={->p:=&x};r.p=&1;";
    rejects(&format!("{call}g(&(r.p),0)"), "E303");
    accepts(&format!("{call}'out{{g(&(r.p),{{'out.leave();->0}})}}"));
}

#[test]
pub(crate) fn borrowed_alias_writes_preserve_unsupported_shapes() {
    accepts("x:1;r:{->c:{->p:=&x};c.p=&x}");
    accepts("x:1;y:2;r:{->p:=&x;p=&y}");
    accepts("x:1;r:{->p<&int32><null>:=null;p=&x}");
    accepts("x:1;y:2;r:{->c:={->p:=&x};c.p=&y}");
    accepts("x:1;r:{->c:={->p:=&x;->n:=0};c.n=1}");
    accepts("x:1;y:2;r:{->c:={->p:=&x};c={->p:=&y}}");
    rejects("x:1;r:={->p:=&[1]}", "B001");
    rejects("x:=1;r:={->p:=&!x}", "B001");
    rejects("r:{x:1;->p:=&x}", "E303");
}
