use super::Case;
use super::exclusive_references::rejects;

#[test]
pub fn nested_elements_mutate_actual_scalar_storage() {
    Case::new(
        r#"
d:@"debug"
xs:=[[1,2],[3,4]];p:&!(xs[2][1]);*p=5;d.print(*p);d.print(xs[2][1]);d.print(xs[1][1])
bs:=[[[true]]];q:&!(bs[1][1][1]);*q=false;d.print(*q)
fs:=[[1.5]];r:&!(fs[1][1]);*r=2.5;d.print(*r)
"#,
    )
    .runs(b"5\n5\n1\nfalse\n2.5\n");
}

#[test]
pub fn mixed_field_index_paths_preserve_external_sibling_independence() {
    Case::new(
        r#"
d:@"debug"
r:={->rows:=[{->xs:=[1];->tag:=2}];->other:=[3]}
p:&!(((r.rows)[1]).xs[1]);q:&!(r.other[1]);*p=4;*q=5;d.print(*p);d.print(*q);d.print(r.rows[1].tag)
"#,
    )
    .runs(b"4\n5\n2\n");
}

#[test]
pub fn each_index_runs_once_and_can_read_reserved_owners() {
    Case::new(
        r#"
d:@"debug"
pos<int32>:(v<int32>){d.print(v);->1}
r:={->rows:=[{->xs:=[1]}];->side:=0}
p:&!(r.rows[pos(7)].xs[{r.side=3;d.print(r.rows[1].xs[1]);->pos(8)}])
*p=4;d.print(*p);d.print(r.side)
"#,
    )
    .runs(b"7\n1\n8\n4\n3\n");
}

#[test]
pub fn returning_indexes_reserve_all_enclosing_collections() {
    for source in [
        "xs:=[[1]];p:&!(xs[{xs=[[2]];->1}][1])",
        "xs:=[[1]];p:&!(xs[1][{xs=[[2]];->1}])",
        "xs:=[[1]];p:&!(xs[1][{xs[1]=[2];->1}])",
        "xs:=[[1]];p:&!(xs[1][{q:&!(xs[1][1]);*q=2;->1}])",
        "xs:=[[1]];p:&!(xs[1][1]);xs[1]=[2];v:*p",
        "xs:=[[1],[2]];p:&!(xs[1][1]);q:&!(xs[2][1]);v:*p;w:*q",
        "xs:=[[1]];s:&(xs[1][1]);p:&!(xs[1][1]);v:*s",
        "r:={->rows:=[{->xs:=[1];->tag:=0}]};p:&!(r.rows[1].xs[{r.rows[1].tag=2;->1}])",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn cancellation_at_any_index_skips_later_effects_and_acquisition() {
    Case::new(
        r#"
d:@"debug"
pos<int32>:(){d.print("unreachable");->1}
xs:=[[1]];'out{p:&!(xs[{xs=[[2]];'out.leave()}][pos()])};d.print(xs[1][1])
'out{p:&!(xs[1][{xs=[[3]];'out.leave()}])};d.print(xs[1][1])
ys:=[[[4]]];'out{p:&!(ys[1][{ys=[[[5]]];'out.leave()}][pos()])};d.print(ys[1][1][1])
"#,
    )
    .runs(b"2\n3\n5\n");
}

#[test]
pub fn later_cancellation_preserves_protection_needed_by_earlier_bounds() {
    for source in [
        "xs:=[[1]];'out{p:&!(xs[{xs=[[2]];->1}][{'out.leave()}])}",
        "xs:=[[[1]]];'out{p:&!(xs[1][{xs=[[[2]]];->1}][{'out.leave()}])}",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn conditional_cancellation_preserves_old_handles() {
    Case::new(r#"
d:@"debug"
f<null>:(flag<boolean>){x:=9;p:=&!x;xs:=[[1]];'out{p=&!(xs[1][{|flag|{xs=[[2]];'out.leave()};->1}])};d.print(*p);d.print(xs[1][1])}
f(true);f(false)
"#).runs(b"9\n2\n1\n1\n");
}

#[test]
pub fn initialized_bounds_fail_at_the_first_invalid_index() {
    for (path, expected, fault) in [
        ("xs[pos(2)][pos(1)]", "2\n", "xs[pos(2)]"),
        ("xs[pos(1)][pos(2)]", "1\n2\n", "&!(xs[pos(1)][pos(2)])"),
    ] {
        let source = format!(
            "d:@\"debug\";pos<int32>:(v<int32>){{d.print(v);->v}};xs<int32[3][3]>:=[[1]];p:&!({path})"
        );
        let start = source.find(fault).unwrap();
        let error = format!(
            "panic[P001]: index 2 is outside initialized length 1 at bytes {start}..{}\n",
            start + fault.len()
        );
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, expected.as_bytes());
            assert_eq!(output.stderr, error.as_bytes());
        }
    }
    rejects("xs:=[[1]];p:&!(xs[0][1])", "E101");
    rejects("xs:=[[1]];p:&!(xs[1][0])", "E101");
    rejects("xs:=[[1]];p:&!(xs[true][1])", "E222");
}

