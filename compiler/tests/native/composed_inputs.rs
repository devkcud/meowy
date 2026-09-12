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
