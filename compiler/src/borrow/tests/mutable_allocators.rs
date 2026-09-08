use super::allocators::{accepts, rejects};

#[test]
pub(crate) fn mutable_allocator_overwrites_preserve_old_copy_lifetimes() {
    accepts("a:=m.heap;{x:1;a=f(&x)};a=m.heap;copy:a");
    accepts("a:=f(&1);a=m.heap;copy:a");
    accepts("x:=1;a:=f(&x);old:a;a=m.heap;x=2;copy:old");
    rejects("a:=m.heap;{x:1;a=f(&x)};copy:a", "E303");
    rejects(
        "a:=m.heap;b:=m.heap;{x:1;a=f(&x);b=a};a=m.heap;copy:b",
        "E303",
    );
    rejects("a:=m.heap;{x:1;a=f(&x)};old:a;a=m.heap", "E303");
}

#[test]
pub(crate) fn allocator_branch_and_short_circuit_versions_follow_executed_writes() {
    accepts("a:=m.heap;{x:1;flag:false;flag&&{a=f(&x);->true}};copy:a");
    accepts("a:=m.heap;flag:=true;{x:1;|flag|a=f(&x)};a=m.heap;copy:a");
    rejects("a:=m.heap;flag:=true;{x:1;|flag|a=f(&x)};copy:a", "E303");
    rejects(
        "a:=m.heap;flag:=true;{x:1;flag&&{a=f(&x);->true}};copy:a",
        "E303",
    );
    rejects(
        "a:=m.heap;flag:=true;{x:1;|flag|a=f(&x)};flag=false;copy:a",
        "E303",
    );
}

#[test]
pub(crate) fn allocator_leave_retains_completed_writes_and_skips_pending_stores() {
    rejects(
        "a:=m.heap;'out{x:1;a={a=f(&x);'out.leave();->m.heap}};copy:a",
        "E303",
    );
    accepts("a:=m.heap;'out{x:1;a={a=f(&x);'out.leave();->m.heap}};a=m.heap;copy:a");
    rejects(
        "a:=m.heap;b:=m.heap;'out{x:1;a=f(&x);b=a;'out.leave()};a=m.heap;copy:b",
        "E303",
    );
}

#[test]
pub(crate) fn allocator_restart_headers_keep_ancestor_bounds_and_expire_iteration_sources() {
    accepts("x:1;a:=m.heap;n:=2;'loop{a=f(&x);copy:a;n=n-1;|n>0|'loop.restart()};copy:a");
    accepts(
        "a:=m.heap;again:=true;'loop{x:1;|again|{again=false;a=f(&x);'loop.restart()}};a=m.heap;copy:a",
    );
    accepts(
        "a:=m.heap;again:=true;'loop{a=m.heap;copy:a;x:1;|again|{again=false;a=f(&x);'loop.restart()}};a=m.heap;copy:a",
    );
    rejects(
        "a:=m.heap;again:=true;'loop{x:1;|again|{again=false;a=f(&x);'loop.restart()}};copy:a",
        "E303",
    );
    rejects("a:=m.heap;'loop{x:1;old:a;a=f(&x);'loop.restart()}", "E303");
}

#[test]
pub(crate) fn allocator_cell_borrows_still_conflict_with_replacement() {
    accepts("x:=1;a:=f(&x);p:&a;x=2;copy:*p");
    rejects("x:1;a:=f(&x);p:&a;a=m.heap;copy:*p", "E302");
    accepts("x:1;a:=f(&x);p:&a;copy:*p;a=m.heap");
}

#[test]
pub(crate) fn unsupported_mutable_allocator_carriers_remain_explicit() {
    accepts("x:1;a<m.Allocator><null>:=f(&x)");
    accepts("x:1;a:={->handle:f(&x)};copy:a");
    rejects("x:1;result:{->handle:=f(&x)}", "B001");
    accepts("result:{->handle:=m.heap;handle=m.heap}");
}

#[test]
pub(crate) fn allocator_bounds_keep_temporary_and_emitted_slot_identity() {
    accepts("a:=f(&1);'loop{a=m.heap};copy:a");
    rejects(
        "a:=m.heap;load<m.Allocator>:(p<&m.Allocator>){->*p};result:{->slot:m.heap;a=load(&slot)};copy:a",
        "E303",
    );
    accepts(
        "a:=m.heap;load<m.Allocator>:(p<&m.Allocator>){->*p};result:{->slot:m.heap;a=load(&slot)};a=m.heap;copy:a",
    );
    accepts(
        "load<m.Allocator>:(p<&m.Allocator>){->*p};'outer{->slot:m.heap;a:=m.heap;n:=2;'loop{a=load(&slot);n=n-1;|n>0|'loop.restart()};copy:a}",
    );
}

#[test]
pub(crate) fn optional_allocator_header_bounds_disappear_only_after_pointer_overwrite() {
    accepts(
        "a<m.Allocator><null>:f(&1);b<m.Allocator><null>:null;p:=&a;again:=true;'loop{p=&b;copy:*p;|again|{again=false;'loop.restart()}};copy:*p",
    );
    rejects(
        "a<m.Allocator><null>:f(&1);b<m.Allocator><null>:null;p:=&a;again:=true;'loop{old:*p;p=&b;|again|{again=false;'loop.restart()}}",
        "E303",
    );
}
