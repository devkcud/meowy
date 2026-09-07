use super::Case;
use super::exclusive_references::rejects;

#[test]
pub fn exclusive_elements_mutate_actual_scalar_storage() {
    Case::new(
        r#"
d:@"debug"
xs:=[1,2];p:&!xs[1];*p=3;d.print(*p);d.print(xs[1]);d.print(xs[2])
bs:=[true,false];q:&!bs[1];*q=false;d.print(*q)
is<int8[1]>:=[126];r:&!is[1];*r=*r+1;d.print(*r)
fs<float32[1]>:=[1.5];s:&!fs[1];*s=2.5;d.print(*s)
"#,
    )
    .runs(b"3\n3\n2\nfalse\n127\n2.5\n");
}

#[test]
pub fn index_reservations_allow_reads_and_evaluate_effects_once() {
    Case::new(
        r#"
d:@"debug"
position<int32>:(){d.print("index");->1}
xs:=[1,2];p:&!xs[position()];*p=3;d.print(*p)
ys:=[1,2];q:&!ys[{s:&ys[1];d.print(*s);->*s}];*q=4;d.print(*q)
"#,
    )
    .runs(b"index\n3\n1\n4\n");
}

#[test]
pub fn returning_index_evaluation_keeps_the_owner_reserved() {
    for source in [
        "xs:=[1,2];p:&!xs[{xs=[3,4];->1}]",
        "xs:=[1,2];p:&!xs[{xs[2]=3;->1}]",
        "xs<int32[3]>:=[1];p:&!xs[{xs=xs.add(2);->1}]",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn elements_conservatively_exclude_other_collection_access() {
    for source in [
        "xs:=[1,2];p:&!xs[1];v:xs[2];w:*p",
        "xs:=[1,2];p:&!xs[1];xs[2]=3;w:*p",
        "xs:=[1,2];p:&!xs[1];xs=[3,4];w:*p",
        "xs:=[1,2];p:&!xs[1];q:&!xs[2];v:*p;w:*q",
        "xs:=[1,2];s:&xs[2];p:&!xs[1];w:*s",
        "xs:=[1,2];p:&!xs[1];s:&xs[2];w:*p",
        "xs:=[1,2];p:&!xs[1];n:xs.size();w:*p",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn moved_handles_and_reborrows_keep_element_authority() {
    Case::new(
        r#"
d:@"debug"
xs:=[1,2];p:=&!xs[1];q:p;s:&*q;d.print(*s);*q=3;d.print(*q)
p=&!xs[2];r:&!*p;*r=4;d.print(*r);*p=5;d.print(*p)
"#,
    )
    .runs(b"1\n3\n4\n5\n");
    rejects("xs:=[1];p:&!xs[1];q:p;v:*p", "E301");
    rejects("xs:=[1];p:&!xs[1];s:&*p;*p=2;v:*s", "E302");
    rejects("xs:=[1];p:&!xs[1];q:&!*p;v:*p;w:*q", "E302");
    rejects(
        "f<null>:(flag<boolean>){xs:=[1];p:&!xs[1];|flag|{q:p};v:*p}",
        "E309",
    );
}

#[test]
pub fn call_and_block_results_preserve_element_roots_and_bounds() {
    Case::new(
        r#"
d:@"debug"
id<&!int32>:(p<&!int32>){->p}
read<int32>:(p<&int32>){->*p}
xs:=[1,2];p:{->id(&!xs[1])};d.print(read(p));*p=3;d.print(*p)
"#,
    )
    .runs(b"1\n3\n");
    rejects(
        "id<&!int32>:(p<&!int32>,q<&boolean>){->p};xs:=[1];flag:=true;p:id(&!xs[1],&flag);flag=false;v:*p",
        "E302",
    );
}

#[test]
pub fn copying_a_shared_list_creates_an_independent_mutable_owner() {
    Case::new(r#"d:@"debug";xs:=[1];view:&xs;copy:=*view;p:&!copy[1];*p=2;d.print(*p);d.print((*view)[1])"#).runs(b"2\n1\n");
}

#[test]
pub fn nonreturning_indices_release_reservations_without_acquisition() {
    Case::new(
        r#"d:@"debug";xs:=[1];'out{p:&!xs[{xs=[2];d.print(xs[1]);'out.leave()}]};d.print(xs[1])"#,
    )
    .runs(b"2\n2\n");
    let case = Case::new(r#"d:@"debug";xs:=[1];p:&!xs[{xs=[2];d.print(xs[1]);d.panic("stop")}]"#);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"2\n");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
    }
}

#[test]
pub fn conditional_index_exits_preserve_old_reference_versions() {
    Case::new(
        r#"
d:@"debug"
f<null>:(flag<boolean>){
 a:=9;p:=&!a;xs:=[1,2]
 'out{p=&!xs[{|flag|{xs=[3,4];'out.leave()};->1}]}
 d.print(*p);d.print(xs[1])
}
f(true);f(false)
"#,
    )
    .runs(b"9\n3\n1\n1\n");
}

#[test]
pub fn element_store_targets_are_captured_once_before_rhs_effects() {
    Case::new(r#"d:@"debug";xs:=[1,2];ys:=[3];p:=&!xs[1];*p={p=&!ys[1];->4};d.print(xs[1]);d.print(*p);'out{*p={ys=[5];'out.leave()}};d.print(ys[1])"#).runs(b"4\n3\n5\n");
    rejects("xs:=[1];p:&!xs[1];*p={xs=[2];->3}", "E302");
}

#[test]
pub fn initialized_length_and_integer_widths_control_runtime_bounds() {
    for (ty, position, length) in [
        ("int8", "-1", 2),
        ("int32", "0", 2),
        ("int32", "3", 2),
        ("uint64", "18446744073709551615", 2),
        ("int32", "1", 0),
    ] {
        let values = if length == 0 { "[]" } else { "[1,2]" };
        let source =
            format!("f<null>:(at<{ty}>){{xs<int32[3]>:={values};p:&!xs[at]}};f({position})");
        let start = source.find("&!xs[at]").unwrap();
        let expected = format!(
            "panic[P001]: index {position} is outside initialized length {length} at bytes {start}..{}\n",
            start + 8
        );
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, expected.as_bytes(), "{source}");
        }
    }
    let case = Case::new("xs<int32[3]>:=[1];p:&!xs[2]");
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("index 2 is outside initialized length 1")
        );
    }
}

