use super::Case;

#[test]
pub fn tagged_allocator_predicates_follow_current_values_and_skip_expired_payloads() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
f<m.Allocator>:(x<&int32>){d.print(*x);->m.heap}
show<null>:(flag<boolean>){
    a<m.Allocator><null>:=null
    |a<null>|d.print("initial")
    {x:7;|flag|a=f(&x)}
    |a<null>|{copy:a;d.print("empty")}
    |a<m.Allocator>|{d.print("bounded");a=null;|a<null>|{copy:a;d.print("cleared")}}
}
show(true)
show(false)
"#,
    )
    .runs(b"initial\n7\nbounded\ncleared\ninitial\nempty\n");
}

#[test]
pub fn tagged_allocator_field_commits_preserve_rhs_effects_and_leave() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
f<m.Allocator>:(x<&int32>){->m.heap}
r:={->h<m.Allocator><null>:=null;->n:=0}
old:r
'out{x:1;r.h={r.h=f(&x);r.n=9;'out.leave();->null}}
|r.h<m.Allocator>|d.print(r.n)
|old.h<null>|d.print(old.n)
r.h={r.n=10;->null}
|r.h<null>|{copy:r;d.print(r.n)}
"#,
    )
    .runs(b"9\n0\n10\n");
}

#[test]
pub fn tagged_allocator_restarts_keep_nested_variants_and_ancestor_bounds() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
f<m.Allocator>:(x<&int32>){->m.heap}
<A>:<{h<m.Allocator><null>;flag<boolean>}>
<B>:<{n<int32>}>
a<A><B>:={->n:2}
x:7
n:=0
'loop{
    |a<B>|d.print(a.n)
    |a<A>|{
        |a.h<null>|{copy:a.h;d.print("empty")}
        |a.h<m.Allocator>|{copy:a.h;d.print("heap")}
    }
    n=n+1
    |n==1|a={->h:f(&x);->flag:true}
    |n==2|a={->flag:false}
    |n<3|'loop.restart()
}
"#,
    )
    .runs(b"2\nheap\nempty\n");
}

#[test]
pub fn tagged_allocator_expiry_stale_predicates_and_cell_conflicts_are_rejected() {
    let prefix = r#"m:@"memory";f<m.Allocator>:(x<&int32>){->m.heap};a<m.Allocator><null>:=null;"#;
    for (body, code) in [
        ("{x:1;a=f(&x)};|a<m.Allocator>|{copy:a}", "E303"),
        ("|a<null>|{{x:1;a=f(&x)};|a<m.Allocator>|{copy:a}}", "E303"),
        (
            "'loop{|a<m.Allocator>|{copy:a};x:1;a=f(&x);'loop.restart()}",
            "E303",
        ),
        ("|a<null>|{p:&a;a=m.heap;copy:*p}", "E302"),
        ("x:1;r:{->h<m.Allocator><null>:=f(&x)}", "B001"),
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
