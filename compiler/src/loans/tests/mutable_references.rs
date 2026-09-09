use super::{accepts, rejects};

#[test]
pub(crate) fn reference_assignments_replace_future_origins_and_preserve_copies() {
    accepts("a:=1;b:=2;p:=&a;p=&b;a=3;v:*p");
    accepts("a:=1;b:=2;p:=&a;old:p;p=&b;v:*old;a=3;w:*p");
    rejects("a:=1;b:=2;p:=&a;old:p;p=&b;a=3;v:*old", "E302");
    rejects("a:=1;b:=2;p:=&a;p=&b;b=3;v:*p", "E302");
    accepts("a:=1;b:=2;p:=&a;p={a=3;->&b};v:*p");
    accepts("a:=1;p:=&a;p=p;v:*p;a=2");
}

#[test]
pub(crate) fn reference_operands_keep_their_version_through_rhs_assignment() {
    accepts("a:=1;b:=2;p:=&a;same:p=={p=&b;->&a};a=3;v:*p");
    rejects("a:=1;b:=2;p:=&a;same:p=={p=&b;a=3;->&b}", "E302");
    rejects("a:=1;b:=2;p:=&a;pair:{->old:p;p=&b;a=3;->new:p}", "E302");
    rejects(
        "a:=1;b:=2;p:=&a;pair:{->old:p;p=&b;->new:p};a=3;v:*(pair.old)",
        "E302",
    );
}

#[test]
pub(crate) fn reference_cell_borrows_block_stores_until_their_final_use() {
    rejects("a:1;b:2;p:=&a;cell:&p;p=&b;v:**cell", "E302");
    rejects("a:1;b:2;p:=&a;cell:&p;p=&b;same:cell==&p", "E302");
    accepts("a:1;b:2;p:=&a;cell:&p;v:**cell;p=&b;w:*p");
    accepts("a:1;b:2;p:=&a;old:*(&p);p=&b;v:*old;w:*p");
    accepts("a:1;p:=&a;cell:&p;p=*cell;v:*p");
    rejects(
        "load<&int32>:(cell<& &int32>){->*cell};a:1;p:=&a;p=load(&p);v:*p",
        "E302",
    );
    rejects(
        "load<&int32>:(cell<& &int32>){->*cell};a:1;b:2;p:=&a;old:load(&p);p=&b;v:*old",
        "E302",
    );
    accepts("load<&int32>:(cell<& &int32>){->*cell};a:1;b:2;p:=&a;old:load(&p);v:*old;p=&b;w:*p");
}

#[test]
pub(crate) fn reference_versions_preserve_transitive_contents_and_all_input_bounds() {
    accepts("a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;p=&right;a=3;v:*(p.view)");
    rejects(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;old:p;p=&right;a=3;v:*(old.view)",
        "E302",
    );
    rejects(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;p=&right;b=3;v:*(p.view)",
        "E302",
    );
    accepts(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};p:=&left;old:p;p=&right;a=3;same:old==&left;v:*(p.view)",
    );
    rejects(
        "first<&int32>:(p<&int32>,other<&string>){->p};a:1;b:2;s:=\"old\";p:=first(&a,&s);old:p;p=&b;s=\"new\";v:*old",
        "E302",
    );
    accepts(
        "first<&int32>:(p<&int32>,other<&string>){->p};a:1;b:2;s:=\"old\";p:=first(&a,&s);p=&b;s=\"new\";v:*p",
    );
}
