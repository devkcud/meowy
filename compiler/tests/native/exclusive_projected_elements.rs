use super::Case;
use super::exclusive_references::rejects;

#[test]
pub fn projected_elements_mutate_actual_storage_and_preserve_siblings() {
    Case::new(
        r#"
d:@"debug"
r:={->7;->inner:={->8;->xs:=[1,2];->ys:=[3]};->tail:=4}
p:&!(((r).inner).xs[1]);q:&!(r.inner.ys[1])
a<int32>:r;b<int32>:r.inner;r.tail=5;*p=6;*q=9
d.print(a);d.print(b);d.print(r.tail);d.print(*p);d.print(*q);d.print(r.inner.xs[1])
"#,
    )
    .runs(b"7\n8\n5\n6\n9\n6\n");
}

#[test]
pub fn emitted_lists_and_fields_keep_exact_slot_storage() {
    Case::new(r#"
d:@"debug"
r:{->xs:=[1,2];p:&!(xs[1]);*p=3;d.print(*p)};d.print(r.xs[1])
s:{->row:={->xs:=[true];->ys<float32[1]>:=[1.5]};p:&!(row.xs[1]);q:&!(row.ys[1]);*p=false;*q=2.5;d.print(*p);d.print(*q)}
d.print(s.row.xs[1]);d.print(s.row.ys[1])
"#).runs(b"3\n3\nfalse\n2.5\nfalse\n2.5\n");
}

#[test]
pub fn projected_reservations_allow_sibling_effects_and_shared_reads() {
    Case::new(
        r#"
d:@"debug"
r:{->row:={->xs:=[1,2];->ys:=[3]}
p:&!(row.xs[{row.ys[1]=4;s:&(row.xs[1]);d.print(*s);->*s}])
q:&!(row.ys[1]);*p=5;*q=6;d.print(*p);d.print(*q)}
d.print(r.row.xs[1]);d.print(r.row.ys[1])
"#,
    )
    .runs(b"1\n5\n6\n5\n6\n");
}

#[test]
pub fn returning_indices_reserve_the_selected_list_and_ancestors() {
    for source in [
        "r:={->xs:=[1,2]};p:&!(r.xs[{r.xs[2]=3;->1}])",
        "r:={->xs:=[1,2]};p:&!(r.xs[{r.xs=[3,4];->1}])",
        "r:={->xs:=[1,2]};p:&!(r.xs[{r={->xs:=[3,4]};->1}])",
        "r:{->xs:=[1,2];p:&!(xs[{xs=[3,4];->1}])}",
        "r:{->row:={->xs:=[1,2]};p:&!(row.xs[{row.xs[2]=3;->1}])}",
        "r:{->row:={->xs:=[1,2]};p:&!(row.xs[{q:&!(row.xs[2]);*q=3;->1}])}",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn live_projected_elements_exclude_collection_and_ancestor_access() {
    for source in [
        "r:={->xs:=[1,2]};p:&!(r.xs[1]);v:r.xs[2];w:*p",
        "r:={->xs:=[1,2]};p:&!(r.xs[1]);q:&!(r.xs[2]);v:*p;w:*q",
        "r:{->row:={->xs:=[1,2]};s:&(row.xs);p:&!(row.xs[1]);v:*s}",
        "r:{->row:={->xs:=[1,2]};p:&!(row.xs[1]);v:row;w:*p}",
        "r:{->row:={->xs:=[1,2]};p:&!(row.xs[1]);v:row.xs.size();w:*p}",
        "r:{->xs:=[1,2];p:&!(xs[1]);xs=[3,4];w:*p}",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn all_owned_path_boundaries_require_mutability() {
    Case::new("r:{->xs:=[1]};p:&!(r.xs[1])").runs(b"");
    Case::new("r:={->inner:{->xs:=[1]}};p:&!(r.inner.xs[1])").runs(b"");
    Case::new("r:{->row:{->xs:=[1]};p:&!(row.xs[1])}").runs(b"");
    for source in [
        "r:={->xs:[1]};p:&!(r.xs[1])",
        "r:{->xs:[1];p:&!(xs[1])}",
        "r:{->row:={->xs:[1]};p:&!(row.xs[1])}",
    ] {
        rejects(source, "E305");
    }
    rejects("r:={->xs:=[1]};p:&!(r.missing[1])", "E201");
}

#[test]
pub fn emitted_element_backing_must_match_the_complete_owner() {
    for source in [
        "f:(flag<boolean>)'out{|flag|{'out->xs:=[1];p:&!(xs[1])};|!flag|'out->xs:=[2,3]}",
        "f:(flag<boolean>)'out{|flag|{'out->row:={->xs:=[1]};p:&!(row.xs[1])};|!flag|'out->row:={->xs:=[2,3]}}",
        "f:(flag<boolean>)'out{|flag|{'out->row:={->xs:=[1];->tag:=true};p:&!(row.xs[1])};|!flag|'out->row:={->xs:=[2];->tag:=3}}",
        "f:(flag<boolean>)'out{|flag|{'out->xs:=[1];p:&!(xs[1])}}",
    ] {
        rejects(source, "B001");
    }
    Case::new(
        r#"d:@"debug";r<{xs<int32[2]>:=}>:{->xs<int32[2]>:=[1];p:&!(xs[1]);*p=2};d.print(r.xs[1])"#,
    )
    .runs(b"2\n");
}

#[test]
pub fn element_handles_follow_target_scope_through_calls_and_blocks() {
    Case::new(r#"
d:@"debug"
id<&!int32>:(p<&!int32>){->p}
bump<null>:(p<&!int32>){*p=*p+1}
r:'out{p:{'out->row:={->xs:=[1];->ys:=[2]};->id(&!(row.xs[1]))};q:p;s:&*q;d.print(*s);bump(&!*q);d.print(*q)}
d.print(r.row.xs[1]);d.print(r.row.ys[1])
"#).runs(b"1\n2\n2\n2\n");
    for source in [
        "p:{r:={->xs:=[1]};->&!(r.xs[1])}",
        "x:=0;p:=&!x;r:{->xs:=[1];p=&!(xs[1])};w:*p",
        "r:{->row:={->xs:=[1]};p:&!(row.xs[1]);->&*p}",
    ] {
        rejects(source, "E303");
    }
    rejects("r:{->xs:=[1];p:&!(xs[1]);q:p;w:*p}", "E301");
}

#[test]
pub fn cancelled_indices_release_projected_reservations_and_keep_effects() {
    Case::new(r#"
d:@"debug"
r:'out{->row:={->xs:=[1]};p:&!(row.xs[{row.xs=[2];'out.leave()}])};d.print(r.row.xs[1])
f<null>:(flag<boolean>){x:=9;p:=&!x;r:={->xs:=[1]};'out{p=&!(r.xs[{|flag|{r.xs=[3];'out.leave()};->1}])};d.print(*p);d.print(r.xs[1])}
f(true);f(false)
"#).runs(b"2\n9\n3\n1\n1\n");
    let case =
        Case::new(r#"d:@"debug";r:{->xs:=[1];p:&!(xs[{xs=[2];d.print(xs[1]);d.panic("stop")}]) }"#);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"2\n");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
    }
}

