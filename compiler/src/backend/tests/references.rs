use super::*;

#[test]
pub(crate) fn shared_references_preserve_storage_identity_and_field_addresses() {
    let ty = Type::Int {
        bits: 32,
        signed: true,
    };
    let reference = Type::Reference(Box::new(ty.clone()));
    let borrow = |root, fields| expr(ExprKind::Borrow(Place { root, fields }), reference.clone());
    let value = record(
        1,
        integer(2, 8, false),
        record(2, integer(3, 16, true), integer(7, 32, true)),
    );
    let shape = value.ty.clone();
    let alias = expr(ExprKind::Local(2), reference.clone());
    let parts = vec![
        binary("==", alias.clone(), borrow(0, vec![]), Type::Bool),
        binary("==", borrow(0, vec![]), borrow(1, vec![]), Type::Bool),
        expr(ExprKind::Deref(Box::new(alias)), ty.clone()),
        expr(ExprKind::Deref(Box::new(borrow(3, vec![0, 0]))), ty.clone()),
        binary(
            "==",
            borrow(3, vec![0, 0]),
            borrow(3, vec![0, 0]),
            Type::Bool,
        ),
        expr(
            ExprKind::Deref(Box::new(expr(
                ExprKind::Borrow(Place {
                    root: 3,
                    fields: vec![],
                }),
                Type::Reference(Box::new(shape.clone())),
            ))),
            shape.clone(),
        ),
    ];
    let program = Program {
        body: Block {
            id: 0,
            ty: Type::Null,
            stmts: vec![
                Stmt::Bind {
                    id: 0,
                    value: integer(42, 32, true),
                },
                Stmt::Bind {
                    id: 1,
                    value: integer(42, 32, true),
                },
                Stmt::Bind {
                    id: 2,
                    value: borrow(0, vec![]),
                },
                Stmt::Bind { id: 3, value },
                Stmt::Expr(expr(
                    ExprKind::Print {
                        parts,
                        newline: true,
                    },
                    Type::Null,
                )),
            ],
        },
        functions: Vec::new(),
        locals: vec![ty.clone(), ty, reference, shape],
    };
    for release in [false, true] {
        let output = native_program(&program, release, false);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"truefalse427true2\n");
    }
}

#[test]
pub(crate) fn element_references_preserve_owner_identity_and_nested_offsets() {
    let int = Type::Int {
        bits: 32,
        signed: true,
    };
    let values = list(
        int.clone(),
        3,
        vec![integer(11, 32, true), integer(22, 32, true)],
    );
    let row = record(
        2,
        integer(7, 16, false),
        list(
            int.clone(),
            4,
            vec![integer(88, 32, true), integer(99, 32, true)],
        ),
    );
    let rows = list(row.ty.clone(), 2, vec![row.clone()]);
    let nested = record(1, expr(ExprKind::Bool(true), Type::Bool), rows.clone());
    let list_ref = Type::Reference(Box::new(values.ty.clone()));
    let first = element_ref(borrow(0, values.ty.clone()), integer(1, 8, false));
    let second = element_ref(borrow(0, values.ty.clone()), integer(2, 16, true));
    let inner = expr(
        ExprKind::Reborrow {
            site: 1,
            value: Box::new(borrow(3, nested.ty.clone())),
            fields: vec![0],
        },
        Type::Reference(Box::new(rows.ty.clone())),
    );
    let Type::Record { fields, .. } = &row.ty else {
        unreachable!()
    };
    let inner = expr(
        ExprKind::Reborrow {
            site: 2,
            value: Box::new(element_ref(inner, integer(1, 32, false))),
            fields: vec![0],
        },
        Type::Reference(Box::new(fields[0].ty.clone())),
    );
    let inner = element_ref(inner, integer(2, 64, false));
    let parts = separated(vec![
        binary("==", first.clone(), first.clone(), Type::Bool),
        binary("==", first.clone(), second.clone(), Type::Bool),
        binary(
            "==",
            first,
            element_ref(borrow(1, values.ty.clone()), integer(1, 32, true)),
            Type::Bool,
        ),
        binary(
            "==",
            second.clone(),
            element_ref(
                expr(ExprKind::Local(2), list_ref.clone()),
                integer(2, 8, true),
            ),
            Type::Bool,
        ),
        expr(ExprKind::Deref(Box::new(second)), int.clone()),
        expr(ExprKind::Deref(Box::new(inner.clone())), int),
        binary("==", inner.clone(), inner, Type::Bool),
    ]);
    let program = Program {
        body: Block {
            id: 0,
            ty: Type::Null,
            stmts: vec![
                Stmt::Bind {
                    id: 0,
                    value: values.clone(),
                },
                Stmt::Bind {
                    id: 1,
                    value: expr(ExprKind::Local(0), values.ty.clone()),
                },
                Stmt::Bind {
                    id: 2,
                    value: borrow(0, values.ty.clone()),
                },
                Stmt::Bind {
                    id: 3,
                    value: nested.clone(),
                },
                Stmt::Expr(expr(
                    ExprKind::Print {
                        parts,
                        newline: true,
                    },
                    Type::Null,
                )),
            ],
        },
        functions: Vec::new(),
        locals: vec![values.ty.clone(), values.ty, list_ref, nested.ty],
    };
    for release in [false, true] {
        let output = native_program(&program, release, false);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"true|false|false|true|22|99|true|\n");
        assert!(output.stderr.is_empty());
    }
}

