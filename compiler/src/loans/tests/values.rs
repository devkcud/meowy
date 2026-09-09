use super::{accepts, rejects};

#[test]
pub(crate) fn projections_only_read_the_selected_record_components() {
    accepts("a:=1;b:=2;r:{->left:&a;->right:&b};b=3;x:*(r.left)");
    accepts("a:=1;r:{->count:3;->view:&a};a=2;x:r.count");
    accepts("a:=1;b:=2;r:{->left:{->view:&a};->right:{->view:&b}};b=3;x:*(r.left.view)");
    accepts("a:=1;r:{->5;->view:&a};a=2;d:@\"debug\";d.print(r);x:r+1;number<int32>:r");
    rejects(
        "a:=1;b:=2;r:{->left:&a;->right:&b};b=3;copy:r;x:*(copy.left)",
        "E302",
    );
    rejects("a:=1;b:=2;r:{->left:&a;->right:&b};a=3;x:*(r.left)", "E302");
}

#[test]
pub(crate) fn record_operands_and_result_slots_hold_all_copied_components() {
    accepts("a:=1;b:=2;r:{->left:&a;->right:&b};same:r.left=={b=3;->&a}");
    rejects(
        "a:=1;b:=2;r:{->left:&a;->right:&b};same:r=={a=3;->r}",
        "E302",
    );
    rejects(
        "a:=1;b:2;left:{->view:&a};right:{->view:&b};same:left=={a=3;->right}",
        "E302",
    );
    rejects("a:=1;b:=2;r:{->left:&a;a=3;->right:&b};x:*(r.left)", "E302");
    accepts(
        "a:=1;again:=true;r:'out {|again|{'out->view:&a;a=2;again=false;'out.restart()};->view:&a};x:*(r.view)",
    );
}

#[test]
pub(crate) fn union_tag_inspection_does_not_read_payloads_or_skip_construction() {
    accepts("a:=1;r<&int32><null>:&a;a=2;|r<null>|{}");
    accepts("a:=1;r:{->view<&int32><null>:&a};a=2;|r.view<&int32>|{}");
    accepts("a:=1;r<&int32><null>:&a;a=2;|r<&int32>|{}");
    rejects("a:=1;flag:=true;|({|flag|->&a;a=2})<null>|{}", "E302");
    rejects("a:=1;r<&int32><null>:&a;a=2;copy:r", "E302");
}

#[test]
pub(crate) fn union_extraction_and_retagging_preserve_projection_demand() {
    accepts(
        "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;|r<Row>|{b=3;x:*(r.left)}",
    );
    accepts(
        "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;wide<Row><null><boolean>:r;|wide<Row>|{b=3;x:*(wide.left)}",
    );
    rejects(
        "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;|r<Row>|{b=3;copy:r;x:*(copy.left)}",
        "E302",
    );
    rejects(
        "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;wide<Row><null><boolean>:r;|wide<Row>|{a=3;x:*(wide.left)}",
        "E302",
    );
}
