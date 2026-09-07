use super::{accepts, rejects};

#[test]
pub(crate) fn forward_leaves_merge_target_versions_and_skip_unfinished_stores() {
    for source in [
        "a:1;b:2;p:=&a;'out{p=&b;'out.leave()};value:*p",
        "a:1;b:2;c:3;p:=&a;'out{p={p=&b;'out.leave();->&c}};value:*p",
        "a:1;b:2;c:3;p:=&a;'out{'inner{p=&b;'out.leave()};p=&c};value:*p",
        "f<null>:(flag<boolean>){a:=1;b:=2;p:=&a;'out{|flag|{p=&b;'out.leave()};p=&a};|flag|{a=3;value:*p};|!flag|{b=3;value:*p}}",
        "d:@\"debug\";f<null>:(flag<boolean>){a:1;b:2;p:=&a;'out{|flag|{p=&b;'out.leave()};d.panic(\"stop\")};value:*p}",
    ] {
        accepts(source);
    }
    rejects(
        "f<null>:(flag<boolean>){a:=1;b:2;p:=&a;'out{|flag|{p=&b;'out.leave()}};a=3;value:*p}",
        "E302",
    );
}

#[test]
pub(crate) fn leave_snapshots_preserve_expiry_without_reading_old_versions() {
    for source in [
        "a:1;b:2;p:=&a;'out{local:3;p=&local;'out.leave()};p=&b;value:*p",
        "a:1;p:=&a;'out{p=&1;'out.leave()};p=&a;value:*p",
        "a:=1;b:2;p:=&a;'out{a=3;'out.leave()};p=&b;value:*p",
        "a:1;p:=&a;record:'out{->n:=2;p=&n;'out.leave()};p=&a;value:*p",
    ] {
        accepts(source);
    }
    for source in [
        "a:1;p:=&a;'out{local:3;p=&local;'out.leave()};value:*p",
        "a:1;p:=&a;'out{p=&1;'out.leave()};value:*p",
        "a:1;p:=&a;record:'out{->n:=2;p=&n;'out.leave()};value:*p",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub(crate) fn leave_result_proofs_keep_disjoint_emissions_live() {
    accepts("f<int32>:(flag<boolean>)'out{a:1;b:2;p:=&a;|flag|{p=&b;'out->*p;'out.leave()};->*p}");
    accepts(
        "choose<&int32>:(flag<boolean>,a<&int32>,b<&int32>)'out{p:=a;|flag|{p=b;'out->p;'out.leave()};->p}",
    );
    rejects(
        "f<int32>:(flag<boolean>){a:=1;b:=2;p:=&a;value:'out{|flag|{p=&b;'out->p;'out.leave()};->p};a=3;b=3;->*value}",
        "E302",
    );
}

#[test]
pub(crate) fn reference_assignment_backedges_preserve_unread_expiry() {
    accepts("a:1;p:=&a;'loop{local:2;p=&local;'loop.restart()}");
    accepts("a:1;p:=&a;p=&1;'loop{'loop.restart()}");
    accepts("unused<null>:()'loop{'loop.restart()};a:1;b:2;p:=&a;'out{p=&b;'out.leave()};value:*p");
}
