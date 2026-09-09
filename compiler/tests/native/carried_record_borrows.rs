use super::Case;

#[test]
pub(crate) fn carried_record_borrows_retain_records_projections_and_old_references() {
    Case::new(
        r#"
d:@"debug"
<Inner>:<{n<int32>}>
<Row>:<{inner<Inner>;name<string>}>
<R>:<{row<Row>}>
x<Row>:{->inner:{->n:1};->name:"old"}
p:=&x
old:p
q:=&x.inner.n
first:=true;i:=0
r<R>:'out{
    'loop{
        |first|{
            'out->row:{d.print("init");->inner:{->n:7};->name:"ready"}
            p=&row
            q=&p.inner.n
            d.print(q==&row.inner.n)
            first=false
        }
        d.print(old.inner.n)
        d.print(p.name)
        d.print(*q)
        i=i+1
        |i<3|'loop.restart()
    }
    d.print(p.inner.n)
}
p=&x;q=&x.inner.n
d.print(r.row.inner.n)
d.print(*q)
"#,
    )
    .runs(b"init\ntrue\n1\nready\n7\n1\nready\n7\n1\nready\n7\n7\n7\n1\n");
}

#[test]
pub(crate) fn carried_record_borrows_reacquire_after_owner_resets() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{n<int32>}>
<R>:<{row<Row>}>
x:9;p:=&x;i:=0
r<R>:'out{
    p=&x
    first:=true
    'loop{
        |first|{'out->row:{->n:i};p=&row.n;first=false;'loop.restart()}
        d.print(*p)
    }
    i=i+1
    |i<2|'out.restart()
}
p=&x
d.print(r.row.n)
d.print(*p)
"#,
    )
    .runs(b"0\n1\n1\n9\n");
}

#[test]
pub(crate) fn carried_record_borrows_preserve_disjoint_writes_and_last_use() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{n<int32>:=;other<int32>:=}>
<R>:<{row<Row>:=}>
first:=true
r<R>:'out{'loop{|first|{
    'out->row:={->n:=7;->other:=1}
    old:row
    p:&row.n
    row.other=9
    d.print(*p)
    row.n=8
    q:&row
    d.print(q.n)
    d.print(q.other)
    row={->n:=10;->other:=2}
    d.print(old.n)
    first=false
    'loop.restart()
}}}
d.print(r.row.n)
"#,
    )
    .runs(b"7\n8\n9\n7\n10\n");
}

#[test]
pub(crate) fn carried_record_borrows_keep_returned_projection_until_leave() {
    Case::new(
        r#"
d:@"debug"
<Inner>:<{n<int32>}>
<Row>:<{inner<Inner>}>
<R>:<{row<Row>}>
keep<&int32>:(value<&int32>){->value}
x:1;p:=&x;first:=true
r<R>:'out{'loop{
    |first|{'out->row:{->inner:{->n:7}};p=keep(&row.inner.n);first=false;'loop.restart()}
    d.print(*p)
    'out.leave()
}}
p=&x
d.print(r.row.inner.n)
"#,
    )
    .runs(b"7\n7\n");
}

#[test]
pub(crate) fn carried_record_borrows_reject_expiry_conflicts_and_exclusive_paths() {
    for (source, code) in [
        (
            "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;x<Row>:{->n:1};p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};p=&row;first=false;'loop.restart()}}};v:p.n",
            "E303",
        ),
        (
            "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;x:1;p:=&x;i:=0;r<R>:'out{v:*p;first:=true;'loop{|first|{'out->row:{->n:i};p=&row.n;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
            "E303",
        ),
        (
            "<Row>:<{n<int32>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7};p:&row.n;row.n=8;v:*p;first=false;'loop.restart()}}}",
            "E302",
        ),
        (
            "<Row>:<{n<int32>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7};p:&row;row={->n:=8};v:p.n;first=false;'loop.restart()}}}",
            "E302",
        ),
        (
            "<Row>:<{n<int32>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7};p:&!row;v:p.n;first=false;'loop.restart()}}}",
            "B001",
        ),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with(&format!("error[{code}]:")),
                "{source}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
