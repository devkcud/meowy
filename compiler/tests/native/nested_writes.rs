use super::Case;

#[test]
pub fn nested_writes_preserve_outer_rows_and_copy_leaf_layouts() {
    Case::new(
        r#"
d:@"debug"
a<int32[3][2]>:=[[10],[20,30]]
copy:a
a[2][1]=a[1][1]+1
d.print(a[1][1]);d.print(a[2][1]);d.print(a[2][2]);d.print(copy[2][1])
d.print(a.size());d.print(a[1].size());d.print(a[2].size())
cube<uint8[2][2][2]>:=[[[1,2]],[[3]]]
cube[2][1][1]=255
d.print(cube[2][1][1]);d.print(cube[1][1][2])
<Point>:<{x<int32>;name<string>}>
rows<Point[2][2]>:=[[{->x:1;->name:"before"}],[{->x:2;->name:"kept"}]]
rows[1][1]={->x:3;->name:"after"}
d.print(rows[1][1].x);d.print(rows[1][1].name);d.print(rows[2][1].name)
<Item>:<int32><string>
items<Item[2][2]>:=[[1],["kept"]]
items[1][1]="new"
expected<Item[2][2]>:[["new"],["kept"]]
d.print(items==expected)
rows2<int32[3][2][2]>:=[[[1]],[[2]]]
rows2[2][1]=[4,5,6]
d.print(rows2[2][1][3]);d.print(rows2[1][1][1])
"#,
    )
    .runs(b"10\n11\n30\n20\n2\n1\n2\n255\n2\n3\nafter\nkept\ntrue\n6\n1\n");
}

#[test]
pub fn nested_write_paths_capture_indices_and_allow_final_shared_reads() {
    Case::new(
        r#"
d:@"debug"
head<&int32>:(items<&int32[3][2]>){->&items[1][2]}
a<int32[3][2]>:=[[1,2],[3]]
view:&a[1][1]
a[1][2]=*view+5
row:&a[1]
a[2][1]=row[2]+1
returned:head(&a)
a[2][1]=*returned+2
i:=1
j:=2
a[i][j]={i=2;j=1;->99}
d.print(a[1][2]);d.print(a[2][1])
a[{i=1;d.print("row");->i}][{i=2;d.print("column");->2}]={d.print("value");->a[2][1]+1}
d.print(a[1][2]);d.print(a[2][1]);d.print(i)
other:=[[4]]
a[{other[1][1]=5;->1}][{other[1][1]=6;->1}]={other[1][1]=7;->other[1][1]}
d.print(a[1][1]);d.print(other[1][1])
((a)[1])[1]=10
d.print(a[1][1])
a=[]
d.print(a.size())
"#,
    )
    .runs(b"99\n8\nrow\ncolumn\nvalue\n9\n8\n2\n7\n7\n10\n0\n");
}

#[test]
pub fn nested_writes_stop_at_each_nonreturning_operand() {
    Case::new(
        r#"
d:@"debug"
a<int32[2][2]>:=[[1],[2]]
'out{a[{d.print("outer-leave");'out.leave();->1}][{d.print("unreachable");->1}]=99}
'out{a[{d.print("outer");->1}][{d.print("inner-leave");'out.leave();->1}]={d.print("unreachable");->99}}
'out{a[1][1]={d.print("value-leave");'out.leave();->99}}
d.print(a[1][1]);d.print(a[2][1])
'out{a[{a=[[3],[4]];'out.leave();->1}][1]=99}
'out{a[1][{a=[[5],[6]];'out.leave();->1}]=99}
'out{a[1][1]={a=[[7],[8]];'out.leave();->99}}
d.print(a[1][1]);d.print(a[2][1])
empty<int32[0][1]>:=[[]]
'out{empty[1][{'out.leave()}]=99}
|false|{a[0][0]=99}
skipped:true||{a[0][0]=99;->true}
d.print(skipped)
i:=0
'retry{
    i=i+1
    a[1][{|i<2|'retry.restart();->1}]=10
}
d.print(a[1][1])
i=0
'retry{
    i=i+1
    a[1][1]={|i<2|'retry.restart();->11}
}
d.print(a[1][1])
"#,
    )
    .runs(b"outer-leave\nouter\ninner-leave\nvalue-leave\n1\n2\n7\n8\ntrue\n10\n11\n");
}

