use super::{accepts, rejects};

pub(crate) const PREFIX: &str =
    "<R>:<{items<int32[3]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[7,8];";
pub(crate) const SUFFIX: &str = "first=false;'loop.restart()}}}";

#[test]
pub(crate) fn exclusive_carried_elements_mutate_and_end_before_restart() {
    accepts(&format!("{PREFIX}p:&!(items[1]);*p=9;{SUFFIX}"));
}

#[test]
pub(crate) fn exclusive_carried_elements_end_before_indexed_writes() {
    accepts(&format!("{PREFIX}p:&!(items[1]);*p=9;items[2]=10;{SUFFIX}"));
}

#[test]
pub(crate) fn exclusive_carried_elements_keep_children_calls_moves_and_last_use() {
    accepts(&format!(
        "bump<null>:(p<&!int32>){{*p=*p+1}};{PREFIX}old:items;p:&!(items[1]);s:&*p;v:*s;q:&!*p;*q=9;*p=10;bump(p);items=[11];w:old[1];{SUFFIX}"
    ));
    rejects(
        &format!("{PREFIX}p:&!(items[1]);q:p;v:*p;w:*q;{SUFFIX}"),
        "E301",
    );
    for body in [
        "p:&!(items[1]);items=[9];v:*p",
        "p:&!(items[1]);v:items[2];w:*p",
        "p:&!(items[1]);q:&!(items[2]);v:*p;w:*q",
        "s:&(items[2]);p:&!(items[1]);v:*s;*p=9",
        "p:&!(items[1]);s:&*p;*p=9;v:*s",
        "p:&!(items[1]);q:&!*p;v:*p;*q=9",
    ] {
        rejects(&format!("{PREFIX}{body};{SUFFIX}"), "E302");
    }
}

#[test]
pub(crate) fn exclusive_carried_elements_reserve_returning_indices_and_keep_cancellation() {
    rejects(
        &format!("{PREFIX}p:&!(items[{{items=[9];->1}}]);{SUFFIX}"),
        "E302",
    );
    accepts(&format!(
        "{PREFIX}'cancel{{p:&!(items[{{items=[9];'cancel.leave()}}])}};{SUFFIX}"
    ));
    accepts(&format!(
        "{PREFIX}p:&!(items[{{s:&items;v:s[1];->1}}]);*p=9;{SUFFIX}"
    ));
    for index in ["0", "-1", "4"] {
        rejects(&format!("{PREFIX}p:&!(items[{index}]);{SUFFIX}"), "E101");
    }
}

#[test]
pub(crate) fn exclusive_carried_elements_keep_nested_fields_and_selected_mutability() {
    accepts(
        "<Row>:<{items<int32[2]>:=;n<int32>:=}>;<R>:<{rows<Row[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->rows:[{->items:=[1];->n:=2}];p:&!(rows[1].items[1]);*p=3;q:rows[1].&!n;*q=4;first=false;'loop.restart()}}}",
    );
    accepts(
        "<Row>:<{items<int32[2][2]>:=;side<int32>:=}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:=[[1]];->side:=2};p:&!(row.items[1][{row.side=3;->1}]);q:row.&!side;*p=4;*q=5;first=false;'loop.restart()}}}",
    );
    rejects(
        &format!("{PREFIX}p:&!(items[1]);{SUFFIX}")
            .replace("items<int32[3]>:=", "items<int32[3]>")
            .replace("->items:=", "->items:"),
        "E305",
    );
    rejects(&format!("{PREFIX}s:&items;p:&!(s[1]);{SUFFIX}"), "B001");
    rejects(&format!("{PREFIX}p:&!items;{SUFFIX}"), "B001");
}

#[test]
pub(crate) fn exclusive_carried_elements_keep_completed_outer_reservations() {
    let prefix =
        "<R>:<{items<int32[2][2]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[[7]];";
    accepts(&format!(
        "{prefix}'cancel{{p:&!(items[1][{{items=[[9]];'cancel.leave()}}])}};{SUFFIX}"
    ));
    rejects(
        &format!(
            "{prefix}'cancel{{p:&!(items[{{items=[[9]];->1}}][{{'cancel.leave()}}])}};{SUFFIX}"
        ),
        "E302",
    );
    rejects(
        &format!("{prefix}p:&!(items[1][{{items=[[9]];->1}}]);{SUFFIX}"),
        "E302",
    );
}

#[test]
pub(crate) fn exclusive_carried_elements_reject_live_reset_ancestry_and_preserve_expiry() {
    for (body, name) in [
        ("p:&!(items[1])", "p"),
        ("p:&!(items[1]);s:&*p", "s"),
        ("p:&!(items[1]);s:&!*p", "s"),
        ("p:&!(items[1]);s:keep(p)", "s"),
    ] {
        rejects(
            &format!(
                "keep<&int32>:(p<&!int32>){{->&*p}};{PREFIX}{body};i:=0;'again{{v:*{name};i=i+1;|i<2|'again.restart()}};{SUFFIX}"
            ),
            "B001",
        );
    }
    rejects(
        &format!("x:1;s:=&x;{PREFIX}p:&!(items[1]);s=&*p;{SUFFIX};v:*s"),
        "E303",
    );
    rejects(
        &format!("x:1;s:=&x;{PREFIX}p:&!(items[1]);s=&*p;first=false;'loop.restart()}};v:*s}}}}"),
        "B001",
    );
    accepts(
        "<R>:<{items<int32[2]>:=}>;i:=0;r<R>:'out{first:=true;'loop{|first|{'out->items:=[i];p:&!(items[1]);*p=*p+1;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
    );
}

#[test]
pub(crate) fn exclusive_carried_elements_preserve_shared_headers_and_opaque_gates() {
    accepts(
        "<R>:<{items<int32[2]>:=;side<int32[2]>}>;seed:[0,1];s:=&seed;old:s;first:=true;r<R>:'out{'loop{|first|{'out->items:=[7];'out->side:[2];s=&side;p:&!(items[1]);*p=8;first=false;'loop.restart()};v:s[1];w:old[1]}};s=&seed",
    );
    rejects(
        &format!(
            "keep<&int32>:(p<&int32>,extra<string>){{->p}};x:1;s:=keep(&x,\"\");{PREFIX}p:&!(items[1]);*p=9;first=false;'loop.restart()}};v:*s}}}}"
        ),
        "B001",
    );
    rejects(
        &format!("{PREFIX}p:&!(items[1]);first=false;*p=9;'loop.restart()}}}}}}"),
        "B001",
    );
}
