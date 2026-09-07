use super::Case;

#[test]
pub fn reference_reassignment_preserves_copies_and_releases_previous_pointees() {
    Case::new(
        r#"
d:@"debug"
a:=1
b:2
p:=&a
old:p
p=&b
d.print(*old);d.print(*p)
a=3
d.print(a)
p=p
d.print(p==&b)
copy:*(&p)
p=&a
d.print(*copy);d.print(*p)
"#,
    )
    .runs(b"1\n2\n3\ntrue\n2\n3\n");
}

#[test]
pub fn reference_reassignment_keeps_cell_addresses_and_last_rhs_reads() {
    Case::new(
        r#"
d:@"debug"
a:1
b:2
p:=&a
cell:&p
p={d.print(**cell);->&b}
d.print(*p)
next:&p
d.print(**next)
old:*next
p=&a
d.print(*old);d.print(*p)
"#,
    )
    .runs(b"1\n2\n2\n2\n1\n");
    for source in [
        "a:1;b:2;p:=&a;cell:&p;p=&b;v:**cell",
        "a:=1;b:2;p:=&a;old:p;p=&b;a=3;v:*old",
        "a:1;b:2;p:=&a;cell:&p;copy:cell;p=&b;v:**copy",
        "<R>:<&int32>;load<R>:(cell<&R>){->*cell};a:1;b:2;p:=&a;old:load(&p);p=&b;v:*old",
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E302\""),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
pub fn reference_reassignment_preserves_call_bounds_and_temporary_expiry() {
    Case::new(
        r#"
d:@"debug"
first<&int32>:(p<&int32>,q<&string>){->p}
a:7
p:=&1
p=&a
d.print(*p)
p=first(&a,&"temporary")
p=&a
d.print(*p)
"#,
    )
    .runs(b"7\n7\n");
    for source in [
        "a:1;p:=&a;p=&2;v:*p",
        "a:1;p:=&a;p={b:2;->&b};v:*p",
        "first<&int32>:(p<&int32>,q<&string>){->p};a:1;p:=&a;p=first(&a,&\"temporary\");v:*p",
        "a:1;p:=&a;old:p;p=&2;v:*p",
    ] {
        let output = Case::new(source).command("build", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E303\""),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
pub fn reference_assignment_rhs_runs_once_before_storing() {
    Case::new(
        r#"
d:@"debug"
identity<&int32>:(p<&int32>){d.print("call");->p}
read<int32>:(){'out{'out.leave()};->5}
a:1
b:2
p:=&a
p=identity(&b)
d.print(*p)
p={d.print("block");d.print(*p);->&a}
d.print(*p)
d.print(read())
"#,
    )
    .runs(b"call\n2\nblock\n2\n1\n5\n");
    let source =
        "d:@\"debug\";a:1;p:=&a;p={d.print(\"rhs\");d.panic(\"stop\");->&a};d.print(\"after\")";
    let call = "d.panic(\"stop\")";
    let start = source.find(call).unwrap();
    let end = start + call.len();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"rhs\n");
        assert_eq!(
            output.stderr,
            format!("panic[P006]: stop at bytes {start}..{end}\n").as_bytes()
        );
    }
}

#[test]
pub fn fixed_reference_bindings_preserve_nested_and_aggregate_pointees() {
    Case::new(
        r#"
d:@"debug"
a:1
b:2
first:{->view:&a;->count:3}
second:{->view:&b;->count:4}
p:=&first
old:p
p=&second
d.print(*old.view);d.print(*p.view)
d.print(p.count)
x:&a
y:&b
cell:=&x
copy:*cell
cell=&y
d.print(*copy);d.print(**cell)
left<int32[2]>:[1]
right<int32[2]>:[2,3]
items:=&left
items=&right
d.print((*items)[2])
one<int32><null>:1
none<int32><null>:null
union:=&one
union=&none
|(*union)<null>|d.print("empty")
"#,
    )
    .runs(b"1\n2\n4\n1\n2\n3\nempty\n");
}

#[test]
pub fn reference_binding_types_and_unsupported_mutation_forms_stay_explicit() {
    for (source, code) in [
        ("a:1;b:2;p:&a;p=&b", "E305"),
        ("a:1;b<uint8>:2;p:=&a;p=&b", "E207"),
        ("a:1;p<&int32><null>:=&a", "B001"),
        ("a:1;holder:={->view:&a}", "B001"),
        ("a:1;holder:{->view:=&a}", "B001"),
        ("a:1;values:[&a]", "B001"),
        ("a:=1;p:&!a", "B001"),
        ("a:1;b:2;p:=&a;cell:&p;*cell=&b", "B001"),
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(&format!("\"code\":\"{code}\"")),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
pub fn reference_operands_keep_the_value_loaded_before_later_argument_effects() {
    Case::new(
        r#"
d:@"debug"
same<boolean>:(a<&int32>,b<&int32>){->a==b}
a:1
b:2
p:=&a
d.print(same(p,{p=&b;->p}))
d.print(*p)
p={p=&a;d.print(*p);->&b}
d.print(*p)
"#,
    )
    .runs(b"false\n2\n1\n2\n");
    let source =
        "same<boolean>:(a<&int32>,b<&int32>){->a==b};a:=1;b:2;p:=&a;v:same(p,{p=&b;a=3;->p})";
    let output = Case::new(source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E302\""));
}

#[test]
pub fn reference_assignments_preserve_restart_header_boundaries() {
    for source in [
        "a:1;p<&int32><null>:=&a;i:=0;'loop{p=&a;i=i+1;|i<2|'loop.restart()}",
        "a:1;p:=&a;p=&2;'out{'out.restart()};v:*p",
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("\"code\":\"B001\""),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
