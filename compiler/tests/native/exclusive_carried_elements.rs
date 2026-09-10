use super::Case;
use super::exclusive_references::rejects;

#[test]
pub(crate) fn exclusive_carried_elements_mutate_widths_nested_paths_and_preserve_copies() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{items<int32[2]>:=;n<uint8>:=}>
<R>:<{rows<Row[2]>;flags<boolean[2]>:=;amounts<float32[2]>:=}>
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->rows:[{->items:=[7];->n:=1}]
            'out->flags:=[true]
            'out->amounts:=[1.5]
            old:rows
            p:&!(rows[1].items[1])
            *p=8
            q:rows[1].&!n
            *q=255
            b:&!(flags[1])
            *b=false
            f:&!(amounts[1])
            *f=2.5
            d.print(old[1].items[1])
            first=false
            'loop.restart()
        }
    }
}
d.print(r.rows[1].items[1])
d.print(r.rows[1].n)
d.print(r.flags[1])
d.print(r.amounts[1])
"#,
    )
    .runs(b"7\n8\n255\nfalse\n2.5\n");
}

#[test]
pub(crate) fn exclusive_carried_elements_preserve_children_calls_and_last_use() {
    Case::new(
        r#"
d:@"debug"
bump<null>:(p<&!int32>){*p=*p+1}
<R>:<{items<int32[3]>:=}>
first:=true
r<R>:'out{
 'loop{
  |first|{
   'out->items:=[7,8]
   p:&!(items[1]);s:&*p;d.print(*s)
   q:&!*p;*q=9;d.print(*q)
   *p=10;moved:p;bump(moved);d.print(items[1])
   items=[12];first=false;'loop.restart()
  }
 }
}
d.print(r.items[1])
"#,
    )
    .runs(b"7\n9\n11\n12\n");
}

#[test]
pub(crate) fn exclusive_carried_elements_order_indices_and_keep_disjoint_shared_headers() {
    Case::new(include_str!(
        "../../examples/exclusive-carried-elements.mwy"
    ))
    .runs(b"1\n7\n3\n2\n0\n8\n");
}

#[test]
pub(crate) fn exclusive_carried_elements_cancel_indices_and_unfinished_stores() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[2][2]>:=}>
run<null>:(stop<boolean>){
 first:=true
 r<R>:'out{
  'loop{
   |first|{
    'out->items:=[[7]]
    'cancel{p:&!(items[{items=[[8]];d.print("outer");'cancel.leave()}][{d.print("never");->1}])}
    'cancel{p:&!(items[1][{items=[[9]];d.print("inner");'cancel.leave()}])}
    p:&!(items[1][1]);*p={d.print("rhs");|stop|'out.leave();->10}
    first=false;'loop.restart()
   }
  }
 }
 d.print(r.items[1][1])
}
run(false);run(true)
"#,
    )
    .runs(b"outer\ninner\nrhs\n10\nouter\ninner\nrhs\n9\n");
}

#[test]
pub(crate) fn exclusive_carried_elements_reacquire_after_owner_reset() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[2]>:=}>
i:=0
r<R>:'out{
 first:=true
 'loop{|first|{'out->items:=[i];p:&!(items[1]);*p=*p+10;d.print(*p);first=false;'loop.restart()}}
 i=i+1;|i<2|'out.restart()
}
d.print(r.items[1])
"#,
    )
    .runs(b"10\n11\n11\n");
}

#[test]
pub(crate) fn exclusive_carried_elements_check_current_length_and_prefix_bounds() {
    for (path, output, fault) in [
        ("items[pos(2)][pos(1)]", "2\n", "items[pos(2)]"),
        (
            "items[pos(1)][pos(2)]",
            "1\n2\n",
            "&!(items[pos(1)][pos(2)])",
        ),
    ] {
        let source = format!(
            "d:@\"debug\";pos<int32>:(n<int32>){{d.print(n);->n}};<R>:<{{items<int32[3][3]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[[7,8],[9]];items=[[1]];p:&!({path});*p=10;first=false;'loop.restart()}}}}}}"
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
    for (ty, value, index, code) in [
        ("int32[2]", "[]", "pos(1)", "P001"),
        ("int32[0]", "[]", "pos(1)", "P001"),
        ("int32[2]", "[1]", "pos(0)", "P001"),
        ("int32[2]", "[1]", "pos(-1)", "P001"),
        ("int32[2]", "[1]", "big()", "P001"),
    ] {
        let case = Case::new(&format!(
            "big<uint64>:(){{->18446744073709551615}};pos<int32>:(n<int32>){{->n}};<R>:<{{items<{ty}>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:={value};p:&!(items[{index}]);first=false;'loop.restart()}}}}}}"
        ));
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with(&format!("panic[{code}]:")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
pub(crate) fn exclusive_carried_elements_preserve_conflicts_frontiers_and_write_gates() {
    for (body, code) in [
        ("p:&!(items[1]);items=[9];v:*p", "E302"),
        ("p:&!(items[1]);q:&!(items[2]);v:*p;w:*q", "E302"),
        ("p:&!(items[{items=[9];->1}])", "E302"),
        ("p:&!(items[1]);s:&*p;*p=9;v:*s", "E302"),
        ("p:&!(items[1]);q:p;v:*p", "E301"),
        ("items[1]=9", "B001"),
        ("p:&!items", "B001"),
        (
            "p:&!(items[1]);i:=0;'again{v:*p;i=i+1;|i<2|'again.restart()}",
            "B001",
        ),
        (
            "p:&!(items[1]);s:&*p;i:=0;'again{v:*s;i=i+1;|i<2|'again.restart()}",
            "B001",
        ),
    ] {
        rejects(
            &format!(
                "<R>:<{{items<int32[3]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[7,8];{body};first=false;'loop.restart()}}}}}}"
            ),
            code,
        );
    }
    rejects(
        "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p:&!(items[1]);first=false;'loop.restart()}}}",
        "E305",
    );
    rejects(
        "x:1;s:=&x;<R>:<{items<int32[2]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[7];p:&!(items[1]);s=&*p;first=false;'loop.restart()}}};v:*s",
        "E303",
    );
    rejects(
        "<R>:<{items<int32[2][2]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[[7]];'cancel{p:&!(items[{items=[[9]];->1}][{'cancel.leave()}])};first=false;'loop.restart()}}}",
        "E302",
    );
}

#[test]
pub(crate) fn exclusive_carried_elements_keep_completed_effects_before_index_panic() {
    for path in [
        "items[{items=[[9]];d.print(items[1][1]);d.panic(\"stop\")}][1]",
        "items[1][{items=[[9]];d.print(items[1][1]);d.panic(\"stop\")}]",
    ] {
        let case = Case::new(&format!(
            "d:@\"debug\";<R>:<{{items<int32[2][2]>:=}}>;run<null>:(stop<boolean>){{first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[[7]];|stop|{{p:&!({path})}};first=false;'loop.restart()}}}}}}}};run(true)"
        ));
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(
                output.stdout,
                b"9\n",
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
        }
    }
}
