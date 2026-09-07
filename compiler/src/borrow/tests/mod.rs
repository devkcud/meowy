mod aggregates;
mod boundaries;
mod control;
mod mutable_references;
mod origins;
mod reference_slots;
mod slots;
mod temporaries;
mod temporary_carriers;
mod transitive;

pub(crate) fn accepts(source: &str) {
    let result = crate::compile(source);
    assert!(result.is_ok(), "{source}: {result:?}");
}

pub(crate) fn rejects(source: &str, code: &str) {
    let errors = crate::compile(source).unwrap_err();
    assert_eq!(errors[0].code, code, "{source}: {errors:?}");
}
