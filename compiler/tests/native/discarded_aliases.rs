use super::Case;

#[test]
pub fn discarded_alias_writes_keep_copies_and_local_effects_before_leave() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:2
'out{
    r:{
        ->p:=&x
        old:p
        p=&y
        d.print(*old)
        x=3
        d.print(*p)
        'out.leave()
    }
}
d.print(x)
"#,
    )
    .runs(b"1\n2\n3\n");
}

#[test]
pub fn discarded_alias_fields_preserve_rhs_and_nested_leave() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
'out{
    r:{
        ->c:={->p:=&x;->q:=&y;->n:=0}
        c.p={c.q=&x;->&y}
        d.print(*c.q)
        'inner{c.p={c={->p:=&y;->q:=&x;->n:=3};'inner.leave();->&x}}
        d.print(*c.p)
        d.print(c.n)
        'out.leave()
    }
}
"#,
    )
    .runs(b"1\n2\n3\n");
}

#[test]
pub fn discarded_aliases_repair_expiry_and_survive_inner_restarts() {
    Case::new(
        r#"
d:@"debug"
x:9
'out{
    r:{
        ->p<&int32><null>:=null
        {short:7;p=&short}
        |p<&int32>|{d.print("expired");p=null}
        n:=2
        'loop{
            p=&x
            |p<&int32>|d.print(*p)
            n=n-1
            |n>0|'loop.restart()
        }
        'out.leave()
    }
}
"#,
    )
    .runs(b"expired\n9\n9\n");
}

#[test]
pub fn discarded_alias_cell_lifetime_follows_its_target() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
'out{
    r:'target{
        cell:{'target->p:=&x;p=&y;->&p}
        d.print(**cell)
        n:=2
        'loop{d.print(**cell);n=n-1;|n>0|'loop.restart()}
        'out.leave()
    }
}
"#,
    )
    .runs(b"2\n2\n2\n");
}

#[test]
pub fn discarded_alias_expiry_loans_and_published_results_stay_checked() {
    for (source, code) in [
        (
            "x:=1;y:2;'out{r:{->p:=&x;old:p;p=&y;x=3;v:*old;'out.leave()}}",
            "E302",
        ),
        (
            "x:1;y:2;'out{r:{->p:=&x;cell:&p;p=&y;v:**cell;'out.leave()}}",
            "E302",
        ),
        ("x:1;'out{r:{->p:=&x;{y:2;p=&y};v:*p;'out.leave()}}", "E303"),
        (
            "x:1;'out{r:{->p:=&x;'loop{v:*p;y:2;p=&y;'loop.restart()};'out.leave()}}",
            "E303",
        ),
        (
            "x:1;y:2;first:&x;cell:=&first;'out{r:'target{{'target->p:=&x;p=&y;cell=&p};'out.leave()}};v:**cell",
            "E303",
        ),
        (
            "x:1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y}};v:*r.p",
            "B001",
        ),
        ("x:1;'out{r:{->p:&x;p=&x;'out.leave()}}", "E305"),
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

#[test]
pub fn discarded_alias_panic_preserves_completed_rhs_writes() {
    let source = r#"d:@"debug";x:1;y:2;r:{->p:=&x;p={p=&y;d.print(*p);d.panic("stop");->&x}}"#;
    let call = "d.panic(\"stop\")";
    let start = source.find(call).unwrap();
    let end = start + call.len();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"2\n");
        assert_eq!(
            output.stderr,
            format!("panic[P006]: stop at bytes {start}..{end}\n").as_bytes()
        );
    }
}
