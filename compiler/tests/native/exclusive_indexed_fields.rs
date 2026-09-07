use super::Case;
use super::exclusive_references::rejects;

#[test]
pub fn indexed_scalar_fields_mutate_actual_storage_and_widths() {
    Case::new(
        r#"
d:@"debug"
rows:=[{->n:=1;->flag:=true;->small<int8>:=126;->f<float32>:=1.5}]
p:&!rows[1].n;*p=2;d.print(*p);d.print(rows[1].n)
q:&!rows[1].flag;*q=false;d.print(*q)
s:&!rows[1].small;*s=*s+1;d.print(*s)
t:&!rows[1].f;*t=2.5;d.print(*t)
"#,
    )
    .runs(b"2\n2\nfalse\n127\n2.5\n");
}

#[test]
pub fn mixed_index_field_paths_preserve_outer_siblings() {
    Case::new(
        r#"
d:@"debug"
r:={->rows:=[{->inner:={->cells:=[[{->n:=1}]]}}];->side:=2}
p:&!((r.rows)[1]).inner.cells[1][1].n;q:&!r.side
*p=3;*q=4;d.print(*p);d.print(*q);d.print(r.rows[1].inner.cells[1][1].n)
"#,
    )
    .runs(b"3\n4\n3\n");
}

#[test]
pub fn field_acquisition_evaluates_indexes_once_and_reserves_owners() {
    Case::new(
        r#"
d:@"debug"
pos<int32>:(v<int32>){d.print(v);->1}
r:={->rows:=[[{->n:=1}]];->side:=0}
p:&!r.rows[pos(7)][{r.side=2;d.print(r.rows[1][1].n);->pos(8)}].n
*p=3;d.print(*p);d.print(r.side)
"#,
    )
    .runs(b"7\n1\n8\n3\n2\n");
    for source in [
        "r:=[{->n:=1}];p:&!r[{r=[{->n:=2}];->1}].n",
        "r:=[{->n:=1;->b:=2}];p:&!r[{r[1].b=3;->1}].n",
        "r:=[[{->n:=1}]];p:&!r[1][{r[1]=[{->n:=2}];->1}].n",
        "r:=[{->n:=1}];p:&!r[{q:&!r[1].n;*q=2;->1}].n",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn scalar_field_loans_exclude_live_collection_access() {
    Case::new(
        r#"d:@"debug";r:=[{->a:=1;->b:=2}];s:&r[1].b;p:&!r[1].a;*p=3;d.print(*p);d.print(*s)"#,
    )
    .runs(b"3\n2\n");
    for source in [
        "r:=[{->n:=1}];p:&!r[1].n;v:r[1].n;w:*p",
        "r:=[{->n:=1}];p:&!r[1].n;r[1].n=2;w:*p",
        "r:=[{->n:=1}];p:&!r[1].n;r=[{->n:=2}];w:*p",
        "r:=[{->n:=1;->b:=2}];p:&!r[1].n;v:r[1].b;w:*p",
        "r:=[{->n:=1;->b:=2}];p:&!r[1].n;q:&!r[1].b;v:*p;w:*q",
        "r:=[{->n:=1},{->n:=2}];p:&!r[1].n;q:&!r[2].n;v:*p;w:*q",
        "r:=[{->n:=1}];s:&r[1].n;p:&!r[1].n;v:*s",
        "r:=[{->n:=1}];p:&!r[1].n;s:&r[1].n;v:*p",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn every_field_boundary_requires_mutability() {
    for source in [
        "r:[{->n:=1}];p:&!r[1].n",
        "r:=[{->n:1}];p:&!r[1].n",
        "r:=[{->inner:{->n:=1}}];p:&!r[1].inner.n",
        "r:=[{->inner:={->n:1}}];p:&!r[1].inner.n",
        "r:={->rows:[{->n:=1}]};p:&!r.rows[1].n",
        "r:{->rows:[{->n:=1}];p:&!rows[1].n}",
    ] {
        rejects(source, "E305");
    }
    rejects("r:=[{->n:=1}];p:&!r[1].missing", "E201");
}

#[test]
pub fn indexed_alias_fields_keep_actual_backing_and_target_lifetime() {
    Case::new(
        r#"
d:@"debug"
r:'out{p:{'out->rows:=[{->n:=1;->b:=2}];->&!rows[1].n};*p=3;d.print(*p)}
d.print(r.rows[1].n);d.print(r.rows[1].b)
s:{->row:={->rows:=[{->n:=4}]};p:&!row.rows[1].n;*p=5};d.print(s.row.rows[1].n)
"#,
    )
    .runs(b"3\n3\n2\n5\n");
    for source in [
        "p:{r:=[{->n:=1}];->&!r[1].n}",
        "x:=0;p:=&!x;r:{->rows:=[{->n:=1}];p=&!rows[1].n};v:*p",
        "r:{->rows:=[{->n:=1}];p:&!rows[1].n;->&*p}",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub fn indexed_fields_transfer_authority_through_moves_children_and_calls() {
    Case::new(
        r#"
d:@"debug"
id<&!int32>:(p<&!int32>){->p}
bump<null>:(p<&!int32>){*p=*p+1}
r:=[{->n:=1}];p:{->id(&!r[1].n)};q:p;s:&*q;d.print(*s);bump(&!*q);d.print(*q)
"#,
    )
    .runs(b"1\n2\n");
    rejects("r:=[{->n:=1}];p:&!r[1].n;q:p;v:*p", "E301");
    rejects("r:=[{->n:=1}];p:&!r[1].n;s:&*p;*p=2;v:*s", "E302");
    rejects("r:=[{->n:=1}];p:&!r[1].n;q:&!*p;v:*p;w:*q", "E302");
    rejects(
        "id<&!int32>:(p<&!int32>,flag<&boolean>){->p};r:=[{->n:=1}];b:=true;p:id(&!r[1].n,&b);b=false;v:*p",
        "E302",
    );
}

#[test]
pub fn cancelled_indexes_skip_field_acquisition_and_later_effects() {
    Case::new(
        r#"
d:@"debug"
pos<int32>:(){d.print("unreachable");->1}
r:=[{->n:=1}];'out{p:&!r[{r=[{->n:=2}];'out.leave()}].n};d.print(r[1].n)
rows:=[[{->n:=3}]];'out{p:&!rows[{rows=[[{->n:=4}]];'out.leave()}][pos()].n};d.print(rows[1][1].n)
'out{p:&!rows[1][{rows=[[{->n:=5}]];'out.leave()}].n};d.print(rows[1][1].n)
"#,
    )
    .runs(b"2\n4\n5\n");
    rejects(
        "rows:=[[{->n:=1}]];'out{p:&!rows[{rows=[[{->n:=2}]];->1}][{'out.leave()}].n}",
        "E302",
    );
}

