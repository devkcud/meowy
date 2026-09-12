use super::Case;

#[test]
pub(crate) fn conditional_record_inputs_select_named_nested_and_composed_fields() {
    for condition in ["true", "!false", "true&&(!false||false)"] {
        let source = format!(
            "d:@\"debug\";row:{{|{condition}|->width<uint8>:4;|false|->width<uint8>:9;|true|->part:{{|true|->n<uint8>:2}}}};copy:{{|{condition}|->row}};n:copy.width;<T>:{{-><int32[n+copy.part.n]>}};v<T>:[7];d.print(v[1]);d.print(copy.width);d.print(copy.part.n)"
        );
        Case::new(&source).runs(b"7\n4\n2\n");
    }
}

#[test]
pub(crate) fn conditional_record_inputs_skip_unevaluated_effects_and_conditions() {
    for body in [
        "|false|d.print(1);->n:4",
        "->n:4;|false|d.print(1)",
        "|false|{d.print(1);unused:=2};->n:4",
        "|true| |!false|->n:4",
        "|true| |false|d.print(1);->n:4",
        "|false&&effect()|d.print(1);->n:4",
        "|true| |false&&effect()|d.print(1);->n:4",
        "|true||true| |false|d.print(1);->n:4",
        "|true| |true| |true|->n:4",
        "|true| |true| |true| |true|->n:4",
    ] {
        let source = format!(
            "d:@\"debug\";effect<boolean>:(){{d.print(9);->true}};row:{{{body}}};<T>:{{-><int32[row.n]>}};v<T>:[7];d.print(v[1])"
        );
        Case::new(&source).runs(b"7\n");
    }
    Case::new("d:@\"debug\";effect<boolean>:(){d.print(9);->true};row:{|true||effect()|->n:4};<T>:{-><int32[row.n]>};v<T>:[7];d.print(v[1])").runs(b"7\n");
}

#[test]
pub(crate) fn conditional_record_inputs_reject_selected_effects_and_folded_runtime_reads() {
    for body in [
        "|true|d.print(1);->n:4",
        "->n:4;|true|d.print(1)",
        "|true|unused:=1;->n:4",
        "|true&&effect()|d.print(1);->n:4",
        "|false||effect()|d.print(1);->n:4",
        "|flag|d.print(1);->n:4",
        "|1<2|->n:4",
        "|true|->part:{->n:4};|true|->bad:{->n:1;d.print(1)};->n:4",
    ] {
        let source = format!(
            "d:@\"debug\";flag:false;effect<boolean>:(){{d.print(9);->true}};row:{{{body}}};<T>:{{n:row.n;-><int32>}}"
        );
        let output = Case::new(&source).command("run", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E211\""), "{source}: {error}");
    }
}

#[test]
pub(crate) fn conditional_record_inputs_retain_selected_sibling_error_spans() {
    for body in [
        "|true|->good<uint8>:4;|true|->bad<uint8>:255+1",
        "->good<uint8>:4;|true|unused<uint8>:255+1;->bad<uint8>:2",
    ] {
        let source = format!(
            "|false|{{row<{{good<uint8>;bad<uint8>}}>:{{{body}}};<T>:{{n:row.good;-><int32>}}}}"
        );
        let error = meowy::compile(&source).unwrap_err().remove(0);
        assert_eq!(error.code, "E107", "{source}: {error:?}");
        assert_eq!(error.span.start, source.find("255+1").unwrap());
    }
}

#[test]
pub(crate) fn conditional_record_inputs_bound_active_branch_depth() {
    for (depth, code) in [(31, None), (32, Some("E211"))] {
        let guards = "|true| ".repeat(depth);
        let source = format!("row:{{{guards}->n:4}};<T>:{{n:row.n;-><int32>}}");
        let result = meowy::compile(&source);
        if let Some(code) = code {
            assert_eq!(result.unwrap_err()[0].code, code);
        } else {
            assert!(result.is_ok(), "{result:?}");
        }
    }
}
