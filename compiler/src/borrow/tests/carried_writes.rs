use super::{accepts, rejects};

pub(crate) const PREFIX: &str =
    "<R>:<{items<int32[3]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[7,8];";
pub(crate) const SUFFIX: &str = "first=false;'loop.restart()}}}";

#[test]
pub(crate) fn carried_writes_update_initialized_elements_before_restart() {
    accepts(&format!("{PREFIX}items[1]=9;{SUFFIX}"));
}

#[test]
pub(crate) fn carried_writes_keep_immutable_slots_read_only() {
    rejects(
        &format!("{PREFIX}items[1]=9;{SUFFIX}")
            .replace("items<int32[3]>:=", "items<int32[3]>")
            .replace("->items:=", "->items:"),
        "E305",
    );
}

#[test]
pub(crate) fn carried_writes_keep_copies_final_reads_and_overlapping_loans() {
    accepts(&format!(
        "{PREFIX}old:items;p:&(items[1]);items[2]=*p+1;s:&items;items[1]=s[2]+1;items=[9];v:old[1];{SUFFIX}"
    ));
    for body in [
        "p:&items;items[1]=9;v:p[1]",
        "p:&(items[1]);items[2]=9;v:*p",
        "p:&!(items[1]);items[2]=9;v:*p",
        "p:&(items[1]);items[2]=*p+1;v:*p",
    ] {
        rejects(&format!("{PREFIX}{body};{SUFFIX}"), "E302");
    }
    accepts(&format!("{PREFIX}p:&!(items[1]);*p=9;items[2]=10;{SUFFIX}"));
}

#[test]
pub(crate) fn carried_writes_preserve_reservations_through_returning_indices_and_rhs() {
    for body in [
        "items[{items=[9];->1}]=10",
        "items[1]={items=[9];->10}",
        "items[{items[2]=9;->1}]=10",
        "items[1]={items[2]=9;->10}",
        "'cancel{items[{items=[9];->1}]={'cancel.leave()}}",
    ] {
        rejects(&format!("{PREFIX}{body};{SUFFIX}"), "E302");
    }
    for body in [
        "'cancel{items[{items=[9];'cancel.leave()}]=10}",
        "'cancel{items[1]={items=[9];'cancel.leave()}}",
    ] {
        accepts(&format!("{PREFIX}{body};{SUFFIX}"));
    }
}

#[test]
pub(crate) fn carried_writes_keep_nested_phases_and_cancelled_stores() {
    let prefix = "<R>:<{items<int32[3][2]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[[7],[8,9]];";
    accepts(&format!(
        "{prefix}i:=1;j:=1;items[i][j]={{i=2;j=2;->10}};items[2]=[11,12];items[1][1]=items[2][2];{SUFFIX}"
    ));
    for body in [
        "'cancel{items[{items=[[10]];'cancel.leave()}][1]=9}",
        "'cancel{items[1][{items=[[10]];'cancel.leave()}]=9}",
        "'cancel{items[1][1]={items=[[10]];'cancel.leave()}}",
    ] {
        accepts(&format!("{prefix}{body};{SUFFIX}"));
    }
    for body in [
        "'cancel{items[{items=[[10]];->1}][{'cancel.leave()}]=9}",
        "'cancel{items[1][{items=[[10]];->1}]={'cancel.leave()}}",
        "items[1][1]={items[2]=[10];->9}",
    ] {
        rejects(&format!("{prefix}{body};{SUFFIX}"), "E302");
    }
}

#[test]
pub(crate) fn carried_writes_support_record_list_and_unit_leaves() {
    accepts(
        "<Row>:<{n<int32>:=;items<int32[3]>:=}>;<R>:<{rows<Row[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->rows:[{->n:=1;->items:=[2]}];rows[1].n=3;rows[1].items=[4,5];rows[1].items[2]=6;first=false;'loop.restart()}}}",
    );
    for (ty, initial, value) in [
        (
            "{n<int32>;label<string>}[2]",
            "[{->n:1;->label:\"old\"}]",
            "{->n:2;->label:\"new\"}",
        ),
        ("int32[3][2]", "[[1]]", "[2,3]"),
        ("null[2]", "[null]", "null"),
        ("string[2]", "[\"old\"]", "\"new\""),
    ] {
        accepts(&format!(
            "<R>:<{{items<{ty}>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:={initial};items[1]={value};first=false;'loop.restart()}}}}}}"
        ));
    }
}

#[test]
pub(crate) fn carried_writes_keep_disjoint_shared_and_exclusive_headers() {
    accepts(
        "<Row>:<{items<int32[2]>:=;side<int32[2]>:=;n<int32>:=}>;<R>:<{row<Row>}>;seed:[0,1];s:=&seed;old:s;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:=[7];->side:=[1];->n:=2};p:row.&!n;row.items[{row.side[1]=3;->1}]=4;s=row.&side;*p=5;first=false;'loop.restart()};v:s[1];w:old[1]}};s=&seed",
    );
    rejects(
        "<Row>:<{n<int32>:=;other<int32>}>;<R>:<{rows<Row[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->rows:[{->n:=1;->other:2}];s:&(rows[1].other);rows[1].n=3;v:*s;first=false;'loop.restart()}}}",
        "E302",
    );
}

#[test]
pub(crate) fn carried_writes_keep_owner_reset_leave_expiry_and_old_copies() {
    accepts(
        "<R>:<{items<int32[2]>:=}>;i:=0;r<R>:'out{first:=true;'loop{|first|{'out->items:=[i];items[1]=i+10;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
    );
    accepts(
        "<R>:<{items<int32[2]>:=}>;run<null>:(stop<boolean>){first:=true;r<R>:'out{'loop{|first|{'out->items:=[7];items[1]={|stop|'out.leave();->9};first=false;'loop.restart()}}}}",
    );
    rejects(
        &format!("x:1;s:=&x;{PREFIX}items[1]=9;s=&(items[1]);{SUFFIX};v:*s"),
        "E303",
    );
    rejects(
        &format!(
            "x:1;s:=&x;old:=s;{PREFIX}s=&(items[1]);old=s;items[2]=9;first=false;'loop.restart()}};v:*old}}}}"
        ),
        "E302",
    );
}

#[test]
pub(crate) fn carried_writes_keep_bounds_types_and_unsupported_roots() {
    for (body, code) in [
        ("items[0]=9", "E101"),
        ("items[-1]=9", "E101"),
        ("items[4]=9", "E101"),
        ("items[true]=9", "E222"),
        ("items[1]=\"wrong\"", "E207"),
        ("s:&items;s[1]=9", "B001"),
        ("p:&!items", "B001"),
    ] {
        rejects(&format!("{PREFIX}{body};{SUFFIX}"), code);
    }
}