#[test]
pub fn conditional_index_exits_preserve_old_field_handles() {
    Case::new(r#"
d:@"debug"
f<null>:(flag<boolean>){x:=9;p:=&!x;rows:=[{->n:=1}];'out{p=&!rows[{|flag|{rows=[{->n:=2}];'out.leave()};->1}].n};d.print(*p);d.print(rows[1].n)}
f(true);f(false)
"#).runs(b"9\n2\n1\n1\n");
}

#[test]
pub fn exact_alias_backing_covers_sibling_fields_and_capacity() {
    for source in [
        "f:(flag<boolean>)'out{|flag|{'out->rows:=[{->n:=1;->tag:=true}];p:&!rows[1].n};|!flag|'out->rows:=[{->n:=2;->tag:=3}]}",
        "f:(flag<boolean>)'out{|flag|{'out->rows:=[{->n:=1}];p:&!rows[1].n};|!flag|'out->rows:=[{->n:=2},{->n:=3}]}",
    ] {
        rejects(source, "B001");
    }
    Case::new(r#"d:@"debug";<R>:<{n<int32>:=}>;r<{rows<R[2]>:=}>:{->rows:=[{->n:=1}];p:&!rows[1].n;*p=2};d.print(r.rows[1].n)"#).runs(b"2\n");
}

#[test]
pub fn index_bounds_fail_before_field_projection_with_prefix_spans() {
    for (path, fault, output) in [
        ("rows[pos(2)][pos(1)].n", "rows[pos(2)]", "2\n"),
        ("rows[pos(1)][pos(2)].n", "rows[pos(1)][pos(2)]", "1\n2\n"),
    ] {
        let source = format!(
            "d:@\"debug\";<R>:<{{n<int32>:=}}>;pos<int32>:(at<int32>){{d.print(at);->at}};rows<R[3][3]>:=[[{{->n:=1}}]];p:&!{path}"
        );
        let start = source.find(fault).unwrap();
        let error = format!(
            "panic[P001]: index 2 is outside initialized length 1 at bytes {start}..{}\n",
            start + fault.len()
        );
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(result.status.code(), Some(1));
            assert_eq!(result.stdout, output.as_bytes());
            assert_eq!(result.stderr, error.as_bytes());
        }
    }
    rejects("r:=[{->n:=1}];p:&!r[0].n", "E101");
    rejects("r:=[{->n:=1}];p:&!r[true].n", "E222");
}

