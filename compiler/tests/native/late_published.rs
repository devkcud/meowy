use super::Case;

#[test]
pub fn late_published_initializer_restarts_and_rhs_leave_keep_effect_order() {
    Case::new(
        r#"
d:@"debug"
x:7
y:9
n:=3
r:'out{
    'loop{
        'out->p:={d.print(n);n=n-1;|n>0|'loop.restart();->&x}
        p={p=&y;'out.leave();->&x}
    }
}
d.print(*(r.p))
"#,
    )
    .runs(b"3\n2\n1\n9\n");
}

#[test]
pub fn late_published_conditional_union_views_keep_backing_and_defaults() {
    Case::new(
        r#"
d:@"debug"
make:(flag<boolean>,x<&int32>,y<&int32>)'out{
    n:=2
    'loop{
        n=n-1
        |n>0|'loop.restart()
        |flag|{'out->p<&int32><null>:=x;p=null;p=y}
        |!flag|{'out->p:="empty"}
    }
}
optional:(flag<boolean>,x<&int32>)'out{
    n:=2
    'loop{n=n-1;|n>0|'loop.restart();|flag|{'out->p:=x;p=x}}
}
x:7
y:9
a:make(true,&x,&y)
b:make(false,&x,&y)
c:optional(false,&x)
|a.p<&int32>|d.print(*(a.p))
|b.p<string>|d.print(b.p)
|c.p<null>|d.print("absent")
"#,
    )
    .runs(b"9\nempty\nabsent\n");
}

#[test]
pub fn late_published_records_keep_field_writes_and_cell_owners() {
    Case::new(
        r#"
d:@"debug"
x:7
y:9
n:=2
r:'out{
    cell:'cell{
        'loop{
            n=n-1
            |n>0|'loop.restart()
            'out->c:={->p:=&x;->q:=&y;->n:=0}
            c.p={c.q=&x;->&y}
            c.n=3
            'cell->&(c.p)
        }
    }
    d.print(**cell)
}
d.print(*(r.c.p))
d.print(*(r.c.q))
d.print(r.c.n)
"#,
    )
    .runs(b"9\n9\n7\n3\n");
}

#[test]
pub fn late_published_aliases_use_headers_only_for_inner_restarts() {
    Case::new(
        r#"
d:@"debug"
x:7
y:9
n:=2
m:=0
r:'out{
    'outer{
        n=n-1
        |n>0|'outer.restart()
        'out->p:=&x
        'inner{d.print(*p);p=&y;m=m+1;|m<2|'inner.restart()}
    }
}
d.print(*(r.p))
"#,
    )
    .runs(b"7\n9\n9\n");
}

#[test]
pub fn late_published_initialization_and_lifetimes_remain_checked() {
    for (source, code) in [
        (
            "x:1;y:2;n:=2;r:'out{'loop{'out->p:=&x;p=&y;n=n-1;|n>0|'loop.restart()}}",
            "B001",
        ),
        (
            "x:1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y;'out->p:=&x}}",
            "E205",
        ),
        (
            "x:1;y:=2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y}};y=3;v:*(r.p)",
            "E302",
        ),
        (
            "x:=1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;old:p;p=&y;x=3;v:*old}}",
            "E302",
        ),
        (
            "x:1;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;local:2;p=&local}}",
            "E303",
        ),
        (
            "x:1;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;'out->cell:&p}}",
            "E303",
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