#[test]
pub fn cancelled_emissions_use_the_original_list_layout() {
    Case::new(r#"
d:@"debug"
f<null>:(flag<boolean>)'done{
 r:'out{|flag|{'out->row:={->xs:=[1]};p:&!(row.xs[1]);*p=2;d.print(*p);'done.leave()};->row:={->flag:=true}}
 d.print(r.row.flag)
}
f(true);f(false)
"#).runs(b"2\ntrue\n");
}

#[test]
pub fn projected_bounds_use_initialized_length_after_one_index_evaluation() {
    for source in [
        "r:={->xs<int32[3]>:=[1]};p:&!(r.xs[position()])",
        "r:{->xs<int32[3]>:=[1];p:&!(xs[position()])}",
        "r:{->row:={->xs<int32[3]>:=[1]};p:&!(row.xs[position()])}",
    ] {
        let source = format!("d:@\"debug\";position<int32>:(){{d.print(7);->2}};{source}");
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, b"7\n");
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("index 2 is outside initialized length 1")
            );
        }
    }
    rejects("r:={->xs:=[1]};p:&!(r.xs[0])", "E101");
}

#[test]
pub fn projected_reference_and_temporary_owners_remain_gated() {
    for source in [
        "r:={->xs:=[1]};view:=&r;p:&!(view.xs[1])",
        "r:={->xs:=[1]};view:&r;p:&!((*view).xs[1])",
        "p:&!(({->xs:=[1]}).xs[1])",
        "x:1;r:={->xs:=[1];->view:&x};p:&!(r.xs[1])",
        "r:{->xs:=[1];p:&!(xs[1]);'again{'again.restart()}}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub fn guarded_emitted_views_keep_canonical_element_identity() {
    Case::new(
        r#"
d:@"debug"
f<null>:(flag<boolean>){
 x:=0;p:=&!x
 r:'out{
  |flag|{'out->row:={->xs:=[1];->ys:=[3]};p=&!(row.xs[1])}
  |!flag|{'out->row:={->xs:=[2];->ys:=[4]};p=&!(row.xs[1])}
  *p=*p+10;d.print(*p)
 }
 p=&!x;*p=5;d.print(r.row.xs[1]);d.print(r.row.ys[1]);d.print(*p)
}
f(true);f(false)
"#,
    )
    .runs(b"11\n11\n3\n5\n12\n12\n4\n5\n");
}

#[test]
pub fn projected_element_store_captures_target_before_reassigning_handle() {
    Case::new(
        r#"
d:@"debug"
r:'out{
 ->row:={->xs:=[1];->ys:=[2]};p:=&!(row.xs[1])
 *p={p=&!(row.ys[1]);->3};d.print(row.xs[1]);d.print(*p)
 *p={row={->xs:=[4];->ys:=[5]};'out.leave()}
}
d.print(r.row.xs[1]);d.print(r.row.ys[1])
"#,
    )
    .runs(b"3\n2\n4\n5\n");
}
