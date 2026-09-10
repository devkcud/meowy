use super::Case;
use super::exclusive_references::rejects;

#[test]
pub(crate) fn carried_writes_preserve_scalar_and_aggregate_values_lengths_and_copies() {
    Case::new(r#"
d:@"debug"
<Row>:<{n<int32>:=;label<string>}>
<R>:<{items<uint8[3]>:=;rows<Row[2]>:=;matrix<int32[3][2]>:=;flags<boolean[2]>:=;amounts<float32[2]>:=;units<null[1]>:=}>
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->items:=[7,8]
            'out->rows:=[{->n:=1;->label:"old"}]
            'out->matrix:=[[1],[2,3]]
            'out->flags:=[true]
            'out->amounts:=[1.5]
            'out->units:=[null]
            old:items
            items[1]=255
            rows[1]={->n:=2;->label:"new"}
            rows[1].n=3
            matrix[1]=[4,5]
            matrix[2][1]=6
            flags[1]=false
            amounts[1]=2.5
            units[1]=null
            d.print(old[1])
            first=false
            'loop.restart()
        }
    }
}
d.print(r.items[1]);d.print(r.items[2]);d.print(r.items.size())
d.print(r.rows[1].n);d.print(r.rows[1].label)
d.print(r.matrix[1].size());d.print(r.matrix[1][2]);d.print(r.matrix[2][1]);d.print(r.matrix[2][2])
d.print(r.flags[1]);d.print(r.amounts[1]);d.print(r.units.size())
"#).runs(b"7\n255\n8\n2\n3\nnew\n2\n5\n6\n3\nfalse\n2.5\n1\n");
}

#[test]
pub(crate) fn carried_writes_capture_indices_before_rhs_and_end_shared_reads() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{items<int32[3][2]>:=;side<int32>:=}>
<R>:<{row<Row>}>
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->row:{->items:=[[1,2],[3]];->side:=7}
            i:=1;j:=2
            row.items[{d.print("row");->i}][{d.print("item");->j}]={d.print("rhs");i=2;j=1;->9}
            view:&(row.items[1][2])
            row.items[2][1]=*view+1
            p:row.&!side
            row.items[1][1]=11
            *p=12
            d.print(i);d.print(j)
            first=false
            'loop.restart()
        }
    }
}
d.print(r.row.items[1][2]);d.print(r.row.items[2][1]);d.print(r.row.items[1][1]);d.print(r.row.side)
"#,
    )
    .runs(b"row\nitem\nrhs\n2\n1\n9\n10\n11\n12\n");
}

#[test]
pub(crate) fn carried_writes_keep_nested_cancellation_and_owner_completion() {
    Case::new(r#"
d:@"debug"
<R>:<{items<int32[2][2]>:=}>
run<null>:(stop<boolean>){
    first:=true
    r<R>:'out{
        'loop{
            |first|{
                'out->items:=[[1],[2]]
                'cancel{items[{items=[[3]];d.print("outer");'cancel.leave()}][{d.print("never");->1}]=99}
                'cancel{items[1][{items=[[4]];d.print("inner");'cancel.leave()}]={d.print("never");->99}}
                'cancel{items[1][1]={items=[[5]];d.print("rhs");'cancel.leave()}}
                items[1][1]={|stop|'out.leave();->6}
                first=false
                'loop.restart()
            }
        }
    }
    d.print(r.items[1][1])
}
run(false);run(true)
"#).runs(b"outer\ninner\nrhs\n6\nouter\ninner\nrhs\n5\n");
}

#[test]
pub(crate) fn carried_writes_reinitialize_on_owner_reset_and_cancel_restarted_stores() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[2]>:=}>
i:=0
r<R>:'out{
    first:=true
    'loop{
        |first|{
            'out->items:=[i]
            items[1]=i+10
            d.print(items[1])
            first=false
            'loop.restart()
        }
    }
    i=i+1
    |i<2|'out.restart()
}
d.print(r.items[1])
"#,
    )
    .runs(b"10\n11\n11\n");
    for statement in [
        "items[{i=i+1;|i<2|'out.restart();->1}]=9",
        "items[1]={i=i+1;|i<2|'out.restart();->9}",
    ] {
        Case::new(&format!("d:@\"debug\";<R>:<{{items<int32[2]>:=}}>;i:=0;r<R>:'out{{first:=true;'loop{{|first|{{'out->items:=[i];d.print(items[1]);{statement};first=false;'loop.restart()}}}}}};d.print(r.items[1])")).runs(b"0\n1\n9\n");
    }
}

