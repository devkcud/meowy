use super::file_modules::case;

#[test]
pub(crate) fn mixed_primary_copies_keep_widths_aliases_and_named_exports() {
    case(
        "m:@\"./data.mwy\";alias:m;copy:alias+0;f:@\"./facade.mwy\";d:@\"debug\";<T>:{n:copy+f.width;-><int32[n]>};v<T>:[3,7];d.print(v[2]);d.print(alias.label);d.print(copy)",
        &[
            ("data.mwy", "base<uint8>:2;->base*2;->label:\"ready\";->row:{->n:9}"),
            ("facade.mwy", "m:@\"./data.mwy\";copy:m+0;->width:copy"),
        ],
    ).runs(b"7\nready\n4\n");
}

#[test]
pub(crate) fn mixed_primary_copies_preserve_checked_integer_failures() {
    for value in ["copy+1", "export.width+1"] {
        let source = format!(
            "m:@\"./data.mwy\";copy:m+0;export:@\"./facade.mwy\";<T>:{{n:{value};-><int32>}}"
        );
        let output = case(
            &source,
            &[
                ("data.mwy", "base<uint8>:255;->base;->label:\"ready\""),
                ("facade.mwy", "m:@\"./data.mwy\";->width:m+0"),
            ],
        )
        .command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E107\""), "{error}");
    }
}

#[test]
pub(crate) fn mixed_primary_copies_require_evidence_and_preserve_runtime_capture_gates() {
    for (source, data, code) in [
        (
            "m:@\"./data.mwy\";copy:m+0;<T>:{n:copy;-><int32>}",
            "d:@\"debug\";->{->4;d.print(\"must not run\")};->label:\"ready\"",
            "E211",
        ),
        (
            "m:@\"./data.mwy\";copy:m+0;f<int32>:(){->copy}",
            "->4;->label:\"ready\"",
            "B001",
        ),
        (
            "row:{->4;->n:2};copy:row+0;<T>:{n:copy;-><int32>}",
            "->4",
            "E211",
        ),
    ] {
        let output = case(source, &[("data.mwy", data)]).command("run", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}
