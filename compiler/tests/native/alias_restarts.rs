use super::Case;

#[test]
pub fn alias_restart_results_reset_slots_and_keep_the_final_write() {
    Case::new(
        r#"
d:@"debug"
x:=1
y:2
n:=0
r:'loop{
    ->p:=&x
    d.print(*p)
    p=&y
    n=n+1
    |n<3|'loop.restart()
}
x=3
d.print(*r.p)
d.print(x)
s:'loop{
    ->p:=&x
    n=n+1
    |n<5|{p=&y;d.print(*p);'loop.restart()}
}
d.print(*s.p)
"#,
    )
    .runs(b"1\n1\n1\n2\n3\n2\n3\n");
}

#[test]
pub fn alias_restart_nested_targets_preserve_rhs_and_leave_effects() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
n:=0
r:'outer{
    inner:{
        ->c:={->p:=&x;->q:=&y;->n:=0}
        n=n+1
        c.p={c.q=&x;|n<2|'outer.restart();->&y}
        c.n=4
    }
    ->inner
}
d.print(*r.c.p)
d.print(*r.c.q)
d.print(r.c.n)
s:'loop{
    ->p:=&x
    n=n+1
    p={p=&y;|n<4|'loop.restart();'loop.leave();->&x}
}
d.print(*s.p)
"#,
    )
    .runs(b"2\n1\n4\n2\n");
}

#[test]
pub fn nullable_alias_restarts_use_fresh_tags_and_canonical_input_versions() {
    Case::new(
        r#"
d:@"debug"
x:7
y:9
q:=&x
n:=0
r:'loop{
    ->p<&int32><null>:=null
    |p<null>|{copy:p;d.print("empty");p=q}
    |p<&int32>|d.print(*p)
    p=&y
    |p<&int32>|q=p
    n=n+1
    |n<2|'loop.restart()
}
|r.p<&int32>|d.print(*r.p)
"#,
    )
    .runs(b"empty\n7\nempty\n9\n9\n");
}

#[test]
pub fn independent_loops_do_not_block_exact_borrowed_alias_writes() {
    Case::new(
        r#"
d:@"debug"
n:=2
'first{n=n-1;|n>0|'first.restart()}
x:3
y:4
r:{->p:=&x;p=&y}
n=2
'second{n=n-1;|n>0|'second.restart()}
d.print(*r.p)
"#,
    )
    .runs(b"4\n");
}

#[test]
pub fn alias_restart_lifetimes_loans_and_surviving_result_gates_are_checked() {
    for (source, code) in [
        (
            "x:1;y:2;n:=2;r:{->p:=&x;'inner{p=&y;n=n-1;|n>0|'inner.restart()}};v:*r.p",
            "B001",
        ),
        (
            "x:1;y:2;r:'outer{'inner{'outer->p:=&x;p=&y;'inner.restart()}}",
            "B001",
        ),
        (
            "x:1;y:=2;n:=0;r:'loop{->p:=&x;p=&y;n=n+1;|n<2|'loop.restart()};y=3;v:*r.p",
            "E302",
        ),
        (
            "x:=1;y:2;n:=0;r:'loop{->p:=&x;old:p;p=&y;x=3;v:*old;n=n+1;|n<2|'loop.restart()}",
            "E302",
        ),
        (
            "x:1;first:&x;cell:=&first;n:=0;r:'loop{v:**cell;->p:=&x;p=&x;cell=&p;n=n+1;|n<2|'loop.restart()}",
            "E303",
        ),
        (
            "x:1;n:=0;r:'loop{->p:=&x;local:2;p=&local;n=n+1;|n<2|'loop.restart()}",
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
