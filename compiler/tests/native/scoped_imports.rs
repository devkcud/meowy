use super::file_modules::case;

#[test]
pub(crate) fn scoped_imports_resolve_function_and_type_identities_without_reinitializing() {
    case("d:@\"debug\";f<int32>:(n<int32>){ops:@\"./ops.mwy\";alias:ops;x<alias.Number>:n;->alias.inc(x)};d.print(f(1));d.print(f(2));g<int32>:(n<int32>){->(@\"./ops.mwy\").inc(n)};d.print(g(3))", &[
        ("ops.mwy","d:@\"debug\";d.print(\"init\");-><Number>:<int32>;->inc<int32>:(n<int32>){->n+1}"),
    ]).runs(b"init\n2\n3\n4\n");
}

#[test]
pub(crate) fn scoped_imports_preserve_runtime_data_and_storage_capture_gates() {
    for (source, code) in [
        ("f<int32>:(){m:@\"./ops.mwy\";->m.n}", "B001"),
        ("f<int32>:(){->(@\"./ops.mwy\").n}", "B001"),
        ("f<null>:(){m:@\"./ops.mwy\";p:&(m.n)}", "B001"),
        ("f<null>:(){m:=@\"./ops.mwy\"}", "B001"),
        ("f<null>:(){m:@\"./ops.mwy\"};m.inc(1)", "E201"),
        ("f<int32>:(){m:@\"./ops.mwy\";->m.private()}", "E201"),
        ("f<null>:(){m:@\"./ops.mwy\";x<m.Private>:1}", "E202"),
    ] {
        let case = case(
            source,
            &[(
                "ops.mwy",
                "private<int32>:(){->1};<Private>:<int32>;->n:7;->inc<int32>:(n<int32>){->n+1}",
            )],
        );
        for profile in ["debug", "release"] {
            let output = case.command("check", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(&format!("\"code\":\"{code}\"")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
pub(crate) fn scoped_imports_keep_exported_reference_and_exclusive_call_rules() {
    case("d:@\"debug\";run<null>:(){ops:@\"./ops.mwy\";x:=7;p:ops.keep(&x);d.print(*p);ops.bump(&!x);d.print(x)};run()", &[
        ("ops.mwy","->keep<&int32>:(p<&int32>){->p};->bump<null>:(p<&!int32>){*p=*p+1}"),
    ]).runs(b"7\n8\n");
    let case = case(
        "run<null>:(){ops:@\"./ops.mwy\";x:=7;p:ops.keep(&x);x=8;v:*p}",
        &[("ops.mwy", "->keep<&int32>:(p<&int32>){->p}")],
    );
    let output = case.command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E302\""));
}
