use super::native_ir;

#[test]
pub(crate) fn generated_strings_copy_transfer_and_cleanup_actual_heap_storage() {
    let ir = r#"
target triple = "x86_64-unknown-linux-gnu"
%handle = type { ptr, i64, i64 }
%failure = type { i32, i64, i64 }
@newline = private constant [1 x i8] c"\0A"
declare void @meowy_write_v1(i32, ptr, i64)
declare ptr @meowy_string_heap_v0()
declare i64 @meowy_string_bytes_v0()
declare i32 @meowy_string_copy_v0(ptr, ptr, i64, ptr, ptr)
declare i32 @meowy_string_view_v0(ptr, ptr, ptr)
declare i64 @meowy_owned_bytes_v0()
declare i32 @meowy_owned_open_v0(ptr, i64, ptr, i64)
declare i32 @meowy_owned_finish_v0(ptr)
declare i32 @meowy_owned_arm_v0(ptr, ptr, ptr)
declare i32 @meowy_owned_transfer_v0(ptr, ptr, ptr, ptr, ptr, ptr)
declare i64 @meowy_cleanup_bytes_v0(i64)
declare i32 @meowy_cleanup_open_v0(ptr, i64, i64)
declare i32 @meowy_cleanup_mark_v0(ptr, ptr)
declare i32 @meowy_cleanup_reserve_v0(ptr, ptr)
declare i32 @meowy_cleanup_unwind_v0(ptr, ptr, i32, ptr)
declare i32 @meowy_cleanup_finish_v0(ptr)
define i32 @main() {
  %osize = call i64 @meowy_owned_bytes_v0()
  %psize = call i64 @meowy_string_bytes_v0()
  %a = alloca i8, i64 %osize, align 16
  %b = alloca i8, i64 %osize, align 16
  %pa = alloca i8, i64 %psize, align 16
  %pb = alloca i8, i64 %psize, align 16
  %failure = alloca %failure, align 8
  %text = alloca [4 x i8], align 1
  store [4 x i8] c"meow", ptr %text
  %heap = call ptr @meowy_string_heap_v0()
  %s0 = call i32 @meowy_owned_open_v0(ptr %a, i64 %osize, ptr %pa, i64 %psize)
  %s1 = call i32 @meowy_owned_open_v0(ptr %b, i64 %osize, ptr %pb, i64 %psize)
  %fsize = call i64 @meowy_cleanup_bytes_v0(i64 2)
  %frame = alloca i8, i64 %fsize, align 16
  %root = alloca %handle, align 8
  %x = alloca %handle, align 8
  %y = alloca %handle, align 8
  %s2 = call i32 @meowy_cleanup_open_v0(ptr %frame, i64 %fsize, i64 2)
  %s3 = call i32 @meowy_cleanup_mark_v0(ptr %frame, ptr %root)
  %s4 = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %x)
  %v0 = or i32 %s0, %s1
  %v1 = or i32 %s2, %s3
  %v2 = or i32 %v0, %v1
  %v3 = or i32 %v2, %s4
  %ready = icmp eq i32 %v3, 0
  br i1 %ready, label %construct, label %invalid
construct:
  %copied = call i32 @meowy_string_copy_v0(ptr %a, ptr %text, i64 4, ptr %heap, ptr %failure)
  %live = icmp eq i32 %copied, 0
  br i1 %live, label %arm, label %invalid
arm:
  store [4 x i8] zeroinitializer, ptr %text
  %armed = call i32 @meowy_owned_arm_v0(ptr %frame, ptr %x, ptr %a)
  %reserved = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %y)
  %v4 = or i32 %armed, %reserved
  %can_move = icmp eq i32 %v4, 0
  br i1 %can_move, label %move, label %invalid
move:
  %moved = call i32 @meowy_owned_transfer_v0(ptr %frame, ptr %x, ptr %a, ptr %frame, ptr %y, ptr %b)
  %done = icmp eq i32 %moved, 0
  br i1 %done, label %view, label %invalid
view:
  %pointer = alloca ptr, align 8
  %length = alloca i64, align 8
  %viewed = call i32 @meowy_string_view_v0(ptr %b, ptr %pointer, ptr %length)
  %visible = icmp eq i32 %viewed, 0
  br i1 %visible, label %print, label %invalid
print:
  %data = load ptr, ptr %pointer
  %size = load i64, ptr %length
  call void @meowy_write_v1(i32 1, ptr %data, i64 %size)
  call void @meowy_write_v1(i32 1, ptr @newline, i64 1)
  %unwound = call i32 @meowy_cleanup_unwind_v0(ptr %frame, ptr %root, i32 0, ptr null)
  %closed = call i32 @meowy_cleanup_finish_v0(ptr %frame)
  %a_done = call i32 @meowy_owned_finish_v0(ptr %a)
  %b_done = call i32 @meowy_owned_finish_v0(ptr %b)
  %v5 = or i32 %unwound, %closed
  %v6 = or i32 %a_done, %b_done
  %result = or i32 %v5, %v6
  ret i32 %result
invalid:
  ret i32 99
}
"#;
    for release in [false, true] {
        let output = native_ir(ir, release, false);
        assert!(output.status.success());
        assert_eq!(output.stdout, b"meow\n");
        assert!(output.stderr.is_empty());
    }
}
