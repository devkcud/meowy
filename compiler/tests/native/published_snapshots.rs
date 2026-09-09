use super::Case;

#[test]
pub fn published_snapshot_tracking_preserves_rhs_effects_and_lexical_scope() {
    Case::new(
        r#"
d:@"debug"
x:1
y:2
n:=0
r:'out{
    {'out->p:=&x;p={n=n+1;->&y}}
    probe:{}
}
d.print(*(r.p))
d.print(n)
"#,
    )
    .runs(b"2\n1\n");
}

#[test]
pub fn published_snapshot_tracking_preserves_readonly_outer_slots_across_restart() {
    Case::new(
        r#"
d:@"debug"
x:7
y:9
n:=0
q:=&x
r:'out{
    {'out->p<&int32><null>:=&x}
    'loop{q=&y;n=n+1;|n<2|'loop.restart()}
    d.print(*q)
}
|r.p<&int32>|d.print(*(r.p))
"#,
    )
    .runs(b"9\n7\n");
}
