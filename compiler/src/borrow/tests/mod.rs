mod aggregates;
mod allocator_carriers;
mod allocator_headers;
mod allocator_records;
mod allocators;
mod boundaries;
mod control;
mod guarded_references;
mod header_activity;
mod header_components;
mod leaves;
mod mutable_allocators;
mod mutable_references;
mod origins;
mod reference_slots;
mod restarts;
mod slots;
mod tagged_allocators;
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
