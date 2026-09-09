use super::Case;

#[test]
pub(crate) fn carried_records_keep_nested_scalar_unit_and_primary_values() {
    Case::new(r#"
d:@"debug"
<Inner>:<{n<uint8>;unit<null>}>
<Row>:<{-><int32>;inner<Inner>;ok<boolean>;name<string>;ratio<float32>}>
<R>:<{row<Row>}>
first:=true;i:=0;builds:=0
r<R>:'out{
    'loop{
        |first|{
            'out->row:{builds=builds+1;->5;->inner:{->n:255;->unit:null};->ok:true;->name:"ready";->ratio:1.5}
            first=false
        }
        i=i+1
        |i<3|'loop.restart()
    }
}
d.print(r.row)
d.print(r.row.inner.n)
d.print(r.row.ok)
d.print(r.row.name)
d.print(r.row.ratio)
d.print(builds)
"#).runs(b"5\n255\ntrue\nready\n1.5\n1\n");
}

#[test]
pub(crate) fn carried_records_keep_old_copies_and_scalar_sibling_loans() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{n<int32>:=;name<string>}>
<R>:<{row<Row>:=;count<int32>:=}>
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->row:={->n:=7;->name:"old"}
            old:row
            row.n=8
            d.print(row.n)
            row={->n:=9;->name:"new"}
            d.print(old.n)
            d.print(old.name)
            'out->count:=1
            p:&!count
            *p=10
            first=false
            'loop.restart()
        }
    }
}
d.print(r.row.n)
d.print(r.row.name)
d.print(r.count)
p:&r.row.n
d.print(*p)
"#,
    )
    .runs(b"8\n7\nold\n9\nnew\n10\n9\n");
}

#[test]
pub(crate) fn carried_records_reinitialize_at_owner_resets_and_complete_leaves() {
    Case::new(r#"
d:@"debug"
<Row>:<{n<int32>}>
<R>:<{row<Row>}>
i:=0
r<R>:'out{
    first:=true
    'loop{|first|{'out->row:{d.print("init");->n:i};first=false;'loop.restart()}}
    i=i+1
    |i<2|'out.restart()
}
d.print(r.row.n)
first:=true
s<R>:'out{'loop{|first|{'out->row:{->n:7};copy:row;p:&copy.n;d.print(*p);first=false;'loop.restart()};'out.leave()}}
d.print(s.row.n)
"#).runs(b"init\ninit\n1\n7\n7\n");
}

#[test]
pub(crate) fn carried_records_do_not_publish_partial_panicking_initializers() {
    let source = r#"
d:@"debug"
<Row>:<{n<int32>}>
<R>:<{row<Row>}>
run<int32>:(stop<boolean>){
    first:=true
    r<R>:'out{'loop{|first|{'out->row:{d.print("init");|stop|d.panic("halt");->n:7};first=false;'loop.restart()}}}
    ->r.row.n
}
"#;
    Case::new(&format!("{source}\nd.print(run(false))")).runs(b"init\n7\n");
    let case = Case::new(&format!("{source}\nd.print(run(true))"));
    for profile in ["debug", "release"] {
        let result = case.command("run", &["--profile", profile]);
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(result.stdout, b"init\n");
        assert!(String::from_utf8_lossy(&result.stderr).contains("P006"));
    }
}

#[test]
pub(crate) fn carried_records_reject_unproved_initialization_and_storage_borrows() {
    for (source, code) in [
        (
            "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->row:{->n:7};first=false};i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;i:=0;r<R>:'out{'loop{'out->row:{->n:7};i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};'out->row:{->n:8};first=false;'loop.restart()}}}",
            "E205",
        ),
        (
            "<Row>:<{n<int32>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7};p:&!row.n;v:*p;first=false;'loop.restart()}}}",
            "B001",
        ),
        (
            "<Row>:<{n<int32><null>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};first=false;'loop.restart()}}}",
            "B001",
        ),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(result.status.code(), Some(1), "{source}: {profile}");
            assert!(result.stdout.is_empty());
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(
                error.starts_with(&format!("error[{code}]:")),
                "{source}: {profile}: {error}"
            );
        }
    }
}
