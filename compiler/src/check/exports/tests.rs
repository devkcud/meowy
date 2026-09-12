use crate::check::inputs::tests::check;

#[test]
pub(crate) fn exported_inputs_retain_declaration_ids_and_private_dependencies() {
    let checker = check("private<uint8>:2;->width:private+2;->row:{->nested:{->n:width}}");
    assert_eq!(checker.module.inputs.len(), 2);
    assert!(!checker.module.inputs.contains_key("private"));
    let width = checker.module.inputs["width"];
    assert_eq!(checker.inputs[&width].value, Some(4));
    let row = checker.module.inputs["row"];
    assert_eq!(
        checker.record_inputs[&row].field(&[0, 0]).unwrap().value,
        Some(4)
    );
}

#[test]
pub(crate) fn exported_inputs_separate_initializer_effects_from_file_effects() {
    let checker =
        check("d:@\"debug\";d.print(1);->good:4;->bad:{->4;d.print(2)};->row:{->n:4;d.print(3)}");
    assert_eq!(checker.module.inputs.keys().collect::<Vec<_>>(), ["good"]);
}

#[test]
pub(crate) fn exported_inputs_exclude_mutable_conditional_and_composed_emissions() {
    for source in [
        "->n:=4",
        "|true|->n:4",
        "row:{->n:4};->row",
        "->{->n:4}",
        "->row:{->n:=4}",
        "->4",
    ] {
        assert!(check(source).module.inputs.is_empty(), "{source}");
    }
}

#[test]
pub(crate) fn exported_inputs_preserve_complete_record_work() {
    let checker = check("row:{->nested:{->n:4};unused:2};->copy:row.nested");
    let id = checker.module.inputs["copy"];
    let record = &checker.record_inputs[&id];
    assert_eq!(record.field(&[0]).unwrap().value, Some(4));
    assert!(record.input.work > 10);
}
