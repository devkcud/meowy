use super::Case;

pub(crate) fn with_lists(source: &str) -> String {
    source
        .replace("<{n<", "<{items<int32[2]>:=;n<")
        .replace("->n:", "->items:=[1];->n:")
}

pub(crate) fn runs(source: &str, expected: &[u8]) {
    Case::new(source).runs(expected);
    Case::new(&with_lists(source)).runs(expected);
}

#[test]
pub(crate) fn exclusive_carried_record_fields_mutate_nested_widths_and_preserve_copies() {
    runs(
        r#"
d:@"debug"
<Inner>:<{n<uint8>:=;other<int32>:=}>
<Row>:<{inner<Inner>:=;flag<boolean>:=;amount<float32>:=;label<string>}>
<R>:<{row<Row>:=}>
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->row:={->inner:={->n:=7;->other:=1};->flag:=true;->amount:=1.5;->label:"ready"}
            old:row
            s:&(old.inner.n)
            p:&!(row.inner.n)
            q:&!(row.inner.other)
            *p=255
            *q=9
            d.print(*s)
            b:&!(row.flag)
            *b=false
            f:&!(row.amount)
            *f=2.5
            first=false
            'loop.restart()
        }
    }
}
d.print(r.row.inner.n)
d.print(r.row.inner.other)
d.print(r.row.flag)
d.print(r.row.amount)
d.print(r.row.label)
"#,
        b"7\n255\n9\nfalse\n2.5\nready\n",
    );
}

#[test]
pub(crate) fn exclusive_carried_record_fields_keep_children_calls_and_final_use() {
    runs(
        r#"
d:@"debug"
<Inner>:<{n<int32>:=;other<int32>:=}>
<Row>:<{inner<Inner>:=}>
<R>:<{row<Row>:=}>
bump<null>:(p<&!int32>){*p=*p+1}
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->row:={->inner:={->n:=7;->other:=1}}
            p:&!(row.inner.n)
            s:&*p
            d.print(*s)
            q:&!*p
            *q=8
            d.print(*q)
            *p=9
            moved:p
            bump(moved)
            d.print(row.inner.n)
            row.inner={->n:=11;->other:=2}
            first=false
            'loop.restart()
        }
    }
}
d.print(r.row.inner.n)
"#,
        b"7\n8\n10\n11\n",
    );
}

#[test]
pub(crate) fn exclusive_carried_record_fields_reacquire_after_owner_resets() {
    runs(
        r#"
d:@"debug"
<Row>:<{n<int32>:=}>
<R>:<{row<Row>:=}>
i:=0
r<R>:'out{
    first:=true
    'loop{
        |first|{
            'out->row:={->n:=i}
            p:&!(row.n)
            *p=*p+10
            d.print(*p)
            first=false
            'loop.restart()
        }
    }
    i=i+1
    |i<2|'out.restart()
}
d.print(r.row.n)
"#,
        b"10\n11\n11\n",
    );
}

#[test]
pub(crate) fn exclusive_carried_record_fields_leave_skips_unfinished_stores() {
    runs(
        r#"
d:@"debug"
<Row>:<{n<int32>:=}>
<R>:<{row<Row>:=}>
run<null>:(stop<boolean>){
    first:=true
    r<R>:'out{
        'loop{
            |first|{
                'out->row:={->n:=3}
                p:&!(row.n)
                *p={d.print("rhs");|stop|'out.leave();->9}
                first=false
                'loop.restart()
            }
        }
    }
    d.print(r.row.n)
}
run(false)
run(true)
"#,
        b"rhs\n9\nrhs\n3\n",
    );
}

#[test]
pub(crate) fn exclusive_carried_record_fields_preserve_shared_headers() {
    runs(
        r#"
d:@"debug"
<Row>:<{n<int32>:=;other<int32>:=}>
<R>:<{row<Row>:=}>
x:1
s:=&x
old:s
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->row:={->n:=7;->other:=2}
            s=&(row.other)
            p:&!(row.n)
            *p=8
            first=false
            'loop.restart()
        }
        d.print(*s)
        d.print(*old)
    }
}
s=&x
d.print(r.row.n)
d.print(*s)
"#,
        b"2\n1\n8\n1\n",
    );
}

#[test]
pub(crate) fn exclusive_carried_record_fields_preserve_rejection_boundaries() {
    for (body, tail, code) in [
        ("p:&!(row.n);q:p;v:*p;w:*q", "", "E301"),
        ("p:&!(row.n);row.n=8;v:*p", "", "E302"),
        ("p:&!(row.n);row={->n:=8};v:*p", "", "E302"),
        ("p:&!(row.n);s:&*p;*p=8;v:*s", "", "E302"),
        ("p:&!(row.n);q:&!(row.n);v:*p;w:*q", "", "E302"),
        ("p:&!row", "", "B001"),
        ("p:&!(row.n);s=&*p", ";v:*s", "E303"),
        (
            "p:&!(row.n);i:=0;'again{v:*p;i=i+1;|i<2|'again.restart()}",
            "",
            "B001",
        ),
        (
            "p:&!(row.n);q:&*p;i:=0;'again{v:*q;i=i+1;|i<2|'again.restart()}",
            "",
            "B001",
        ),
        (
            "p:&!(row.n);q:keep(p);i:=0;'again{v:*q;i=i+1;|i<2|'again.restart()}",
            "",
            "B001",
        ),
    ] {
        let source = format!(
            "keep<&int32>:(p<&!int32>){{->&*p}};x:1;s:=&x;<Row>:<{{n<int32>:=}}>; <R>:<{{row<Row>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->row:={{->n:=7}};{body};first=false;'loop.restart()}}}}}}{tail}"
        );
        for source in [source.clone(), with_lists(&source)] {
            let case = Case::new(&source);
            for profile in ["debug", "release"] {
                let output = case.command("run", &["--profile", profile]);
                assert_eq!(output.status.code(), Some(1), "{body}: {profile}");
                assert!(output.stdout.is_empty());
                assert!(
                    String::from_utf8_lossy(&output.stderr).starts_with(&format!("error[{code}]:")),
                    "{body}: {profile}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

#[test]
pub(crate) fn exclusive_carried_record_fields_replace_list_siblings_with_shared_headers() {
    Case::new(include_str!(
        "../../examples/exclusive-carried-list-fields.mwy"
    ))
    .runs(b"1\n3\n0\n8\n2\n");
}

#[test]
pub(crate) fn exclusive_carried_record_fields_keep_list_conflicts_and_indexed_gates() {
    for (body, code) in [
        ("p:row.&!n;s:row.&items;row.items=[2];v:s[1];w:*p", "E302"),
        ("s:&row;p:row.&!n;*p=8;v:s.items[1]", "E302"),
        ("p:row.&!n;s:&!*p;v:*p;*s=8", "E302"),
        ("p:row.&!items", "B001"),
        ("s:row.&items;s[1]=9", "B001"),
    ] {
        super::exclusive_references::rejects(
            &format!(
                "<Row>:<{{n<int32>:=;items<int32[2]>:=}}>; <R>:<{{row<Row>}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->row:{{->n:=7;->items:=[1]}};{body};first=false;'loop.restart()}}}}}}"
            ),
            code,
        );
    }
}
