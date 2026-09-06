use super::{accepts, rejects};

#[test]
pub(crate) fn emitted_slot_borrows_share_canonical_writes_and_restart_liveness() {
    for source in [
        "r:{->n:=1;p:&n;v:*p;n=2}",
        "r:{->row:={->left:=1;->right:=2};p:&row.left;row.right=3;v:*p}",
        "r:{->items:=[1,2];p:&items[1];items[2]=*p+1}",
        "i:=0;r:'again{->n:=1;p:&n;v:*p;i=i+1;|i<2|'again.restart()}",
    ] {
        accepts(source);
    }
    for source in [
        "r:{->n:=1;p:&n;n=2;v:*p}",
        "r:{->items:=[1,2];p:&items[1];items[2]=3;v:*p}",
        "f<null>:(flag<boolean>){r:'out{|flag|{'out->n:=1};|!flag|{'out->n:=2;p:&n;n=3;v:*p}}}",
        "r:{->n:=1;p:&n;i:=0;'loop{v:*p;n=2;i=i+1;|i<2|'loop.restart()}}",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub(crate) fn alias_mutations_publish_unknown_activity_without_losing_sibling_origins() {
    for source in [
        "r:{->tag<int32><null>:=null;tag=1};a:=1;p:{|r.tag<int32>|->view:&a};a=2;|p.view<&int32>|{v:*p.view<&int32>}",
        "r:{->row:={->tag<int32><null>:=null};row.tag=1};a:=1;p:{|r.row.tag<int32>|->view:&a};a=2;|p.view<&int32>|{v:*p.view<&int32>}",
        "a:=1;r:{->view:&a;->tag<int32><null>:=null;tag=1};a=2;v:*r.view",
    ] {
        rejects(source, "E302");
    }
    accepts("a:=1;r:{->view:&a;->tag<int32><null>:=null;tag=1};v:*r.view;a=2");
    rejects("r:{->items:=[1,2];items[{items=[3,4];->1}]=5}", "E302");
    accepts("r:{->items:=[1,2];items[1]=items[2]+1}");
}
