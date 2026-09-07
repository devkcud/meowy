use super::guarded_references::program;
use super::{accepts, rejects};

#[test]
pub(crate) fn leave_joins_preserve_selected_and_skipped_reference_versions() {
    accepts(&program(
        "a:=1;b:=2;p:=&a;'out{|flag|{p=&b;'out.leave()};p=&a};|flag|a=3;|!flag|b=4;v:*p",
    ));
    rejects(
        &program("a:=1;b:=2;p:=&a;'out{|flag|{p=&b;'out.leave()};p=&a};|flag|b=3;v:*p"),
        "E302",
    );
    rejects(
        &program("a:=1;b:=2;p:=&a;'out{|flag|{p=&b;'out.leave()};p=&a};|!flag|a=3;v:*p"),
        "E302",
    );
    accepts(&program(
        "a:=1;b:=2;p:=&a;'out{|flag|'out.leave();p=&b};|flag|b=3;|!flag|a=4;v:*p",
    ));
    rejects(
        &program(
            "a:=1;b:=2;c:=3;p:=&a;'out{|flag|{p=&b;'out.leave()};|other|{p=&c;'out.leave()};p=&a};|!flag&&other|c=4;v:*p",
        ),
        "E302",
    );
}

#[test]
pub(crate) fn nested_leave_keeps_finished_writes_and_skips_unfinished_outer_stores() {
    accepts(&program(
        "a:=1;b:=2;c:=3;p:=&a;'out{p={p=&b;|flag|'out.leave();->&c}};a=4;|!flag|b=5;v:*p",
    ));
    rejects(
        &program("a:=1;b:=2;c:=3;p:=&a;'out{p={p=&b;|flag|'out.leave();->&c}};|flag|b=4;v:*p"),
        "E302",
    );
    accepts("a:=1;b:=2;p:=&a;'out{'inner{p=&b;'inner.leave()};a=3;v:*p};w:*p");
    rejects(
        "a:=1;b:=2;p:=&a;'out{'inner{p=&b;'inner.leave()};b=3};w:*p",
        "E302",
    );
    rejects(
        "a:=1;b:=2;p:=&a;same:p=='out{p=&b;a=3;->p;'out.leave()}",
        "E302",
    );
    accepts(&program(
        "a:=1;b:=2;c:=3;p:=&a;'out{'inner{|flag|{p=&b;'out.leave()};p=&c};a=4};|!flag|b=5;v:*p",
    ));
}

#[test]
pub(crate) fn leave_snapshots_add_no_reads_and_preserve_existing_copy_and_cell_loans() {
    accepts("a:=1;b:2;p:=&a;'out{a=3;'out.leave()};p=&b;value:*p");
    accepts(&program(
        "a:=1;b:=2;p:=&a;'out{|flag|{p=&b;'out.leave()};a=3};p=&b;v:*p",
    ));
    rejects(
        "a:=1;b:=2;p:=&a;old:p;'out{p=&b;'out.leave()};a=3;v:*old",
        "E302",
    );
    accepts(&program(
        "a:1;b:2;p:=&a;cell:&p;'out{|flag|{p=&b;'out.leave()}};|!flag|v:**cell;w:*p",
    ));
    rejects(
        &program("a:1;b:2;p:=&a;cell:&p;'out{|flag|{p=&b;'out.leave()}};|flag|v:**cell"),
        "E302",
    );
    rejects(
        "load<&int32>:(cell<& &int32>){->*cell};a:1;b:2;p:=&a;old:load(&p);'out{p=&b;'out.leave()};v:*old",
        "E302",
    );
}

#[test]
pub(crate) fn leave_result_slots_and_projected_summaries_keep_their_own_versions() {
    rejects(
        "a:=1;b:=2;p:=&a;row:'out{->old:p;p=&b;'out.leave()};a=3;v:*row.old",
        "E302",
    );
    accepts("a:=1;b:=2;p:=&a;row:'out{{'out->old:p;p=&b;'out.leave()}};v:*row.old;a=3;w:*p");
    accepts(&program(
        "a:=1;b:=2;p:=&a;row:'out{p=&b;|flag|{'out->view:p;'out.leave()};p=&a;->view:p};|flag|a=3;|!flag|b=4;v:*row.view;w:*p",
    ));
    accepts(&program(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};pair:{->first:&left;->second:&right};p:=pair.first;'out{|flag|{p=pair.second;'out.leave()}};|flag|a=3;|!flag|b=4;v:*p.view",
    ));
    rejects(
        &program(
            "a:=1;b:=2;left:{->view:&a};right:{->view:&b};pair:{->first:&left;->second:&right};p:=pair.first;'out{|flag|{p=pair.second;'out.leave()}};|flag|b=3;v:*p.view",
        ),
        "E302",
    );
}

#[test]
pub(crate) fn leave_captures_allow_expired_values_to_be_replaced_without_reading() {
    accepts("a:1;p:=&a;'out{p=&2;'out.leave()};p=&a;v:*p");
    rejects("a:1;p:=&a;'out{p=&2;'out.leave()};v:*p", "E303");
    accepts("a:1;p:=&a;'out{{short:2;p=&short;'out.leave()}};p=&a;v:*p");
    rejects(
        "a:1;p:=&a;'out{{short:2;p=&short;'out.leave()}};v:*p",
        "E303",
    );
    rejects("cell:&1;p:=&cell;'again{p=&cell;'again.restart()}", "B001");
}

#[test]
pub(crate) fn repeated_forward_exits_normalize_surviving_origins() {
    let exits = "|flag|{p=&b;'out.leave()};".repeat(64);
    accepts(&program(&format!(
        "a:=1;b:=2;p:=&a;'out{{{exits}}};|flag|a=3;|!flag|b=4;v:*p"
    )));
}