#[test]
pub(crate) fn element_reference_failures_preserve_index_widths_and_initialized_length() {
    let values = list(Type::Bool, 4, vec![expr(ExprKind::Bool(true), Type::Bool)]);
    for release in [false, true] {
        for bits in [8, 16, 32, 64] {
            for signed in [false, true] {
                let number = if signed {
                    -(1i128 << (bits - 1))
                } else {
                    (1i128 << bits) - 1
                };
                let mut value =
                    element_ref(borrow(0, values.ty.clone()), integer(number, bits, signed));
                value.span = Span { start: 12, end: 34 };
                let program = Program {
                    body: Block {
                        id: 0,
                        ty: Type::Null,
                        stmts: vec![
                            Stmt::Bind {
                                id: 0,
                                value: values.clone(),
                            },
                            Stmt::Expr(value),
                        ],
                    },
                    functions: Vec::new(),
                    locals: vec![values.ty.clone()],
                };
                let output = native_program(&program, release, false);
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert_eq!(output.stderr, format!("panic[P001]: index {number} is outside initialized length 1 at bytes 12..34\n").as_bytes());
            }
        }
    }
}

#[test]
pub(crate) fn element_references_evaluate_parent_then_index_and_stop_on_divergence() {
    let print = |text: &str| {
        Stmt::Expr(expr(
            ExprKind::Print {
                parts: vec![expr(ExprKind::String(text.into()), Type::String)],
                newline: true,
            },
            Type::Null,
        ))
    };
    for release in [false, true] {
        for panic in [false, true] {
            let values = list(Type::Bool, 0, Vec::new());
            let parent = expr(
                ExprKind::Block(Block {
                    id: 1,
                    ty: Type::Reference(Box::new(values.ty.clone())),
                    stmts: vec![
                        print("parent"),
                        Stmt::Emit {
                            id: 0,
                            target: 1,
                            field: None,
                            value: borrow(0, values.ty.clone()),
                        },
                    ],
                }),
                Type::Reference(Box::new(values.ty.clone())),
            );
            let result = if panic {
                Type::Never
            } else {
                integer(1, 32, true).ty
            };
            let end = if panic {
                Stmt::Expr(Expr {
                    kind: ExprKind::Panic {
                        parts: vec![expr(ExprKind::String("stop".into()), Type::String)],
                    },
                    ty: Type::Never,
                    span: Span { start: 30, end: 40 },
                })
            } else {
                Stmt::Emit {
                    id: 1,
                    target: 2,
                    field: None,
                    value: integer(1, 32, true),
                }
            };
            let index = expr(
                ExprKind::Block(Block {
                    id: 2,
                    ty: result.clone(),
                    stmts: vec![print("index"), end],
                }),
                result,
            );
            let mut value = element_ref(parent, index);
            value.span = Span { start: 10, end: 50 };
            let program = Program {
                body: Block {
                    id: 0,
                    ty: Type::Null,
                    stmts: vec![
                        Stmt::Bind {
                            id: 0,
                            value: values.clone(),
                        },
                        Stmt::Expr(value),
                    ],
                },
                functions: Vec::new(),
                locals: vec![values.ty],
            };
            if panic {
                assert!(
                    !emit_ir(&program)
                        .unwrap()
                        .contains("call void @meowy_index_fail_v1")
                );
            }
            let output = native_program(&program, release, false);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, b"parent\nindex\n");
            assert_eq!(
                output.stderr,
                if panic {
                    b"panic[P006]: stop at bytes 30..40\n".as_slice()
                } else {
                    b"panic[P001]: index 1 is outside initialized length 0 at bytes 10..50\n"
                        .as_slice()
                }
            );
        }
    }
}
