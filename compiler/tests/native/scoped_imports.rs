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

#[test]
pub(crate) fn scoped_imports_example_initializes_unused_and_inactive_dependencies() {
    case(
        include_str!("../../examples/scoped-imports/main.mwy"),
        &[
            (
                "side.mwy",
                include_str!("../../examples/scoped-imports/side.mwy"),
            ),
            (
                "ops.mwy",
                include_str!("../../examples/scoped-imports/ops.mwy"),
            ),
        ],
    )
    .runs(b"side\nops\nentry\n8\nagain 9\n");
}

#[test]
pub(crate) fn scoped_imports_report_missing_paths_and_nested_cycles_at_the_import() {
    for source in [
        "unused<null>:(){m:@\"./missing.mwy\"}",
        "|false|{m:@\"./missing.mwy\"}",
        "<Items>:<int32[(@\"./missing.mwy\").n]>",
    ] {
        let case = case(source, &[]);
        let output = case.command("check", &["--json"]);
        let error = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1));
        assert!(error.contains("\"code\":\"E501\""), "{error}");
        assert!(
            error.contains(&format!("\"start\":{}", source.find('@').unwrap())),
            "{error}"
        );
    }
    let closing = "|false|{a:@\"./a.mwy\"}";
    let case = case(
        "m:@\"./a.mwy\"",
        &[
            ("a.mwy", "unused<null>:(){b:@\"./b.mwy\"}"),
            ("b.mwy", closing),
        ],
    );
    let output = case.command("check", &["--json"]);
    let error = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(error.contains("\"code\":\"E502\""), "{error}");
    assert!(
        error.contains(&format!(
            "\"path\":\"{}\"",
            case.path.join("b.mwy").display()
        )),
        "{error}"
    );
    assert!(
        error.contains(&format!("\"start\":{}", closing.find('@').unwrap())),
        "{error}"
    );
}

#[test]
pub(crate) fn scoped_imports_preserve_initializer_failure_and_ignore_literal_text() {
    let source = "d:@\"debug\";d.panic(\"startup\")";
    let case = case(
        "d:@\"debug\";unused<null>:(){m:@\"./bad.mwy\"};d.print(\"never\")",
        &[("bad.mwy", source)],
    );
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: startup at \"{}\" bytes 11..{}\n",
                case.path.join("bad.mwy").display(),
                source.len()
            )
            .as_bytes()
        );
    }
    super::Case::new("d:@\"debug\";# @\"./missing.mwy\" #d.print(\"@\\\"./missing.mwy\\\"\")")
        .runs(b"@\"./missing.mwy\"\n");
}

#[cfg(unix)]
#[test]
pub(crate) fn scoped_imports_protect_dependencies_used_only_inside_functions() {
    let source = "->n:7";
    let case = case(
        "unused<null>:(){m:@\"./value.mwy\"}",
        &[("value.mwy", source)],
    );
    let output = case.path.join("artifact");
    std::fs::hard_link(case.path.join("value.mwy"), &output).unwrap();
    let result = case.command("build", &["--output", output.to_str().unwrap()]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("hard link to the source"));
    assert_eq!(std::fs::read_to_string(output).unwrap(), source);
}
