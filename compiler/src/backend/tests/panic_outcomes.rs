use super::{emit_ir, native_ir};
use std::os::unix::process::ExitStatusExt;

pub(crate) fn probe(source: &str, fatal: bool) -> String {
    let program = crate::compile(source).unwrap();
    let mut ir = emit_ir(&program)
        .unwrap()
        .replace("define i32 @main()", "define internal i32 @unused_main()");
    let drop = if fatal {
        r#"%size = call i64 @meowy_cleanup_panic_bytes_v0()
  %text = alloca [14 x i8], align 1
  store [14 x i8] c"release failed", ptr %text
  %status = call i32 @meowy_cleanup_panic_init_v0(ptr %panic, i64 %size, i32 6, ptr %text, i64 14)
  store [14 x i8] zeroinitializer, ptr %text"#
    } else {
        r#"%value = load i64, ptr %data
  call void @meowy_int_v1(i32 1, i64 %value)
  call void @meowy_write_v1(i32 1, ptr @newline, i64 1)"#
    };
    ir.push_str(&format!(
        r#"
%handle = type {{ ptr, i64, i64 }}
@drop_name = private constant [7 x i8] c"fixture"
@newline = private constant [1 x i8] c"\0A"
declare i32 @setrlimit(i32, ptr)
declare i64 @meowy_cleanup_bytes_v0(i64)
declare i32 @meowy_cleanup_open_v0(ptr, i64, i64)
declare i32 @meowy_cleanup_mark_v0(ptr, ptr)
declare i32 @meowy_cleanup_reserve_v0(ptr, ptr)
declare i32 @meowy_cleanup_arm_v0(ptr, ptr, ptr, ptr, ptr, i64)
declare i32 @meowy_cleanup_unwind_v0(ptr, ptr, i32, ptr)
declare i32 @meowy_cleanup_finish_v0(ptr)
declare i32 @meowy_cleanup_panic_init_v0(ptr, i64, i32, ptr, i64)
define void @drop(ptr %data, ptr %panic) {{
  {drop}
  ret void
}}
define i32 @main() {{
  %limit = alloca {{ i64, i64 }}, align 8
  store {{ i64, i64 }} zeroinitializer, ptr %limit
  %limited = call i32 @setrlimit(i32 4, ptr %limit)
  %size = call i64 @meowy_cleanup_bytes_v0(i64 2)
  %frame = alloca i8, i64 %size, align 16
  %mark = alloca %handle, align 8
  %a = alloca %handle, align 8
  %b = alloca %handle, align 8
  %x = alloca i64, align 8
  %y = alloca i64, align 8
  store i64 1, ptr %x
  store i64 2, ptr %y
  %s0 = call i32 @meowy_cleanup_open_v0(ptr %frame, i64 %size, i64 2)
  %s1 = call i32 @meowy_cleanup_mark_v0(ptr %frame, ptr %mark)
  %s2 = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %a)
  %s3 = call i32 @meowy_cleanup_arm_v0(ptr %frame, ptr %a, ptr %x, ptr @drop, ptr @drop_name, i64 7)
  %s4 = call i32 @meowy_cleanup_reserve_v0(ptr %frame, ptr %b)
  %s5 = call i32 @meowy_cleanup_arm_v0(ptr %frame, ptr %b, ptr %y, ptr @drop, ptr @drop_name, i64 7)
  %v0 = or i32 %s0, %s1
  %v1 = or i32 %s2, %s3
  %v2 = or i32 %s4, %s5
  %v3 = or i32 %v0, %v1
  %v4 = or i32 %v2, %v3
  %v5 = or i32 %v4, %limited
  %ready = icmp eq i32 %v5, 0
  br i1 %ready, label %run, label %invalid
run:
  %bytes = call i64 @meowy_cleanup_panic_bytes_v0()
  %panic = alloca i8, i64 %bytes, align 16
  %copy = alloca i8, i64 %bytes, align 16
  call void @meowy_panic_begin_v0(ptr %panic, i32 0)
  call void @meowy_panic_begin_v0(ptr %copy, i32 0)
  %success = call i1 @meowy_entry(ptr %panic)
  call void @meowy_panic_copy_v0(ptr %copy, ptr %panic)
  call void @meowy_panic_begin_v0(ptr %panic, i32 0)
  %reason = select i1 %success, i32 0, i32 3
  %cause = select i1 %success, ptr null, ptr %copy
  %unwound = call i32 @meowy_cleanup_unwind_v0(ptr %frame, ptr %mark, i32 %reason, ptr %cause)
  %closed = call i32 @meowy_cleanup_finish_v0(ptr %frame)
  %status = or i32 %unwound, %closed
  ret i32 %status
invalid:
  ret i32 99
}}
"#
    ));
    ir
}

