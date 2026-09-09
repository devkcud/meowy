use super::Case;

#[test]
pub fn borrowed_alias_writes_publish_current_values_and_release_replaced_loans() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:2
r:{
    ->p:=&x
    old:p
    p=&y
    d.print(*old)
    x=3
    d.print(*p)
}
d.print(*(r.p))
d.print(x)
safe:{local:4;s:{->p:=&local;p=&y};->s}
d.print(*(safe.p))
"#,
    )
    .runs(b"1\n2\n2\n3\n2\n");
}

#[test]
pub fn borrowed_alias_fields_keep_rhs_siblings_and_leave_results() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
r:'out{
    ->c:={->p:=&x;->q:=&y;->n:=0}
    c.p={c.q=&x;->&y}
    d.print(*(c.q))
    c.n={c={->p:=&x;->q:=&y;->n:=3};->4}
    c.p={c.q=&x;'out.leave();->&y}
}
d.print(*(r.c.p))
d.print(*(r.c.q))
d.print(r.c.n)
"#,
    )
    .runs(b"1\n1\n1\n4\n");
}

#[test]
pub fn borrowed_alias_branch_and_function_results_follow_completed_writes() {
    Case::new(
        r#"
d:@"debug"
make:(x<&int32>,y<&int32>,flag<boolean>)'out{
    ->p:=x
    |flag|{p=y;'out.leave()}
    p=x
}
x:1
y:2
a:make(&x,&y,true)
b:make(&x,&y,false)
d.print(*(a.p))
d.print(*(b.p))
choose:(flag<boolean>,x<&int32>,y<&int32>)'out{
    |flag|{'out->p:=x;p=y}
    |!flag|{'out->p:=y;p=x}
}
c:choose(true,&x,&y)
e:choose(false,&x,&y)
d.print(*(c.p))
d.print(*(e.p))
"#,
    )
    .runs(b"2\n1\n2\n1\n");
}

#[test]
pub fn borrowed_alias_nullable_tags_and_allocator_roles_stay_separate() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
f<m.Allocator>:(p<&int32>){->m.heap}
x:=1
y:2
r:{
    ->p<&int32><null>:=&x
    p=null
    x=3
    |p<null>|{copy:p;d.print("clear")}
    p=&y
    |p<&int32>|d.print(*p)
}
|r.p<&int32>|d.print(*(r.p))
s:{->c:={->p:=&x;->h:=m.heap};c.p=&y;c.h=f(&x);x=4}
copy:s
d.print(*(s.c.p))
d.print(x)
"#,
    )
    .runs(b"clear\n2\n2\n2\n4\n");
}

#[test]
pub fn borrowed_alias_result_loans_and_remaining_backing_gates_are_checked() {
    for (source, code) in [
        ("x:=1;y:2;r:{->p:=&x;old:p;p=&y;x=3;v:*old}", "E302"),
        ("x:1;y:=2;r:{->p:=&x;p=&y;y=3};v:*(r.p)", "E302"),
        ("x:1;y:2;r:{->p:=&x;cell:&p;p=&y;v:**cell}", "E302"),
        ("x:1;r:{->p:=&x;y:2;p=&y}", "E303"),
        ("x:1;r:{->c:{->p:=&x};c.p=&x}", "E305"),
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
