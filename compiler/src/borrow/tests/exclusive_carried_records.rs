pub(crate) fn with_lists(source: &str) -> String {
    source
        .replace("<{n<", "<{items<int32[2]>:=;n<")
        .replace("->n:", "->items:=[1];->n:")
}

pub(crate) fn accepts(source: &str) {
    super::accepts(source);
    super::accepts(&with_lists(source));
}

pub(crate) fn rejects(source: &str, code: &str) {
    super::rejects(source, code);
    super::rejects(&with_lists(source), code);
}

pub(crate) const PREFIX: &str = "<Inner>:<{n<int32>:=;other<int32>:=}>;<Row>:<{inner<Inner>:=;flag<boolean>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->inner:={->n:=7;->other:=1};->flag:=true};";
pub(crate) const SUFFIX: &str = "first=false;'loop.restart()}}}";

#[test]
pub(crate) fn exclusive_carried_record_fields_mutate_nested_storage_and_end_before_restart() {
    accepts(&format!(
        "{PREFIX}old:row;s:&(old.inner.n);p:&!(row.inner.n);q:&!(row.inner.other);*p=8;*q=9;v:*s;b:&!(row.flag);*b=false;{SUFFIX}"
    ));
}

#[test]
pub(crate) fn exclusive_carried_record_fields_keep_moves_children_and_calls_local() {
    accepts(&format!(
        "bump<null>:(p<&!int32>){{*p=*p+1}};{PREFIX}p:&!(row.inner.n);s:&*p;v:*s;q:&!*p;*q=8;*p=9;moved:p;bump(moved);{SUFFIX}"
    ));
    rejects(
        &format!("{PREFIX}p:&!(row.inner.n);q:p;v:*p;w:*q;{SUFFIX}"),
        "E301",
    );
}

#[test]
pub(crate) fn exclusive_carried_record_fields_preserve_conflicts_and_final_use() {
    for body in [
        "p:&!(row.inner.n);row.inner.n=8;v:*p",
        "p:&!(row.inner.n);row.inner={->n:=8;->other:=2};v:*p",
        "p:&!(row.inner.n);v:row;w:*p",
        "p:&!(row.inner.n);q:&!(row.inner.n);v:*p;w:*q",
        "p:&!(row.inner.n);s:&*p;*p=8;v:*s",
        "p:&!(row.inner.n);s:&row;v:*p;w:s.inner.n",
    ] {
        rejects(&format!("{PREFIX}{body};{SUFFIX}"), "E302");
    }
    accepts(&format!(
        "{PREFIX}p:&!(row.inner.n);row.inner.other=9;v:*p;row.inner.n=8;q:&!(row.inner.n);*q=10;row={{->inner:={{->n:=11;->other:=2}};->flag:=false}};{SUFFIX}"
    ));
}

#[test]
pub(crate) fn exclusive_carried_record_fields_reject_live_backedge_descendants() {
    for (borrow, name) in [
        ("p:&!(row.inner.n)", "p"),
        ("p:&!(row.inner.n);s:&*p", "s"),
        ("p:&!(row.inner.n);s:&!*p", "s"),
        ("p:&!(row.inner.n);s:keep(p)", "s"),
    ] {
        rejects(
            &format!(
                "keep<&int32>:(p<&!int32>){{->&*p}};{PREFIX}{borrow};i:=0;'again{{v:*{name};i=i+1;|i<2|'again.restart()}};{SUFFIX}"
            ),
            "B001",
        );
    }
    for value in ["&*p", "keep(p)"] {
        rejects(
            &format!(
                "keep<&int32>:(p<&!int32>){{->&*p}};x:1;s:=&x;{PREFIX}p:&!(row.inner.n);s={value};first=false;'loop.restart()}};v:*s}}}}"
            ),
            "B001",
        );
    }
}

