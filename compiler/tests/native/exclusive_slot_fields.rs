use super::Case;
use super::exclusive_references::rejects;

#[test]
pub fn emitted_field_borrows_mutate_actual_slots_once() {
    Case::new(r#"
d:@"debug"
mark<int32>:(){d.print("init");->1}
r:{->row:={->n:=mark();->flag:=true;->f<float32>:=1.5};copy:row;p:&!row.n;q:&!row.flag;s:&!row.f;*p=2;*q=false;*s=2.5;d.print(copy.n);d.print(*p);d.print(*q);d.print(*s)}
d.print(r.row.n);d.print(r.row.flag);d.print(r.row.f)
"#).runs(b"init\n1\n2\nfalse\n2.5\n2\nfalse\n2.5\n");
}

#[test]
pub fn emitted_siblings_and_primary_regions_remain_disjoint() {
    Case::new(
        r#"
d:@"debug"
r:{
 ->row:={->7;->inner:={->8;->n:=1;->other:=2};->tail:=3}
 p:&!((row).inner).n;q:&!row.tail
 a<int32>:row;b<int32>:row.inner
 row.inner.other=4;*p=5;*q=6
 d.print(a);d.print(b);d.print(row.inner.other);d.print(*p);d.print(*q)
}
"#,
    )
    .runs(b"7\n8\n4\n5\n6\n");
}

#[test]
pub fn alias_roots_and_every_crossed_field_require_mutability() {
    for source in [
        "r:{->row:{->n:=1};p:&!row.n}",
        "r:{->row:={->n:1};p:&!row.n}",
        "r:{->row:={->inner:{->n:=1}};p:&!row.inner.n}",
        "r:{->row:={->inner:={->n:1}};p:&!row.inner.n}",
    ] {
        rejects(source, "E305");
    }
    rejects("r:{->row:={->n:=1};p:&!row.missing}", "E201");
}

#[test]
pub fn alias_fields_ancestors_and_owner_replacements_conflict() {
    for source in [
        "r:{->row:={->n:=1};p:&!row.n;v:row.n;w:*p}",
        "r:{->row:={->n:=1};p:&!row.n;row.n=2;w:*p}",
        "r:{->row:={->n:=1};p:&!row.n;copy:row;w:*p}",
        "r:{->row:={->n:=1};p:&!row.n;row={->n:=2};w:*p}",
        "r:{->row:={->inner:={->n:=1}};p:&!row.inner.n;row.inner={->n:=2};w:*p}",
        "r:{->row:={->n:=1};s:&row;p:&!row.n;w:s.n}",
        "r:{->row:={->n:=1};p:&!row.n;s:&row;w:*p}",
        "r:{->row:={->n:=1};s:&row.n;p:&!row.n;w:*s}",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn projected_slot_pointers_outlive_lexical_alias_scope() {
    Case::new(
        r#"
d:@"debug"
r:'out{p:{'out->row:={->n:=1;->other:=2};->&!row.n};*p=3;d.print(*p)}
d.print(r.row.n);d.print(r.row.other)
"#,
    )
    .runs(b"3\n3\n2\n");
}

#[test]
pub fn guarded_record_views_preserve_target_identity() {
    Case::new(
        r#"
d:@"debug"
f<null>:(flag<boolean>){
 x:=0;p:=&!x
 r:'out{
  |flag|{'out->row:={->n:=1;->other:=3};p=&!row.n}
  |!flag|{'out->row:={->n:=2;->other:=4};p=&!row.n}
  *p=*p+10;d.print(*p)
 }
 p=&!x;*p=5;d.print(r.row.n);d.print(r.row.other);d.print(*p)
}
f(true);f(false)
"#,
    )
    .runs(b"11\n11\n3\n5\n12\n12\n4\n5\n");
}

#[test]
pub fn field_reference_moves_children_and_calls_keep_authority() {
    Case::new(r#"
d:@"debug"
id<&!int32>:(p<&!int32>){->p}
bump<null>:(p<&!int32>){*p=*p+1}
r:{->row:={->a:=1;->b:=2};p:=&!row.a;q:p;p=&!row.b;s:&*q;d.print(*s);bump(&!*q);t:{->id(p)};*t=3;d.print(*q);d.print(*t)}
d.print(r.row.a);d.print(r.row.b)
"#).runs(b"1\n2\n3\n2\n3\n");
    rejects("r:{->row:={->n:=1};p:&!row.n;q:p;w:*p}", "E301");
    rejects("r:{->row:={->n:=1};p:&!row.n;s:&*p;*p=2;w:*s}", "E302");
}

#[test]
pub fn exact_backing_covers_the_whole_record() {
    for source in [
        "f:(flag<boolean>)'out{|flag|{'out->row:={->n:=1;->tag:=true};p:&!row.n};|!flag|'out->row:={->n:=2;->tag:=3}}",
        "f:(flag<boolean>)'out{|flag|{'out->row:={->n:=1};p:&!row.n}}",
        "<R>:<{n<int32>:=}>;r<{row<R><null>:=}>:{->row:={->n:=1};p:&!row.n}",
    ] {
        rejects(source, "B001");
    }
    Case::new(r#"d:@"debug";<R>:<{n<int32>:=}>;r<{row<R>:=}>:{->row:={->n:=1};p:&!row.n;*p=2};d.print(r.row.n)"#).runs(b"2\n");
}

#[test]
pub fn captured_field_stores_preserve_rhs_changes_and_leave() {
    Case::new(
        r#"
d:@"debug"
r:'out{
 ->row:={->a:=1;->b:=2};p:=&!row.a
 *p={row.b=3;p=&!row.b;->4};d.print(row.a);d.print(*p)
 *p={row={->a:=5;->b:=6};'out.leave()}
}
d.print(r.row.a);d.print(r.row.b)
"#,
    )
    .runs(b"4\n3\n5\n6\n");
    rejects(
        "r:{->row:={->a:=1;->b:=2};p:&!row.a;*p={row={->a:=3;->b:=4};->5}}",
        "E302",
    );
}

#[test]
pub fn cancelled_record_backing_preserves_declared_layout_and_effects() {
    Case::new(
        r#"
d:@"debug"
'done{r:{->row:={->n:=1};p:&!row.n;*p=2;d.print(*p);'done.leave()}}
f<null>:(flag<boolean>)'done{
 r:'out{|flag|{'out->row:={->n:=3};p:&!row.n;*p=4;d.print(*p);'done.leave()};->row:={->flag:=true}}
 d.print(r.row.flag)
}
f(true);f(false)
"#,
    )
    .runs(b"2\n4\ntrue\n");
}

#[test]
pub fn target_scope_and_input_bounds_prevent_escaping_views() {
    for source in [
        "x:=0;p:=&!x;r:{->row:={->n:=1};p=&!row.n};w:*p",
        "r:{->row:={->n:=1};p:&!row.n;->&*p}",
        "bad<&!int32>:()'out{r:{->row:={->n:=1};'out->&!row.n;'out.leave()}}",
    ] {
        rejects(source, "E303");
    }
    rejects(
        "first<&!int32>:(p<&!int32>,flag<&boolean>){->p};r:{->row:={->n:=1;->flag:=true};p:first(&!row.n,&row.flag);row.flag=false;w:*p}",
        "E302",
    );
}

#[test]
pub fn copied_records_and_sibling_collections_remain_independent() {
    Case::new(r#"
d:@"debug"
r:{->row:={->n:=1;->items:=[2,3]};copy:=row;p:&!row.n;q:&!copy.n;row.items[1]=4;*p=5;*q=6;d.print(*p);d.print(*q);d.print(row.items[1])}
d.print(r.row.n)
"#).runs(b"5\n6\n4\n5\n");
}

#[test]
pub fn wider_alias_pointees_and_paths_remain_gated() {
    for source in [
        "r:{->row:={->n:=1};p:&!row}",
        "r:{->row:={->inner:={->n:=1}};p:&!row.inner}",
        "r:{->row:={->label:=\"x\"};p:&!row.label}",
        "r:{->row:={->n:=1};s:&row;p:&!s.n}",
        "r:{->row:={->n:=1};p:&!row.n;'again{'again.restart()}}",
        "x:1;r:{->row:={->n:=1;->view:&x};p:&!row.n}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub fn panic_cancels_projected_storage_after_mutation() {
    let case =
        Case::new(r#"d:@"debug";r:{->row:={->n:=1};p:&!row.n;*p=2;d.print(*p);d.panic("stop")}"#);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"2\n");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
    }
}