#[test]
pub fn nested_emitted_storage_follows_target_lifetime() {
    Case::new(r#"
d:@"debug"
id<&!int32>:(p<&!int32>){->p}
r:'out{p:{'out->rows:=[{->xs:=[1]}];->id(&!(rows[1].xs[1]))};q:p;s:&*q;d.print(*s);t:&!*q;*t=2;d.print(*t)}
d.print(r.rows[1].xs[1])
s:{->row:={->xs:=[[3]]};p:&!(row.xs[1][1]);*p=4};d.print(s.row.xs[1][1])
"#).runs(b"1\n2\n2\n4\n");
    for source in [
        "p:{xs:=[[1]];->&!(xs[1][1])}",
        "x:=0;p:=&!x;r:{->xs:=[[1]];p=&!(xs[1][1])};v:*p",
        "r:{->xs:=[[1]];p:&!(xs[1][1]);->&*p}",
    ] {
        rejects(source, "E303");
    }
    rejects("xs:=[[1]];p:&!(xs[1][1]);q:p;v:*p", "E301");
}

#[test]
pub fn mutable_boundaries_and_complete_backing_remain_required() {
    Case::new("xs:=[{->inner:{->ys:=[1]}}];p:&!(xs[1].inner.ys[1])").runs(b"");
    for source in [
        "xs:[[1]];p:&!(xs[1][1])",
        "xs:=[{->ys:[1]}];p:&!(xs[1].ys[1])",
        "r:{->xs:[[1]];p:&!(xs[1][1])}",
    ] {
        rejects(source, "E305");
    }
    rejects(
        "f:(flag<boolean>)'out{|flag|{'out->xs:=[[1]];p:&!(xs[1][1])};|!flag|'out->xs:=[[2,3]]}",
        "B001",
    );
    rejects(
        "f:(flag<boolean>)'out{|flag|{'out->xs:=[{->ys:=[1];->tag:=true}];p:&!(xs[1].ys[1])};|!flag|'out->xs:=[{->ys:=[2];->tag:=3}]}",
        "B001",
    );
}

#[test]
pub fn discarded_nested_backing_and_panic_preserve_completed_effects() {
    Case::new(r#"
d:@"debug"
f<null>:(flag<boolean>)'done{r:'out{|flag|{'out->xs:=[[1]];p:&!(xs[1][1]);*p=2;d.print(*p);'done.leave()};->xs:=true};d.print(r.xs)}
f(true);f(false)
"#).runs(b"2\ntrue\n");
    for path in [
        "xs[{xs=[[2]];d.print(xs[1][1]);d.panic(\"stop\")}][1]",
        "xs[1][{xs=[[2]];d.print(xs[1][1]);d.panic(\"stop\")}]",
    ] {
        let source = format!("d:@\"debug\";xs:=[[1]];p:&!({path})");
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, b"2\n");
            assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
        }
    }
}

#[test]
pub fn reference_temporary_and_non_scalar_targets_remain_gated() {
    for source in [
        "xs:=[[1]];s:=&xs;p:&!(s[1][1])",
        "xs:=[[1]];s:&xs;p:&!((*s)[1][1])",
        "p:&!([[1]][1][1])",
        "xs:=[[{->n:=1}]];p:&!(xs[1][1])",
        "xs:=[[1]];p:&!(xs[1])",
        "xs:=[[1]];p:&!(xs[1][1]);'again{'again.restart()}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub fn guarded_aliases_keep_nested_element_identity() {
    Case::new(
        r#"
d:@"debug"
f<null>:(flag<boolean>){
 x:=0;p:=&!x
 r:'out{
  |flag|{'out->xs:=[[1]];p=&!(xs[1][1])}
  |!flag|{'out->xs:=[[2]];p=&!(xs[1][1])}
  *p=*p+10;d.print(*p)
 }
 p=&!x;d.print(r.xs[1][1])
}
f(true);f(false)
"#,
    )
    .runs(b"11\n11\n12\n12\n");
}

#[test]
pub fn nested_store_capture_and_short_circuit_keep_evaluation_order() {
    Case::new(
        r#"
d:@"debug"
xs:=[[1]];ys:=[[2]];p:=&!(xs[1][1])
*p={p=&!(ys[1][1]);->3};d.print(xs[1][1]);d.print(*p)
'out{*p={ys=[[4]];'out.leave()}};d.print(ys[1][1])
v:false&&(*(&!(xs[0][0]))>0);xs[1][1]=5;d.print(xs[1][1])
"#,
    )
    .runs(b"3\n2\n4\n5\n");
}

#[test]
pub fn nested_bounds_preserve_signed_unsigned_and_empty_lengths() {
    for (ty, at, values, path, detail) in [
        (
            "int8",
            "-1",
            "[[1]]",
            "xs[at][1]",
            "index -1 is outside initialized length 1",
        ),
        (
            "uint64",
            "18446744073709551615",
            "[[1]]",
            "xs[1][at]",
            "index 18446744073709551615 is outside initialized length 1",
        ),
        (
            "int32",
            "1",
            "[[]]",
            "xs[1][at]",
            "index 1 is outside initialized length 0",
        ),
        (
            "int32",
            "1",
            "[]",
            "xs[at][1]",
            "index 1 is outside initialized length 0",
        ),
    ] {
        let source =
            format!("f<null>:(at<{ty}>){{xs<int32[3][3]>:={values};p:&!({path})}};f({at})");
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(detail),
                "{source}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
