use super::*;
use std::os::unix::process::ExitStatusExt;

pub(crate) fn fixture(capacity: u64, fatal: bool) -> String {
    let abi = super::generated_cleanup::ABI;
    let success = capacity >= 16;
    let expected = if success { 0 } else { 2 };
    let reason = if fatal { 3 } else { 0 };
    let failure = if fatal {
        r#"
  %text = alloca [14 x i8], align 1
  store [14 x i8] c"release failed", ptr %text
  %bytes = call i64 @meowy_cleanup_panic_bytes_v0()
  %captured = call i32 @meowy_cleanup_panic_init_v0(ptr %panic, i64 %bytes, i32 6, ptr %text, i64 14)
  call void @require(i32 %captured)
  store [14 x i8] zeroinitializer, ptr %text
"#
    } else {
        ""
    };
    format!(
        r#"{abi}
@ops = internal global [128 x i8] zeroinitializer, align 16
@owned_name = private constant [13 x i8] c"owned fixture"
@first = private constant [11 x i8] c"body failed"
declare i64 @meowy_owned_bytes_v0()
declare i64 @meowy_owned_ops_bytes_v0()
declare i32 @meowy_owned_ops_init_v0(ptr, i64, i64, i64, ptr, ptr, ptr, i64)
declare i32 @meowy_owned_open_v0(ptr, i64, ptr, i64)
declare i32 @meowy_owned_reserve_v0(ptr, ptr)
declare ptr @meowy_owned_data_v0(ptr)
declare i32 @meowy_owned_commit_v0(ptr)
declare i32 @meowy_owned_finish_v0(ptr)
declare i32 @meowy_owned_arm_v0(ptr, ptr, ptr)
declare i32 @meowy_owned_transfer_v0(ptr, ptr, ptr, ptr, ptr, ptr)
declare void @llvm.lifetime.start.p0(i64, ptr nocapture)
declare void @llvm.lifetime.end.p0(i64, ptr nocapture)
declare void @abort() noreturn

define void @require(i32 %status) {{
  %valid = icmp eq i32 %status, 0
  br i1 %valid, label %ok, label %bad
bad:
  call void @abort()
  unreachable
ok:
  ret void
}}
define void @print(i64 %value) {{
  call void @meowy_int_v1(i32 1, i64 %value)
  call void @meowy_write_v1(i32 1, ptr @line, i64 1)
  ret void
}}
define void @move(ptr %to, ptr %from) {{
  call void @llvm.lifetime.start.p0(i64 16, ptr %to)
  %old = load {{ ptr, i64 }}, ptr %from
  %value = extractvalue {{ ptr, i64 }} %old, 1
  %new0 = insertvalue {{ ptr, i64 }} poison, ptr %to, 0
  %new1 = insertvalue {{ ptr, i64 }} %new0, i64 %value, 1
  store {{ ptr, i64 }} %new1, ptr %to
  store {{ ptr, i64 }} zeroinitializer, ptr %from
  call void @llvm.lifetime.end.p0(i64 16, ptr %from)
  call void @print(i64 77)
  ret void
}}
define void @drop(ptr %data, ptr %panic) {{
  %value = load {{ ptr, i64 }}, ptr %data
  %self = extractvalue {{ ptr, i64 }} %value, 0
  %valid = icmp eq ptr %self, %data
  %status = select i1 %valid, i32 0, i32 1
  call void @require(i32 %status)
  %number = extractvalue {{ ptr, i64 }} %value, 1
  call void @print(i64 %number)
  {failure}
  call void @llvm.lifetime.end.p0(i64 16, ptr %data)
  ret void
}}
define i32 @main() {{
  %limit = alloca {{ i64, i64 }}, align 8
  store {{ i64, i64 }} zeroinitializer, ptr %limit
  %limited = call i32 @setrlimit(i32 4, ptr %limit)
  call void @require(i32 %limited)
  %s0 = call i32 @meowy_owned_ops_init_v0(ptr @ops, i64 128, i64 16, i64 8, ptr @move, ptr @drop, ptr @owned_name, i64 13)
  call void @require(i32 %s0)
  %bytes = call i64 @meowy_cleanup_bytes_v0(i64 1)
  %a = alloca i8, i64 %bytes, align 16
  %b = alloca i8, i64 %bytes, align 16
  %root_a = alloca %handle, align 8
  %root_b = alloca %handle, align 8
  %old = alloca %handle, align 8
  %next = alloca %handle, align 8
  %s1 = call i32 @meowy_cleanup_open_v0(ptr %a, i64 %bytes, i64 1)
  %s2 = call i32 @meowy_cleanup_open_v0(ptr %b, i64 %bytes, i64 1)
  call void @require(i32 %s1)
  call void @require(i32 %s2)
  %s3 = call i32 @meowy_cleanup_mark_v0(ptr %a, ptr %root_a)
  %s4 = call i32 @meowy_cleanup_mark_v0(ptr %b, ptr %root_b)
  call void @require(i32 %s3)
  call void @require(i32 %s4)
  %size = call i64 @meowy_owned_bytes_v0()
  %source = alloca i8, i64 %size, align 16
  %target = alloca i8, i64 %size, align 16
  %from = alloca {{ ptr, i64 }}, align 8
  %to = alloca [16 x i8], align 8
  %s5 = call i32 @meowy_owned_open_v0(ptr %source, i64 %size, ptr %from, i64 16)
  %s6 = call i32 @meowy_owned_open_v0(ptr %target, i64 %size, ptr %to, i64 {capacity})
  call void @require(i32 %s5)
  call void @require(i32 %s6)
  %s7 = call i32 @meowy_owned_reserve_v0(ptr %source, ptr @ops)
  call void @require(i32 %s7)
  call void @llvm.lifetime.start.p0(i64 16, ptr %from)
  %value0 = insertvalue {{ ptr, i64 }} poison, ptr %from, 0
  %value1 = insertvalue {{ ptr, i64 }} %value0, i64 42, 1
  store {{ ptr, i64 }} %value1, ptr %from
  %s8 = call i32 @meowy_owned_commit_v0(ptr %source)
  call void @require(i32 %s8)
  %s9 = call i32 @meowy_cleanup_reserve_v0(ptr %a, ptr %old)
  call void @require(i32 %s9)
  %s10 = call i32 @meowy_owned_arm_v0(ptr %a, ptr %old, ptr %source)
  call void @require(i32 %s10)
  %s11 = call i32 @meowy_cleanup_reserve_v0(ptr %b, ptr %next)
  call void @require(i32 %s11)
  %moved = call i32 @meowy_owned_transfer_v0(ptr %a, ptr %old, ptr %source, ptr %b, ptr %next, ptr %target)
  %matches = icmp eq i32 %moved, {expected}
  %checked = select i1 %matches, i32 0, i32 1
  call void @require(i32 %checked)
  %owner = select i1 {success}, ptr %target, ptr %source
  %ptr = call ptr @meowy_owned_data_v0(ptr %owner)
  %payload = load {{ ptr, i64 }}, ptr %ptr
  %number = extractvalue {{ ptr, i64 }} %payload, 1
  call void @print(i64 %number)
  %s12 = call i32 @meowy_cleanup_unwind_v0(ptr %a, ptr %root_a, i32 1, ptr null)
  call void @require(i32 %s12)
  %s13 = call i32 @meowy_cleanup_finish_v0(ptr %a)
  %s14 = call i32 @meowy_owned_finish_v0(ptr %source)
  call void @require(i32 %s13)
  call void @require(i32 %s14)
  %pbytes = call i64 @meowy_cleanup_panic_bytes_v0()
  %panic = alloca i8, i64 %pbytes, align 16
  %s15 = call i32 @meowy_cleanup_panic_init_v0(ptr %panic, i64 %pbytes, i32 6, ptr @first, i64 11)
  call void @require(i32 %s15)
  %cause = select i1 {fatal}, ptr %panic, ptr null
  %s16 = call i32 @meowy_cleanup_unwind_v0(ptr %b, ptr %root_b, i32 {reason}, ptr %cause)
  call void @require(i32 %s16)
  %s17 = call i32 @meowy_cleanup_finish_v0(ptr %b)
  %s18 = call i32 @meowy_owned_finish_v0(ptr %target)
  call void @require(i32 %s17)
  call void @require(i32 %s18)
  ret i32 0
}}
"#
    )
}

#[test]
pub(crate) fn generated_ownership_relocates_payloads_and_retains_failed_sources() {
    for capacity in [8, 16] {
        for release in [false, true] {
            let output = native_ir(&fixture(capacity, false), release, false);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.stdout,
                if capacity == 16 {
                    b"77\n42\n42\n".as_slice()
                } else {
                    b"42\n42\n".as_slice()
                }
            );
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
pub(crate) fn generated_owned_drop_preserves_the_enclosing_panic_cause() {
    for release in [false, true] {
        let output = native_ir(&fixture(16, true), release, false);
        assert_eq!(output.status.signal(), Some(6));
        assert_eq!(output.stdout, b"77\n42\n42\n");
        assert_eq!(output.stderr, b"panic[P008]: panic during cleanup\noriginal: panic P006: body failed\ncleanup: owned fixture\nsecond: P006: release failed\n");
    }
}
