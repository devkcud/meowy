use super::{accepts, rejects};

#[test]
pub(crate) fn restart_headers_preserve_initial_sources_before_entry_without_adding_reads() {
    accepts("a:=1;b:=2;p:=&a;b=3;i:=0;'again{v:*p;p=&b;i=i+1;|i<2|'again.restart()};w:*p");
    accepts("a:=1;b:2;p:=&a;i:=0;'again{a=3;p=&b;i=i+1;|i<2|'again.restart()}");
    accepts("a:=1;b:2;p:=&a;i:=0;'again{p=&b;a=3;v:*p;i=i+1;|i<2|'again.restart()}");
    accepts("a:=1;b:=2;p:=&a;|false|'again{p=&b;'again.restart()};b=3;v:*p");
}

#[test]
pub(crate) fn restart_headers_check_first_and_later_iteration_owner_conflicts() {
    rejects(
        "a:=1;b:=2;p:=&a;i:=0;'again{a=3;v:*p;p=&b;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    rejects(
        "a:=1;b:=2;p:=&a;i:=0;'again{b=3;v:*p;p=&b;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    rejects(
        "a:=1;b:=2;p:=&b;flag:=false;i:=0;'again{|flag|{a=3;v:*p};|!flag|p=&a;flag=true;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    rejects(
        "check<null>:(flag<boolean>){a:=1;b:2;p:=&a;|flag|'again{a=3;v:*p;p=&b;|flag|'again.restart()}}",
        "E302",
    );
}

#[test]
pub(crate) fn restart_versions_keep_old_copies_cell_loans_and_public_bounds() {
    rejects(
        "a:=1;b:2;p:=&a;old:p;i:=0;'again{p=&b;i=i+1;|i<2|'again.restart()};a=3;v:*old",
        "E302",
    );
    rejects(
        "a:1;b:2;p:=&a;cell:&p;i:=0;'again{p=&b;i=i+1;|i<2|'again.restart()};v:**cell",
        "E302",
    );
    accepts("a:1;b:2;p:=&a;cell:&p;v:**cell;i:=0;'again{p=&b;i=i+1;|i<2|'again.restart()};w:*p");
    rejects(
        "first<&int32>:(p<&int32>,s<&string>){->p};a:1;b:2;s:=\"old\";p:=first(&a,&s);old:p;i:=0;'again{p=&b;i=i+1;|i<2|'again.restart()};s=\"new\";v:*old",
        "E302",
    );
    accepts(
        "first<&int32>:(p<&int32>,s<&string>){->p};a:1;b:2;s:=\"old\";p:=first(&a,&s);i:=0;'again{p=&b;i=i+1;|i<2|'again.restart()};s=\"new\";v:*p",
    );
}

#[test]
pub(crate) fn restart_fixed_points_propagate_rotating_reference_sources() {
    accepts(
        "a:1;b:2;c:3;d:4;p:=&b;q:=&c;r:=&d;i:=0;'again{v:*p;p=q;q=r;r=&a;i=i+1;|i<4|'again.restart()};w:*p",
    );
    rejects(
        "a:=1;b:2;c:3;d:4;p:=&b;q:=&c;r:=&d;i:=0;'again{a=5;v:*p;p=q;q=r;r=&a;i=i+1;|i<4|'again.restart()}",
        "E302",
    );
    accepts(
        "<Row>:<{value<int32>}>;a<Row>:{->value:1};b<Row>:{->value:2};p:=&a;i:=0;'again{v:p.value;p=&b;i=i+1;|i<2|'again.restart()};w:p.value",
    );
    accepts("a:[1,2];b:[3,4];p:=&a;i:=0;'again{v:p[1];p=&b;i=i+1;|i<2|'again.restart()};w:p[2]");
}

#[test]
pub(crate) fn nested_restarts_use_their_own_headers_and_keep_completed_rhs_updates() {
    accepts(
        "a:1;b:2;c:3;p:=&a;i:=0;j:=0;'outer{j=0;'inner{v:*p;p=&b;j=j+1;|j<2|'inner.restart()};p=&c;i=i+1;|i<2|'outer.restart()};w:*p",
    );
    accepts(
        "a:1;b:2;c:3;p:=&a;i:=0;'outer{'inner{p=&b;i=i+1;|i<2|'outer.restart()};p=&c;'outer.leave()};v:*p",
    );
    accepts(
        "a:1;b:2;c:3;p:=&a;i:=0;'again{p={p=&b;i=i+1;|i<2|'again.restart();->&c};'again.leave()};v:*p",
    );
    rejects(
        "a:=1;b:=2;p:=&a;i:=0;'outer{'inner{b=3;v:*p;p=&b;i=i+1;|i<2|'outer.restart()};'outer.leave()}",
        "E302",
    );
}

#[test]
pub(crate) fn restarted_carried_sources_cannot_reuse_iteration_storage() {
    rejects(
        "a:1;p:=&a;'again{local:2;p=&local;'again.restart()}",
        "B001",
    );
    rejects(
        "a:1;p:=&a;'again{->local:2;p=&local;'again.restart()}",
        "B001",
    );
    rejects("a:1;p:=&a;'again{p=&2;'again.restart()}", "B001");
    rejects(
        "a:1;holder:{->view<&int32><null>:&a};p:=&holder;'again{p=&holder;'again.restart()}",
        "B001",
    );
    accepts("a:1;i:=0;'again{local:2;p:=&local;p=&a;v:*p;i=i+1;|i<2|'again.restart()}");
}
