use super::Case;
use super::exclusive_references::rejects;

#[test]
pub(crate) fn carried_lists_keep_length_payload_and_independent_copies() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[3]>:=}>
first:=true
builds:=0
r<R>:'out{
    'loop{
        |first|{
            'out->items:={builds=builds+1;d.print("init");->[1,2]}
            old:items
            items=[9]
            items=items.add(10)
            d.print(old.size())
            d.print(old[2])
            d.print(items.size())
            d.print(items[2])
            first=false
            'loop.restart()
        }
    }
}
d.print(r.items[1])
d.print(builds)
"#,
    )
    .runs(b"init\n2\n2\n2\n10\n9\n1\n");
}

#[test]
pub(crate) fn carried_lists_keep_nested_record_unit_empty_and_primary_values() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{value<uint8>;unit<null>;name<string>}>
<R>:<{rows<Row[3][2]>;empty<int32[0]>;units<null[2]>}>
first:=true
r<R>:'out{'loop{|first|{
    'out->rows:[[{->value:255;->unit:null;->name:"ready"}],[]]
    'out->empty:[]
    'out->units:[null]
    first=false
    'loop.restart()
}}}
d.print(r.rows[1][1].value)
d.print(r.rows[1][1].name)
d.print(r.rows[2].size())
d.print(r.empty.size())
d.print(r.units.size())
first=true
s<int32[3]>:'out{'loop{|first|{'out->[1,2];first=false;'loop.restart()}}}
d.print(s[2])
"#,
    )
    .runs(b"255\nready\n0\n0\n1\n2\n");
}

#[test]
pub(crate) fn carried_lists_replace_record_fields_and_keep_sibling_loans() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{items<int32[3]>:=;other<int32>:=}>
<R>:<{row<Row>:=;n<int32>:=}>
first:=true
r<R>:'out{'loop{|first|{
    'out->row:={->items:=[1,2];->other:=3}
    old:=row.items
    p:&(old[2])
    row.items=[9]
    row.other=4
    d.print(*p)
    old[2]=8
    d.print(old[2])
    'out->n:=5
    q:&!n
    *q=6
    first=false
    'loop.restart()
}}}
d.print(r.row.items.size())
d.print(r.row.items[1])
d.print(r.row.other)
d.print(r.n)
p:&(r.row.items[1])
d.print(*p)
"#,
    )
    .runs(b"2\n8\n1\n9\n4\n6\n9\n");
}

#[test]
pub(crate) fn carried_lists_reinitialize_at_owner_resets_and_preserve_leaves() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[2]>:=}>
i:=0
r<R>:'out{
    first:=true
    'loop{|first|{'out->items:={d.print("init");->[i]};first=false;'loop.restart()}}
    i=i+1
    |i<2|'out.restart()
}
d.print(r.items[1])
run<null>:(stop<boolean>){
    first:=true
    r<R>:'out{'loop{|first|{
        'out->items:=[3]
        items={d.print("rhs");|stop|'out.leave();->[9,10]}
        first=false
        'loop.restart()
    }}}
    d.print(r.items.size())
    d.print(r.items[1])
}
run(false)
run(true)
"#,
    )
    .runs(b"init\ninit\n1\nrhs\n2\n9\nrhs\n1\n3\n");
}

#[test]
pub(crate) fn carried_lists_publish_only_completed_initializers() {
    let source = r#"
d:@"debug"
<R>:<{items<int32[2]>}>
run<int32>:(stop<boolean>){
    first:=true
    r<R>:'out{'loop{|first|{
        'out->items:[{d.print("first");->1},{d.print("second");|stop|d.panic("halt");->2}]
        first=false
        'loop.restart()
    }}}
    ->r.items[2]
}
"#;
    Case::new(&format!("{source}d.print(run(false))")).runs(b"first\nsecond\n2\n");
    let case = Case::new(&format!("{source}d.print(run(true))"));
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"first\nsecond\n");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]:"));
    }
}

#[test]
pub(crate) fn carried_lists_keep_runtime_bounds_tied_to_retained_length() {
    let source = r#"
d:@"debug"
<R>:<{items<int32[2]>:=}>
run<null>:(more<boolean>,at<int32>){
    first:=true
    r<R>:'out{'loop{|first|{
        'out->items:=[1]
        |more|items=[7,8]
        first=false
        'loop.restart()
    }}}
    d.print(r.items[at])
}
"#;
    Case::new(&format!("{source}run(false,1);run(true,2)")).runs(b"1\n8\n");
    let case = Case::new(&format!("{source}run(false,2)"));
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .starts_with("panic[P001]: index 2 is outside initialized length 1")
        );
    }
}

#[test]
pub(crate) fn carried_lists_preserve_initialization_storage_and_type_rejections() {
    for (source, code) in [
        (
            "<R>:<{items<int32[2]>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->items:[1];first=false};i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<R>:<{items<int32[2]>}>;i:=0;r<R>:'out{'loop{'out->items:[1];i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[1];'out->items:[2];first=false;'loop.restart()}}}",
            "E205",
        ),
        (
            "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[true];first=false;'loop.restart()}}}",
            "E207",
        ),
        (
            "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[1];items=[2];first=false;'loop.restart()}}}",
            "E305",
        ),
    ] {
        rejects(source, code);
    }
    for body in ["p:&items", "p:&(items[1])", "p:&!(items[1])", "items[1]=9"] {
        rejects(
            &format!(
                "<R>:<{{items<int32[2]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[1];{body};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
}
