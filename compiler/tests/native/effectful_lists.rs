use super::Case;

#[test]
pub fn effectful_list_contexts_select_scalar_and_aggregate_shapes() {
    Case::new(
        r#"
d:@"debug"
count:=0
wide<uint8[1]><uint16[1]>:[{d.print("wide");count=count+1;->300}]
|wide<uint16[1]>|d.print(wide[1])
compound<int8[1]><int16[1]>:[{d.print("compound");count=count+1;->(127+1)-1}]
|compound<int16[1]>|d.print(compound[1])
<Small>:<{value<uint8>;name<string>}>
<Wide>:<{value<uint16>;name<string>}>
rows<Small[1]><Wide[1]>:[{d.print("row");count=count+1;->value:300;->name:"chosen"}]
|rows<Wide[1]>|{d.print(rows[1].value);d.print(rows[1].name)}
nested<uint8[2][1]><uint16[2][1]>:[{d.print("nested");count=count+1;->[1,300]}]
|nested<uint16[2][1]>|d.print(nested[1][2])
logic<boolean[1]><string[1]>:[{d.print("logic");->false&&(1/0==0)}]
|logic<boolean[1]>|d.print(logic[1])
d.print(count)
<R>:<{n<int32>}>
<S>:<{-><R><int32>;n<int32>}>
<U>:<R><S>
composed<R[1]><U[1]>:[{d.print("forward");->{->n:1};->n:2}]
|composed<U[1]>|{item:composed[1];|item<S>|d.print(item.n)}
"#,
    )
    .runs(
        b"wide\n300\ncompound\n127\nrow\n300\nchosen\nnested\n300\nlogic\nfalse\n4\nforward\n2\n",
    );
}

#[test]
pub fn effectful_list_prefix_locals_keep_default_types_and_shadowing() {
    Case::new(
        r#"
d:@"debug"
x<uint8>:7
numbers<uint8[1]><int32[1]>:[{x:1;d.print("default");->x}]
|numbers<int32[1]>|d.print(numbers[1])
names<uint8[1]><string[1]>:[{x<string>:"inner";d.print(x);->x}]
|names<string[1]>|d.print(names[1])
bytes<uint8[1]><uint16[1]>:[{byte<uint8>:4;d.print("typed");->byte+1}]
|bytes<uint8[1]>|d.print(bytes[1])
d.print(x)
"#,
    )
    .runs(b"default\n1\ninner\ninner\ntyped\n5\n7\n");
}

#[test]
pub fn effectful_list_contexts_lower_prefixes_once_in_source_order() {
    Case::new(
        r#"
d:@"debug"
count<int16>:=0
values<int8[2]><int16[2]>:[{d.print("first");count=count+1;->127+1},{d.print("second");count=count+1;->count}]
|values<int16[2]>|{d.print(values[1]);d.print(values[2])}
d.print(count)
owner:=10
view:&owner
read<uint8[1]><uint16[1]>:[{d.print(*view);owner=20;->300}]
|read<uint16[1]>|d.print(read[1])
d.print(owner)
i:=0
'loop{
    values<uint8[1]><uint16[1]>:[{i=i+1;{|i<2|'loop.restart()};d.print("iteration");->300}]
    |values<uint16[1]>|d.print(values[1])
}
d.print(i)
"#,
    )
    .runs(b"first\nsecond\n128\n2\n2\n10\n300\n20\niteration\n300\n2\n");
}

#[test]
pub fn effectful_list_context_prefix_exits_preserve_flow_and_prior_effects() {
    Case::new(
        r#"
d:@"debug"
'out{
    values<uint8[2]><uint16[2]>:[{d.print("leave");'out.leave();->300},{d.print("unreachable");->1}]
    d.print("unreachable")
}
d.print("after")
"#,
    )
    .runs(b"leave\nafter\n");
    let source =
        "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(\"prefix\");d.panic(\"stop\");->300}]";
    let start = source.find("d.panic(\"stop\")").unwrap();
    let end = start + "d.panic(\"stop\")".len();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"prefix\n");
        assert_eq!(
            output.stderr,
            format!("panic[P006]: stop at bytes {start}..{end}\n").as_bytes()
        );
    }
}

#[test]
pub fn effectful_list_contexts_preserve_errors_and_explicit_boundaries() {
    for (source, code) in [
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1;->2}]",
            "E205",
        ),
        (
            "<A>:<{x<uint8>}>;<B>:<{x<uint16>}>;values<A[1]><B[1]>:[{x:1;->x:300}]",
            "E203",
        ),
        (
            "d:@\"debug\";values<uint8[2]><uint16[2]>:[{d.print(1);->1},{byte<uint8>:2;->byte}]",
            "B001",
        ),
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1}]",
            "E207",
        ),
        (
            "d:@\"debug\";values<int8[1]><int16[1]>:[{d.panic(\"stop\");->127+1}]",
            "E207",
        ),
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{missing();->300}]",
            "E201",
        ),
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->70000}]",
            "E207",
        ),
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.panic(\"stop\");bad<int32[1/0]>:[];->300}]",
            "E107",
        ),
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->300;d.print(2)}]",
            "B001",
        ),
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:['row{d.print(1);->300}]",
            "B001",
        ),
        (
            "d:@\"debug\";x<uint8>:1;<A>:<{x<uint16>;y<uint8>}>;<B>:<{x<uint16>;y<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->x:300;->y:x}]",
            "B001",
        ),
        (
            "d:@\"debug\";owner:=1;view:&owner;values<uint8[1]><uint16[1]>:[{owner=2;->300}];d.print(*view)",
            "E302",
        ),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("build", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(
                output.stdout.is_empty(),
                "checking executed source effects: {source}"
            );
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains(&format!("\"code\":\"{code}\"")),
                "{source}: {error}"
            );
            assert!(!case.path.join("build").exists());
        }
    }
}