#[test]
pub(crate) fn exclusive_carried_record_fields_keep_shared_headers_and_old_copies() {
    accepts(&format!(
        "x:1;s:=&x;old:s;{PREFIX}s=&(row.inner.other);p:&!(row.inner.n);*p=8;first=false;'loop.restart()}};v:*s;w:*old}}}};s=&x;v:*s"
    ));
    accepts(&format!(
        "x:1;s:=&x;{PREFIX}p:&!(row.inner.n);s=&*p;v:*s;s=&x;{SUFFIX};v:*s"
    ));
}

#[test]
pub(crate) fn exclusive_carried_record_fields_keep_owner_reset_leave_and_expiry_rules() {
    accepts(
        "<Row>:<{n<int32>:=}>;<R>:<{row<Row>:=}>;i:=0;r<R>:'out{first:=true;'loop{|first|{'out->row:={->n:=i};p:&!(row.n);*p=*p+1;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
    );
    accepts(
        "<Row>:<{n<int32>:=}>;<R>:<{row<Row>:=}>;run<null>:(stop<boolean>){first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=3};p:&!(row.n);*p={|stop|'out.leave();->9};first=false;'loop.restart()}}}}",
    );
    rejects(
        &format!("x:1;s:=&x;{PREFIX}p:&!(row.inner.n);s=&*p;{SUFFIX};v:*s"),
        "E303",
    );
}

#[test]
pub(crate) fn exclusive_carried_record_fields_keep_mutability_shapes_and_boolean_gates() {
    for body in [
        "p:&!row",
        "p:&!(row.inner)",
        "p:&!(row.inner.n);first=false;*p=8",
    ] {
        let suffix = if body.contains("first=false") {
            "'loop.restart()}}}"
        } else {
            SUFFIX
        };
        rejects(&format!("{PREFIX}{body};{suffix}"), "B001");
    }
    for (ty, value) in [
        ("string", "\"text\""),
        ("null", "null"),
        ("int32[2]", "[1,2]"),
    ] {
        rejects(
            &format!(
                "<Row>:<{{n<{ty}>:=}}>; <R>:<{{row<Row>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->row:={{->n:={value}}};p:&!(row.n);first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:7};p:&!(row.n);first=false;'loop.restart()}}}",
        "E305",
    );
}

pub(crate) const LIST_PREFIX: &str = "<Inner>:<{n<int32>:=;items<int32[3]>:=}>;<Row>:<{inner<Inner>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->inner:{->n:=7;->items:=[1]}};";

#[test]
pub(crate) fn exclusive_carried_record_fields_allow_list_sibling_replacement_and_shared_headers() {
    super::accepts(&format!(
        "{LIST_PREFIX}old:row;p:row.inner.&!n;row.inner.items=[2,3];s:row.inner.&items;*p=8;v:s[2];w:old.inner.items[1];{SUFFIX}"
    ));
    super::accepts(&format!(
        "seed<int32[3]>:[0];s:=&seed;old:s;{LIST_PREFIX}s=row.inner.&items;p:row.inner.&!n;*p=8;first=false;'loop.restart()}};v:s[1];w:old[1]}}}};s=&seed"
    ));
    for body in [
        "p:row.inner.&!n;s:row.inner.&items;row.inner.items=[2];v:s[1];w:*p",
        "s:&row;p:row.inner.&!n;*p=8;v:s.inner.items[1]",
        "p:row.inner.&!n;s:&!*p;v:*p;*s=8",
    ] {
        super::rejects(&format!("{LIST_PREFIX}{body};{SUFFIX}"), "E302");
    }
}

#[test]
pub(crate) fn exclusive_carried_record_fields_keep_list_paths_and_permissions_gated() {
    for body in [
        "p:row.inner.&!items",
        "p:&!(row.inner.items[1])",
        "row.inner.items[1]=9",
    ] {
        super::rejects(&format!("{LIST_PREFIX}{body};{SUFFIX}"), "B001");
    }
    super::rejects(
        &format!("{LIST_PREFIX}p:row.inner.&!n;*p=8;{SUFFIX}")
            .replace("n<int32>:=", "n<int32>")
            .replace("->n:=7", "->n:7"),
        "E305",
    );
    super::rejects(
        &format!("{LIST_PREFIX}s:&row;p:s.inner.&!n;{SUFFIX}"),
        "B001",
    );
}
