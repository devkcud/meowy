use super::Case;

#[test]
pub fn reference_field_construction_reads_aliases_and_preserves_old_copies() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:2
r:={->p:=&x;d.print(*p);->q:=&y;->n:=0}
old:r.p
r.p=&y
d.print(*old)
x=3
d.print(*r.p)
d.print(*r.q)
d.print(x)
s:{->c:={->p:=&x};copy:=c;copy.p=&y;d.print(*copy.p)}
d.print(*s.c.p)
"#,
    )
    .runs(b"1\n1\n2\n2\n3\n2\n3\n");
}

#[test]
pub fn nested_reference_fields_preserve_rhs_and_leave_effects() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
r:={->c:={->p:=&x;->n:=0};->q:=&y}
r.c.p={r.q=&x;->&y}
d.print(*r.c.p)
d.print(*r.q)
r.c.p={r={->c:={->p:=&x;->n:=3};->q:=&y};->&y}
d.print(r.c.n)
'out{r.c.p={r.q=&x;'out.leave();->&x}}
d.print(*r.c.p)
d.print(*r.q)
"#,
    )
    .runs(b"2\n1\n3\n2\n1\n");
}

#[test]
pub fn nullable_reference_fields_use_current_tags_in_functions_and_restarts() {
    Case::new(
        r#"
d:@"debug"
<R>:<{p<&int32><null>:=;n<int32>:=}>
show<null>:(flag<boolean>){
    r<R>:={->n:=0}
    {x:7;|flag|r.p=&x}
    |r.p<null>|{copy:r;d.print("empty")}
    |r.p<&int32>|{d.print("expired");r.p=null;copy:r;d.print("clear")}
}
show(true)
show(false)
x:9
r<R>:={->p:=null;|p<null>|{copy:p;d.print("initial")};->n:=0}
'loop{
    |r.p<null>|d.print("null")
    |r.p<&int32>|d.print(*r.p)
    r.n=r.n+1
    |r.n==1|r.p=&x
    |r.n==2|r.p=null
    |r.n<3|'loop.restart()
}
"#,
    )
    .runs(b"expired\nclear\nempty\ninitial\nnull\n9\nnull\n");
}

#[test]
pub fn reference_fields_retain_transitive_and_allocator_lifetime_roles() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
f<m.Allocator>:(p<&int32>){->m.heap}
x:=1
y:2
p:&x
q:&y
r:={->p:=&p;->h:=m.heap}
r.p=&q
r.h=f(&x)
x=3
copy:r
d.print(**r.p)
d.print(x)
"#,
    )
    .runs(b"2\n3\n");
}

#[test]
pub fn reference_field_lifetimes_alias_writes_and_mutability_are_checked() {
    for (source, code) in [
        ("x:1;r:={->p:=&x};{y:2;r.p=&y};v:*r.p", "E303"),
        ("x:=1;y:2;r:={->p:=&x};old:r.p;r.p=&y;x=3;v:*old", "E302"),
        ("x:1;y:2;r:={->p:=&x};cell:&r.p;r.p=&y;v:**cell", "E302"),
        (
            "r:={->p<&int32><null>:=null};'loop{|r.p<&int32>|{v:*r.p};x:1;r.p=&x;'loop.restart()}",
            "E303",
        ),
        (
            "x:1;y:2;r:{->p:=&x;n:=2;'loop{p=&y;n=n-1;|n>0|'loop.restart()}}",
            "B001",
        ),
        (
            "x:1;y:2;r:{->c:={->p:=&x};n:=2;'loop{c.p=&y;n=n-1;|n>0|'loop.restart()}}",
            "B001",
        ),
        (
            "x:1;r:{->c:={->p:=&x;->n:=0};n:=2;'loop{c.n=1;n=n-1;|n>0|'loop.restart()}}",
            "B001",
        ),
        ("r:{x:1;->p:=&x}", "E303"),
        ("x:1;r:{->p:=&x};r.p=&x", "E305"),
        ("x:1;r:={->c:{->p:=&x}};r.c.p=&x", "E305"),
        ("<R>:<{p<&int32>:=}>;x:1;r<R>:{->p:&x}", "E206"),
        ("x:1;r:={->p:=&[1]}", "B001"),
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
