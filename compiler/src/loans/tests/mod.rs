mod access;
mod active_restarts;
mod aliases;
mod authority;
mod blocks;
mod budgets;
mod control;
mod elements;
mod exclusive_fields;
mod exclusive_restarts;
mod exclusive_slots;
mod expired_restarts;
mod fields;
mod functions;
mod guarded_references;
mod indexed_fields;
mod init;
mod leaving_references;
mod mixed_headers;
mod mutable_references;
mod nested_elements;
mod places;
mod reference_temporaries;
mod restarting_references;
mod returns;
mod temporaries;
mod transitive;
mod transitive_restarts;
mod values;

use super::{Facts, Flow, Graph, Proofs};

pub(crate) fn accepts(source: &str) {
    let result = crate::compile(source);
    assert!(result.is_ok(), "{source}: {result:?}");
}

pub(crate) fn rejects(source: &str, code: &str) {
    let errors = crate::compile(source).unwrap_err();
    assert_eq!(errors[0].code, code, "{source}: {errors:?}");
}
