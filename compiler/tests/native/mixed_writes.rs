use super::*;

#[test]
pub fn mixed_write_paths_preserve_nested_payloads_lengths_and_copies() {
    Case::new(
        r#"
d:@"debug"
<Item>:<{n<int32>:=;label<string>}>
holder:={->items<Item[3]>:=[{->n:=1;->label:"first"},{->n:=2;->label:"second"}];->other:=9}
copy:holder
holder.items[2].n=7
holder.items[1].n=holder.items[2].n+1
d.print(holder.items[1].n);d.print(holder.items[2].n);d.print(copy.items[2].n)
d.print(holder.items[1].label);d.print(holder.items.size());d.print(holder.other)
<Row>:<{items<int32[3]>:=}>
rows<Row[2]>:=[{->items:=[1,2]},{->items:=[3]}]
rows[1].items[2]=11
rows[2].items=[4,5]
d.print(rows[1].items[2]);d.print(rows[2].items[2]);d.print(rows[2].items.size())
"#,
    )
    .runs(b"8\n7\n2\nfirst\n2\n9\n11\n5\n2\n");
}

#[test]
pub fn mixed_write_regions_allow_holder_siblings_and_final_shared_reads() {
    Case::new(
        r#"
d:@"debug"
<Item>:<{n<int32>:=}>
h:={->items<Item[2]>:=[{->n:=1},{->n:=2}];->other:=10}
outside:&(h.other)
h.items[1].n=3
d.print(*outside)
inside:&(h.items[1].n)
h.items[2].n=*inside+1
h.items[{h.other=20;d.print("index");->1}].n={h.other=21;d.print("rhs");->h.other}
d.print(h.items[1].n);d.print(h.items[2].n);d.print(h.other)
<Holder>:<{items<int32[2]>:=;tag<int32><string>:=}>
proof<Holder>:={->items:=[1];->tag:=7}
|proof.tag<int32>|{proof.items[1]=2;copy<int32>:proof.tag;d.print(copy)}
"#,
    )
    .runs(b"10\nindex\nrhs\n21\n4\n21\n7\n");
}

#[test]
pub fn mixed_write_indices_are_captured_once_before_rhs() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{items<int32[3]>:=}>
h:={->rows<Row[2]>:=[{->items:=[1,2]},{->items:=[3,4]}]}
i:=1
j:=2
h.rows[{d.print("row");->i}].items[{d.print("item");->j}]={d.print("rhs");i=2;j=1;->99}
d.print(h.rows[1].items[2]);d.print(h.rows[2].items[1]);d.print(i);d.print(j)
(((h).rows)[2].items)[1]=8
d.print(h.rows[2].items[1])
"#,
    )
    .runs(b"row\nitem\nrhs\n99\n3\n2\n1\n8\n");
}

#[test]
pub fn mixed_write_bounds_report_each_prefix_and_skip_remaining_effects() {
    for (cap, initial, ty, row, item, prefix, shown, length, stdout) in [
        (
            3,
            "[{->items:=[10]},{->items:=[20,30]}]",
            "int8",
            "-1",
            "1",
            1,
            "-1",
            2,
            "row\n",
        ),
        (
            3,
            "[{->items:=[10]},{->items:=[20,30]}]",
            "int32",
            "1",
            "2",
            2,
            "2",
            1,
            "row\nitem\n",
        ),
        (
            3,
            "[{->items:=[10]},{->items:=[20,30]}]",
            "uint64",
            "2",
            "18446744073709551615",
            2,
            "18446744073709551615",
            2,
            "row\nitem\n",
        ),
        (3, "[]", "int32", "1", "1", 1, "1", 0, "row\n"),
        (
            0,
            "[{->items:=[]}]",
            "int32",
            "1",
            "1",
            2,
            "1",
            0,
            "row\nitem\n",
        ),
    ] {
        let first = "h.rows[{d.print(\"row\");->row}]";
        let target = format!("{first}.items[{{d.print(\"item\");->item}}]");
        let source = format!(
            "#é🙂#\nd:@\"debug\";<Row>:<{{items<int32[{cap}]>:=}}>;set<null>:(row<{ty}>,item<{ty}>){{h:={{->rows<Row[3]>:={initial}}};{target}={{d.print(\"unreachable\");->99}}}};set({row},{item})"
        );
        let start = source.find(&target).unwrap();
        let end = start
            + if prefix == 1 {
                first.len()
            } else {
                target.len()
            };
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert_eq!(output.stdout, stdout.as_bytes());
            assert_eq!(output.stderr, format!("panic[P001]: index {shown} is outside initialized length {length} at bytes {start}..{end}\n").as_bytes());
        }
    }
}

