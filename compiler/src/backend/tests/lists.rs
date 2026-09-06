use super::*;

#[test]
pub(crate) fn lists_compare_initialized_prefixes_and_preserve_element_equality() {
    let int = Type::Int {
        bits: 32,
        signed: true,
    };
    let values = list(
        int.clone(),
        3,
        vec![integer(11, 32, true), integer(22, 32, true)],
    );
    let appended = expr(
        ExprKind::ListAdd {
            value: Box::new(values.clone()),
            item: Box::new(integer(33, 32, true)),
        },
        values.ty.clone(),
    );
    let empty = list(Type::Bool, 0, Vec::new());
    let short = list(int, 3, vec![integer(11, 32, true)]);
    let float = Type::Float { bits: 64 };
    let zero = list(
        float.clone(),
        2,
        vec![expr(ExprKind::Float(0.0), float.clone())],
    );
    let negative = list(
        float.clone(),
        2,
        vec![expr(ExprKind::Float(-0.0), float.clone())],
    );
    let nan = binary(
        "/",
        expr(ExprKind::Float(0.0), float.clone()),
        expr(ExprKind::Float(0.0), float.clone()),
        float.clone(),
    );
    let nan = list(float, 1, vec![nan]);
    for release in [false, true] {
        let output = native(
            separated(vec![
                binary("==", empty.clone(), empty.clone(), Type::Bool),
                size(values.clone()),
                index(values.clone(), integer(2, 8, false)),
                size(appended.clone()),
                index(appended.clone(), integer(3, 64, true)),
                size(values.clone()),
                binary("==", values.clone(), short.clone(), Type::Bool),
                binary("==", zero.clone(), negative.clone(), Type::Bool),
                binary("==", nan.clone(), nan.clone(), Type::Bool),
            ]),
            release,
            false,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"true|2|22|3|33|2|false|true|false|\n");
    }
}

#[test]
pub(crate) fn lists_preserve_nested_and_padded_union_payloads() {
    let record = record(
        1,
        expr(ExprKind::Bool(true), Type::Bool),
        integer(41, 64, true),
    );
    let values = list(record.ty.clone(), 3, vec![record.clone()]);
    let union = Type::union([Type::Null, values.ty.clone()]);
    let packed = coerce(values.clone(), &union);
    let restored = coerce(packed.clone(), &values.ty);
    let first = index(restored.clone(), integer(1, 32, true));
    let inner = list(Type::Bool, 3, vec![expr(ExprKind::Bool(true), Type::Bool)]);
    let outer = list(inner.ty.clone(), 2, vec![inner]);
    let outer_union = Type::union([Type::Null, outer.ty.clone()]);
    let nested = coerce(coerce(outer.clone(), &outer_union), &outer.ty);
    for release in [false, true] {
        let output = native(
            separated(vec![
                expr(ExprKind::Primary(Box::new(first.clone())), Type::Bool),
                expr(
                    ExprKind::Field {
                        value: Box::new(first.clone()),
                        index: 0,
                    },
                    Type::Int {
                        bits: 64,
                        signed: true,
                    },
                ),
                size(restored.clone()),
                binary("==", packed.clone(), packed.clone(), Type::Bool),
                index(
                    index(nested.clone(), integer(1, 32, true)),
                    integer(1, 32, true),
                ),
            ]),
            release,
            false,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"true|41|1|true|true|\n");
    }
}

#[test]
pub(crate) fn list_failures_keep_index_sign_length_capacity_and_site() {
    let values = list(
        Type::Bool,
        256,
        vec![expr(ExprKind::Bool(true), Type::Bool); 256],
    );
    for release in [false, true] {
        for (position, text) in [
            (integer(-1, 8, true), "-1"),
            (integer(0, 32, true), "0"),
            (integer(u64::MAX.into(), 64, false), "18446744073709551615"),
        ] {
            let mut value = index(values.clone(), position);
            value.span = Span { start: 10, end: 20 };
            let output = native(vec![value], release, false);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(
                String::from_utf8_lossy(&output.stderr),
                format!(
                    "panic[P001]: index {text} is outside initialized length 256 at bytes 10..20\n"
                )
            );
        }
        let value = list(Type::Bool, 1, vec![expr(ExprKind::Bool(true), Type::Bool)]);
        let ty = value.ty.clone();
        let mut full = expr(
            ExprKind::ListAdd {
                value: Box::new(value),
                item: Box::new(expr(ExprKind::Bool(false), Type::Bool)),
            },
            ty,
        );
        full.span = Span { start: 30, end: 40 };
        let output = native(vec![size(full)], release, false);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            b"panic[P003]: bounded list is full (length 1, capacity 1) at bytes 30..40\n"
        );
    }
}