#[test]
pub(crate) fn returned_panics_drive_cleanup_with_owned_original_causes() {
    let cases = [
        (
            r#"d:@"debug";f<int8>:(x<int8>){->x+1};g<int8>:(x<int8>){->f(x)};g(127);d.print("late")"#,
            "x+1",
            2,
            "int8 + overflow (left 127, right 1; range -128..127)",
        ),
        (
            r#"d:@"debug";f<int32>:(i<int32>){items:[7];->items[i]};f(2);d.print("late")"#,
            "items[i]",
            1,
            "index 2 is outside initialized length 1",
        ),
        (
            r#"d:@"debug";f<int32[1]>:(v<int32[1]>){->v.add(2)};f([1]);d.print("late")"#,
            "v.add(2)",
            3,
            "bounded list is full (length 1, capacity 1)",
        ),
        (
            r#"d:@"debug";f<never>:(){d.panic("inner")};d.panic("outer {f()} tail")"#,
            r#"d.panic("inner")"#,
            6,
            "inner",
        ),
    ];
    for (source, expression, code, message) in cases {
        let start = source.find(expression).unwrap();
        let evidence = format!("{message} at bytes {start}..{}", start + expression.len());
        let prefix = if code == 6 { "panic[P006]: outer " } else { "" };
        let stream = format!("{prefix}panic[P{code:03}]: {evidence}\n");
        for fatal in [false, true] {
            let ir = probe(source, fatal);
            for release in [false, true] {
                let output = native_ir(&ir, release, false);
                if fatal {
                    assert_eq!(output.status.signal(), Some(6));
                    assert!(output.stdout.is_empty());
                    assert_eq!(output.stderr, format!("{stream}panic[P008]: panic during cleanup\noriginal: panic P{code:03}: {evidence}\ncleanup: fixture\nsecond: P006: release failed\n").as_bytes());
                } else {
                    assert!(output.status.success());
                    assert_eq!(output.stdout, b"2\n1\n");
                    assert_eq!(output.stderr, stream.as_bytes());
                }
            }
        }
    }
}

#[test]
pub(crate) fn abandoned_pending_panics_leave_success_outcomes_empty() {
    let source =
        r#"d:@"debug";'out {d.panic("before {{'out.leave();->0}} after")};d.print("continued")"#;
    for fatal in [false, true] {
        let ir = probe(source, fatal);
        for release in [false, true] {
            let output = native_ir(&ir, release, false);
            if fatal {
                assert_eq!(output.status.signal(), Some(6));
                assert_eq!(output.stdout, b"continued\n");
                assert_eq!(output.stderr, b"panic[P006]: before panic[P008]: panic during cleanup\noriginal: complete\ncleanup: fixture\nsecond: P006: release failed\n");
            } else {
                assert!(output.status.success());
                assert_eq!(output.stdout, b"continued\n2\n1\n");
                assert_eq!(output.stderr, b"panic[P006]: before ");
            }
        }
    }
}

#[test]
pub(crate) fn streamed_panic_snapshots_keep_full_output_and_bounded_utf8_evidence() {
    let message = format!("{}🐱tail", "a".repeat(254));
    let source = format!("d:@\"debug\";d.panic(\"{message}\")");
    let start = source.find("d.panic").unwrap();
    let suffix = format!(" at bytes {start}..{}", source.len());
    let ir = probe(&source, true);
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert_eq!(output.status.signal(), Some(6));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, format!("panic[P006]: {message}{suffix}\npanic[P008]: panic during cleanup\noriginal: panic P006: {} [truncated from {} bytes]\ncleanup: fixture\nsecond: P006: release failed\n", "a".repeat(254), message.len() + suffix.len()).as_bytes());
    }
}

#[test]
pub(crate) fn function_outcomes_publish_results_only_after_success() {
    let source = r#"d:@"debug";f<int32>:(x<int32>){->x;|x==0|d.panic("late")}"#;
    for value in [0, 42] {
        let expected = if value == 0 { -99 } else { value };
        let success = value != 0;
        let ir = probe(source, false).replace(
            "%success = call i1 @meowy_entry(ptr %panic)",
            &format!(
                "%result = alloca i32, align 4\n  store i32 -99, ptr %result\n  %success = call i1 @meowy_fn_0(ptr %panic, ptr %result, i32 {value})\n  %value = load i32, ptr %result\n  %valid_result = icmp eq i32 %value, {expected}\n  %valid_state = icmp eq i1 %success, {success}\n  %valid = and i1 %valid_result, %valid_state\n  br i1 %valid, label %observed, label %invalid\nobserved:"
            ),
        );
        for release in [false, true] {
            let output = native_ir(&ir, release, false);
            assert!(output.status.success());
            assert_eq!(output.stdout, b"2\n1\n");
            if value == 0 {
                let start = source.find("d.panic").unwrap();
                assert_eq!(
                    output.stderr,
                    format!(
                        "panic[P006]: late at bytes {start}..{}\n",
                        start + "d.panic(\"late\")".len()
                    )
                    .as_bytes()
                );
            } else {
                assert!(output.stderr.is_empty());
            }
        }
    }
}

#[test]
pub(crate) fn panic_capture_formats_scalar_union_and_primary_values() {
    let source = r#"d:@"debug";u<uint64>:18446744073709551615;x<float32>:1.25;r:{->7};v<int32><null>:null;d.panic("{true}|{false}|{null}|{-1}|{u}|{x}|{r}|{v}")"#;
    let start = source.find("d.panic").unwrap();
    let evidence = format!(
        "true|false|null|-1|18446744073709551615|1.25|7|null at bytes {start}..{}",
        source.len()
    );
    let ir = probe(source, true);
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert_eq!(output.status.signal(), Some(6));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, format!("panic[P006]: {evidence}\npanic[P008]: panic during cleanup\noriginal: panic P006: {evidence}\ncleanup: fixture\nsecond: P006: release failed\n").as_bytes());
    }
}
