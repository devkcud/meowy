use super::Case;

#[test]
pub fn allocator_carriers_replace_sources_and_preserve_rhs_effects() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
f<m.Allocator>:(p<&int32>){->m.heap}
x:=1
y:=2
r:={->h:=m.heap;->p:&x;->n:=0}
old:r
r.n={r={->h:=m.heap;->p:&y;->n:=7};->8}
d.print(*old.p)
x=3
r.h=f(&x)
x=4
copy:r.h
d.print(*r.p)
d.print(r.n)
'out{
    r.n={r={->h:=m.heap;->p:&x;->n:=9};'out.leave();->10}
}
d.print(*r.p)
d.print(r.n)
"#,
    )
    .runs(b"1\n2\n8\n4\n9\n");
}

#[test]
pub fn allocator_carriers_follow_nullable_reference_activity_in_function_bodies() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
show<null>:(flag<boolean>){
    r:={->h:m.heap;->p<&int32><null>:null}
    {x:7;|flag|r={->h:m.heap;->p<&int32><null>:&x}}
    |r.p<null>|{copy:r;d.print("empty")}
    |r.p<&int32>|{copy:r.h;d.print("expired");r={->h:m.heap;->p<&int32><null>:null}}
    copy:r
    d.print("clear")
}
show(true)
show(false)
"#,
    )
    .runs(b"expired\nclear\nempty\nclear\n");
}

#[test]
pub fn allocator_carriers_restart_with_live_and_inactive_reference_members() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
x:7
r:={->h:m.heap;->p<&int32><null>:null}
n:=0
'loop{
    |r.p<null>|{copy:r.p;d.print("empty")}
    |r.p<&int32>|d.print(*r.p)
    n=n+1
    |n==1|r={->h:m.heap;->p<&int32><null>:&x}
    |n==2|r={->h:m.heap;->p<&int32><null>:null}
    |n<3|'loop.restart()
}
copy:r
"#,
    )
    .runs(b"empty\n7\nempty\n");
}

#[test]
pub fn allocator_carrier_lifetimes_loans_and_remaining_gates_are_checked() {
    let prefix = r#"m:@"memory";f<m.Allocator>:(p<&int32>){->m.heap};x:=1;y:=2;r:={->h:=m.heap;->p:&x;->n:=0};"#;
    for (body, code) in [
        ("{z:3;r.h=f(&z)};copy:r", "E303"),
        ("{z:3;r={->h:=m.heap;->p:&z;->n:=0}};copy:r.p", "E303"),
        ("old:r;r={->h:=m.heap;->p:&y;->n:=0};x=3;copy:old", "E302"),
        ("r.n={r={->h:=m.heap;->p:&y;->n:=0};->1};y=3;copy:r", "E302"),
        (
            "'loop{copy:r.p;z:3;r={->h:=m.heap;->p:&z;->n:=0};'loop.restart()}",
            "E303",
        ),
        ("s:{->h:m.heap;->p:=&x;p=&x;'loop{'loop.restart()}}", "B001"),
        ("s:={->h:m.heap;->p:&x;->list:[1]}", "B001"),
        ("s:{->h:=f(&x);->p:&x}", "B001"),
    ] {
        let case = Case::new(&format!("{prefix}{body}"));
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with(&format!("error[{code}]:")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