#[test]
pub fn mixed_write_early_exits_end_reservations_at_the_last_actual_phase() {
    Case::new(
        r#"
d:@"debug"
<Item>:<{n<int32>:=}>
h:={->items<Item[2]>:=[{->n:=1}];->other:=2}
'out{h.items[{h.items=[];d.print("index-leave");'out.leave();->1}].n=99}
d.print(h.items.size())
h.items=[{->n:=3}]
'out{h.items[1].n={h={->items:=[];->other:=4};d.print("rhs-leave");'out.leave();->99}}
d.print(h.items.size());d.print(h.other)
<Row>:<{items<int32[2]>:=}>
rows<Row[2]>:=[{->items:=[1]}]
'out{rows[1].items[{rows=[];d.print("later-leave");'out.leave();->1}]=99}
d.print(rows.size())
pure:={->a:=1;->b:=2}
pure.a={pure={->a:=3;->b:=4};->5}
d.print(pure.a);d.print(pure.b)
"#,
    )
    .runs(b"index-leave\n0\nrhs-leave\n0\n4\nlater-leave\n0\n5\n4\n");
}

#[test]
pub fn mixed_write_paths_protect_the_first_collection_and_owning_fields() {
    for source in [
        "h:={->items:=[{->n:=1}];->other:=2};r:&(h.items[1].n);h.items[1].n=2;x:*r",
        "h:={->items:=[{->n:=1;->other:=2}]};r:&(h.items[1].other);h.items[1].n=3;x:*r",
        "h:={->items:=[{->n:=1}];->other:=2};r:&h;h.items[1].n=2;x:r.other",
        "h:={->items:=[{->n:=1}]};h.items[{h.items=[];->1}].n=2",
        "h:={->items:=[{->n:=1}]};h.items[1].n={h.items=[];->2}",
        "h:={->items:=[{->n:=1},{->n:=2}]};h.items[1].n={h.items[2].n=3;->4}",
        "d:@\"debug\";<R>:<{items<int32[2]>:=}>;rows<R[2]>:=[{->items:=[1]}];rows[{rows=[];->1}].items[{d.panic(\"stop\")}]=2",
        "d:@\"debug\";<R>:<{items<int32[2]>:=}>;rows<R[2]>:=[{->items:=[1]}];rows[1].items[{rows=[];->1}]=d.panic(\"stop\")",
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E302\""), "{source}: {error}");
    }
}

#[test]
pub fn mixed_write_paths_enforce_mutability_types_and_remaining_boundaries() {
    Case::new("h:{->items:=[1]};h.items[1]=2").runs(b"");
    for (source, code) in [
        ("h:={->items:[1]};h.items[1]=2", "E305"),
        ("rows:=[{->n:1}];rows[1].n=2", "E305"),
        ("rows:=[{->items:[1]}];rows[1].items[1]=2", "E305"),
        ("h:={->items:=[{->n:=1}]};h.items[0].n=2", "E101"),
        ("h:={->items:=[{->n:=1}]};h.items[1].n=\"wrong\"", "E207"),
        ("h:={->items:=[{->n:=1}]};p:&h;p.items[1].n=2", "B001"),
        ("h:={->items:=[{->n:=1}]};p:&h;(*p).items[1].n=2", "B001"),
        ("[{->n:=1}][1].n=2", "B001"),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("build", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains(&format!("\"code\":\"{code}\"")),
                "{source}: {error}"
            );
            assert!(!case.path.join("build").exists());
        }
    }
}
