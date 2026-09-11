use super::{Case, file_modules::case};

#[test]
pub(crate) fn computed_fields_keep_local_module_values_types_and_initialization_order() {
    case(
        "m:@\"./types.mwy\";d:@\"debug\";items<m.Items>:[3,7];d.print(items[2]);d.print(m.get())",
        &[("types.mwy", "d:@\"debug\";d.print(\"init\");settings:{->other<uint8>:2;->width<uint8>:other*2};alias:settings;-><Items>:{-><int32[alias.width]>};->get<int32>:(){<Local>:{n:settings.width;-><int32[n]>};items<Local>:[4,8];->items[2]}")],
    ).runs(b"init\n7\n8\n");
}

#[test]
pub(crate) fn computed_fields_preserve_errors_in_unselected_siblings() {
    let source = "#é#\n|false|{row<{good<uint8>;bad<uint8>}>:{->good<uint8>:4;->bad<uint8>:255+1};<T>:{n:row.good;-><int32>}}";
    let case = case("m:@\"./types.mwy\"", &[("types.mwy", source)]);
    for action in ["check", "build", "run"] {
        let output = case.command(action, &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E107\""), "{error}");
        assert!(
            error.contains(&format!(
                "\"path\":\"{}\"",
                case.path.join("types.mwy").display()
            )),
            "{error}"
        );
        assert!(
            error.contains(&format!("\"start\":{}", source.find("255+1").unwrap())),
            "{error}"
        );
    }
}

#[test]
pub(crate) fn computed_fields_do_not_execute_effectful_record_initializers() {
    let source = "d:@\"debug\";settings:{->width:4;d.print(\"must not run\")};<T>:{n:settings.width;-><int32[n]>}";
    let case = Case::new(source);
    for action in ["check", "build", "run"] {
        let output = case.command(action, &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E211\""));
    }
}

#[test]
pub(crate) fn computed_fields_keep_imported_data_and_runtime_captures_gated() {
    for (source, code) in [
        ("m:@\"./data.mwy\";<T>:{n:m.width;-><int32>}", "B001"),
        (
            "m:@\"./data.mwy\";copy:m.width;<T>:{n:copy;-><int32>}",
            "E211",
        ),
        (
            "m:@\"./scalar.mwy\";copy:m+0;<T>:{n:copy;-><int32>}",
            "B001",
        ),
        ("row:{->width:4};f<int32>:(){->row.width}", "B001"),
    ] {
        let case = case(source, &[("data.mwy", "->width:4"), ("scalar.mwy", "->4")]);
        let output = case.command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}

#[test]
pub(crate) fn computed_fields_charge_the_whole_record_work_on_every_read() {
    let tail = (1..110)
        .map(|id| format!("v{id}:v{}+1;", id - 1))
        .collect::<String>();
    let source = format!("settings:{{->width:4;v0:1;{tail}}};<T>:{{n:settings.width;-><int32>}}");
    let output = Case::new(&source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("\"code\":\"B001\""), "{error}");
    assert!(error.contains("computed type bootstrap budget"), "{error}");
}

#[test]
pub(crate) fn nested_records_preserve_module_initialization_and_subrecord_aliases() {
    case(
        "m:@\"./types.mwy\";d:@\"debug\";items<m.Items>:[3,7];d.print(items[2]);d.print(m.get())",
        &[("types.mwy", "d:@\"debug\";d.print(\"init\");settings:{->limits:{->width<uint8>:4}};alias:settings.limits;-><Items>:{-><int32[alias.width]>};->get<int32>:(){<Local>:{n:settings.limits.width;-><int32[n]>};v<Local>:[4,8];->v[2]}")],
    ).runs(b"init\n7\n8\n");
}

#[test]
pub(crate) fn nested_records_keep_errors_outside_a_projected_subrecord() {
    let source = "#é#\npart:{->n<uint8>:4};|false|{row<{nested<{n<uint8>}>;bad<uint8>}>:{->nested:part;->bad<uint8>:255+1};alias:row.nested;<T>:{n:alias.n;-><int32>}}";
    let case = case("m:@\"./types.mwy\"", &[("types.mwy", source)]);
    for action in ["check", "build", "run"] {
        let output = case.command(action, &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E107\""), "{error}");
        assert!(
            error.contains(&format!(
                "\"path\":\"{}\"",
                case.path.join("types.mwy").display()
            )),
            "{error}"
        );
        assert!(
            error.contains(&format!("\"start\":{}", source.find("255+1").unwrap())),
            "{error}"
        );
    }
}

#[test]
pub(crate) fn nested_records_cannot_drop_ancestor_effects_or_work() {
    let source = "d:@\"debug\";row:{->nested:{->n:4};d.print(\"must not run\")};alias:row.nested;<T>:{n:alias.n;-><int32>}";
    let output = Case::new(source).command("run", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E211\""));
    let tail = (1..110)
        .map(|id| format!("v{id}:v{}+1;", id - 1))
        .collect::<String>();
    let source = format!(
        "row:{{->nested:{{->n:4}};v0:1;{tail}}};alias:row.nested;<T>:{{n:alias.n;-><int32>}}"
    );
    let output = Case::new(&source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("\"code\":\"B001\""), "{error}");
    assert!(error.contains("computed type bootstrap budget"), "{error}");
}

#[test]
pub(crate) fn nested_records_keep_imported_values_and_runtime_captures_gated() {
    for (source, code) in [
        ("m:@\"./data.mwy\";<T>:{n:m.nested.n;-><int32>}", "B001"),
        (
            "m:@\"./data.mwy\";alias:m.nested;<T>:{n:alias.n;-><int32>}",
            "E211",
        ),
        ("row:{->nested:{->n:4}};f<int32>:(){->row.nested.n}", "B001"),
    ] {
        let case = case(source, &[("data.mwy", "->nested:{->n:4}")]);
        let output = case.command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}
