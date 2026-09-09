use super::{accepts, rejects};

#[test]
pub(crate) fn transitive_headers_keep_initial_and_backedge_component_sources_separate() {
    accepts(
        "a:=1;b:=2;left:{->view:&a};p:=&left;b=3;right:{->view:&b};i:=0;'again{v:*(p.view);p=&right;i=i+1;|i<2|'again.restart()};w:*(p.view)",
    );
    accepts(
        "a:=1;b:=2;first:&a;p:=&first;b=3;second:&b;i:=0;'again{v:**p;p=&second;i=i+1;|i<2|'again.restart()};w:**p",
    );
    rejects(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;i:=0;'again{a=3;v:*(p.view);p=&right;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    rejects(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;i:=0;'again{b=3;v:*(p.view);p=&right;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    rejects(
        "a:=1;b:=2;first:&a;second:&b;p:=&first;i:=0;'again{b=3;v:**p;p=&second;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
}

#[test]
pub(crate) fn transitive_headers_only_keep_demanded_pointee_loans_live() {
    accepts(
        "a:=1;b:=2;left:{->view:&a;->count:3};right:{->view:&b;->count:4};p:=&left;i:=0;'again{a=5;b=6;copy:p;same:copy==p;v:p.count;p=&right;i=i+1;|i<2|'again.restart()}",
    );
    accepts(
        "a:=1;cell:&a;p:=&cell;i:=0;'again{a=2;same:p==&cell;p=&cell;i=i+1;|i<2|'again.restart()}",
    );
    rejects(
        "a:=1;cell:&a;p:=&cell;i:=0;'again{a=2;copy:*p;p=&cell;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    rejects(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;i:=0;'again{a=3;b=4;copy:*p;p=&right;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    accepts(
        "d:@\"debug\";a:=1;b:=2;left:{->5;->view:&a};right:{->6;->view:&b};p:=&left;i:=0;'again{a=3;b=4;d.print(*p);p=&right;i=i+1;|i<2|'again.restart()}",
    );
}

#[test]
pub(crate) fn repeated_sources_keep_distinct_field_and_dereference_paths() {
    accepts(
        "a:=1;b:=2;left:{->x:&a;->y:&a};right:{->x:&b;->y:&a};pair:{->first:&left;->second:&right};p:=pair.first;i:=0;'again{b=3;v:*(p.y);p=pair.second;i=i+1;|i<2|'again.restart()};w:*(p.y)",
    );
    rejects(
        "a:=1;b:=2;left:{->x:&a;->y:&a};right:{->x:&b;->y:&a};pair:{->first:&left;->second:&right};p:=pair.first;i:=0;'again{a=3;v:*(p.y);p=pair.second;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    accepts(
        "a:1;first:&a;second:&first;p:=&second;i:=0;'again{v:***p;p=&second;i=i+1;|i<2|'again.restart()};w:***p",
    );
    accepts(
        "a:1;b:2;c:3;first:&a;second:&b;third:&c;p:=&first;q:=&second;i:=0;'again{v:**p;p=q;q=&third;i=i+1;|i<3|'again.restart()};w:**p",
    );
}

#[test]
pub(crate) fn transitive_restart_copies_preserve_cell_and_nested_public_bounds() {
    rejects(
        "a:=1;b:2;left:{->view:&a};right:{->view:&b};p:=&left;old:p;i:=0;'again{p=&right;i=i+1;|i<2|'again.restart()};a=3;v:*(old.view)",
        "E302",
    );
    rejects(
        "a:1;b:2;left:{->view:&a};right:{->view:&b};p:=&left;cell:&p;i:=0;'again{p=&right;i=i+1;|i<2|'again.restart()};same:cell==&p",
        "E302",
    );
    accepts(
        "a:1;b:2;left:{->view:&a};right:{->view:&b};p:=&left;old:*(&p);i:=0;'again{p=&right;i=i+1;|i<2|'again.restart()};v:*(old.view);w:*(p.view)",
    );
    rejects(
        "<C>:<{view<&int32>}>;id<&C>:(p<&C>,s<&string>){->p};a:1;b:2;s:=\"old\";left<C>:{->view:&a};right<C>:{->view:&b};p:=id(&left,&s);i:=0;'again{copy:*p;s=\"new\";v:*(copy.view);p=&right;i=i+1;|i<2|'again.restart()}",
        "E302",
    );
    accepts(
        "<C>:<{view<&int32>}>;id<&C>:(p<&C>,s<&string>){->p};a:1;b:2;s:=\"old\";left<C>:{->view:&a};right<C>:{->view:&b};p:=id(&left,&s);i:=0;'again{copy:*p;s=\"new\";p=&right;i=i+1;|i<2|'again.restart()};w:*(p.view)",
    );
}

#[test]
pub(crate) fn nested_transitive_restarts_keep_exact_targets_and_later_loans() {
    accepts(
        "a:1;b:2;left:{->view:&a};right:{->view:&b};p:=&left;i:=0;j:=0;'outer{j=0;'inner{v:*(p.view);p=&right;j=j+1;|j<2|'inner.restart()};p=&left;i=i+1;|i<2|'outer.restart()};w:*(p.view)",
    );
    rejects(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;i:=0;'outer{'inner{b=3;v:*(p.view);p=&right;i=i+1;|i<2|'outer.restart()};'outer.leave()}",
        "E302",
    );
    accepts(
        "a:1;b:2;left:{->view:&a};right:{->view:&b};p:=&left;i:=0;'again{p={p=&right;i=i+1;|i<2|'again.restart();->&left};'again.leave()};v:*(p.view)",
    );
}

#[test]
pub(crate) fn transitive_restart_headers_keep_activity_and_source_lifetime_boundaries() {
    accepts(
        "value<int32><null>:1;cell:&value;p:=&cell;i:=0;'again{v:**p;p=&cell;i=i+1;|i<2|'again.restart()}",
    );
    accepts("a:1;holder:{->view<&int32><null>:&a};p:=&holder;'again{p=&holder;'again.restart()}");
    accepts(
        "a:1;holder:{->tag<int32><null>:null;->view:&a};p:=&holder;'again{p=&holder;'again.restart()}",
    );
    accepts("cell:&1;p:=&cell;'again{p=&cell;'again.restart()}");
    accepts("a:1;first:&a;p:=&first;'again{local:&a;p=&local;'again.restart()}");
    accepts(
        "a:1;holder:{->view:&a};p:=&holder;|false|'again{p=&holder;'again.restart()};v:*(p.view)",
    );
    accepts(
        "<C>:<{view<&int32>}>;d:@\"debug\";stop<&C>:(){d.panic(\"stop\")};run<null>:(flag<boolean>){a:1;holder<C>:{->view:&a};p:=&holder;i:=0;'again{|flag|{p=stop();'again.restart()};p=&holder;i=i+1;|i<2|'again.restart()};v:*(p.view)}",
    );
}
