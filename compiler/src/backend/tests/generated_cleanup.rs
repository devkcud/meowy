use super::*;
use std::os::unix::process::ExitStatusExt;

pub(crate) const ABI: &str = r#"
%handle = type { ptr, i64, i64 }
@name = private constant [7 x i8] c"fixture"
@line = private constant [1 x i8] c"\0A"
declare i64 @meowy_cleanup_bytes_v0(i64)
declare i32 @meowy_cleanup_open_v0(ptr, i64, i64)
declare i32 @meowy_cleanup_mark_v0(ptr, ptr)
declare i32 @meowy_cleanup_reserve_v0(ptr, ptr)
declare i32 @meowy_cleanup_arm_v0(ptr, ptr, ptr, ptr, ptr, i64)
declare i32 @meowy_cleanup_unwind_v0(ptr, ptr, i32, ptr)
declare i32 @meowy_cleanup_finish_v0(ptr)
declare i64 @meowy_cleanup_panic_bytes_v0()
declare i32 @meowy_cleanup_panic_init_v0(ptr, i64, i32, ptr, i64)
declare void @meowy_int_v1(i32, i64)
declare void @meowy_write_v1(i32, ptr, i64)
declare i32 @setrlimit(i32, ptr)
"#;

#[test]
pub(crate) fn generated_callbacks_unwind_lifo_and_skip_uninitialized_storage() {
    for reason in [0, 1, 2, 4] {
        let ir = format!(
            r#"{ABI}
define void @drop(ptr %data, ptr %panic) {{
  %value = load i64, ptr %data
  call void @meowy_int_v1(i32 1, i64 %value)
  call void @meowy_write_v1(i32 1, ptr @line, i64 1)
  ret void
}}
define i32 @main() {{
  %size = call i64 @meowy_cleanup_bytes_v0(i64 3)
  %frame = alloca i8, i64 %size, align 16
  %root = alloca %handle, align 8
  %inner = alloca %handle, align 8
  %a = alloca %handle, align 8
  %empty = alloca %handle, align 8
  %b = alloca %handle, align 8
  %x = alloca i64, align 8
  %y = alloca i64, align 8
  store i64 1, ptr %x
  store i64 2, ptr %y
  %s0 = call i32 @meowy_cleanup_open_v0(ptr %frame, i64 %size, i64 3)
  %s1 = call i32 @meowy_cleanup_mark_v0(ptr %frame, ptr %root)
  %s2 = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %a)
  %s3 = call i32 @meowy_cleanup_arm_v0(ptr %frame, ptr %a, ptr %x, ptr @drop, ptr @name, i64 7)
  %s4 = call i32 @meowy_cleanup_mark_v0(ptr %frame, ptr %inner)
  %s5 = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %empty)
  %s6 = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %b)
  %s7 = call i32 @meowy_cleanup_arm_v0(ptr %frame, ptr %b, ptr %y, ptr @drop, ptr @name, i64 7)
  %s8 = call i32 @meowy_cleanup_unwind_v0(ptr %frame, ptr %inner, i32 {reason}, ptr null)
  %s9 = call i32 @meowy_cleanup_unwind_v0(ptr %frame, ptr %root, i32 0, ptr null)
  %s10 = call i32 @meowy_cleanup_finish_v0(ptr %frame)
  %v0 = or i32 %s0, %s1
  %v1 = or i32 %s2, %s3
  %v2 = or i32 %s4, %s5
  %v3 = or i32 %s6, %s7
  %v4 = or i32 %s8, %s9
  %v5 = or i32 %v0, %v1
  %v6 = or i32 %v2, %v3
  %v7 = or i32 %v4, %s10
  %v8 = or i32 %v5, %v6
  %result = or i32 %v8, %v7
  ret i32 %result
}}
"#
        );
        for release in [false, true] {
            let output = native_ir(&ir, release, false);
            assert!(output.status.success());
            assert_eq!(output.stdout, b"2\n1\n");
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
pub(crate) fn generated_cleanup_panic_owns_text_before_callback_storage_ends() {
    let ir = format!(
        r#"{ABI}
@first = private constant [11 x i8] c"body failed"
define void @drop(ptr %data, ptr %panic) {{
  %text = alloca [14 x i8], align 1
  store [14 x i8] c"release failed", ptr %text
  %size = call i64 @meowy_cleanup_panic_bytes_v0()
  %status = call i32 @meowy_cleanup_panic_init_v0(ptr %panic, i64 %size, i32 6, ptr %text, i64 14)
  store [14 x i8] zeroinitializer, ptr %text
  ret void
}}
define i32 @main() {{
  %limit = alloca {{ i64, i64 }}, align 8
  store {{ i64, i64 }} zeroinitializer, ptr %limit
  %limited = call i32 @setrlimit(i32 4, ptr %limit)
  %size = call i64 @meowy_cleanup_bytes_v0(i64 1)
  %frame = alloca i8, i64 %size, align 16
  %root = alloca %handle, align 8
  %token = alloca %handle, align 8
  %pbytes = call i64 @meowy_cleanup_panic_bytes_v0()
  %panic = alloca i8, i64 %pbytes, align 16
  %s0 = call i32 @meowy_cleanup_open_v0(ptr %frame, i64 %size, i64 1)
  %s1 = call i32 @meowy_cleanup_mark_v0(ptr %frame, ptr %root)
  %s2 = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %token)
  %s3 = call i32 @meowy_cleanup_arm_v0(ptr %frame, ptr %token, ptr null, ptr @drop, ptr @name, i64 7)
  %s4 = call i32 @meowy_cleanup_panic_init_v0(ptr %panic, i64 %pbytes, i32 6, ptr @first, i64 11)
  %s5 = call i32 @meowy_cleanup_unwind_v0(ptr %frame, ptr %root, i32 3, ptr %panic)
  ret i32 99
}}
"#
    );
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert_eq!(output.status.signal(), Some(6));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"panic[P008]: panic during cleanup\noriginal: panic P006: body failed\ncleanup: fixture\nsecond: P006: release failed\n");
    }
}
