use super::allocators::{accepts, rejects};

pub(crate) const NULLABLE: &str = "a<m.Allocator><null>:=null;";
pub(crate) const RECORD: &str = "r:={->h<m.Allocator><null>:=null;->n:=0};";

#[test]
pub(crate) fn tagged_allocator_replacement_keeps_old_copy_bounds() {
    accepts(&format!("{NULLABLE}{{x:1;a=f(&x)}};a=null;copy:a"));
    rejects(&format!("{NULLABLE}{{x:1;a=f(&x)}};copy:a"), "E303");
    accepts(&format!("{NULLABLE}old:a;{{x:1;a=f(&x)}};copy:old"));
    rejects(
        &format!("{NULLABLE}b<m.Allocator><null>:=null;{{x:1;a=f(&x);b=a}};a=null;copy:b"),
        "E303",
    );
}

#[test]
pub(crate) fn tagged_allocator_predicates_skip_inactive_payloads() {
    accepts(&format!("{NULLABLE}{{x:1;a=f(&x)}};|a<null>|{{copy:a}}"));
    rejects(
        &format!("{NULLABLE}{{x:1;a=f(&x)}};|a<m.Allocator>|{{copy:a}}"),
        "E303",
    );
    accepts(&format!(
        "{NULLABLE}flag:=true;{{x:1;|flag|a=f(&x)}};|a<null>|{{copy:a}}"
    ));
    rejects(
        &format!("{NULLABLE}flag:=true;{{x:1;|flag|a=f(&x)}};|a<m.Allocator>|{{copy:a}}"),
        "E303",
    );
}

#[test]
pub(crate) fn tagged_allocator_nested_variants_keep_parent_activity() {
    let types = "<A>:<{h<m.Allocator><null>;flag<boolean>}>;<B>:<{n<int32>}>;";
    accepts(&format!(
        "{types}a<A><B>:={{->n:2}};{{x:1;a={{->h:f(&x);->flag:true}}}};|a<B>|{{n:a.n}};|a<A>|{{|a.h<null>|{{copy:a.h}}}}"
    ));
    rejects(
        &format!(
            "{types}a<A><B>:={{->n:2}};{{x:1;a={{->h:f(&x);->flag:true}}}};|a<A>|{{|a.h<m.Allocator>|{{copy:a.h}}}}"
        ),
        "E303",
    );
    accepts(&format!(
        "{types}a<A><B>:={{->n:2}};again:=true;'loop{{|a<B>|{{n:a.n}};x:1;|again|{{again=false;a={{->h:f(&x);->flag:true}};'loop.restart()}}}};a={{->n:3}};copy:a"
    ));
}

#[test]
pub(crate) fn tagged_allocator_writes_invalidate_prior_predicates() {
    rejects(
        &format!("{NULLABLE}|a<null>|{{{{x:1;a=f(&x)}};|a<m.Allocator>|{{copy:a}}}}"),
        "E303",
    );
    accepts(&format!(
        "{NULLABLE}|a<null>|{{x:1;a=f(&x);|a<m.Allocator>|{{copy:a}}}}"
    ));
    accepts(&format!(
        "{NULLABLE}{{x:1;a=f(&x)}};|a<m.Allocator>|{{a=null;|a<null>|{{copy:a}}}}"
    ));
}

#[test]
pub(crate) fn tagged_allocator_fields_keep_siblings_and_rhs_writes() {
    accepts(&format!(
        "{RECORD}{{x:1;r.h=f(&x)}};number:r.n;|r.h<null>|{{copy:r.h}};r.h=null;whole:r"
    ));
    rejects(
        &format!("{RECORD}{{x:1;r.h=f(&x)}};r.n=2;|r.h<m.Allocator>|{{copy:r.h}}"),
        "E303",
    );
    rejects(
        &format!(
            "{RECORD}|r.h<null>|{{{{x:1;r.n={{r.h=f(&x);->2}}}};|r.h<m.Allocator>|{{copy:r.h}}}}"
        ),
        "E303",
    );
    accepts(&format!("{RECORD}{{x:1;r.h={{r.h=f(&x);->null}}}};whole:r"));
}

#[test]
pub(crate) fn tagged_allocator_restart_headers_preserve_activity_and_expiry() {
    accepts(&format!(
        "{NULLABLE}x:1;n:=2;'loop{{a=f(&x);|a<m.Allocator>|{{copy:a}};a=null;n=n-1;|n>0|'loop.restart()}};copy:a"
    ));
    accepts(&format!(
        "{NULLABLE}again:=true;'loop{{|a<null>|{{copy:a}};x:1;|again|{{again=false;a=f(&x);'loop.restart()}}}};a=null;copy:a"
    ));
    rejects(
        &format!("{NULLABLE}'loop{{|a<m.Allocator>|{{copy:a}};x:1;a=f(&x);'loop.restart()}}"),
        "E303",
    );
    accepts(&format!(
        "{RECORD}again:=true;'loop{{r.h=null;copy:r.h;x:1;|again|{{again=false;r.h=f(&x);'loop.restart()}}}};r.h=null;whole:r"
    ));
}

#[test]
pub(crate) fn tagged_allocator_leave_preserves_completed_writes() {
    rejects(
        &format!(
            "{NULLABLE}'out{{x:1;a={{a=f(&x);'out.leave();->null}}}};|a<m.Allocator>|{{copy:a}}"
        ),
        "E303",
    );
    accepts(&format!(
        "{NULLABLE}'out{{x:1;a={{a=f(&x);'out.leave();->null}}}};a=null;copy:a"
    ));
}

#[test]
pub(crate) fn tagged_allocator_cell_loans_and_remaining_carriers_stay_checked() {
    rejects(&format!("{NULLABLE}p:&a;a=m.heap;copy:*p"), "E302");
    accepts(&format!("{NULLABLE}p:&a;copy:*p;a=m.heap"));
    accepts("x:1;r:={->h:f(&x);->p:&x}");
    rejects("x:1;r:={->h:f(&x);->list:[1]}", "B001");
    rejects("x:1;r:{->h<m.Allocator><null>:=f(&x)}", "B001");
}