#[test]
pub fn captured_field_stores_and_short_circuit_keep_effect_order() {
    Case::new(
        r#"
d:@"debug"
a:=[{->n:=1}];b:=[{->n:=2}];p:=&!a[1].n
*p={p=&!b[1].n;->3};d.print(a[1].n);d.print(*p)
'out{*p={b=[{->n:=4}];'out.leave()}};d.print(b[1].n)
v:false&&(*(&!a[0].n)>0);a[1].n=5;d.print(a[1].n)
"#,
    )
    .runs(b"3\n2\n4\n5\n");
}

#[test]
pub fn cancelled_alias_layouts_and_panics_preserve_completed_work() {
    Case::new(r#"
d:@"debug"
f<null>:(flag<boolean>)'done{r:'out{|flag|{'out->rows:=[{->n:=1}];p:&!rows[1].n;*p=2;d.print(*p);'done.leave()};->rows:=true};d.print(r.rows)}
f(true);f(false)
"#).runs(b"2\ntrue\n");
    let case = Case::new(
        r#"d:@"debug";r:=[{->n:=1}];p:&!r[{r=[{->n:=2}];d.print(r[1].n);d.panic("stop")}].n"#,
    );
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"2\n");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
    }
}

#[test]
pub fn wider_leaves_and_reference_roots_remain_gated() {
    for source in [
        "r:=[{->n:=1}];s:=&r;p:&!s[1].n",
        "r:=[{->n:=1}];s:&r;p:&!(*s)[1].n",
        "p:&![{->n:=1}][1].n",
        "r:=[{->inner:={->n:=1}}];p:&!r[1].inner",
        "r:=[{->xs:=[1]}];p:&!r[1].xs",
        "r:=[{->s:=\"text\"}];p:&!r[1].s",
        "r:=[{->n:=1}];p:&!r[1].n;'again{'again.restart()}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub fn guarded_alias_views_preserve_indexed_field_identity() {
    Case::new(
        r#"
d:@"debug"
f<null>:(flag<boolean>){x:=0;p:=&!x;r:'out{
 |flag|{'out->rows:=[{->n:=1}];p=&!rows[1].n}
 |!flag|{'out->rows:=[{->n:=2}];p=&!rows[1].n}
 *p=*p+10;d.print(*p)
};p=&!x;d.print(r.rows[1].n)}
f(true);f(false)
"#,
    )
    .runs(b"11\n11\n12\n12\n");
}

#[test]
pub fn field_bounds_handle_signed_unsigned_and_empty_lists() {
    for (ty, at, values, detail) in [
        (
            "int8",
            "-1",
            "[{->n:=1}]",
            "index -1 is outside initialized length 1",
        ),
        (
            "uint64",
            "18446744073709551615",
            "[{->n:=1}]",
            "index 18446744073709551615 is outside initialized length 1",
        ),
        (
            "int32",
            "1",
            "[]",
            "index 1 is outside initialized length 0",
        ),
    ] {
        let source = format!(
            "<R>:<{{n<int32>:=}}>;f<null>:(at<{ty}>){{rows<R[3]>:={values};p:&!rows[at].n}};f({at})"
        );
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains(detail));
        }
    }
}
