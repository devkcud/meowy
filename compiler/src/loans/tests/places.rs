use super::{accepts, rejects};

#[test]
pub(crate) fn mixed_writes_keep_first_list_regions_and_returning_phase_uses() {
    for source in [
        "r:={->items:=[1,2];->other:=3};p:&(r.other);r.items[1]=4;v:*p",
        "r:={->items:=[1,2];->other:=3};r.items[{r.other=4;->1}]=5",
        "r:={->left:=[1,2];->right:=[3,4]};p:&(r.left[1]);r.right[1]=5;v:*p",
        "r:={->items:=[1,2]};p:&(r.items[1]);r.items[2]=*p+1",
        "d:@\"debug\";r:={->rows:=[{->items:=[1,2]}]};r.rows[1].items[{r.rows=[{->items:=[3,4]}];d.panic(\"stop\")}]=5",
        "r:={->rows:=[{->n:=1}]};'out{r.rows[1].n={r.rows=[{->n:=2}];'out.leave()}}",
    ] {
        accepts(source);
    }
    for source in [
        "r:={->items:=[1,2]};p:&(r.items[1]);r.items[2]=3;v:*p",
        "r:=[{->left:=1;->right:=2}];p:&(r[1].left);r[1].right=3;v:*p",
        "r:={->items:=[1,2]};r.items[{r.items=[3,4];->1}]=5",
        "r:={->items:=[1,2]};r.items[1]={r.items=[3,4];->5}",
        "d:@\"debug\";r:={->rows:=[{->items:=[1,2]}]};r.rows[{r.rows=[{->items:=[3,4]}];->1}].items[{d.panic(\"stop\")}]=5",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub(crate) fn nested_store_reservations_cover_each_returning_index() {
    for source in [
        "a:=[[1,2],[3,4]];r:&(a[1][1]);a[2][2]=*r+1",
        "a:=[[1,2],[3,4]];b:=0;a[{b=1;->1}][{b=2;->2}]=3",
        "d:@\"debug\";a:=[[1,2],[3,4]];a[1][{a=[[5,6],[7,8]];d.panic(\"stop\")}]=9",
        "d:@\"debug\";a:=[[1,2],[3,4]];a[1][2]={a=[[5,6],[7,8]];d.panic(\"stop\")}",
        "a:=[[1,2],[3,4]];'out{a[1][{a=[[5,6],[7,8]];'out.leave()}]=9}",
        "a:=[[1,2],[3,4]];|false|a[1][{a=[[5,6],[7,8]];->2}]=9",
    ] {
        accepts(source);
    }
    for source in [
        "a:=[[1,2],[3,4]];r:&(a[1][1]);a[2][2]=5;v:*r",
        "a:=[[1,2],[3,4]];a[{a=[[5,6],[7,8]];->1}][1]=9",
        "a:=[[1,2],[3,4]];a[1][{a=[[5,6],[7,8]];->1}]=9",
        "a:=[[1,2],[3,4]];a[1][1]={a[2][2]=8;->9}",
        "d:@\"debug\";a:=[[1,2],[3,4]];a[{a=[[5,6],[7,8]];->1}][{d.panic(\"stop\")}]=9",
        "d:@\"debug\";a:=[[1,2],[3,4]];a[1][{a=[[5,6],[7,8]];->1}]=d.panic(\"stop\")",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub(crate) fn element_store_reservations_end_at_each_actual_access() {
    for source in [
        "a:=[1,2];r:&(a[1]);a[2]=*r+1",
        "a:=[1,2];r:&a;a[2]=r[1]+1",
        "a:=[1,2];a[2]={r:&(a[1]);->*r+1}",
        "a:=[1,2];b:=[3,4];a[{b[1]=5;->1}]={b[2]=6;->7}",
        "d:@\"debug\";a:=[1,2];a[{a=[3,4];d.panic(\"stop\")}]=5",
        "d:@\"debug\";a:=[1,2];a[1]={a=[3,4];d.panic(\"stop\")}",
        "a:=[1,2];'out{a[1]={a=[3,4];'out.leave()}}",
        "a:=[1,2];'out{a[{a=[3,4];'out.leave()}]=5}",
        "a:=[1,2];|false|a[1]={a=[3,4];->5}",
        "a:=[1,2];i:=0;'loop{r:&(a[1]);a[2]=*r+1;i=i+1;|i<2|'loop.restart()}",
    ] {
        accepts(source);
    }
    for source in [
        "a:=[1,2];r:&(a[1]);a[2]=3;v:*r",
        "a:=[1,2];r:&a;a[2]=3;v:r[1]",
        "a:=[1,2];a[{a=[3,4];->1}]=5",
        "a:=[1,2];a[1]={a=[3,4];->5}",
        "a:=[1,2];a[{a[2]=3;->1}]=5",
        "a:=[1,2];a[1]={a[2]=3;->5}",
        "d:@\"debug\";a:=[1,2];a[{a=[3,4];->1}]=d.panic(\"stop\")",
        "first<&int32>:(a<&int32[2]>,b<&int32[2]>){->&(a[1])};a:=[1,2];b:=[3,4];r:first(&a,&b);b[2]=5;v:*r",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub(crate) fn checked_element_borrows_keep_parent_and_inherited_loans_live() {
    for source in [
        "a:=[1,2];r:&(a[1]);value:*r;a=[3,4]",
        "a:=[1,2];r:&(a[1]);copy:&*r;value:*copy;a=[3,4]",
        "a:=[1,2];'loop{r:&(a[1]);value:*r;a=[3,4]}",
        "d:@\"debug\";a:=[1,2];r:&(a[{a=[3,4];d.panic(\"stop\")}])",
        "a:=[1,2];'out{r:&(a[{a=[3,4];'out.leave()}])}",
        "first<&int32>:(p<&int32[2]>,other<&string>){->&(p[1])};a:=[1,2];other:=\"x\";r:first(&a,&other);value:*r;other=\"y\";a=[3,4]",
    ] {
        accepts(source);
    }
    for source in [
        "a:=[1,2];r:&(a[1]);a=[3,4];value:*r",
        "a:=[1,2];r:&(a[{a=[3,4];->1}])",
        "a:=[1,2];&(a[{a=[3,4];->1}])",
        "a:=[[1,2],[3,4]];r:&(a[1][{a=[[5,6],[7,8]];->1}])",
        "a:=[1,2];r:&(a[1]);same:r==&(a[{a=[3,4];->2}])",
        "first<&int32>:(p<&int32[2]>,other<&string>){->&(p[1])};a:[1,2];other:=\"x\";r:first(&a,&other);other=\"y\";value:*r",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub(crate) fn assignments_resume_after_the_final_reference_use() {
    accepts("a:=1;r:&a;s:r;v:*s;a=2");
    accepts("a:=1;r:&a;a=*r+1");
    accepts("a:=1;r:&a;same:r==&a;a=2");
    rejects("a:=1;r:&a;a=2;v:*r", "E302");
    rejects("a:=1;r:&a;s:r;a=2;v:*s", "E302");
    rejects("a:=1;r:&a;a=*r+1;v:*r", "E302");
}

#[test]
pub(crate) fn whole_record_writes_overlap_borrowed_fields() {
    accepts("a:={->x:1;->y:2};r:&(a.x);v:*r;a={->x:3;->y:4}");
    rejects("a:={->x:1;->y:2};r:&(a.x);a={->x:3;->y:4};v:*r", "E302");
    rejects(
        "a:={->nested:{->x:1}};r:&(a.nested.x);a={->nested:{->x:2}};v:*r",
        "E302",
    );
}
