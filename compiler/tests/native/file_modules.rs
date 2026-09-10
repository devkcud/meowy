use super::{Case, fs};

pub(crate) fn case(source: &str, files: &[(&str, &str)]) -> Case {
    let case = Case::new(source);
    for (name, source) in files {
        let path = case.path.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    case
}

#[test]
pub(crate) fn file_modules_initialize_diamonds_once_before_entry_in_source_order() {
    case(
        "d:@\"debug\";d.print(\"entry\");a:@\"./a.mwy\";b:@\"./b.mwy\";d.print(a.n+b.n)",
        &[
            (
                "a.mwy",
                "d:@\"debug\";s:@\"./shared.mwy\";d.print(\"a\");->n:s.n+1",
            ),
            (
                "b.mwy",
                "d:@\"debug\";s:@\"./shared.mwy\";d.print(\"b\");->n:s.n+2",
            ),
            ("shared.mwy", "d:@\"debug\";d.print(\"shared\");->n:7"),
        ],
    )
    .runs(b"shared\na\nb\nentry\n17\n");
}

#[test]
pub(crate) fn file_modules_read_immutable_records_lists_strings_and_primary_exports() {
    case("d:@\"debug\";m:@\"./value.mwy\";alias:m;s:@\"./scalar.mwy\";d.print(alias.n);d.print(m.items[2]);d.print(m.row.label);d.print(s+1)", &[
        ("value.mwy", "private:99;->n:7;->items:[1,2];->row:{->label:\"ready\"}"),
        ("scalar.mwy", "->8"),
    ]).runs(b"7\n2\nready\n9\n");
}

#[test]
pub(crate) fn file_modules_map_dependency_parse_and_semantic_diagnostics() {
    for (module, code, text) in [
        ("#é🙂#\n->n:missing", "E201", "missing"),
        ("->n:)", "E002", ")"),
    ] {
        let case = case("m:@\"./value.mwy\"", &[("value.mwy", module)]);
        let output = case.command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(&format!("\"code\":\"{code}\"")), "{error}");
        assert!(
            error.contains(&format!(
                "\"path\":\"{}\"",
                case.path.join("value.mwy").display()
            )),
            "{error}"
        );
        assert!(
            error.contains(&format!("\"start\":{}", module.find(text).unwrap())),
            "{error}"
        );
    }
}

#[test]
pub(crate) fn file_modules_stop_entry_after_initialization_panic() {
    let case = case(
        "d:@\"debug\";d.print(\"never\");m:@\"./value.mwy\"",
        &[(
            "value.mwy",
            "d:@\"debug\";d.print(\"init\");d.panic(\"stop\")",
        )],
    );
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stdout,
            b"init\n",
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
    }
}

#[test]
pub(crate) fn file_modules_protect_all_source_files_from_output_replacement() {
    let module = "->n:7";
    let case = case("m:@\"./value.mwy\"", &[("value.mwy", module)]);
    for flag in ["--output", "--emit-llvm"] {
        let output = case.command(
            "build",
            &[flag, case.path.join("value.mwy").to_str().unwrap()],
        );
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            fs::read_to_string(case.path.join("value.mwy")).unwrap(),
            module
        );
    }
    #[cfg(unix)]
    {
        let alias = case.path.join("artifact");
        fs::hard_link(case.path.join("value.mwy"), &alias).unwrap();
        for flag in ["--output", "--emit-llvm"] {
            let output = case.command("build", &[flag, alias.to_str().unwrap()]);
            assert_eq!(output.status.code(), Some(2));
            assert!(String::from_utf8_lossy(&output.stderr).contains("hard link to the source"));
            assert_eq!(fs::read_to_string(&alias).unwrap(), module);
        }
    }
}

#[test]
pub(crate) fn file_modules_resolve_from_each_importer_and_keep_interpolated_spans() {
    case(
        "d:@\"debug\";m:@\"./sub/value.mwy\";d.print(m.n)",
        &[
            (
                "sub/value.mwy",
                "d:@\"debug\";s:@\"../shared.mwy\";d.print(\"value {s.n}\");->n:s.n+1",
            ),
            ("shared.mwy", "->n:7"),
        ],
    )
    .runs(b"value 7\n8\n");
    let module = "#é🙂#\nd:@\"debug\";d.print(\"value {missing}\");->n:1";
    let case = case("m:@\"./value.mwy\"", &[("value.mwy", module)]);
    let output = case.command("check", &["--json"]);
    let error = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(error.contains("\"code\":\"E201\""), "{error}");
    assert!(
        error.contains(&format!("\"start\":{}", module.find("missing").unwrap())),
        "{error}"
    );
    assert!(error.contains("value.mwy"));
}

