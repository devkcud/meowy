use super::file_modules::case;

#[test]
pub(crate) fn imported_inputs_check_without_running_diamond_initializers() {
    let case = case(
        "d:@\"debug\";a:@\"./a.mwy\";b:@\"./b.mwy\";<T>:{n:a.width+b.row.n;-><int32[n]>};v<T>:[7];d.print(\"entry\");d.print(v[1])",
        &[
            (
                "data.mwy",
                "d:@\"debug\";d.print(\"data\");->width<uint8>:2;->row:{->n:width}",
            ),
            (
                "a.mwy",
                "d:@\"debug\";m:@\"./data.mwy\";->width:m.width;d.print(\"a\")",
            ),
            (
                "b.mwy",
                "d:@\"debug\";m:@\"./data.mwy\";->row:m.row;d.print(\"b\")",
            ),
        ],
    );
    for action in ["check", "build"] {
        for profile in ["debug", "release"] {
            let output = case.command(action, &["--profile", profile]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.is_empty());
            assert!(output.stderr.is_empty());
        }
    }
    case.runs(b"data\na\nb\nentry\n7\n");
}

#[test]
pub(crate) fn imported_inputs_refuse_transitive_ancestor_effects_without_execution() {
    for export in ["->row:m.row", "part:m.row.nested;->row:part"] {
        let source = if export.contains("part:") {
            "m:@\"./facade.mwy\";<T>:{n:m.row.n;-><int32>}"
        } else {
            "m:@\"./facade.mwy\";part:m.row.nested;copy:part.n;<T>:{n:copy;-><int32>}"
        };
        let facade = format!("m:@\"./data.mwy\";{export}");
        let case = case(
            source,
            &[
                (
                    "data.mwy",
                    "d:@\"debug\";d.print(\"must not run\");->row:{->nested:{->n:4};d.print(\"ancestor effect\")}",
                ),
                ("facade.mwy", &facade),
            ],
        );
        for action in ["check", "build", "run"] {
            let output = case.command(action, &["--json"]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(error.contains("\"code\":\"E211\""), "{error}");
        }
    }
}

#[test]
pub(crate) fn imported_inputs_keep_dependency_error_spans_and_integer_widths() {
    let data = "#é🙂#\n->row:{->nested:{->n<uint8>:4};->bad<uint8>:255+1}";
    let case = case(
        "m:@\"./data.mwy\";<T>:{n:m.row.nested.n;-><int32>}",
        &[("data.mwy", data)],
    );
    for action in ["check", "build", "run"] {
        let output = case.command(action, &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E107\""), "{error}");
        assert!(
            error.contains(&format!(
                "\"path\":\"{}\"",
                case.path.join("data.mwy").display()
            )),
            "{error}"
        );
        assert!(
            error.contains(&format!("\"start\":{}", data.find("255+1").unwrap())),
            "{error}"
        );
    }
    let source = "m:@\"./width.mwy\";<T>:{n:m.width+1;-><int32>}";
    let output = super::file_modules::case(source, &[("width.mwy", "->width<uint8>:255")])
        .command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("\"code\":\"E107\""), "{error}");
    assert!(
        error.contains(&format!("\"start\":{}", source.find("m.width+1").unwrap())),
        "{error}"
    );
}

#[test]
pub(crate) fn imported_inputs_charge_ancestor_work_again_after_reexport_and_projection() {
    let tail = (1..40)
        .map(|id| format!("v{id}:v{}+1;", id - 1))
        .collect::<String>();
    let data = format!("->row:{{->nested:{{->n:4}};v0:1;{tail}}}");
    let files = [
        ("data.mwy", data.as_str()),
        ("facade.mwy", "m:@\"./data.mwy\";->row:m.row.nested"),
    ];
    let one = case(
        "m:@\"./facade.mwy\";<T>:{n:m.row.n;-><int32[n]>};v<T>:[1]",
        &files,
    )
    .command("check", &["--json"]);
    assert!(
        one.status.success(),
        "{}",
        String::from_utf8_lossy(&one.stderr)
    );
    let output = case(
        "m:@\"./facade.mwy\";copy:m.row;<T>:{a:copy.n;b:copy.n;-><int32>}",
        &files,
    )
    .command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("\"code\":\"B001\""), "{error}");
    assert!(error.contains("computed type bootstrap budget"), "{error}");
}
