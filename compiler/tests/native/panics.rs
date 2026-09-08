use super::Case;

#[test]
pub fn dynamic_arithmetic_panics_in_both_profiles() {
    for (ty, body, args) in [
        ("int8", "x+1", "127"),
        ("uint8", "x+1", "255"),
        ("int32", "10/x", "0"),
        ("int32", "-x", "-2147483648"),
        ("int32", "x/-1", "-2147483648"),
    ] {
        let case = Case::new(&format!(
            "d:@\"debug\";f<{ty}>:(x<{ty}>){{->{body}}};d.print(f({args}))"
        ));
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(
                result.status.code(),
                Some(1),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("P002"),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}

#[test]
pub fn arithmetic_panics_report_typed_original_operands_and_source_bytes() {
    for (ty, operation, args, evidence) in [
        (
            "int8",
            "left+right",
            "127,1",
            "int8 + overflow (left 127, right 1; range -128..127)",
        ),
        (
            "uint8",
            "left-right",
            "0,1",
            "uint8 - overflow (left 0, right 1; range 0..255)",
        ),
        (
            "int16",
            "left*right",
            "-32768,-1",
            "int16 * overflow (left -32768, right -1; range -32768..32767)",
        ),
        (
            "uint64",
            "left+right",
            "18446744073709551615,1",
            "uint64 + overflow (left 18446744073709551615, right 1; range 0..18446744073709551615)",
        ),
        (
            "int64",
            "left/right",
            "-9223372036854775808,-1",
            "int64 / overflow (left -9223372036854775808, right -1; range -9223372036854775808..9223372036854775807)",
        ),
        (
            "int8",
            "left/right",
            "-7,0",
            "int8 / zero divisor (left -7, right 0; range -128..127)",
        ),
        (
            "uint64",
            "left%right",
            "18446744073709551615,0",
            "uint64 % zero divisor (left 18446744073709551615, right 0; range 0..18446744073709551615)",
        ),
    ] {
        let source = format!(
            "#é🙂#\r\nd:@\"debug\";f<{ty}>:(left<{ty}>,right<{ty}>){{->{operation}}};d.print(f({args}))"
        );
        let start = source.find(operation).unwrap();
        let end = start + operation.len();
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{ty}: {profile}");
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                format!("panic[P002]: {evidence} at bytes {start}..{end}\n").as_bytes()
            );
        }
    }
    let source = "#é#\nd:@\"debug\";f<int8>:(value<int8>){->-value};d.print(f(-128))";
    let start = source.find("-value").unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, format!("panic[P002]: int8 unary - overflow (value -128; range -128..127) at bytes {start}..{}\n", start + "-value".len()).as_bytes());
    }
}

#[test]
pub fn arithmetic_evidence_does_not_repeat_operand_effects() {
    let source = r#"
d:@"debug"
left<int8>:(){d.print("left");->127}
right<int8>:(){d.print("right");->1}
d.print(left()+right())
"#;
    let operation = "left()+right()";
    let start = source.find(operation).unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"left\nright\n");
        assert_eq!(output.stderr, format!("panic[P002]: int8 + overflow (left 127, right 1; range -128..127) at bytes {start}..{}\n", start + operation.len()).as_bytes());
    }
}

#[test]
pub fn explicit_panic_appends_call_site_after_streamed_effects() {
    let source = r#"#é🙂#
d:@"debug"
mark<string>:(value<string>){d.print(value);->value}
die:d.panic
die("first={mark("A")}, second={mark("B")}")
"#;
    let call = r#"die("first={mark("A")}, second={mark("B")}")"#;
    let start = source.find(call).unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"A\nB\n");
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: first=A, second=B at bytes {start}..{}\n",
                start + call.len()
            )
            .as_bytes()
        );
    }
}

#[test]
pub fn interrupted_panic_messages_do_not_append_the_outer_site() {
    let called = r#"d:@"debug";stop<never>:(){d.panic("inner")};d.panic("outer {stop()} tail")"#;
    let call = r#"d.panic("inner")"#;
    let start = called.find(call).unwrap();
    let case = Case::new(called);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: outer panic[P006]: inner at bytes {start}..{}\n",
                start + call.len()
            )
            .as_bytes()
        );
    }
    let nested = r#"d:@"debug";d.panic("outer {d.panic("inner")} tail")"#;
    let inner = r#"d.panic("inner")"#;
    let start = nested.find(inner).unwrap();
    let case = Case::new(nested);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: outer panic[P006]: inner at bytes {start}..{}\n",
                start + inner.len()
            )
            .as_bytes()
        );
    }
    let arithmetic =
        r#"d:@"debug";f<never>:(value<int8>){d.panic("before {value+1} after")};f(127)"#;
    let start = arithmetic.find("value+1").unwrap();
    let case = Case::new(arithmetic);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, format!("panic[P006]: before panic[P002]: int8 + overflow (left 127, right 1; range -128..127) at bytes {start}..{}\n", start + "value+1".len()).as_bytes());
    }
    let source =
        r#"d:@"debug";'out {d.panic("before {{'out.leave();->0}} after")};d.print("continued")"#;
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert!(output.status.success());
        assert_eq!(output.stdout, b"continued\n");
        assert_eq!(output.stderr, b"panic[P006]: before ");
    }
}

#[test]
pub fn recursive_failure_skips_pending_arguments_and_caller_effects() {
    let source = r#"
d:@"debug"
f<int32>:(x<int32>)'out{|x==0|{'out ->10/x;'out.leave()};->f(x-1)}
mark<int32>:(){d.print("late argument");->1}
add<int32>:(a<int32>,b<int32>){d.print("entered");->a+b}
d.print("start")
value:=0
value=add(f(3),mark())
d.print("late caller")
"#;
    let start = source.find("10/x").unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stdout,
            b"start\n",
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stderr, format!("panic[P002]: int32 / zero divisor (left 10, right 0; range -2147483648..2147483647) at bytes {start}..{}\n", start + 4).as_bytes());
    }
}
