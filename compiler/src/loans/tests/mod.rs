mod aliases;
mod budgets;
mod control;
mod fields;
mod functions;
mod guarded_references;
mod leaving_references;
mod mutable_references;
mod places;
mod reference_temporaries;
mod temporaries;
mod transitive;
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
