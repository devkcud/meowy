use super::Case;

#[test]
pub fn fixed_published_results_keep_writes_outside_restarted_bodies() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:2
n:=0
r:{
    ->p:=&x
    old:p
    p=&y
    'loop{d.print(*p);n=n+1;|n<2|'loop.restart()}
    d.print(*old)
    x=3
}
d.print(*r.p)
d.print(x)
s:{->p:=&x;'loop{n=n+1;|n<4|'loop.restart()};p=&y}
d.print(*s.p)
"#,
    )
    .runs(b"2\n2\n1\n2\n3\n2\n");
}

#[test]
pub fn fixed_published_rhs_restarts_finish_once_and_keep_leave_effects() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
n:=0
r:{->p:=&x;p={'loop{d.print(*p);n=n+1;|n<2|'loop.restart()};->&y}}
d.print(*r.p)
s:'out{
    ->p:=&x
    p={p=&y;'loop{d.print(*p);n=n+1;|n<4|'loop.restart()};'out.leave();->&x}
}
d.print(*s.p)
d.print(n)
"#,
    )
    .runs(b"1\n1\n2\n2\n2\n2\n4\n");
}

#[test]
pub fn fixed_published_conditional_backing_keeps_variant_tags_and_absent_fields() {
    Case::new(
        r#"
d:@"debug"
make:(flag<boolean>,x<&int32>,y<&int32>)'out{
    |flag|{'out->p<&int32><null>:=x;p=y}
    |!flag|{'out->p:="empty";p="idle"}
    n:=0
    'loop{n=n+1;|n<2|'loop.restart()}
}
optional:(flag<boolean>,x<&int32>,y<&int32>)'out{
    |flag|{'out->p:=x;p=y}
    n:=0
    'loop{n=n+1;|n<2|'loop.restart()}
}
x:7
y:9
a:make(true,&x,&y)
b:make(false,&x,&y)
c:optional(false,&x,&y)
|a.p<&int32>|d.print(*a.p)
|b.p<string>|d.print(b.p)
|c.p<null>|d.print("absent")
"#,
    )
    .runs(b"9\nidle\nabsent\n");
}

#[test]
pub fn fixed_published_source_lifetimes_and_initialization_remain_checked() {
    for (source, code) in [
        (
            "x:1;y:=2;n:=0;r:{->p:=&x;p=&y;'loop{y=3;n=n+1;|n<2|'loop.restart()}};v:*r.p",
            "E302",
        ),
        (
            "x:=1;y:2;n:=0;r:{->p:=&x;old:p;p=&y;'loop{x=3;n=n+1;|n<2|'loop.restart()};v:*old}",
            "E302",
        ),
        (
            "x:1;n:=0;r:{->p:=&x;local:2;p=&local;'loop{n=n+1;|n<2|'loop.restart()}};v:*r.p",
            "E303",
        ),
        (
            "x:1;y:2;n:=0;r:'out{'loop{n=n+1;|n<2|'loop.restart();'out->p:=&x;p=&y}};v:*r.p",
            "B001",
        ),
        (
            "x:1;y:2;n:=0;r:'out{'loop{n=n+1;|n<2|'loop.restart();'out->p:=&x;p={p=&y;->&x}}}",
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
