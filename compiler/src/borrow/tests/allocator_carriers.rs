use super::allocators::{accepts, rejects};

pub(crate) const RECORD: &str = "x:=1;y:=2;r:={->h:=m.heap;->p:&x;->n:=0};";

#[test]
pub(crate) fn allocator_carriers_replace_physical_and_lifetime_sources() {
    accepts(&format!(
        "{RECORD}r={{->h:=m.heap;->p:&y;->n:=3}};r.h=f(&x);x=4;copy:r"
    ));
    rejects(
        &format!("{RECORD}y=4;r={{->h:=m.heap;->p:&y;->n:=3}};y=5;copy:r"),
        "E302",
    );
    accepts(&format!(
        "{RECORD}old:r;r={{->h:=m.heap;->p:&y;->n:=3}};copy:old;y=4"
    ));
    rejects(
        &format!("{RECORD}old:r;r={{->h:=m.heap;->p:&y;->n:=3}};x=4;copy:old"),
        "E302",
    );
}

#[test]
pub(crate) fn allocator_carriers_select_live_fields_and_repair_bounds() {
    accepts(&format!(
        "{RECORD}{{z:3;r.h=f(&z)}};v:*(r.p);r.n=4;r.h=m.heap;copy:r"
    ));
    rejects(&format!("{RECORD}{{z:3;r.h=f(&z)}};r.n=4;copy:r"), "E303");
    accepts(&format!(
        "{RECORD}{{z:3;r={{->h:=m.heap;->p:&z;->n:=3}}}};handle:r.h;r={{->h:=m.heap;->p:&x;->n:=4}};copy:r"
    ));
    rejects(
        &format!("{RECORD}{{z:3;r={{->h:=m.heap;->p:&z;->n:=3}}}};copy:r.p"),
        "E303",
    );
}

#[test]
pub(crate) fn allocator_carriers_commit_fields_after_rhs_effects() {
    accepts(&format!(
        "{RECORD}r.n={{r={{->h:=m.heap;->p:&y;->n:=7}};->8}};x=4;copy:r"
    ));
    rejects(
        &format!("{RECORD}r.n={{r={{->h:=m.heap;->p:&y;->n:=7}};->8}};y=4;copy:r"),
        "E302",
    );
    rejects(
        &format!(
            "{RECORD}'out{{z:3;r.n={{r={{->h:=m.heap;->p:&z;->n:=7}};'out.leave();->8}}}};copy:r.p"
        ),
        "E303",
    );
}

#[test]
pub(crate) fn allocator_carriers_preserve_nullable_reference_activity() {
    let record = "r:={->h:=m.heap;->p<&int32><null>:null};";
    accepts(&format!(
        "{record}{{x:1;r={{->h:=m.heap;->p<&int32><null>:&x}}}};|r.p<null>|{{copy:r.p}};r={{->h:=m.heap;->p<&int32><null>:null}};copy:r"
    ));
    rejects(
        &format!(
            "{record}{{x:1;r={{->h:=m.heap;->p<&int32><null>:&x}}}};|r.p<&int32>|{{copy:r.p}}"
        ),
        "E303",
    );
    rejects(
        &format!(
            "{record}|r.p<null>|{{{{x:1;r={{->h:=m.heap;->p<&int32><null>:&x}}}};|r.p<&int32>|{{copy:r.p}}}}"
        ),
        "E303",
    );
}

#[test]
pub(crate) fn allocator_carriers_restart_with_required_reference_coverage() {
    accepts(&format!(
        "{RECORD}n:=2;'loop{{r={{->h:=m.heap;->p:&y;->n:=n}};r.h=f(&x);copy:r;n=n-1;|n>0|'loop.restart()}};copy:r"
    ));
    rejects(
        &format!("{RECORD}'loop{{copy:r.p;z:3;r={{->h:=m.heap;->p:&z;->n:=3}};'loop.restart()}}"),
        "E303",
    );
    accepts(&format!(
        "{RECORD}again:=true;'loop{{r={{->h:=m.heap;->p:&x;->n:=0}};copy:r;z:3;|again|{{again=false;r={{->h:=m.heap;->p:&z;->n:=3}};'loop.restart()}}}};r={{->h:=m.heap;->p:&x;->n:=0}};copy:r"
    ));
    rejects(
        &format!("{RECORD}'loop{{copy:r;z:3;r.h=f(&z);'loop.restart()}}"),
        "E303",
    );
}

#[test]
pub(crate) fn allocator_carriers_keep_transitive_bounds_and_cell_loans() {
    accepts(&format!(
        "{RECORD}p:&r;copy:*p;r={{->h:=m.heap;->p:&y;->n:=0}}"
    ));
    rejects(
        &format!("{RECORD}p:&r;r={{->h:=m.heap;->p:&y;->n:=0}};copy:*p"),
        "E302",
    );
    rejects(
        "a:m.heap;r:={->h:=m.heap;->p:&a};{z:2;b:f(&z);r={->h:=m.heap;->p:&b}};copy:*(r.p)",
        "E303",
    );
    accepts("x:1;r:={->h:m.heap;->p:=&x}");
    rejects("x:1;r:={->h:m.heap;->p:&x;->list:[1]}", "B001");
    rejects("x:1;r:{->h:=f(&x);->p:&x}", "B001");
    accepts("x:1;r:={->p:&x}");
}

#[test]
pub(crate) fn allocator_carriers_validate_transitive_call_inputs_after_all_arguments() {
    let prefix = "g<null>:(p<&m.Allocator>,n<int32>){};a:m.heap;r:={->h:=m.heap;->p:&a};";
    rejects(&format!("{prefix}r.h=f(&1);g(&(r.h),0)"), "E303");
    accepts(&format!(
        "{prefix}r.h=f(&1);'out{{g(&(r.h),{{'out.leave();->0}})}}"
    ));
    rejects("x:=1;p:&x;r:={->h:m.heap;->p:&p};x=2;v:*(*(r.p))", "E302");
    accepts("x:=1;y:=2;p:&x;q:&y;r:={->h:m.heap;->p:&p};r={->h:m.heap;->p:&q};x=3;v:*(*(r.p))");
}

#[test]
pub(crate) fn allocator_carriers_merge_reference_variants_across_branches_and_restarts() {
    let prefix = "x:=1;y:=2;r:={->h:m.heap;->p<&int32><null>:null};flag:=true;";
    accepts(&format!(
        "{prefix}|flag|r={{->h:m.heap;->p<&int32><null>:&x}};|r.p<null>|{{x=3}};r={{->h:m.heap;->p<&int32><null>:&y}};x=4;v:r"
    ));
    rejects(
        &format!(
            "{prefix}|flag|r={{->h:m.heap;->p<&int32><null>:&x}};x=3;|r.p<&int32>|{{v:*(r.p)}}"
        ),
        "E302",
    );
    accepts(&format!(
        "{prefix}'loop{{|r.p<null>|{{v:r.p}};z:3;|flag|{{flag=false;r={{->h:m.heap;->p<&int32><null>:&z}};'loop.restart()}}}};r={{->h:m.heap;->p<&int32><null>:null}};v:r"
    ));
    rejects(
        &format!(
            "{prefix}'loop{{|r.p<&int32>|{{v:*(r.p)}};z:3;r={{->h:m.heap;->p<&int32><null>:&z}};'loop.restart()}}"
        ),
        "E303",
    );
}