#[test]
pub(crate) fn carried_writes_check_current_length_before_later_indices_and_rhs() {
    for (path, output, fault) in [
        ("items[pos(2)][pos(1)]", "2\n", "items[pos(2)]"),
        ("items[pos(1)][pos(2)]", "1\n2\n", "items[pos(1)][pos(2)]"),
    ] {
        let source = format!(
            "d:@\"debug\";pos<int32>:(n<int32>){{d.print(n);->n}};<R>:<{{items<int32[3][3]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[[7,8],[9]];items=[[1]];{path}={{d.print(\"never\");->10}};first=false;'loop.restart()}}}}}}"
        );
        let start = source.find(fault).unwrap();
        let error = format!(
            "panic[P001]: index 2 is outside initialized length 1 at bytes {start}..{}\n",
            start + fault.len()
        );
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(result.status.code(), Some(1));
            assert_eq!(result.stdout, output.as_bytes());
            assert_eq!(result.stderr, error.as_bytes());
        }
    }
    for (capacity, initial, ty, index, shown, length) in [
        (2, "[7]", "int8", "-1", "-1", 1),
        (2, "[7]", "int32", "0", "0", 1),
        (
            2,
            "[7]",
            "uint64",
            "18446744073709551615",
            "18446744073709551615",
            1,
        ),
        (2, "[]", "int32", "1", "1", 0),
        (0, "[]", "int32", "1", "1", 0),
    ] {
        let access = "items[{d.print(\"index\");->index}]";
        let source = format!(
            "d:@\"debug\";<R>:<{{items<int32[{capacity}]>:=}}>;run<null>:(index<{ty}>){{first:=true;r<R>:'out{{'loop{{|first|{{'out->items:={initial};{access}={{d.print(\"never\");->9}};first=false;'loop.restart()}}}}}}}};run({index})"
        );
        let start = source.find(access).unwrap();
        let error = format!(
            "panic[P001]: index {shown} is outside initialized length {length} at bytes {start}..{}\n",
            start + access.len()
        );
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(result.status.code(), Some(1));
            assert_eq!(result.stdout, b"index\n");
            assert_eq!(result.stderr, error.as_bytes());
        }
    }
}

#[test]
pub(crate) fn carried_writes_preserve_panic_effects_at_each_cancelled_phase() {
    for (statement, expected) in [
        (
            "items[{items=[[9]];d.print(\"outer\");d.panic(\"stop\")}][1]=10",
            "outer\n",
        ),
        (
            "items[1][{items=[[9]];d.print(\"inner\");d.panic(\"stop\")}]=10",
            "inner\n",
        ),
        (
            "items[1][1]={items=[[9]];d.print(\"rhs\");d.panic(\"stop\")}",
            "rhs\n",
        ),
    ] {
        let source = format!(
            "d:@\"debug\";<R>:<{{items<int32[2][2]>:=}}>;run<null>:(stop<boolean>){{first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[[7]];|stop|{{{statement}}};first=false;'loop.restart()}}}}}}}};run(true)"
        );
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(
                output.stdout,
                expected.as_bytes(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
        }
    }
}

#[test]
pub(crate) fn carried_writes_preserve_reservation_conflicts_and_permission_errors() {
    for (body, code) in [
        ("p:&items;items[1]=9;v:p[1]", "E302"),
        ("p:&!(items[1]);items[2]=9;v:*p", "E302"),
        ("items[{items=[9];->1}]=10", "E302"),
        ("items[1]={items[2]=9;->10}", "E302"),
        ("'cancel{items[{items=[9];->1}]={'cancel.leave()}}", "E302"),
        ("items[0]=9", "E101"),
        ("items[1]=\"wrong\"", "E207"),
        ("p:&items;p[1]=9", "B001"),
        ("p:&!items", "B001"),
    ] {
        rejects(
            &format!(
                "<R>:<{{items<int32[3]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[7,8];{body};first=false;'loop.restart()}}}}}}"
            ),
            code,
        );
    }
    rejects(
        "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];items[1]=9;first=false;'loop.restart()}}}",
        "E305",
    );
    rejects(
        "x:1;s:=&x;<R>:<{items<int32[2]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[7];items[1]=9;s=&(items[1]);first=false;'loop.restart()}}};v:*s",
        "E303",
    );
}

#[test]
pub(crate) fn carried_writes_keep_disjoint_shared_headers_and_exclusive_siblings() {
    Case::new(include_str!("../../examples/carried-writes.mwy"))
        .runs(b"row\nitem\nrhs\n7\n3\n0\n8\n9\n");
}
