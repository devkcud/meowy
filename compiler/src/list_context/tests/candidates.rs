use super::rejects;

#[test]
pub(crate) fn literals_use_the_same_sign_and_width_rules_as_single_contexts() {
    for source in [
        "values<int8[1]><uint8[1]>:[255]",
        "values<int8[1]><uint8[1]>:[-128]",
        "values<int8[1]><uint8[1]>:[-0]",
        "values<float32[1]><float64[1]>:[1e39]",
        "values<int32[1]><float32[1]>:[1]",
    ] {
        assert!(crate::compile(source).is_ok(), "{source}");
    }
    rejects("values<int8[1]><uint8[1]>:[-(128)]", "E207");
    rejects("values<uint8[1]><string[1]>:[-0]", "E207");
    rejects("values<float32[1]><float64[1]>:[1.0]", "E207");
    rejects("values<uint8[2]><string[1]>:[256,1]", "E216");
}

#[test]
pub(crate) fn unresolved_effectful_contexts_are_explicit_and_can_be_annotated() {
    rejects(
        "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1}]",
        "E207",
    );
    assert!(
        crate::compile(
            "d:@\"debug\";byte<uint8>:1;values<uint8[2]><uint16[2]>:[{d.print(1);->1},byte]"
        )
        .is_ok()
    );
    rejects(
        "<Inner>:<int32[1]><int32[2]>;values<Inner[1]><string[1]>:[[1]]",
        "E207",
    );
    rejects(
        "<Inner>:<int32[1]><int32[2]>;values<Inner[1]><int32[2][1]>:[[1]]",
        "B001",
    );
}

#[test]
pub(crate) fn candidate_count_and_probe_work_are_bounded() {
    let types = (0..=super::MAX_CONTEXTS)
        .map(|capacity| format!("<int32[{capacity}]>"))
        .collect::<String>();
    rejects(&format!("values{types}:[]"), "B001");
    let types = (2..=super::MAX_CONTEXTS + 1)
        .map(|capacity| format!("<int32[{capacity}]>"))
        .collect::<String>();
    let source = format!("values{types}:[{}]", "0".repeat(20_000));
    rejects(&source, "B001");
    let fields = (0..4095)
        .map(|index| format!(";field{index}<null>"))
        .collect::<String>();
    let source =
        format!("<Row>:<{{-><int32>{fields}}}>;use<null>:(row<Row>){{values{types}:[row,row]}}");
    rejects(&source, "B001");
}
