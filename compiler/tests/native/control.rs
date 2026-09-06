use super::Case;

#[test]
pub fn emissions_keep_executing_and_dispatch_evaluates_once() {
    Case::new(
        r#"
debug:@"debug"
source<int32>:(){debug.print("source");->21;debug.print("after")}
double<int32>:(x<int32>){->x*2}
debug.print(source().(double))
"#,
    )
    .runs(b"source\nafter\n42\n");
}

#[test]
pub fn never_calls_evaluate_arguments_and_end_the_returning_continuation() {
    let source = r#"
d:@"debug"
mark<int32>:(value<int32>){d.print(value);->value}
stop<never>:(left<int32>,right<int32>){d.print(left+right);d.panic("stop")}
sum<int32>:(left<int32>,right<int32>){d.print("sum");->left+right}
d.print(sum(mark(1),stop(mark(2),mark(3))))
d.print("after")
"#;
    let call = r#"d.panic("stop")"#;
    let start = source.find(call).unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"1\n2\n3\n5\n");
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: stop at bytes {start}..{}\n",
                start + call.len()
            )
            .as_bytes()
        );
    }
}

#[test]
pub fn short_circuit_blocks_and_nonreturning_operands_preserve_effects() {
    Case::new(
        r#"
d:@"debug"
d.print(false&&{d.print("unexpected");->true})
d.print(true||{d.print("unexpected");->false})
f<boolean>:(flag<boolean>){->flag&&d.panic("stop")}
g<boolean>:(flag<boolean>){->flag||d.panic("stop")}
d.print(f(false))
d.print(g(true))
'outer {
    value:{'outer.leave()}&&true
    d.print("unexpected")
}
d.print("done")
"#,
    )
    .runs(b"false\ntrue\nfalse\ntrue\ndone\n");
    for expr in [
        "true&&d.panic(\"stop\")",
        "false||d.panic(\"stop\")",
        "d.panic(\"stop\")&&false",
    ] {
        let case = Case::new(&format!(
            "d:@\"debug\";value:{expr};d.print(\"unexpected\")"
        ));
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(result.status.code(), Some(1), "{expr}");
            assert!(result.stdout.is_empty(), "{expr}");
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(error.contains("P006"), "{expr}: {error}");
        }
    }
}
