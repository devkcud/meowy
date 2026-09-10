use super::{emit_ir, native_ir};
use std::os::unix::process::ExitStatusExt;

pub(crate) const DECLS: &str = "
@file = private constant [10 x i8] c\"a\\22\\5C\\0A\\C3\\A9.mwy\"
declare void @meowy_panic_site_file_v0(ptr, i64, i64, ptr, i64)
declare void @meowy_arithmetic_capture_file_v0(ptr, i32, i32, i32, i64, i64, i64, i64, ptr, i64)
declare void @meowy_index_capture_file_v0(ptr, i64, i64, i32, i64, i64, ptr, i64)
declare void @meowy_list_capture_file_v0(ptr, i64, i64, i64, i64, ptr, i64)
";
pub(crate) const SHOWN: &str = "a\\\"\\\\\\u000aé.mwy";

#[test]
pub(crate) fn runtime_file_sites_preserve_legacy_output_and_escape_labels() {
    let calls = [
        (
            "meowy_arithmetic_capture",
            "i32 43, i32 8, i32 1, i64 127, i64 1",
            "panic[P002]: int8 + overflow (left 127, right 1; range -128..127)",
        ),
        (
            "meowy_index_capture",
            "i64 2, i64 1, i32 1",
            "panic[P001]: index 2 is outside initialized length 1",
        ),
        (
            "meowy_list_capture",
            "i64 1, i64 1",
            "panic[P003]: bounded list is full (length 1, capacity 1)",
        ),
        ("meowy_panic_site", "", ""),
    ];
    for (name, args, prefix) in calls {
        for mapped in [false, true] {
            let args = if args.is_empty() {
                String::new()
            } else {
                format!(", {args}")
            };
            let suffix = if mapped { "_file_v0" } else { "_v0" };
            let file = if mapped { ", ptr @file, i64 10" } else { "" };
            let mut ir = emit_ir(&crate::compile("").unwrap())
                .unwrap()
                .replace("define i32 @main()", "define internal i32 @unused_main()");
            ir.push_str(DECLS);
            ir.push_str(&format!("\ndefine i32 @main() {{\n %size = call i64 @meowy_cleanup_panic_bytes_v0()\n %panic = alloca i8, i64 %size, align 16\n call void @meowy_panic_begin_v0(ptr %panic, i32 6)\n call void @{name}{suffix}(ptr %panic{args}, i64 12, i64 34{file})\n ret i32 0\n}}\n"));
            let site = if mapped {
                format!(" at \"{SHOWN}\" bytes 12..34\n")
            } else {
                " at bytes 12..34\n".into()
            };
            for release in [false, true] {
                let output = native_ir(&ir, release, false);
                assert!(output.status.success());
                assert!(output.stdout.is_empty());
                assert_eq!(output.stderr, format!("{prefix}{site}").as_bytes());
            }
        }
    }
}

#[test]
pub(crate) fn runtime_file_sites_survive_panic_copy_and_cleanup_failure() {
    let mut ir = super::panic_outcomes::probe("", true).replace(
        "%success = call i1 @meowy_entry(ptr %panic)",
        "call void @meowy_index_capture_file_v0(ptr %panic, i64 2, i64 1, i32 1, i64 12, i64 34, ptr @file, i64 10)\n  %success = icmp eq i32 0, 1",
    );
    ir.push_str(DECLS);
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert_eq!(output.status.signal(), Some(6));
        let site = format!("index 2 is outside initialized length 1 at \"{SHOWN}\" bytes 12..34");
        assert_eq!(output.stderr, format!("panic[P001]: {site}\npanic[P008]: panic during cleanup\noriginal: panic P001: {site}\ncleanup: fixture\nsecond: P006: release failed\n").as_bytes());
    }
}
