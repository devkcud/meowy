use super::file_modules::case;

#[test]
pub(crate) fn composed_inputs_preserve_primary_named_records_and_aliases() {
    case(
        "m:@\"./outer.mwy\";alias:m;copy:m.width+0;row:m.row.nested;d:@\"debug\";<T>:{n<uint8>:alias;-><int32[n+copy+row.n]>};v<T>:[3,7];<Shape>:{->m<>};same<Shape>:{->m};f<int32>:(){<U>:{-><int32[m.width+m+0]>};u<U>:[9];->u[1]};d.print(v[2]);d.print(same.label);d.print(m+1);d.print(f())",
        &[
            ("data.mwy", "base<uint8>:2;->base;->width:base+1;->row:{->nested:{->n:base}};->label:\"ready\""),
            ("facade.mwy", "m:@\"./data.mwy\";alias:m;->(alias)"),
            ("outer.mwy", "m:@\"./facade.mwy\";->m"),
        ],
    ).runs(b"7\nready\n3\n9\n");
}

#[test]
pub(crate) fn composed_inputs_keep_exact_widths_privacy_and_runtime_captures() {
    for (body, code) in [
        ("<T>:{n:m+1;-><int32>}", "E107"),
        ("<T>:{n:m.width+1;-><int32>}", "E107"),
        ("<T>:{n:m.row.n+1;-><int32>}", "E107"),
        ("<T>:{n<uint16>:m;-><int32>}", "E207"),
        ("<T>:{n:m.private;-><int32>}", "E201"),
        ("<T>:{n:m;-><int32>}", "E211"),
        ("f<uint8>:(){->m+0}", "B001"),
        ("f<uint8>:(){->m.width}", "B001"),
        ("m.hidden()", "E201"),
        ("v<m.Hidden>:1", "E202"),
    ] {
        let source = format!("m:@\"./facade.mwy\";{body}");
        let output = case(&source, &[
            ("data.mwy", "private<uint8>:255;->private;->width:private;->row:{->n:private};->hidden<int32>:(){->4};-><Hidden>:<int32>"),
            ("facade.mwy", "m:@\"./data.mwy\";->m"),
        ]).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{body}: {error}"
        );
    }
}

#[test]
pub(crate) fn composed_inputs_keep_each_initializer_eligible_independently() {
    for (data, body) in [
        ("d:@\"debug\";->{->4;d.print(1)};->width:4", "n<int32>:m"),
        ("d:@\"debug\";->4;->width:{->4;d.print(1)}", "n:m.width"),
        ("d:@\"debug\";->4;->row:{->n:4;d.print(1)}", "n:m.row.n"),
        ("f<int32>:(){->4};->4;->width:f()", "n:m.width"),
    ] {
        let source = format!("m:@\"./facade.mwy\";<T>:{{{body};-><int32>}}");
        let output = case(
            &source,
            &[("data.mwy", data), ("facade.mwy", "m:@\"./data.mwy\";->m")],
        )
        .command("run", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E211\""), "{data}: {error}");
    }
}

#[test]
pub(crate) fn composed_inputs_keep_conditional_and_nonmodule_compositions_unavailable() {
    for facade in [
        "m:@\"./data.mwy\";|true|->m",
        "row:{->width:4};->row",
        "->{->width:4}",
        "m:@\"./data.mwy\";copy:{->m};->copy",
    ] {
        let output = case(
            "m:@\"./facade.mwy\";<T>:{n:m.width;-><int32>}",
            &[("data.mwy", "->width:4"), ("facade.mwy", facade)],
        )
        .command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E211\""), "{facade}: {error}");
    }
}

