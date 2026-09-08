use super::allocators::{accepts, rejects};

pub(crate) const RECORD: &str = "r:={->h:=m.heap;->other:=m.heap;->n:=0};";

#[test]
pub(crate) fn allocator_record_whole_replacement_preserves_old_copies() {
    accepts("r:={->h:m.heap};{x:1;r={->h:f(&x)}};r={->h:m.heap};copy:r");
    rejects("r:={->h:m.heap};{x:1;r={->h:f(&x)}};copy:r", "E303");
    rejects(
        "r:={->h:m.heap};old:={->h:m.heap};{x:1;r={->h:f(&x)};old=r};r={->h:m.heap};copy:old",
        "E303",
    );
    accepts("r:={->h:m.heap};old:r;{x:1;r={->h:f(&x)}};copy:old");
}

#[test]
pub(crate) fn allocator_record_field_updates_keep_sibling_bounds_and_selective_reads() {
    accepts(&format!(
        "{RECORD}{{x:1;r.h=f(&x)}};number:r.n;copy:r.other;r.h=m.heap;whole:r"
    ));
    rejects(
        &format!("{RECORD}{{x:1;r.h=f(&x)}};r.other=m.heap;copy:r.h"),
        "E303",
    );
    rejects(
        &format!("{RECORD}{{x:1;r.h=f(&x)}};number:r.n;whole:r"),
        "E303",
    );
    accepts("r:={->h:m.heap;->0};{x:1;r={->h:f(&x);->0}};d:@\"debug\";d.print(r)");
}

#[test]
pub(crate) fn allocator_record_field_commit_uses_the_state_after_rhs_effects() {
    rejects(
        &format!("{RECORD}{{x:1;r.h={{r.other=f(&x);->m.heap}}}};copy:r.other"),
        "E303",
    );
    accepts(&format!(
        "{RECORD}{{x:1;r.h={{r.other=f(&x);->m.heap}}}};copy:r.h;r.other=m.heap;whole:r"
    ));
    rejects(
        &format!(
            "make:(x<&int32>){{->h:=m.heap;->other:=m.heap;->n:=0}};{RECORD}{{x:1;r.h={{r=make(&x);->m.heap}}}};copy:r.other"
        ),
        "E303",
    );
    rejects(
        &format!("{RECORD}'out{{x:1;r.h={{r.other=f(&x);'out.leave();->m.heap}}}};copy:r.other"),
        "E303",
    );
}

#[test]
pub(crate) fn nested_allocator_record_paths_preserve_independent_components() {
    let record = "r:={->inner:={->h:=m.heap;->n:=0};->other:=m.heap};";
    accepts(&format!(
        "{record}{{x:1;r.inner.h=f(&x)}};number:r.inner.n;copy:r.other;r.inner.h=m.heap;whole:r"
    ));
    rejects(
        &format!("{record}{{x:1;r.inner.h=f(&x)}};r.other=m.heap;copy:r.inner"),
        "E303",
    );
    accepts(&format!(
        "{record}{{x:1;r.inner.h=f(&x)}};r.inner={{->h:=m.heap;->n:=0}};whole:r"
    ));
}

#[test]
pub(crate) fn allocator_record_branches_and_restart_expiry_keep_current_versions() {
    accepts(&format!(
        "{RECORD}x:1;n:=2;'loop{{r.h=f(&x);copy:r.h;n=n-1;|n>0|'loop.restart()}};whole:r"
    ));
    accepts(&format!(
        "{RECORD}again:=true;'loop{{r.h=m.heap;copy:r.h;x:1;|again|{{again=false;r.h=f(&x);'loop.restart()}}}};r.h=m.heap;whole:r"
    ));
    rejects(
        &format!(
            "{RECORD}again:=true;'loop{{x:1;|again|{{again=false;r.h=f(&x);'loop.restart()}}}};copy:r.h"
        ),
        "E303",
    );
    rejects(
        &format!("{RECORD}flag:=true;{{x:1;|flag|r.h=f(&x)}};copy:r.h"),
        "E303",
    );
    accepts(&format!(
        "{RECORD}{{x:1;false&&{{r.h=f(&x);->true}}}};whole:r"
    ));
}

#[test]
pub(crate) fn fixed_record_mutation_keeps_cell_access_rules_and_remaining_gates() {
    accepts(&format!(
        "{RECORD}{{x:1;r.h=f(&x)}};p:&!r.n;r.h=m.heap;*p=7;number:*p"
    ));
    rejects(&format!("{RECORD}p:&r.h;r.h=m.heap;copy:*p"), "E302");
    rejects(
        &format!("{RECORD}p:&r.n;r={{->h:=m.heap;->other:=m.heap;->n:=0}};number:*p"),
        "E302",
    );
    rejects("x:1;r:={->h:f(&x);->items:[1]}", "B001");
    rejects("x:1;r:={->h:f(&x);->maybe<int32><null>:null}", "B001");
    rejects("x:1;r:{->h:=f(&x)}", "B001");
}
