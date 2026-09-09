use super::Case;

#[test]
pub fn changing_published_loops_publish_current_values_and_keep_old_copies() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:2
n:=0
r:{
    ->p:=&x
    old:p
    'loop{d.print(*p);p=&y;n=n+1;|n<3|'loop.restart()}
    d.print(*old)
    x=3
}
d.print(*(r.p))
d.print(x)
"#,
    )
    .runs(b"1\n2\n2\n1\n2\n3\n");
}

#[test]
pub fn changing_published_backedges_and_null_results_keep_widened_tags() {
    Case::new(
        r#"
d:@"debug"
<R>:<{p<&int32><null><string>:=}>
x:=7
n:=0
r<R>:{
    ->p<&int32><null>:=null
    'loop{
        n=n+1
        |n==1|{p=&x;'loop.restart()}
        |p<&int32>|d.print(*p)
    }
}
|r.p<&int32>|d.print(*(r.p))
n=0
s<R>:{->p<&int32><null>:=&x;'loop{p=null;n=n+1;|n<2|'loop.restart()}}
x=8
|s.p<null>|d.print(x)
"#,
    )
    .runs(b"7\n7\n8\n");
}

#[test]
pub fn changing_published_field_stores_keep_siblings_and_single_rhs_effects() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
n:=0
r:{
    ->c:={->p:=&x;->q:=&y;->n:=0}
    'loop{
        d.print(*(c.p))
        c.p={d.print("rhs");c.q=&x;->&y}
        c.n=c.n+1
        n=n+1
        |n<2|'loop.restart()
    }
}
d.print(*(r.c.p))
d.print(*(r.c.q))
d.print(r.c.n)
"#,
    )
    .runs(b"1\nrhs\n2\nrhs\n2\n1\n2\n");
}

#[test]
pub fn changing_published_leave_and_lexical_exit_keep_the_last_assignment() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
n:=0
r:'out{
    ->p:=&x
    'loop{p={d.print("rhs");p=&y;n=n+1;|n<2|'loop.restart();'out.leave();->&x}}
}
d.print(*(r.p))
d.print(n)
n=0
s:{->p:=&x;'middle{'loop{n=n+1;|n<2|{p=&y;'loop.restart()};'middle.leave()}}}
d.print(*(s.p))
t:'out{{'out->p:=&x;'loop{p=&y;n=n+1;|n<4|'loop.restart()}}}
d.print(*(t.p))
"#,
    )
    .runs(b"rhs\nrhs\n2\n2\n2\n2\n");
}

#[test]
pub fn changing_published_nested_and_transitive_headers_keep_current_sources() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
z:3
n:=0
m:=0
r:{
    ->p:=&x
    'outer{
        d.print(*p)
        m=0
        'inner{p=&y;m=m+1;|m<2|'inner.restart()}
        p=&z
        n=n+1
        |n<2|'outer.restart()
    }
}
d.print(*(r.p))
a:&x
b:&y
s:{->p:=&a;'loop{p=&b;n=n+1;|n<4|'loop.restart()}}
d.print(*(*(s.p)))
"#,
    )
    .runs(b"1\n3\n3\n2\n");
}

#[test]
pub fn changing_published_conditional_views_preserve_completed_record_shape() {
    Case::new(
        r#"
d:@"debug"
make:(flag<boolean>,x<&int32>,y<&int32>)'out{
    |flag|{
        'out->p<&int32><null>:=null
        n:=0
        'loop{p=x;n=n+1;|n<2|'loop.restart();p=y}
    }
    |!flag|{'out->p:="none"}
}
x:7
y:9
a:make(true,&x,&y)
b:make(false,&x,&y)
|a.p<&int32>|d.print(*(a.p))
|b.p<string>|d.print(b.p)
"#,
    )
    .runs(b"9\nnone\n");
}

#[test]
pub fn changing_published_loans_expiry_and_initialization_gates_remain_checked() {
    for (source, code) in [
        (
            "x:1;y:=2;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{p=&y;'loop.restart()}}};y=3;v:*(r.p)",
            "E302",
        ),
        (
            "x:=1;y:2;n:=0;r:{->p:=&x;old:p;'loop{p=&y;n=n+1;|n<2|'loop.restart()};x=3;v:*old}",
            "E302",
        ),
        (
            "x:1;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{local:2;p=&local;'loop.restart()}}};v:*(r.p)",
            "E303",
        ),
        (
            "x:1;n:=0;r:{->p:=&x;'loop{n=n+1;|n<2|{p=&2;'loop.restart()}}};v:*(r.p)",
            "E303",
        ),
        (
            "x:1;y:2;n:=2;r:'out{'loop{'out->p:=&x;p=&y;n=n-1;|n>0|'loop.restart()}}",
            "B001",
        ),
        (
            "x:1;y:2;n:=0;r:{->p:=&x;cell:&p;'loop{p=&y;n=n+1;|n<2|'loop.restart()};v:**cell}",
            "E302",
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
