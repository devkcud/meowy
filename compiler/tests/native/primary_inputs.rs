use super::file_modules::case;

#[test]
pub(crate) fn primary_inputs_support_aliases_copies_reexports_and_function_types() {
    case(
        "m:@\"./primary.mwy\";alias:m;copy:alias+0;f:@\"./facade.mwy\";d:@\"debug\";<T>:{n:alias;-><int32[n]>};v<T>:[3,7];<U>:{n:copy+f.width;-><int32[n]>};u<U>:[9];d.print(v[2]);d.print(u[1]);d.print(f.get());d.print(m+1)",
        &[
            ("data.mwy", "base<uint8>:2;->{->base*2;unused:7}"),
            ("primary.mwy", "m:@\"./data.mwy\";->m"),
            ("facade.mwy", "m:@\"./primary.mwy\";->width:m;->get<int32>:(){<T>:{n:m;-><int32[n]>};v<T>:[8];->v[1]}"),
        ],
    ).runs(b"7\n9\n8\n5\n");
}

#[test]
pub(crate) fn primary_inputs_keep_exact_widths_in_required_arithmetic() {
    for value in ["m", "alias", "copy"] {
        let source = format!("m:@\"./data.mwy\";alias:m;copy:m+0;<T>:{{n:{value}+1;-><int32>}}");
        let output = case(&source, &[("data.mwy", "value<uint8>:255;->value")])
            .command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E107\""), "{error}");
    }
}

#[test]
pub(crate) fn primary_inputs_reject_unproven_initializers_without_execution() {
    for data in [
        "d:@\"debug\";->{->4;d.print(\"must not run\")}",
        "value:=4;->value",
        "|true|->4",
        "f<int32>:(){->4};->f()",
    ] {
        let output = case(
            "m:@\"./data.mwy\";<T>:{n:m;-><int32>}",
            &[("data.mwy", data)],
        )
        .command("run", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E211\""), "{data}: {error}");
    }
}

#[test]
pub(crate) fn primary_inputs_keep_module_records_and_runtime_captures_gated() {
    for (source, data, code) in [
        ("m:@\"./data.mwy\";f<int32>:(){->m+1}", "->4", "B001"),
        (
            "m:@\"./data.mwy\";copy:m+0;f<int32>:(){->copy}",
            "->4",
            "B001",
        ),
        (
            "m:@\"./data.mwy\";<T>:{n:m+0;-><int32>}",
            "->4;->width:2",
            "B001",
        ),
        ("row:{->width:4};f<int32>:(){->row.width}", "->4", "B001"),
    ] {
        let output = case(source, &[("data.mwy", data)]).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}

#[test]
pub(crate) fn primary_inputs_check_silently_and_initialize_shared_dependencies_once() {
    let case = case(
        "a:@\"./a.mwy\";b:@\"./b.mwy\";d:@\"debug\";<T>:{-><int32[a+b.width]>};v<T>:[7];f<int32>:(){m:@\"./data.mwy\";<U>:{-><int32[m]>};u<U>:[9];->u[1]};d.print(\"entry\");d.print(v[1]);d.print(f())",
        &[
            (
                "data.mwy",
                "d:@\"debug\";d.print(\"data\");value<uint8>:2;->value;d.print(\"after\")",
            ),
            (
                "a.mwy",
                "m:@\"./data.mwy\";d:@\"debug\";d.print(\"a\");->m+1",
            ),
            (
                "b.mwy",
                "m:@\"./data.mwy\";d:@\"debug\";d.print(\"b\");->width:m",
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
    case.runs(b"data\nafter\na\nb\nentry\n7\n9\n");
}

#[test]
pub(crate) fn primary_inputs_cannot_hide_effects_behind_copies_and_reexports() {
    let case = case(
        "m:@\"./facade.mwy\";alias:m;copy:alias+0;<T>:{n:copy;-><int32>}",
        &[
            (
                "data.mwy",
                "d:@\"debug\";d.print(\"must not run\");->{->4;d.print(\"initializer effect\")}",
            ),
            ("facade.mwy", "m:@\"./data.mwy\";copy:m+0;->copy"),
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

#[test]
pub(crate) fn primary_inputs_preserve_original_dependency_failure_spans() {
    let data = "#é🙂#\nbase<uint8>:255;->{->4;unused<uint8>:base+1}";
    let case = case(
        "m:@\"./data.mwy\";<T>:{n:m;-><int32>}",
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
            error.contains(&format!("\"start\":{}", data.find("base+1").unwrap())),
            "{error}"
        );
    }
}

#[test]
pub(crate) fn primary_inputs_charge_transitive_tail_work_on_every_required_read() {
    let tail = (1..40)
        .map(|id| format!("v{id}:v{}+1;", id - 1))
        .collect::<String>();
    let data = format!("->{{->4;v0:1;{tail}}}");
    let files = [
        ("data.mwy", data.as_str()),
        ("facade.mwy", "m:@\"./data.mwy\";->m"),
    ];
    let one = case("m:@\"./facade.mwy\";<T>:{-><int32[m]>};v<T>:[1]", &files)
        .command("check", &["--json"]);
    assert!(
        one.status.success(),
        "{}",
        String::from_utf8_lossy(&one.stderr)
    );
    for values in ["a:m;b:m", "a:copy;b:copy"] {
        let source = format!("m:@\"./facade.mwy\";copy:m+0;<T>:{{{values};-><int32>}}");
        let output = case(&source, &files).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"B001\""), "{error}");
        assert!(error.contains("computed type bootstrap budget"), "{error}");
    }
}

#[test]
pub(crate) fn primary_inputs_do_not_remove_runtime_initializer_failure() {
    let case = case(
        "m:@\"./data.mwy\";<T>:{n:m;-><int32[n]>};d:@\"debug\";d.print(\"entry must not run\")",
        &[(
            "data.mwy",
            "d:@\"debug\";stop<boolean>:(){->true};d.print(\"init\");|stop()|d.panic(\"stop\");->4",
        )],
    );
    for profile in ["debug", "release"] {
        let checked = case.command("check", &["--profile", profile]);
        assert!(
            checked.status.success(),
            "{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        assert!(checked.stdout.is_empty());
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"init\n");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("panic[P006]: stop"));
    }
}
