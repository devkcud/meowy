use crate::ast::Span;
use crate::borrow::header::Shape;
use crate::borrow_value::{Origin, Source, State, Step};
use crate::flow::{Flow, TRUE};
use crate::hir::{FoundationType, Type};

#[test]
pub(crate) fn allocator_header_can_have_no_bounds_without_weakening_reference_coverage() {
    let mut flow = Flow::default();
    let span = Span::default();
    let allocator = Type::Foundation(FoundationType::Allocator);
    let shape = Shape::new(&allocator, &mut flow, span).unwrap();
    shape
        .validate(&State::default(), true, &mut flow, span)
        .unwrap();
    let reference = Type::Reference(Box::new(allocator));
    let shape = Shape::new(&reference, &mut flow, span).unwrap();
    let mut state = crate::borrow_contract::input(0, &reference, &mut flow, span).unwrap();
    state.bounds.push(Origin {
        component: vec![Step::Deref],
        source: Source::Expired { id: 7 },
        guard: TRUE,
    });
    shape.validate(&state, true, &mut flow, span).unwrap();
    state.origins.clear();
    assert_eq!(
        shape
            .validate(&state, true, &mut flow, span)
            .unwrap_err()
            .code,
        "B001"
    );
}

#[test]
pub(crate) fn mixed_allocator_header_requires_each_physical_reference_origin() {
    let mut flow = Flow::default();
    let span = Span::default();
    let ty = Type::Record {
        primary: Box::new(Type::Foundation(FoundationType::Allocator)),
        fields: vec![crate::hir::Field {
            name: "p".into(),
            ty: Type::Reference(Box::new(Type::Foundation(FoundationType::Allocator))),
            mutable: false,
        }],
    };
    let shape = Shape::new(&ty, &mut flow, span).unwrap();
    let mut state = crate::borrow_contract::input(0, &ty, &mut flow, span).unwrap();
    shape.validate(&state, true, &mut flow, span).unwrap();
    state.bounds = state.origins.clone();
    state.origins.clear();
    assert_eq!(
        shape
            .validate(&state, true, &mut flow, span)
            .unwrap_err()
            .code,
        "B001"
    );
}
