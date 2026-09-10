use super::{Source, Span, emit_ir_with_sources, native_ir};

pub(crate) fn check(source: &str, fault: &str, message: &str, stdout: &[u8]) {
    let parsed = crate::parser::parse_documented_at(source, 100).unwrap();
    let program = crate::check::check(&parsed.block).unwrap();
    let sources = [Source {
        path: "math.mwy".into(),
        span: Span::new(100, 100 + source.len()),
    }];
    let start = source.find(fault).unwrap();
    for mapped in [false, true] {
        let ir = emit_ir_with_sources(&program, if mapped { &sources } else { &[] }).unwrap();
        let suffix = if mapped {
            format!(" at \"math.mwy\" bytes {start}..{}\n", start + fault.len())
        } else {
            format!(" at bytes {}..{}\n", 100 + start, 100 + start + fault.len())
        };
        for release in [false, true] {
            let output = native_ir(&ir, release, false);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, stdout);
            assert_eq!(output.stderr, format!("{message}{suffix}").as_bytes());
        }
    }
    assert!(
        emit_ir_with_sources(
            &program,
            &[Source {
                path: "bad.mwy".into(),
                span: Span::new(0, 1)
            }]
        )
        .is_err()
    );
}

#[test]
pub(crate) fn mapped_checks_preserve_arithmetic_causes_and_operand_order() {
    for (source, fault, message) in [
        (
            "f<int8>:(x<int8>){->x+1};f(127)",
            "x+1",
            "int8 + overflow (left 127, right 1; range -128..127)",
        ),
        (
            "f<int8>:(x<int8>){->-x};f(-128)",
            "-x",
            "int8 unary - overflow (value -128; range -128..127)",
        ),
        (
            "f<int32>:(x<int32>,y<int32>){->x/y};f(8,0)",
            "x/y",
            "int32 / zero divisor (left 8, right 0; range -2147483648..2147483647)",
        ),
        (
            "f<int8>:(x<int8>,y<int8>){->x/y};f(-128,-1)",
            "x/y",
            "int8 / overflow (left -128, right -1; range -128..127)",
        ),
    ] {
        check(source, fault, &format!("panic[P002]: {message}"), b"");
    }
    check(
        "d:@\"debug\";f<int8>:(x<int8>){->{d.print(\"left\");->x}+{d.print(\"right\");->1}};f(127);d.print(\"never\")",
        "{d.print(\"left\");->x}+{d.print(\"right\");->1}",
        "panic[P002]: int8 + overflow (left 127, right 1; range -128..127)",
        b"left\nright\n",
    );
}

#[test]
pub(crate) fn mapped_checks_preserve_nested_bounds_prefixes_and_capacity_effects() {
    check(
        "d:@\"debug\";f<null>:(i<int32>){xs:[[1]];v:xs[i][{d.print(\"never\");->1}]};f(2)",
        "xs[i]",
        "panic[P001]: index 2 is outside initialized length 1",
        b"",
    );
    check(
        "d:@\"debug\";f<null>:(i<int32>){xs:[[1]];v:xs[1][{d.print(\"inner\");->i}]};f(2)",
        "xs[1][{d.print(\"inner\");->i}]",
        "panic[P001]: index 2 is outside initialized length 1",
        b"inner\n",
    );
    check(
        "d:@\"debug\";f<int32[1]>:(xs<int32[1]>){->xs.add({d.print(\"item\");->2})};f([1])",
        "xs.add({d.print(\"item\");->2})",
        "panic[P003]: bounded list is full (length 1, capacity 1)",
        b"item\n",
    );
}
