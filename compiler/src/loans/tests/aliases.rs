use super::{accepts, rejects};

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
