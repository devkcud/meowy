use super::*;

pub(crate) fn check(source: &str) -> Checker {
    let parsed = crate::parser::parse_documented(source).unwrap();
    let mut checker = Checker::new();
    checker.block(&parsed.block, None, None).unwrap();
    checker
}

#[test]
pub(crate) fn initializer_inputs_track_immutable_integer_dependencies() {
    let checker = check("base<uint8>:2;next:base+2;alias:next;mutable:=4");
    assert_eq!(checker.inputs.len(), 3);
    assert!(checker.inputs.values().all(|input| input.error.is_none()));
    let costs = checker
        .inputs
        .values()
        .map(|input| input.work)
        .collect::<Vec<_>>();
    assert!(costs[1] > costs[0]);
    assert!(costs[2] > costs[1]);
}

#[test]
pub(crate) fn initializer_inputs_exclude_folded_blocks_calls_and_mutable_reads() {
    for source in [
        "x:{->4};alias:x",
        "d:@\"debug\";x:{d.print(1);->4};alias:x",
        "x:=4;copy:x;alias:copy",
        "f<int32>:(x<int32>){copy:x;->copy};result:f(4)",
        "row:{->n:4};copy:row.n",
    ] {
        assert!(check(source).inputs.is_empty(), "{source}");
    }
}

#[test]
pub(crate) fn initializer_inputs_retain_unreachable_arithmetic_failures() {
    let checker = check("|false|{base<uint8>:255;bad:base+1;alias:bad}");
    let errors = checker
        .inputs
        .values()
        .filter_map(|input| input.error.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| error.code == "E107"));
    assert_eq!(errors[0].span, errors[1].span);
}

#[test]
pub(crate) fn initializer_inputs_retain_values_after_lexical_scopes_close() {
    let checker = check("base<uint8>:252;complement:~base;wide<uint64>:4294967296;next:wide+1");
    assert_eq!(
        checker
            .inputs
            .values()
            .map(|input| input.value)
            .collect::<Vec<_>>(),
        [Some(252), Some(3), Some(4294967296), Some(4294967297)]
    );
}

#[test]
pub(crate) fn initializer_inputs_keep_first_invalid_child_before_parent_errors() {
    let source = "|false|{bad<uint8>:(255+1)+(254+2);alias:bad}";
    let checker = check(source);
    let start = source.find("255+1").unwrap();
    for input in checker.inputs.values() {
        assert!(input.value.is_none());
        assert_eq!(input.error.as_ref().unwrap().span.start, start);
    }
}
