use super::{Block, Program, Type, emit_ir, ir_type, native_ir};
use crate::hir::FoundationType;

#[test]
pub(crate) fn nominal_layouts_match_the_native_string_descriptor() {
    let owner = Type::Foundation(FoundationType::OwnedString);
    let failure = Type::Foundation(FoundationType::AllocationFailure);
    let allocator = Type::Foundation(FoundationType::Allocator);
    let ir = format!(
        r#"
target triple = "x86_64-unknown-linux-gnu"
declare i64 @meowy_string_bytes_v0()
declare i64 @meowy_string_alignment_v0()
define i32 @main() {{
  %native_size = call i64 @meowy_string_bytes_v0()
  %native_align = call i64 @meowy_string_alignment_v0()
  %end = getelementptr {owner}, ptr null, i32 1
  %size = ptrtoint ptr %end to i64
  %a = icmp eq i64 %native_size, %size
  %b = icmp eq i64 %native_size, {bytes}
  %c = icmp eq i64 %native_align, {alignment}
  %field = getelementptr {failure}, ptr null, i32 0, i32 1
  %offset = ptrtoint ptr %field to i64
  %d = icmp eq i64 %offset, 8
  %last = getelementptr {failure}, ptr null, i32 0, i32 2
  %last_offset = ptrtoint ptr %last to i64
  %e = icmp eq i64 %last_offset, 16
  %failure_end = getelementptr {failure}, ptr null, i32 1
  %failure_size = ptrtoint ptr %failure_end to i64
  %f = icmp eq i64 %failure_size, {failure_bytes}
  %allocator_end = getelementptr {allocator}, ptr null, i32 1
  %allocator_size = ptrtoint ptr %allocator_end to i64
  %g = icmp eq i64 %allocator_size, {allocator_bytes}
  %v0 = and i1 %a, %b
  %v1 = and i1 %c, %d
  %v2 = and i1 %e, %f
  %v3 = and i1 %v0, %v1
  %v4 = and i1 %v2, %g
  %valid = and i1 %v3, %v4
  %result = select i1 %valid, i32 0, i32 1
  ret i32 %result
}}
"#,
        bytes = owner.layout().unwrap().0,
        alignment = owner.layout().unwrap().1,
        failure_bytes = failure.layout().unwrap().0,
        allocator_bytes = allocator.layout().unwrap().0,
        owner = ir_type(&owner),
        failure = ir_type(&failure),
        allocator = ir_type(&allocator),
    );
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert!(output.status.success());
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
}

#[test]
pub(crate) fn native_failure_facts_survive_source_calls_borrows_and_unions() {
    let source = r#"m:@"memory";<E>:<m.AllocationFailure>;take<E>:(e<&E>){->*e};maybe<E><null>:(e<E>,keep<boolean>){|keep|->take(&e)}"#;
    let program = crate::compile(source).unwrap();
    let mut ir = emit_ir(&program)
        .unwrap()
        .replace("define i32 @main()", "define internal i32 @unused_main()");
    let ty = ir_type(&program.functions[1].result);
    ir.push_str(&format!(r#"
define i32 @main() {{
  %bytes = call i64 @meowy_cleanup_panic_bytes_v0()
  %panic = alloca i8, i64 %bytes, align 16
  call void @meowy_panic_begin_v0(ptr %panic, i32 0)
  %result = alloca {ty}, align 8
  %ok = call i1 @meowy_fn_1(ptr %panic, ptr %result, {{ i32, i64, i64 }} {{ i32 1, i64 18446744073709551615, i64 16 }}, i1 true)
  br i1 %ok, label %inspect, label %invalid
inspect:
  %tag = load i32, ptr %result
  %tag_ok = icmp eq i32 %tag, 1
  br i1 %tag_ok, label %facts, label %invalid
facts:
  %payload = getelementptr {ty}, ptr %result, i32 0, i32 1
  %value = load {{ i32, i64, i64 }}, ptr %payload
  %cause = extractvalue {{ i32, i64, i64 }} %value, 0
  %size = extractvalue {{ i32, i64, i64 }} %value, 1
  %alignment = extractvalue {{ i32, i64, i64 }} %value, 2
  %a = icmp eq i32 %cause, 1
  %b = icmp eq i64 %size, 18446744073709551615
  %c = icmp eq i64 %alignment, 16
  %ab = and i1 %a, %b
  %valid = and i1 %ab, %c
  br i1 %valid, label %absent, label %invalid
absent:
  %empty_ok = call i1 @meowy_fn_1(ptr %panic, ptr %result, {{ i32, i64, i64 }} {{ i32 1, i64 7, i64 1 }}, i1 false)
  %empty_tag = load i32, ptr %result
  %null = icmp eq i32 %empty_tag, 0
  %passed = and i1 %empty_ok, %null
  %code = select i1 %passed, i32 0, i32 1
  ret i32 %code
invalid:
  ret i32 99
}}
"#));
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert!(output.status.success());
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
}

#[test]
pub(crate) fn backend_rejects_unscheduled_owners_but_not_exclusive_loans() {
    let owner = Type::Foundation(FoundationType::OwnedString);
    let reference = Type::Reference(Box::new(owner.clone()));
    let exclusive = Type::Exclusive(Box::new(Type::Int {
        bits: 32,
        signed: true,
    }));
    assert!(!owner.is_copy() && owner.has_drop());
    assert!(reference.is_copy() && !reference.has_drop());
    assert!(!exclusive.is_copy() && !exclusive.has_drop());
    for ty in [
        owner.clone(),
        Type::List {
            element: Box::new(owner),
            capacity: 0,
        },
    ] {
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: vec![],
            },
            functions: vec![],
            locals: vec![ty],
        };
        assert!(emit_ir(&program).unwrap_err().contains("cleanup schedules"));
    }
}