#[test]
pub fn failing_bounds_keep_index_effects_once_and_handle_zero_capacity() {
    for (source, expected, detail) in [
        (
            "d:@\"debug\";position<int32>:(){d.print(77);->3};xs<int32[3]>:=[1];p:&!xs[position()]",
            b"77\n".as_slice(),
            "index 3 is outside initialized length 1",
        ),
        (
            "xs<int32[0]>:=[];at:=1;p:&!xs[at]",
            b"".as_slice(),
            "index 1 is outside initialized length 0",
        ),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, expected);
            assert!(String::from_utf8_lossy(&output.stderr).contains(detail));
        }
    }
}

#[test]
pub fn static_positions_mutability_and_owner_lifetimes_keep_exact_codes() {
    for source in [
        "xs:=[1];p:&!xs[0]",
        "xs:=[1];p:&!xs[-1]",
        "xs:=[1];p:&!xs[2]",
    ] {
        rejects(source, "E101");
    }
    rejects("xs:=[1];p:&!xs[true]", "E222");
    rejects("xs:[1];p:&!xs[1]", "E305");
    rejects("f<null>:(xs<int32[2]>){p:&!xs[1]}", "E305");
    rejects("p:{xs:=[1];->&!xs[1]}", "E303");
    rejects("bad<&!int32>:(){xs:=[1];->&!xs[1]}", "E303");
}

#[test]
pub fn broader_roots_and_reference_upgrades_remain_gated() {
    for source in [
        "xs:=[1];view:=&xs;p:&!view[1]",
        "xs:=[1];view:&xs;p:&!(*view)[1]",
        "r:{->xs:=[1];p:&!xs[1]}",
        "r:={->xs:=[1]};p:&!r.xs[1]",
        "p:&![1][1]",
        "xs:=[[1]];p:&!xs[1][1]",
        "xs:=[{->n:=1}];p:&!xs[1].n",
        "xs:=[{->n:=1}];p:&!xs[1]",
        "xs:=[\"text\"];p:&!xs[1]",
        "xs:=[1];p:&!xs",
        "xs:=[1];p:&!xs[1];'again{'again.restart()}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub fn short_circuit_acquisitions_end_at_their_last_use() {
    Case::new(
        r#"
d:@"debug"
xs:=[1];v:false&&(*(&!xs[0])>0);xs[1]=2;d.print(xs[1])
f<null>:(flag<boolean>){ys:=[1];v:flag&&(*(&!ys[1])>0);ys[1]=3;d.print(v);d.print(ys[1])}
f(true);f(false)
"#,
    )
    .runs(b"2\ntrue\n3\nfalse\n3\n");
}
