use super::Case;

#[test]
pub fn dynamic_suffix_locals_preserve_primitive_types_and_runtime_values() {
    Case::new(
        r#"
d:@"debug"
number<int16>:(){d.print("number");->300}
text<string>:(){d.print("text");->"hello"}
float<float32>:(){d.print("float");->1.5}
flag<boolean>:(){d.print("flag");->false}
none<null>:(){d.print("none")}
values<uint8[1]><int16[1]>:[{x:number();d.print("value");->x+1}]
|values<int16[1]>|d.print(values[1])
bytes<uint8[1]><uint16[1]>:[{x<uint8>:=255;x=1;d.print("mutable");->x+1}]
|bytes<uint8[1]>|d.print(bytes[1])
words<int32[1]><string[1]>:[{x:text();->x}]
|words<string[1]>|d.print(words[1])
floats<float32[1]><float64[1]>:[{x:float();->x+0.5}]
|floats<float32[1]>|d.print(floats[1]==2.0)
flags<boolean[1]><int32[1]>:[{x:flag();->!x}]
|flags<boolean[1]>|d.print(flags[1])
nulls<null[1]><int32[1]>:[{x:none();->x}]
|nulls<null[1]>|d.print(nulls[1])
"#,
    )
    .runs(b"number\nvalue\n301\nmutable\n2\ntext\nhello\nfloat\ntrue\nflag\ntrue\nnone\nnull\n");
}

#[test]
pub fn dynamic_suffix_locals_support_parameters_aggregates_and_shadowing() {
    Case::new(
        r#"
d:@"debug"
read<uint16>:(){d.print("read");->300}
<Small>:<{n<uint8>}>
<Wide>:<{n<uint16>}>
x<uint8>:7
rows<Small[1]><Wide[1]>:[{x:read();->{->n:x}}]
|rows<Wide[1]>|d.print(rows[1].n)
nested<uint8[2][1]><uint16[2][1]>:[{x:read();->[x,x+1]}]
|nested<uint16[2][1]>|d.print(nested[1][2])
parameter<null>:(x<uint8>){
    values<uint8[1]><uint16[1]>:[{d.print("parameter");->x+1}]
    |values<uint8[1]>|d.print(values[1])
}
parameter(5)
d.print(x)
"#,
    )
    .runs(b"read\n300\nread\n301\nparameter\n6\n7\n");
}

#[test]
pub fn dynamic_suffix_reads_remain_in_source_order_and_keep_loans() {
    Case::new(
        r#"
d:@"debug"
x<uint8>:=1
values<uint8[2]><uint16[2]>:[x+1,{x=9;->x}]
|values<uint8[2]>|{d.print(values[1]);d.print(values[2])}
d.print(x)
read<uint8>:(){d.print("read");->4}
owner:=10
view:&owner
selected<uint8[1]><uint16[1]>:[{x:read();d.print(*view);owner=20;->x+1}]
|selected<uint8[1]>|d.print(selected[1])
d.print(owner)
"#,
    )
    .runs(b"2\n9\n9\nread\n10\n5\n20\n");
    let source = "read<uint8>:(){->1};owner:=10;view:&owner;values<uint8[1]><uint16[1]>:[{x:read();owner=20;->x}];copy:*view";
    let output = Case::new(source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E302\""));
}

#[test]
pub fn dynamic_suffix_boolean_probes_preserve_live_short_circuit_facts() {
    Case::new(
        r#"
d:@"debug"
check<null>:(flag<boolean>){
    |!flag|{
        values<boolean[1]><string[1]>:[{d.print("and");->((flag))&&(1/0==0)}]
        |values<boolean[1]>|d.print(values[1])
        duplicates<boolean[1]><int32[1]>:[{d.print("duplicate");->(flag)&&{->true;->false}}]
        |duplicates<boolean[1]>|d.print(duplicates[1])
    }
    |flag|{
        values<boolean[1]><string[1]>:[{d.print("or");->((flag))||(1/0==0)}]
        |values<boolean[1]>|d.print(values[1])
    }
}
check(false)
check(true)
"#,
    )
    .runs(b"and\nfalse\nduplicate\nfalse\nor\ntrue\n");
}

#[test]
pub fn dynamic_suffix_arithmetic_keeps_runtime_overflow_checks() {
    for (source, expression, stdout, message) in [
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{x<uint8>:=1;x=255;d.print(\"value\");->x+1}]",
            "x+1",
            "value\n",
            "uint8 + overflow (left 255, right 1; range 0..255)",
        ),
        (
            "d:@\"debug\";read<int8>:(){d.print(\"read\");->-128};values<int8[1]><int16[1]>:[{x:read();->-x}]",
            "-x",
            "read\n",
            "int8 unary - overflow (value -128; range -128..127)",
        ),
    ] {
        let start = source.rfind(expression).unwrap();
        let end = start + expression.len();
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert_eq!(output.stdout, stdout.as_bytes());
            assert_eq!(
                output.stderr,
                format!("panic[P002]: {message} at bytes {start}..{end}\n").as_bytes()
            );
        }
    }
}

#[test]
pub fn dynamic_suffix_unknown_values_do_not_enable_unsupported_inference() {
    for (source, code) in [
        (
            "x<uint8>:=1;values<uint8[1]><uint16[1]>:[{x=2;->x+300}]",
            "E207",
        ),
        (
            "flag:=false;values<boolean[1]><string[1]>:[{flag=true;->flag&&(1/0==0)}]",
            "E107",
        ),
        (
            "read<uint8>:(){->1};values<uint8[1]><uint8[2]>:[{x:read();->x}]",
            "E207",
        ),
        (
            "x:=1;f<null>:(){values<int32[1]><string[1]>:[{0;->x}]}",
            "B001",
        ),
        (
            "values<int32[1]><string[1]>:[{x<int32><string>:=1;->x}]",
            "B001",
        ),
        (
            "owner:1;values<int32[1]><string[1]>:[{x:&owner;->*x}]",
            "B001",
        ),
        (
            "<A>:<{x<uint8>;y<uint8>}>;<B>:<{x<uint16>;y<uint16>}>;read<uint8>:(){->1};values<A[1]><B[1]>:[{x:read();->x:1;->y:x}]",
            "B001",
        ),
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}
