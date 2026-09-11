use crate::check::inputs::tests::check;

#[test]
pub(crate) fn record_inputs_preserve_field_order_aliases_and_local_emission_dependencies() {
    let checker = check("row:{seed<uint8>:2;->z<uint8>:seed+1;->a<uint8>:{->z+1}};alias:row");
    assert_eq!(checker.record_inputs.len(), 2);
    for record in checker.record_inputs.values() {
        assert_eq!(record.values, [Some(4), Some(3)]);
        assert!(record.input.error.is_none());
    }
}

#[test]
pub(crate) fn record_inputs_reject_effects_mutability_and_noninteger_shapes() {
    for source in [
        "d:@\"debug\";row:{->n:4;d.print(1)}",
        "row:{unused:=0;->n:4}",
        "row:={->n:4}",
        "row:{->n:=4}",
        "row:{->n:4;->text:\"x\"}",
        "row:{->nested:{->n:4}}",
        "row:{->4;->n:4}",
        "row:{|true|->n:4}",
    ] {
        let checker = check(source);
        assert!(checker.record_inputs.is_empty(), "{source}");
    }
}

#[test]
pub(crate) fn record_inputs_retain_whole_initializer_errors_and_field_limits() {
    let checker =
        check("|false|{row<{good<uint8>;bad<uint8>}>:{->good<uint8>:4;->bad<uint8>:255+1}}");
    let record = checker.record_inputs.values().last().unwrap();
    assert_eq!(record.input.error.as_ref().unwrap().code, "E107");
    assert_eq!(record.values, [None, Some(4)]);
    let fields = (0..257).map(|id| format!("->n{id}:1;")).collect::<String>();
    assert!(check(&format!("row:{{{fields}}}")).record_inputs.is_empty());
}
