use super::*;

#[test]
pub(crate) fn integer_boundaries_fail_in_each_profile() {
    for release in [false, true] {
        for bits in [8, 16, 32, 64] {
            for signed in [false, true] {
                let ty = Type::Int { bits, signed };
                let maximum = (1i128 << (bits - u32::from(signed))) - 1;
                let minimum = if signed { -(1i128 << (bits - 1)) } else { 0 };
                let name = if signed { "int" } else { "uint" };
                for (op, left, right) in [("+", maximum, 1), ("-", minimum, 1), ("*", maximum, 2)] {
                    let mut value = binary(
                        op,
                        integer(left, bits, signed),
                        integer(right, bits, signed),
                        ty.clone(),
                    );
                    value.span = Span { start: 11, end: 17 };
                    let output = native(vec![value], release, false);
                    assert_eq!(
                        output.status.code(),
                        Some(1),
                        "bits={bits}, signed={signed}, release={release}, op={op}"
                    );
                    assert_eq!(output.stderr, format!("panic[P002]: {name}{bits} {op} overflow (left {left}, right {right}; range {minimum}..{maximum}) at bytes 11..17\n").as_bytes());
                }
            }
        }
        let ty = Type::Int {
            bits: 64,
            signed: true,
        };
        for op in ["/", "%"] {
            let value = binary(
                op,
                integer(i64::MIN.into(), 64, true),
                integer(-1, 64, true),
                ty.clone(),
            );
            let output = native(vec![value], release, false);
            if op == "/" {
                assert_eq!(output.status.code(), Some(1));
                assert_eq!(output.stderr, b"panic[P002]: int64 / overflow (left -9223372036854775808, right -1; range -9223372036854775808..9223372036854775807) at bytes 0..0\n");
            } else {
                assert!(output.status.success());
                assert_eq!(output.stdout, b"0\n");
            }
            let value = binary(op, integer(8, 64, true), integer(0, 64, true), ty.clone());
            let output = native(vec![value], release, false);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stderr, format!("panic[P002]: int64 {op} zero divisor (left 8, right 0; range -9223372036854775808..9223372036854775807) at bytes 0..0\n").as_bytes());
        }
        let minimum = expr(
            ExprKind::Unary {
                op: "-".into(),
                value: Box::new(integer(i64::MIN.into(), 64, true)),
            },
            ty,
        );
        let output = native(vec![minimum], release, false);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, b"panic[P002]: int64 unary - overflow (value -9223372036854775808; range -9223372036854775808..9223372036854775807) at bytes 0..0\n");
    }
}

#[test]
pub(crate) fn strings_numbers_and_short_circuit_execute_in_each_profile() {
    for release in [false, true] {
        let ty = Type::Int {
            bits: 64,
            signed: true,
        };
        let divide = binary("/", integer(1, 64, true), integer(0, 64, true), ty.clone());
        let danger = binary("==", divide, integer(0, 64, true), Type::Bool);
        let short = binary(
            "&&",
            expr(ExprKind::Bool(false), Type::Bool),
            danger.clone(),
            Type::Bool,
        );
        let other = binary(
            "||",
            expr(ExprKind::Bool(true), Type::Bool),
            danger,
            Type::Bool,
        );
        let order = binary(
            ">",
            expr(ExprKind::String("é".into()), Type::String),
            expr(ExprKind::String("z".into()), Type::String),
            Type::Bool,
        );
        let equal = binary(
            "==",
            expr(ExprKind::String("a\0b".into()), Type::String),
            expr(ExprKind::String("a\0b".into()), Type::String),
            Type::Bool,
        );
        let parts = vec![
            short,
            other,
            order,
            equal,
            binary("/", integer(-7, 64, true), integer(3, 64, true), ty.clone()),
            binary("%", integer(-7, 64, true), integer(3, 64, true), ty),
            integer(u64::MAX.into(), 64, false),
            expr(ExprKind::String("猫\0".into()), Type::String),
            expr(ExprKind::Float(1.25), Type::Float { bits: 32 }),
            expr(ExprKind::Float(-0.0), Type::Float { bits: 64 }),
        ];
        let output = native(parts, release, false);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            "falsetruetruetrue-2-118446744073709551615猫\x001.25-0\n".as_bytes()
        );
    }
}

#[test]
pub(crate) fn output_failure_is_not_success() {
    let output = native(
        vec![expr(ExprKind::String("hello".into()), Type::String)],
        false,
        true,
    );
    assert_eq!(output.status.code(), Some(1));
}
