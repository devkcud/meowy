use super::Case;

#[test]
pub fn element_writes_preserve_other_elements_and_copy_value_types() {
    Case::new(
        r#"
d:@"debug"
a<int8[4]>:=[10,20,30]
b:a
a[2]=a[1]+1
d.print(a[1]);d.print(a[2]);d.print(a[3]);d.print(a.size());d.print(b[2])
rows<{x<int32>;name<string>}[2]>:=[{->x:1;->name:"old"},{->x:2;->name:"kept"}]
rows[1]={->x:3;->name:"new"}
d.print(rows[1].x);d.print(rows[1].name);d.print(rows[2].name)
<Item>:<int32><string>
items<Item[2]>:=[1,"before"]
items[1]="after"
same<Item[2]>:["after","before"]
d.print(items==same)
matrix<int32[3][2]>:=[[1],[2,3]]
matrix[1]=[4,5,6]
d.print(matrix[1].size());d.print(matrix[1][3]);d.print(matrix[2][2])
"#,
    )
    .runs(b"10\n11\n30\n3\n20\n3\nnew\nkept\ntrue\n3\n6\n3\n");
}

#[test]
pub fn element_writes_allow_final_borrow_uses_and_independent_owners() {
    Case::new(
        r#"
d:@"debug"
head<&int32>:(items<&int32[3]>){->&items[1]}
a<int32[3]>:=[10,20]
view:&a[1]
a[2]=*view+1
whole:&a
a[1]=whole[2]+1
returned:head(&a)
a[2]=*returned+1
position:=1
index:&position
a[*index]={position=2;->a[2]+1}
other:=[1]
a[{other[1]=7;->1}]={other[1]=9;->a[2]+1}
d.print(a[1]);d.print(a[2]);d.print(position);d.print(other[1])
a=[]
d.print(a.size())
"#,
    )
    .runs(b"14\n13\n2\n9\n0\n");
}

#[test]
pub fn element_write_operands_and_early_exits_run_once_in_order() {
    Case::new(
        r#"
d:@"debug"
a:=[10,20]
a[{d.print("index");->2}]={d.print("value");->a[1]+1}
d.print(a[2])
'out{a[{d.print("index-leave");'out.leave();->1}]={d.print("unreachable");->99}}
'out{a[1]={d.print("value-leave");'out.leave();->99}}
'out{a[{a=[30,40];'out.leave();->1}]=99}
'out{a[1]={a=[50,60];'out.leave();->99}}
d.print(a[1]);d.print(a[2])
empty<int32[0]>:=[]
'out{empty[{'out.leave()}]=99}
|false|{a[0]=99}
skipped:true||{a[0]=99;->true}
d.print(skipped)
i:=1
'loop{
    |i>2|'loop.leave()
    a[i]=i
    i=i+1
    'loop.restart()
}
d.print(a[1]);d.print(a[2])
i=0
'retry{
    i=i+1
    a[1]={|i<2|'retry.restart();->7}
}
d.print(a[1])
"#,
    )
    .runs(b"index\nvalue\n11\nindex-leave\nvalue-leave\n50\n60\ntrue\n1\n2\n7\n");
}

#[test]
pub fn element_writes_reject_live_aliases_and_intervening_owner_writes() {
    for source in [
        "a:=[1,2];r:&a[1];a[2]=3;x:*r",
        "a:=[1,2];r:&a;a[1]=3;x:r[2]",
        "a:=[1,2];r:{->view:&a[1]};a[2]=3;x:*r.view",
        "head<&int32>:(a<&int32[2]>){->&a[1]};a:=[1,2];r:head(&a);a[2]=3;x:*r",
        "a:=[1,2];r:&a[1];a[2]=*r+1;x:*r",
        "a<int32[2]>:=[1];a[{a=[];->1}]=3",
        "a<int32[2]>:=[1];a[1]={a=[];->3}",
        "a:=[1,2];a[{a[2]=3;->1}]=4",
        "a:=[1,2];a[1]={a[2]=3;->4}",
        "a:=[1,2];r:&a[1];i:=1;'loop{x:*r;a[2]=3;|i>1|'loop.leave();i=i+1;'loop.restart()}",
        "a:=[1,2];|true|{r:&a[1];a[2]=3;x:*r}",
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E302\""), "{source}: {error}");
    }
}

#[test]
pub fn element_writes_check_mutability_types_and_remaining_place_boundaries() {
    for (source, code) in [
        ("a:[1];a[1]=2", "E305"),
        ("f<null>:(a<int32[1]>){a[1]=2}", "E305"),
        ("a:=[1];a[0]=2", "E101"),
        ("a:=[1];a[-1]=2", "E101"),
        ("a:=[1];a[2]=2", "E101"),
        ("a:=[1];a[true]=2", "E222"),
        ("a:=[1];a[1]=\"wrong\"", "E207"),
        ("a:=[1];r:&a;r[1]=2", "B001"),
        ("a:=[1];r:&a;(*r)[1]=2", "B001"),
        ("a:={->items:[1]};a.items[1]=2", "E305"),
        ("[1][1]=2", "B001"),
        ("a:=[1];r:&!a[1]", "B001"),
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
pub fn element_write_dynamic_bounds_run_before_rhs_and_preserve_diagnostics() {
    for (capacity, initial, ty, index, shown, length) in [
        (3, "[10]", "int8", "-1", "-1", 1),
        (3, "[10]", "usize", "2", "2", 1),
        (
            3,
            "[10]",
            "uint64",
            "18446744073709551615",
            "18446744073709551615",
            1,
        ),
        (0, "[]", "int64", "1", "1", 0),
    ] {
        let access = "items[{d.print(\"index\");->position}]";
        let source = format!(
            "#é🙂#\nd:@\"debug\";set<null>:(position<{ty}>){{items<int32[{capacity}]>:={initial};{access}={{d.print(\"unreachable\");->99}}}};set({index})"
        );
        let start = source.find(access).unwrap();
        let end = start + access.len();
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert_eq!(output.stdout, b"index\n");
            assert_eq!(output.stderr, format!("panic[P001]: index {shown} is outside initialized length {length} at bytes {start}..{end}\n").as_bytes());
        }
    }
}

#[test]
pub fn element_write_operand_panics_preserve_prior_effects() {
    for (index, value, expected) in [
        (
            "{d.print(\"index\");d.panic(\"stop\")}",
            "{d.print(\"unreachable\");->1}",
            b"index\n".as_slice(),
        ),
        (
            "{d.print(\"index\");->1}",
            "{d.print(\"value\");d.panic(\"stop\")}",
            b"index\nvalue\n".as_slice(),
        ),
    ] {
        let source = format!("d:@\"debug\";a:=[1];a[{index}]={value}");
        let site = source.find("d.panic(\"stop\")").unwrap();
        let end = site + "d.panic(\"stop\")".len();
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, expected);
            assert_eq!(
                output.stderr,
                format!("panic[P006]: stop at bytes {site}..{end}\n").as_bytes()
            );
        }
    }
}
