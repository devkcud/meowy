use super::*;

#[test]
pub(crate) fn invalid_ir_returns_a_bridge_error() {
    let path = std::env::temp_dir().join(format!(
        "meowy-invalid-{}-{}.o",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let error = emit_object("invalid LLVM", &path, false).unwrap_err();
    assert!(error.contains("LLVM IR parser"));
    assert!(!path.exists());
}

#[test]
pub(crate) fn invalid_ssa_is_rejected_before_object_output() {
    let path = std::env::temp_dir().join(format!(
        "meowy-invalid-ssa-{}-{}.o",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let error = emit_object(
        "define i32 @main() { entry: ret i32 %value dead: %value = add i32 1, 2 ret i32 0 }",
        &path,
        false,
    )
    .unwrap_err();
    assert!(error.contains("LLVM verification"));
    assert!(!path.exists());
}

#[test]
pub(crate) fn object_is_an_x86_64_elf_in_each_profile() {
    let program = Program {
        body: Block {
            id: 0,
            ty: Type::Null,
            stmts: Vec::new(),
        },
        functions: Vec::new(),
        locals: Vec::new(),
    };
    let ir = emit_ir(&program).unwrap();
    let size = ir.len();
    let bounded = format!("{ir}invalid trailing bytes");
    for release in [false, true] {
        let path = std::env::temp_dir().join(format!(
            "meowy-object-{}-{}.o",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        emit_object(&bounded[..size], &path, release).unwrap();
        let object = std::fs::read(&path).unwrap();
        assert_eq!(&object[..4], b"\x7fELF");
        assert_eq!(object[4], 2);
        assert_eq!(u16::from_le_bytes([object[18], object[19]]), 62);
        std::fs::remove_file(path).unwrap();
    }
}
