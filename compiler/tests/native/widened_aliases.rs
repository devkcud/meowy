use super::Case;

#[test]
pub fn optional_alias_writes_publish_changed_and_absent_members() {
    Case::new(
        r#"
d:@"debug"
make:(x<&int32>,y<&int32>,flag<boolean>)'out{
    |flag|{'out->p:=x;p=y}
}
x:7
y:9
a:make(&x,&y,true)
b:make(&x,&y,false)
|a.p<&int32>|d.print(*(a.p))
|b.p<null>|{copy:b.p;d.print("empty")}
"#,
    )
    .runs(b"9\nempty\n");
}

#[test]
pub fn heterogeneous_alias_writes_keep_outer_member_identity() {
    Case::new(
        r#"
d:@"debug"
make:(x<&int32>,y<&int32>,a<&string>,b<&string>,flag<boolean>)'out{
    |flag|{'out->p:=x;p=y}
    |!flag|{'out->p:=a;p=b}
}
x:1
y:2
a:"before"
b:"after"
one:make(&x,&y,&a,&b,true)
two:make(&x,&y,&a,&b,false)
|one.p<&int32>|d.print(*(one.p))
|two.p<&string>|d.print(*(two.p))
"#,
    )
    .runs(b"2\nafter\n");
}

#[test]
pub fn widened_record_alias_fields_preserve_rhs_leave_and_nested_tags() {
    Case::new(
        r#"
d:@"debug"
<C>:<{p<&int32>:=;q<&int32>:=;n<int32>:=}>
x:1
y:2
flag:=true
r:'out{
    |flag|{
        'out->c:={->p:=&x;->q:=&y;->n:=0}
        c.p={c.q=&x;->&y}
        c.n={c={->p:=&y;->q:=&x;->n:=3};->4}
        c.p={c.q=&y;'out.leave();->&x}
    }
}
|r.c<C>|{d.print(*(r.c.p));d.print(*(r.c.q));d.print(r.c.n)}
<H>:<{p<&int32><null>:=}>
s:'out{|flag|{'out->c:={->p<&int32><null>:=&x};c.p=null;|c.p<null>|d.print("local null")}}
|s.c<H>|{|s.c.p<null>|d.print("result null")}
"#,
    )
    .runs(b"2\n2\n4\nlocal null\nresult null\n");
}

#[test]
pub fn widened_alias_restarts_reset_optional_storage_and_keep_copies() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:2
flag:=true
n:=0
r:'loop{
    |flag|{
        'loop->p:=&x
        old:p
        p=&y
        d.print(*old)
    }
    n=n+1
    |n<2|'loop.restart()
}
x=3
|r.p<&int32>|d.print(*(r.p))
d.print(x)
"#,
    )
    .runs(b"1\n1\n2\n3\n");
}

#[test]
pub fn widened_alias_loans_lifetimes_and_union_view_gates_are_checked() {
    for (source, code) in [
        (
            "x:=1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;old:p;p=&y;x=3;v:*old}}",
            "E302",
        ),
        (
            "x:1;y:=2;flag:=true;r:'out{|flag|{'out->p:=&x;p=&y}};y=3;|r.p<&int32>|{v:*(r.p)}",
            "E302",
        ),
        (
            "x:1;y:2;flag:=true;r:'out{|flag|{'out->p:=&x;cell:&p;p=&y;v:**cell}}",
            "E302",
        ),
        (
            "outer:1;flag:=true;result:{local:2;r:'out{|flag|{'out->p:=&outer;p=&local}};->r}",
            "E303",
        ),
        (
            "x:1;y:2;flag:=true;n:=2;r:'out{|flag|{'loop{'out->p:=&x;p=&y;n=n-1;|n>0|'loop.restart()}}};copy:r",
            "B001",
        ),
        ("x:1;flag:=true;r:'out{|flag|{'out->p:&x;p=&x}}", "E305"),
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
