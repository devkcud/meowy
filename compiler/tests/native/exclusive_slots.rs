use super::Case;
use super::exclusive_references::rejects;

#[test]
pub fn emitted_scalar_borrows_mutate_the_completed_result_storage() {
    Case::new(
        r#"
d:@"debug"
mark<int32>:(){d.print("init");->1}
r:{->n:=mark();copy:n;p:&!n;*p=2;d.print(*p);d.print(copy);n=3}
d.print(r.n)
"#,
    )
    .runs(b"init\n2\n1\n3\n");
}

#[test]
pub fn emitted_scalar_widths_and_sibling_slots_remain_independent() {
    Case::new(
        r#"
d:@"debug"
r:{
 ->small<int8>:=126;->flag:=true;->value<float32>:=1.5
 p:&!small;q:&!flag;s:&!value
 *p=*p+1;*q=false;*s=2.5
 d.print(*p);d.print(*q);d.print(*s)
}
d.print(r.small);d.print(r.flag);d.print(r.value)
"#,
    )
    .runs(b"127\nfalse\n2.5\n127\nfalse\n2.5\n");
}

#[test]
pub fn emitted_slot_read_write_and_borrow_conflicts_are_canonical() {
    for source in [
        "r:{->n:=1;p:&!n;v:n;w:*p}",
        "r:{->n:=1;p:&!n;n=2;w:*p}",
        "r:{->n:=1;p:&!n;q:&!n;v:*p;w:*q}",
        "r:{->n:=1;s:&n;p:&!n;w:*s}",
        "r:{->n:=1;p:&!n;s:&n;w:*p}",
        "r:{->n:=1;p:&!n;*p={n=2;->3}}",
    ] {
        rejects(source, "E302");
    }
    Case::new(r#"d:@"debug";r:{->a:=1;->b:=2;p:&!a;b=3;d.print(b);*p=4;d.print(*p)};d.print(r.a)"#)
        .runs(b"3\n4\n4\n");
}

#[test]
pub fn nested_alias_scopes_keep_the_target_slot_alive() {
    Case::new(
        r#"
d:@"debug"
r:'out{
 p:{'out->n:=7;->&!n}
 *p=8;d.print(*p)
}
d.print(r.n)
"#,
    )
    .runs(b"8\n8\n");
}

#[test]
pub fn guarded_views_share_one_target_slot_without_losing_identity() {
    Case::new(
        r#"
d:@"debug"
f<null>:(flag<boolean>){
 local:=0;p:=&!local
 r:'out{
  |flag|{'out->n:=1;p=&!n}
  |!flag|{'out->n:=2;p=&!n}
  *p=*p+10;d.print(*p)
 }
 p=&!local;*p=3;d.print(r.n);d.print(*p)
}
f(true);f(false)
"#,
    )
    .runs(b"11\n11\n3\n12\n12\n3\n");
}

#[test]
pub fn slot_reference_moves_reinitialize_and_preserve_children() {
    Case::new(
        r#"
d:@"debug"
r:{
 ->a:=1;->b:=2;p:=&!a;q:p;p=&!b
 s:&*q;d.print(*s);*q=3;t:&!*p;*t=4;d.print(*t)
 *p=5;d.print(*q);d.print(*p)
}
d.print(r.a);d.print(r.b)
"#,
    )
    .runs(b"1\n4\n3\n5\n3\n5\n");
    rejects("r:{->n:=1;p:&!n;q:p;w:*p}", "E301");
    rejects(
        "f<null>:(flag<boolean>){r:{->n:=1;p:&!n;|flag|{q:p};w:*p}}",
        "E309",
    );
    rejects("r:{->n:=1;p:&!n;s:&*p;*p=2;w:*s}", "E302");
    rejects("r:{->n:=1;p:&!n;q:&!*p;w:*p;v:*q}", "E302");
}

#[test]
pub fn slot_pointers_survive_direct_calls_and_anonymous_results() {
    Case::new(
        r#"
d:@"debug"
id<&!int32>:(p<&!int32>){->p}
add<null>:(p<&!int32>,n<int32>){*p=*p+n}
r:{->n:=1;p:{->id(&!n)};add(&!*p,2);d.print(*p)}
d.print(r.n)
"#,
    )
    .runs(b"3\n3\n");
    rejects(
        "first<&!int32>:(p<&!int32>,b<&boolean>){->p};r:{->n:=1;->flag:=true;p:first(&!n,&flag);flag=false;w:*p}",
        "E302",
    );
}

#[test]
pub fn indirect_slot_targets_are_captured_before_holder_replacement() {
    Case::new(
        r#"
d:@"debug"
r:{->a:=1;->b:=2;p:=&!a;*p={p=&!b;->3};d.print(a);d.print(*p)}
d.print(r.a);d.print(r.b)
"#,
    )
    .runs(b"3\n2\n3\n2\n");
}

#[test]
pub fn leave_and_cancelled_result_cells_preserve_completed_effects() {
    Case::new(
        r#"
d:@"debug"
r:'out{->n:=1;p:&!n;*p={n=2;'out.leave()}}
d.print(r.n)
'done{r:{->n:=3;p:&!n;*p=4;d.print(*p);'done.leave()}}
'done{r:'out{p:{'out->n:=5;->&!n};*p=6;d.print(*p);'done.leave()}}
"#,
    )
    .runs(b"2\n4\n6\n");
    Case::new(r#"d:@"debug";f<null>:(flag<boolean>)'done{r:'out{|flag|{'out->n:=1;p:&!n;*p=2;d.print(*p);'done.leave()};->n:=true};d.print(r.n)};f(true);f(false)"#).runs(b"2\ntrue\n");
}