#[test]
pub(crate) fn file_modules_keep_manifest_and_unsupported_context_gates() {
    for (source, files, code) in [
        (
            "m:@\"./pkg/value.mwy\"",
            vec![("pkg/mod.mwy", ""), ("pkg/value.mwy", "->n:1")],
            "B001",
        ),
        ("m:@\"./mod.mwy\"", vec![("mod.mwy", "")], "B001"),
        ("m:@\"value\"", vec![("value.mwy", "->n:1")], "B001"),
        (
            "'scope{m:@\"./value.mwy\"}",
            vec![("value.mwy", "->n:1")],
            "B001",
        ),
        (
            "m:@\"./value.mwy\";x:m.private",
            vec![("value.mwy", "private:1;->n:2")],
            "E201",
        ),
        ("m:@\"./value.mwy\"", vec![("value.mwy", "->n:=1")], "B001"),
        (
            "m:@\"./value.mwy\";f<int32>:(){->m.n}",
            vec![("value.mwy", "->n:1")],
            "B001",
        ),
    ] {
        let case = case(source, &files);
        for profile in ["debug", "release"] {
            let output = case.command("build", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(&format!("\"code\":\"{code}\"")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[cfg(unix)]
#[test]
pub(crate) fn file_modules_initialize_symlink_aliases_once() {
    let case = case(
        "d:@\"debug\";a:@\"./value.mwy\";b:@\"./alias.mwy\";d.print(a.n+b.n)",
        &[("value.mwy", "d:@\"debug\";d.print(\"init\");->n:7")],
    );
    std::os::unix::fs::symlink("value.mwy", case.path.join("alias.mwy")).unwrap();
    case.runs(b"init\n14\n");
}

#[test]
pub(crate) fn file_modules_example_executes_in_both_profiles() {
    case(
        include_str!("../../examples/modules/main.mwy"),
        &[
            ("left.mwy", include_str!("../../examples/modules/left.mwy")),
            (
                "right.mwy",
                include_str!("../../examples/modules/right.mwy"),
            ),
            (
                "shared.mwy",
                include_str!("../../examples/modules/shared.mwy"),
            ),
        ],
    )
    .runs(b"shared\nleft\nright\nentry\n17\n");
}

#[test]
pub(crate) fn file_function_exports_call_annotated_definitions_and_private_helpers() {
    case("d:@\"debug\";m:@\"./ops.mwy\";->twice<int32>:(x<int32>){->m.inc(m.inc(x))};d.print(twice(1));alias:m.inc;d.print(alias(4))", &[
        ("ops.mwy", "helper<int32>:(x<int32>){->x+1};->inc<int32>:(x<int32>){->helper(x)}"),
    ]).runs(b"3\n5\n");
    Case::new("d:@\"debug\";->inc<int32>:(x<int32>){->x+1};d.print(inc(7))").runs(b"8\n");
}

#[test]
pub(crate) fn file_function_exports_preserve_privacy_annotations_and_capture_gates() {
    for (entry, source, code) in [
        ("m:@\"./ops.mwy\"", "->inc:(x<int32>){->x+1}", "E214"),
        (
            "m:@\"./ops.mwy\";m.helper(1)",
            "helper<int32>:(x<int32>){->x};->inc<int32>:(x<int32>){->helper(x)}",
            "E201",
        ),
        ("m:@\"./ops.mwy\"", "x:=1;->inc<int32>:(){->x}", "B001"),
        (
            "m:@\"./ops.mwy\";m.inc(true)",
            "->inc<int32>:(x<int32>){->x+1}",
            "E212",
        ),
        ("m:@\"./ops.mwy\"", "->inc<int32>:(){->true}", "E207"),
        ("m:@\"./ops.mwy\"", "->inc<int32>:(){->1};->inc:2", "E205"),
        ("m:@\"./ops.mwy\"", "->inc:2;->inc<int32>:(){->1}", "E205"),
        (
            "m:@\"./ops.mwy\"",
            "->inc<int32>:(){->1};->inc<int32>:(){->2}",
            "E205",
        ),
    ] {
        let case = case(entry, &[("ops.mwy", source)]);
        for profile in ["debug", "release"] {
            let output = case.command("check", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{entry}: {source}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(&format!("\"code\":\"{code}\"")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
