use super::Case;

#[test]
pub fn union_alias_writes_remap_tags_between_different_union_layouts() {
    Case::new(
        r#"
d:@"debug"
make:(x<&int32>,y<&int32>,flag<boolean>)'out{
    |flag|{
        'out->p<&int32><null>:=null
        p=x
        |p<&int32>|d.print(*p)
        p=null
        |p<null>|d.print("clear")
        p=y
    }
    |!flag|{'out->p:="before";p="after"}
}
x:7
y:9
a:make(&x,&y,true)
b:make(&x,&y,false)
|a.p<&int32>|d.print(*a.p)
|b.p<string>|d.print(b.p)
"#,
    )
    .runs(b"7\nclear\n9\nafter\n");
}

#[test]
pub fn union_alias_nested_members_and_leave_preserve_result_activity() {
    Case::new(
        r#"
d:@"debug"
<A>:<{p<&int32><null>}>
<B>:<{n<int32>}>
x:7
flag:=true
r:'out{
    |flag|{
        'out->c<A><B>:={->n:1}
        c={->p:&x}
        |c<A>|{|c.p<&int32>|d.print(*c.p)}
        c={c={->n:9};'out.leave();->{->p:&x}}
    }
    |!flag|{'out->c:=false}
}
|r.c<B>|d.print(r.c.n)
"#,
    )
    .runs(b"7\n9\n");
}

#[test]
pub fn declared_union_alias_backing_keeps_old_copies_and_current_sources() {
    Case::new(
        r#"
d:@"debug"
<R>:<{p<&int32><null><string>:=}>
x:=1
y:2
r<R>:{
    ->p<&int32><null>:=&x
    old:p
    p=&y
    |old<&int32>|d.print(*old)
    x=3
}
|r.p<&int32>|d.print(*r.p)
d.print(x)
"#,
    )
    .runs(b"1\n2\n3\n");
}

#[test]
pub fn union_alias_reset_iterations_preserve_local_and_backing_tag_domains() {
    Case::new(
        r#"
d:@"debug"
<R>:<{p<&int32><null><string>:=}>
x:9
n:=0
r<R>:'loop{
    ->p<&int32><null>:=null
    |p<null>|d.print("empty")
    p=&x
    n=n+1
    |n<2|'loop.restart()
}
|r.p<&int32>|d.print(*r.p)
"#,
    )
    .runs(b"empty\nempty\n9\n");
}

#[test]
pub fn union_alias_source_loans_addresses_and_surviving_results_stay_checked() {
    for (source, code) in [
        (
            "<R>:<{p<&int32><null><string>:=}>;x:=1;y:2;r<R>:{->p<&int32><null>:=&x;old:p;p=&y;x=3;copy:old}",
            "E302",
        ),
        (
            "<R>:<{p<&int32><null><string>:=}>;x:1;y:=2;r<R>:{->p<&int32><null>:=&x;p=&y};y=3;copy:r",
            "E302",
        ),
        (
            "<R>:<{p<&int32><null><string>:=}>;x:1;r<R>:{->p<&int32><null>:=null;local:2;p=&local}",
            "E303",
        ),
        (
            "<R>:<{p<&int32><null><string>:=}>;x:1;r<R>:{->p<&int32><null>:=null;cell:&p;p=&x}",
            "B001",
        ),
        (
            "<R>:<{p<&int32><null><string>:=}>;x:1;n:=2;r<R>:'out{'inner{n=n-1;|n>0|'inner.restart();'out->p<&int32><null>:=null;p=&x}}",
            "B001",
        ),
        (
            "<R>:<{p<&int32><null><string>:=}>;r<R>:{->p<&int32><null>:=null;p=\"x\"}",
            "E207",
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