#[test]
pub fn nested_write_bounds_use_each_parent_length_and_prefix_span() {
    for (ty, initial, outer, inner, prefix, shown, length, stdout) in [
        ("int8", "[[10],[20,30]]", "-1", "1", 1, "-1", 2, "outer\n"),
        (
            "uint64",
            "[[10],[20,30]]",
            "18446744073709551615",
            "1",
            1,
            "18446744073709551615",
            2,
            "outer\n",
        ),
        (
            "int32",
            "[[10],[20,30]]",
            "1",
            "2",
            2,
            "2",
            1,
            "outer\ninner\n",
        ),
        (
            "int8",
            "[[10],[20,30]]",
            "2",
            "-1",
            2,
            "-1",
            2,
            "outer\ninner\n",
        ),
        (
            "uint64",
            "[[10],[20,30]]",
            "2",
            "18446744073709551615",
            2,
            "18446744073709551615",
            2,
            "outer\ninner\n",
        ),
        (
            "int32",
            "[[],[20,30]]",
            "1",
            "1",
            2,
            "1",
            0,
            "outer\ninner\n",
        ),
        ("int32", "[]", "1", "1", 1, "1", 0, "outer\n"),
    ] {
        let first = "items[{d.print(\"outer\");->row}]";
        let target = format!("{first}[{{d.print(\"inner\");->column}}]");
        let source = format!(
            "#é🙂#\nd:@\"debug\";set<null>:(row<{ty}>,column<{ty}>){{items<int32[3][3]>:={initial};{target}={{d.print(\"unreachable\");->99}}}};set({outer},{inner})"
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
pub fn nested_writes_protect_every_parent_phase_and_retained_alias() {
    for source in [
        "a:=[[1,2],[3,4]];r:&a[1][1];a[2][2]=5;x:*r",
        "a:=[[1,2],[3,4]];r:&a[1];a[2][2]=5;x:r[1]",
        "a:=[[1,2],[3,4]];r:&a;a[2][2]=5;x:r[1][1]",
        "a:=[[1,2],[3,4]];a[{a=[[5,6],[7,8]];->1}][1]=9",
        "a:=[[1,2],[3,4]];a[1][{a=[[5,6],[7,8]];->1}]=9",
        "a:=[[1,2],[3,4]];a[1][1]={a=[[5,6],[7,8]];->9}",
        "a:=[[1,2],[3,4]];a[1][{a[2]=[5,6];->1}]=9",
        "a:=[[1,2],[3,4]];a[1][1]={a[2][1]=5;->9}",
        "d:@\"debug\";a:=[[1]];a[{a=[[2]];->1}][{d.panic(\"stop\")}]=3",
        "d:@\"debug\";a:=[[1]];a[1][{a=[[2]];->1}]=d.panic(\"stop\")",
        "head<&int32>:(a<&int32[2][2]>,other<&int32[2][2]>){->&a[1][1]};a:=[[1,2],[3,4]];b:=[[1,2],[3,4]];r:head(&a,&b);b[2][2]=5;x:*r",
        "a:=[[1,2],[3,4]];r:&a[1][1];i:=1;'loop{x:*r;a[2][2]=5;|i>1|'loop.leave();i=i+1;'loop.restart()}",
        "a:=[[1]];r:'out{a[1][1]={'out->&a[1][1];->2}};x:*r",
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E302\""), "{source}: {error}");
    }
}

#[test]
pub fn nested_writes_check_depth_types_and_unsupported_owner_boundaries() {
    for (source, code) in [
        ("a:[[1]];a[1][1]=2", "E305"),
        ("a:=[[1]];a[0][1]=2", "E101"),
        ("a:=[[1]];a[1][0]=2", "E101"),
        ("a:=[[1]];a[1][2]=2", "E101"),
        ("a:=[[1]];a[1][true]=2", "E222"),
        ("a:=[[1]];a[1][1]=\"wrong\"", "E207"),
        ("a<uint8[1][1]>:=[[1]];a[1][1]=256", "E216"),
        ("a:=[[1]];r:&a;r[1][1]=2", "B001"),
        ("a:=[[1]];r:&a;(*r)[1][1]=2", "B001"),
        ("a:={->items:[[1]]};a.items[1][1]=2", "B001"),
        ("a:=[{->items:[1]}];a[1].items[1]=2", "B001"),
        ("a:=[[{->x:1}]];a[1][1].x=2", "B001"),
        ("[[1]][1][1]=2", "B001"),
        ("a:=[[1]];r:&!a[1][1]", "B001"),
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

#[test]
pub fn nested_write_panics_keep_only_completed_operand_effects() {
    for (outer, inner, value, stdout) in [
        (
            "{d.print(\"outer\");d.panic(\"stop\")}",
            "{d.print(\"unreachable\");->1}",
            "1",
            "outer\n",
        ),
        (
            "{d.print(\"outer\");->1}",
            "{d.print(\"inner\");d.panic(\"stop\")}",
            "{d.print(\"unreachable\");->1}",
            "outer\ninner\n",
        ),
        (
            "{d.print(\"outer\");->1}",
            "{d.print(\"inner\");->1}",
            "{d.print(\"value\");d.panic(\"stop\")}",
            "outer\ninner\nvalue\n",
        ),
    ] {
        let source = format!("d:@\"debug\";a:=[[1]];a[{outer}][{inner}]={value}");
        let start = source.find("d.panic(\"stop\")").unwrap();
        let end = start + "d.panic(\"stop\")".len();
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert_eq!(output.stdout, stdout.as_bytes());
            assert_eq!(
                output.stderr,
                format!("panic[P006]: stop at bytes {start}..{end}\n").as_bytes()
            );
        }
    }
}
