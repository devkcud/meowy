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
        (
            "m:@\"./data.mwy\";copy:m+0;<T>:{n:copy;-><int32>}",
            "->4;->width:2",
            "E211",
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
