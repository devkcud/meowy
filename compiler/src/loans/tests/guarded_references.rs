use super::{accepts, rejects};

pub(crate) fn program(body: &str) -> String {
    format!("check<null>:(flag<boolean>,other<boolean>){{{body}}}")
}

#[test]
pub(crate) fn guarded_versions_only_protect_the_selected_owner() {
    accepts(&program(
        "a:=1;b:=2;p:=&a;|flag|p=&b;|flag|a=3;|!flag|b=4;v:*p",
    ));
    rejects(
        &program("a:=1;b:=2;p:=&a;|flag|p=&b;|flag|b=3;v:*p"),
        "E302",
    );
    rejects(
        &program("a:=1;b:=2;p:=&a;|flag|p=&b;|!flag|a=3;v:*p"),
        "E302",
    );
    accepts(&program(
        "a:=1;b:=2;c:=3;p:=&a;|flag|{|other|p=&b;|!other|p=&c};|flag|a=4;|!flag|{b=5;c=6};v:*p",
    ));
    rejects(
        &program("a:=1;b:=2;c:=3;p:=&a;|flag|{|other|p=&b;|!other|p=&c};|flag&&other|b=4;v:*p"),
        "E302",
    );
}

#[test]
pub(crate) fn guarded_merges_preserve_captured_values_and_cell_loans() {
    accepts(&program(
        "a:=1;b:=2;p:=&a;old:p;|flag|p=&b;|flag|a=3;|!flag|v:*old;w:*p",
    ));
    rejects(
        &program("a:=1;b:=2;p:=&a;old:p;|flag|p=&b;|flag|a=3;v:*old"),
        "E302",
    );
    accepts(&program(
        "a:1;b:2;p:=&a;cell:&p;|flag|p=&b;|!flag|v:**cell;w:*p",
    ));
    rejects(
        &program("a:1;b:2;p:=&a;cell:&p;|flag|p=&b;|flag|v:**cell"),
        "E302",
    );
    accepts(&program("a:1;p:=&a;cell:&p;|flag|p=*cell;v:*p"));
    rejects(
        &program("a:=1;b:=2;p:=&a;same:p=={|flag|p=&b;a=3;->p}"),
        "E302",
    );
}

#[test]
pub(crate) fn merge_bookkeeping_has_no_reads_and_excludes_nonreturning_arms() {
    accepts(&program("a:=1;b:=2;p:=&a;|flag|p=&b;|!flag|a=3"));
    accepts(&program("a:=1;b:=2;p:=&a;|flag|{a=3;p=&b};a=4;b=5"));
    accepts(&program(
        "d:@\"debug\";a:=1;b:=2;p:=&a;|flag|{p=&b;d.panic(\"stop\")};b=3;v:*p",
    ));
    accepts(&program(
        "d:@\"debug\";a:=1;b:=2;p:=&a;|flag|{|other|p=&b;|other|d.panic(\"stop\")};b=3;v:*p",
    ));
    rejects(
        "dead<&int32>:(s<&string>){->dead(s)};check<null>:(flag<boolean>){s:\"s\";a:=1;p:=&a;|flag|p=dead(&s);a=2;v:*p}",
        "E302",
    );
}

#[test]
pub(crate) fn short_circuit_joins_keep_skipped_and_returning_versions_separate() {
    for (op, active, skipped) in [("&&", "flag", "!flag"), ("||", "!flag", "flag")] {
        accepts(&program(&format!(
            "a:=1;b:=2;p:=&a;v:flag{op}{{p=&b;->true}};|{active}|a=3;|{skipped}|b=4;w:*p"
        )));
        rejects(
            &program(&format!(
                "a:=1;b:=2;p:=&a;v:flag{op}{{p=&b;->true}};|{active}|b=3;w:*p"
            )),
            "E302",
        );
        accepts(&program(&format!(
            "d:@\"debug\";a:=1;b:=2;p:=&a;v:flag{op}{{p=&b;d.panic(\"stop\")}};b=3;w:*p"
        )));
    }
    accepts("a:=1;b:=2;p:=&a;v:false&&{p=&b;->true};b=3;w:*p");
    accepts("a:=1;b:=2;p:=&a;v:true||{p=&b;->false};b=3;w:*p");
}

#[test]
pub(crate) fn guarded_projected_versions_preserve_summary_paths_and_public_bounds() {
    accepts(&program(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};pair:{->first:&left;->second:&right};p:=pair.first;|flag|p=pair.second;|flag|a=3;|!flag|b=4;v:*(p.view)",
    ));
    rejects(
        &program(
            "a:=1;b:=2;left:{->view:&a};right:{->view:&b};pair:{->first:&left;->second:&right};p:=pair.first;|flag|p=pair.second;|flag|b=3;v:*(p.view)",
        ),
        "E302",
    );
    accepts(&program(
        "a:=1;b:=2;left:{->view:&a};right:{->view:&b};pair:{->first:&left;->second:&right};p:=pair.first;|flag|p=pair.second;a=3;b=4;same:p==p",
    ));
    rejects(
        "first<&int32>:(p<&int32>,s<&string>){->p};check<null>:(flag<boolean>){a:1;b:2;s:=\"s\";p:=first(&a,&s);|flag|p=&b;|!flag|s=\"new\";v:*p}",
        "E302",
    );
    accepts(
        "first<&int32>:(p<&int32>,s<&string>){->p};check<null>:(flag<boolean>){a:1;b:2;s:=\"s\";p:=first(&a,&s);|flag|p=&b;|flag|s=\"new\";v:*p}",
    );
}

#[test]
pub(crate) fn repeated_joins_normalize_origins_and_obey_the_loan_work_budget() {
    let source = |count| program(&format!("a:1;p:=&a;{}v:*p", "|flag|p=p;".repeat(count)));
    accepts(&source(64));
    let errors = crate::compile(&source(1024)).unwrap_err();
    assert_eq!(errors[0].code, "B001", "{errors:?}");
    assert!(
        errors[0].message.contains("loan-analysis budget"),
        "{errors:?}"
    );
}