#[test]
pub(crate) fn composed_inputs_check_silently_and_preserve_startup_order() {
    let case = case(
        "a:@\"./a.mwy\";b:@\"./b.mwy\";d:@\"debug\";<T>:{n<uint8>:a;-><int32[n+b.width]>};v<T>:[7];d.print(\"entry\");d.print(v[1]);d.print(a.label)",
        &[
            (
                "data.mwy",
                "d:@\"debug\";d.print(\"data\");base<uint8>:4;->base;->width:base;->label:{d.print(\"label\");->\"ready\"}",
            ),
            (
                "a.mwy",
                "m:@\"./data.mwy\";d:@\"debug\";d.print(\"a before\");->m;d.print(\"a after\")",
            ),
            ("b.mwy", "m:@\"./data.mwy\";d:@\"debug\";->m;d.print(\"b\")"),
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
    case.runs(b"data\nlabel\na before\na after\nb\nentry\n7\nready\n");
}

#[test]
pub(crate) fn composed_inputs_charge_transitive_work_for_primary_fields_and_subrecords() {
    let tail = (1..40)
        .map(|id| format!("v{id}:v{}+1;", id - 1))
        .collect::<String>();
    let integer = format!("{{->4;v0:1;{tail}}}");
    let data = format!("->{integer};->width:{integer};->row:{{->nested:{{->n:4}};v0:1;{tail}}}");
    let files = [
        ("data.mwy", data.as_str()),
        ("facade.mwy", "m:@\"./data.mwy\";->m"),
        ("outer.mwy", "m:@\"./facade.mwy\";->m"),
    ];
    for value in ["m+0", "m.width", "m.row.nested.n", "part.n"] {
        let one = format!(
            "m:@\"./outer.mwy\";part:m.row.nested;<T>:{{n:{value};-><int32[n]>}};<U>:{{n:{value};-><int32[n]>}}"
        );
        let output = case(&one, &files).command("check", &["--json"]);
        assert!(
            output.status.success(),
            "{value}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let two =
            format!("m:@\"./outer.mwy\";part:m.row.nested;<T>:{{a:{value};b:{value};-><int32>}}");
        let output = case(&two, &files).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"B001\""), "{error}");
        assert!(error.contains("computed type bootstrap budget"), "{error}");
    }
}

#[test]
pub(crate) fn composed_inputs_cannot_hide_ancestor_effects_behind_projected_reexports() {
    let case = case(
        "m:@\"./outer.mwy\";copy:m.part;<T>:{n:copy.n;-><int32>}",
        &[
            (
                "data.mwy",
                "d:@\"debug\";->row:{->nested:{->n:4};->sibling:{->n:1;d.print(\"must not run\")}}",
            ),
            ("facade.mwy", "m:@\"./data.mwy\";->m"),
            ("part.mwy", "m:@\"./facade.mwy\";->part:m.row.nested"),
            ("outer.mwy", "m:@\"./part.mwy\";->m"),
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
pub(crate) fn composed_inputs_preserve_original_dependency_error_spans() {
    let data = "#é🙂#\n->row:{->nested:{->n:4};->bad<uint8>:255+1}";
    let case = case(
        "m:@\"./facade.mwy\";part:m.row.nested;<T>:{n:part.n;-><int32>}",
        &[("data.mwy", data), ("facade.mwy", "m:@\"./data.mwy\";->m")],
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
}

#[test]
pub(crate) fn composed_inputs_keep_runtime_failure_before_facade_execution() {
    let case = case(
        "m:@\"./facade.mwy\";<T>:{n<int32>:m;-><int32[n+m.width]>};d:@\"debug\";d.print(\"entry must not run\")",
        &[
            (
                "data.mwy",
                "d:@\"debug\";->4;->width:2;stop<boolean>:(){->true};d.print(\"init\");|stop()|d.panic(\"stop\")",
            ),
            (
                "facade.mwy",
                "m:@\"./data.mwy\";d:@\"debug\";->m;d.print(\"facade must not run\")",
            ),
        ],
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

#[test]
pub(crate) fn local_composed_inputs_keep_values_shapes_and_function_type_reads() {
    super::Case::new(
        "d:@\"debug\";row:{->z<uint8>:2;->a:{->n<uint8>:3}};copy:{->row;->extra<uint8>:1};part:{->(copy.a)};value:copy.z;<T>:{-><int32[value+part.n+copy.extra]>};v<T>:[7];f<int32>:(){<U>:{-><int32[copy.a.n]>};v<U>:[9];->v[1]};d.print(v[1]);d.print(f());d.print(copy.z);d.print(part.n)",
    ).runs(b"7\n9\n2\n3\n");
}

#[test]
pub(crate) fn local_composed_inputs_keep_ancestor_effects_shapes_and_widths() {
    for (source, code) in [
        (
            "d:@\"debug\";row:{->a:{->n:4};->b:{->n:1;d.print(1)}};copy:{->row.a};<T>:{n:copy.n;-><int32>}",
            "E211",
        ),
        (
            "row:{->n:4};copy:{->row;unused:=1};<T>:{n:copy.n;-><int32>}",
            "E211",
        ),
        ("row:{->n:=4};copy:{->row};<T>:{n:copy.n;-><int32>}", "E211"),
        ("row:={->n:4};copy:{->row};<T>:{n:copy.n;-><int32>}", "E211"),
        (
            "row:{->1;->n:4};copy:{->row};<T>:{n:copy.n;-><int32>}",
            "E211",
        ),
        (
            "row:{->n:4;->label:\"x\"};copy:{->row};<T>:{n:copy.n;-><int32>}",
            "E211",
        ),
        (
            "row:{->n:4};copy:{|true|->row};<T>:{n:copy.n;-><int32>}",
            "E211",
        ),
        (
            "row:{->n<uint8>:255};copy:{->row};<T>:{n:copy.n+1;-><int32>}",
            "E107",
        ),
        ("row:{->n:4};copy:{->row};f<int32>:(){->copy.n}", "B001"),
    ] {
        let output = super::Case::new(source).command("run", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}

#[test]
pub(crate) fn local_composed_inputs_retain_unreachable_sibling_error_spans() {
    for body in [
        "row<{good<uint8>;bad<uint8>}>:{->good<uint8>:4;->bad<uint8>:255+1};copy<{good<uint8>;bad<uint8>}>:{->row}",
        "row<{good<uint8>}>:{->good<uint8>:4;unused<uint8>:255+1};copy<{good<uint8>}>:{->row}",
    ] {
        let source = format!("|false|{{{body};<T>:{{n:copy.good;-><int32>}}}}");
        let error = meowy::compile(&source).unwrap_err().remove(0);
        assert_eq!(error.code, "E107", "{source}: {error:?}");
        assert_eq!(error.span.start, source.find("255+1").unwrap());
    }
}