#[test]
pub fn expired_slot_and_self_containing_views_report_lifetime_errors() {
    for source in [
        "a:=0;p:=&!a;r:{->n:=1;p=&!n};w:*p",
        "r:{->n:=1;p:&!n;->&*p}",
        "bad<&!int32>:() 'out{r:{->n:=1;'out->&!n;'out.leave()}}",
        "a:=0;p:=&!a;'done{r:{->n:=1;p=&!n;'done.leave()}};w:*p",
        "first<&!int32>:(p<&!int32>,b<&boolean>){->p};r:'out{p:{'out->n:=1;flag:true;->first(&!n,&flag)};w:*p}",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub fn exclusive_slot_backing_types_must_remain_exact() {
    for source in [
        "f:(flag<boolean>)'out{|flag|{'out->n:=1;p:&!n;*p=2};|!flag|'out->n:=true}",
        "f:(flag<boolean>)'out{|flag|{'out->n:=1;p:&!n;w:*p}}",
        "r<{n<int32><null>:=}>:{->n:=1;p:&!n}",
        "r:{->n<int32><null>:=null;p:&!n}",
    ] {
        rejects(source, "B001");
    }
    Case::new(r#"d:@"debug";r<{n<int32>:=}>:{->n:=1;p:&!n;*p=2};d.print(r.n)"#).runs(b"2\n");
}

#[test]
pub fn immutable_and_wider_alias_boundaries_remain_explicit() {
    rejects("r:{->n:1;p:&!n}", "E305");
    rejects("p:&!n;r:{->n:=1}", "E201");
    for source in [
        "r:{->row:={->n:=1};p:&!row}",
        "r:{->label:=\"x\";p:&!label}",
        "r:{->n:=1;p:&!n;'again{'again.restart()}}",
        "r:{->n:=1;->&!n}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub fn panic_cancels_scalar_slot_storage_after_real_mutation() {
    let case = Case::new(r#"d:@"debug";r:{->n:=1;p:&!n;*p=2;d.print(*p);d.panic("stop")}"#);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"2\n");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
    }
}

#[test]
pub fn emitted_names_start_after_initializer_completion() {
    Case::new(r#"d:@"debug";n:=9;r:{->n:={p:&!n;*p=10;->1};q:&!n;*q=2};d.print(n);d.print(r.n)"#)
        .runs(b"10\n2\n");
    rejects("r:{->n:={p:&!n;->1}}", "E201");
}
