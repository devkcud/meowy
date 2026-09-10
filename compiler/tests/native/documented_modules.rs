use super::file_modules::case;

#[test]
pub(crate) fn documented_modules_example_runs_with_checked_facade_links() {
    case(
        include_str!("../../examples/documented-modules/main.mwy"),
        &[
            (
                "facade.mwy",
                include_str!("../../examples/documented-modules/facade.mwy"),
            ),
            (
                "counter.mwy",
                include_str!("../../examples/documented-modules/counter.mwy"),
            ),
        ],
    )
    .runs(b"counter initialized\n8\n");
}

#[test]
pub(crate) fn documented_modules_report_local_diagnostics_for_check_build_and_run() {
    for (source, code, token) in [
        ("#| Orphan. |#->1", "E801", "#|"),
        ("#||\r\n É [[missing]].\r\n||#x:1", "E802", "[[missing]]"),
        (
            "#||\n```meowy mystery\nx:1\n```\n||#x:1",
            "E803",
            "```meowy",
        ),
        (
            "#| [[x]] |#->f<int32>:(#| [[missing]] |#x<int32>){->x}",
            "E802",
            "[[missing]]",
        ),
    ] {
        let case = case("m:@\"./value.mwy\"", &[("value.mwy", source)]);
        for action in ["check", "build", "run"] {
            let output = case.command(action, &["--json"]);
            assert_eq!(output.status.code(), Some(1), "{action}");
            assert!(output.stdout.is_empty());
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
                error.contains(&format!("\"start\":{}", source.find(token).unwrap())),
                "{error}"
            );
        }
    }
}

#[test]
pub(crate) fn documented_modules_check_unused_files_and_keep_budget_failures_explicit() {
    for (source, code) in [
        ("#!| [[missing]] |!#->n:1".into(), "E802"),
        (format!("#|{}|#->n:1", "x".repeat(1024 * 1024)), "B001"),
    ] {
        let case = case(
            "unused<null>:(){m:@\"./value.mwy\"}",
            &[("value.mwy", &source)],
        );
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
    }
}

#[test]
pub(crate) fn documented_modules_keep_examples_and_initializers_out_of_check_and_build() {
    let case = case(
        "m:@\"./value.mwy\";d:@\"debug\";d.print(m.n)",
        &[(
            "value.mwy",
            "#||\n```meowy run\nm:@\"./missing-example.mwy\";d:@\"debug\";d.panic(\"example\")\n```\n```output\nunused\n```\n||#->n:7;d:@\"debug\";d.print(\"init\")",
        )],
    );
    for action in ["check", "build"] {
        let output = case.command(action, &[]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
    }
    case.runs(b"init\n7\n");
}

#[test]
pub(crate) fn documented_modules_preserve_signature_capture_and_storage_gates() {
    for (source, module, code) in [
        (
            "m:@\"./value.mwy\";x:m.hidden",
            "#| Private. |#hidden:1;->n:2",
            "E201",
        ),
        (
            "m:@\"./value.mwy\";f<int32>:(){->m.n}",
            "#| Public. |#->n:2",
            "B001",
        ),
        ("m:@\"./value.mwy\"", "x:1;#| Shared. |#->ref:&x", "B001"),
        (
            "m:@\"./value.mwy\"",
            "#| Function. |#->f<int32>:(){->true}",
            "E207",
        ),
    ] {
        let case = case(source, &[("value.mwy", module)]);
        let output = case.command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(&format!("\"code\":\"{code}\"")), "{error}");
    }
}

#[test]
pub(crate) fn documented_modules_keep_site_generation_explicitly_unsupported() {
    let case = case(
        "#!| Module. |!#m:@\"./value.mwy\"",
        &[("value.mwy", "#| Value. |#->n:1")],
    );
    for action in ["check", "build"] {
        let output_path = case.path.join("site");
        let mut args = vec!["--standalone", "--json"];
        if action == "build" {
            args.extend(["--output", output_path.to_str().unwrap()]);
        }
        let output = super::documentation::doc(&case, action, &args);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"B001\""), "{error}");
    }
}
