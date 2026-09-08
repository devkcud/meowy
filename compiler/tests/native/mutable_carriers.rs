use super::Case;

#[test]
pub fn mutable_reference_carriers_preserve_replacement_and_field_rhs_effects() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:=2
r:={->p:&x;->n:=0}
old:r
r.n={r={->p:&y;->n:=3};->4}
d.print(*old.p)
x=5
d.print(*r.p)
d.print(r.n)
'out{r.n={r={->p:&x;->n:=6};'out.leave();->7}}
d.print(*r.p)
d.print(r.n)
"#,
    )
    .runs(b"1\n2\n4\n5\n6\n");
}

#[test]
pub fn nullable_reference_versions_support_expired_tag_inspection_and_repair() {
    Case::new(
        r#"
d:@"debug"
show<null>:(flag<boolean>){
    p<&int32><null>:=null
    |p<null>|d.print("initial")
    {x:7;|flag|p=&x}
    |p<null>|{copy:p;d.print("empty")}
    |p<&int32>|{d.print("expired");p=null;|p<null>|{copy:p;d.print("cleared")}}
}
show(true)
show(false)
"#,
    )
    .runs(b"initial\nexpired\ncleared\ninitial\nempty\n");
}

#[test]
pub fn reference_only_nested_variants_restart_with_current_activity() {
    Case::new(
        r#"
d:@"debug"
<A>:<{p<&int32><null>}>
<B>:<{n<int32>}>
r<A><B>:={->n:1}
x:7
n:=0
'loop{
    |r<B>|d.print(r.n)
    |r<A>|{
        |r.p<&int32>|d.print(*r.p)
        |r.p<null>|{copy:r.p;d.print("empty")}
    }
    n=n+1
    |n==1|r={->p:&x}
    |n==2|r={->p:null}
    |n<3|'loop.restart()
}
"#,
    )
    .runs(b"1\n7\nempty\n");
}

#[test]
pub fn reference_carriers_preserve_operand_versions_and_returned_input_bounds() {
    Case::new(
        r#"
d:@"debug"
same<boolean>:(a<&int32><null>,b<&int32><null>){->a==b}
make:(p<&int32>){->view:p;->n:3}
x:1
y:2
p<&int32><null>:=&x
d.print(same(p,{p=&y;->p}))
|p<&int32>|d.print(*p)
r:=make(&x)
r=make(&y)
d.print(*r.view)
d.print(r.n)
"#,
    )
    .runs(b"false\n2\n2\n3\n");
}

#[test]
pub fn mutable_reference_carriers_keep_expiry_loans_and_storage_gates() {
    for (source, code) in [
        (
            "p<&int32><null>:=null;{x:1;p=&x};|p<&int32>|{copy:*p}",
            "E303",
        ),
        (
            "p<&int32><null>:=null;|p<null>|{{x:1;p=&x};|p<&int32>|{copy:*p}}",
            "E303",
        ),
        (
            "p<&int32><null>:=null;'loop{|p<&int32>|{copy:*p};x:1;p=&x;'loop.restart()}",
            "E303",
        ),
        ("x:=1;y:2;r:={->p:&x};old:r;r={->p:&y};x=3;copy:old", "E302"),
        (
            "x:1;y:2;r:={->p:&x;->n:=0};cell:&r.n;r.n=2;copy:*cell",
            "E302",
        ),
        (
            "same<boolean>:(a<&int32><null>,b<&int32><null>){->a==b};x:=1;y:2;p<&int32><null>:=&x;v:same(p,{p=&y;x=3;->p})",
            "E302",
        ),
        (
            "x:1;r:{->p:=&x;p=&x;n:=2;'loop{n=n-1;|n>0|'loop.restart()}}",
            "B001",
        ),
        ("x:1;r:={->p:&x;->list:[1]}", "B001"),
        ("x:=1;r:={->p:&!x}", "B001"),
        ("x:1;r:={->p:&x};r.p=&x", "E305"),
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
