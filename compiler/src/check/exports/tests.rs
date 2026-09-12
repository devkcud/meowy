use crate::check::inputs::tests::check;

#[test]
pub(crate) fn exported_inputs_retain_declaration_ids_and_private_dependencies() {
    let checker = check("private<uint8>:2;->width:private+2;->row:{->nested:{->n:width}}");
    assert_eq!(checker.module.inputs.len(), 2);
    assert!(!checker.module.inputs.contains_key("private"));
    let width = checker.module.inputs["width"].id;
    assert_eq!(checker.inputs[&width].value, Some(4));
    let row = checker.module.inputs["row"].id;
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
    let id = checker.module.inputs["copy"].id;
    let record = &checker.record_inputs[&id];
    assert_eq!(record.field(&[0]).unwrap().value, Some(4));
    assert!(record.input.work > 10);
}

#[test]
pub(crate) fn primary_inputs_preserve_checked_identity_width_and_runtime_expression() {
    use crate::hir::{ExprKind, Stmt, Type};
    let parsed = crate::parser::parse_documented("base<uint8>:2;->base+2").unwrap();
    let mut checker = crate::check::Checker::new();
    let block = checker.block(&parsed.block, None, None).unwrap();
    let (id, input) = checker.module.primary.unwrap();
    assert_eq!(input.value, Some(4));
    assert_eq!(block.stmts.len(), 2);
    assert!(
        matches!(&block.stmts[1], Stmt::Emit { id: found, field: None, value, .. }
        if *found == id && matches!(value.kind, ExprKind::Binary { .. })
            && value.ty == Type::Int { bits: 8, signed: false })
    );
}

#[test]
pub(crate) fn primary_inputs_keep_private_dependencies_tail_work_and_file_effects() {
    let checker =
        check("d:@\"debug\";d.print(1);base:2;->{->base*2;unused:3};d.print(2);->named:7");
    let (_, input) = checker.module.primary.unwrap();
    assert_eq!(input.value, Some(4));
    assert!(input.work > 6);
    assert_eq!(checker.module.inputs.len(), 1);
    assert_eq!(
        checker.inputs[&checker.module.inputs["named"].id].value,
        Some(7)
    );
}

#[test]
pub(crate) fn primary_inputs_exclude_effects_mutable_sources_conditionals_and_nonintegers() {
    for source in [
        "d:@\"debug\";->{->4;d.print(1)}",
        "value:=4;->value",
        "|true|->4",
        "|false|->4",
        "->{->n:4}",
        "->true",
        "->4.0",
        "f<int32>:(){->4};->f()",
    ] {
        assert!(check(source).module.primary.is_none(), "{source}");
    }
}
