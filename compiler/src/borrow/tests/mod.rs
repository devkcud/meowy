mod aggregates;
mod alias_restarts;
mod alias_writes;
mod allocator_carriers;
mod allocator_headers;
mod allocator_records;
mod allocators;
mod binding_fields;
mod boundaries;
mod carried_borrows;
mod carried_list_borrows;
mod carried_lists;
mod carried_proof;
mod carried_record_borrows;
mod carried_records;
mod carried_scalars;
mod changing_published;
mod control;
mod discarded_aliases;
mod exclusive_carried;
mod exclusive_carried_elements;
mod exclusive_carried_records;
mod fixed_published;
mod guarded_references;
mod header_activity;
mod header_components;
mod late_published;
mod leaves;
mod mixed_headers;
mod mutable_allocators;
mod mutable_carriers;
mod mutable_references;
mod origins;
mod published;
mod reference_fields;
mod reference_slots;
mod restarts;
mod slots;
mod tagged_allocators;
mod temporaries;
mod temporary_carriers;
mod transitive;
mod union_aliases;
mod widened_aliases;

pub(crate) fn accepts(source: &str) {
    let result = crate::compile(source);
    assert!(result.is_ok(), "{source}: {result:?}");
}

pub(crate) fn rejects(source: &str, code: &str) {
    let errors = crate::compile(source).unwrap_err();
    assert_eq!(errors[0].code, code, "{source}: {errors:?}");
}
