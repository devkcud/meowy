use super::{accepts, rejects};

#[test]
pub(crate) fn transitive_headers_preserve_nested_cells_and_selected_field_sources() {
    for source in [
        "a:1;b:2;left:&a;right:&b;p:=&left;count:=0;'loop{p=&right;count=count+1;|count<2|'loop.restart()};copy:*p;value:*copy",
        "a:=1;b:=2;c:=3;left:{->first:&a;->second:&b};right:{->first:&a;->second:&c};p:=&left;count:=0;'loop{p=&right;count=count+1;|count<2|'loop.restart()};b=4;c=4;value:*p.first",
        "a:=1;cell:&a;p:=&cell;count:=0;'loop{p=&cell;count=count+1;|count<2|'loop.restart()};a=2;same:p==&cell",
    ] {
        accepts(source);
    }
    rejects(
        "a:=1;b:2;left:&a;right:&b;p:=&right;count:=0;'loop{p=&left;count=count+1;|count<2|'loop.restart()};a=3;value:**p",
        "E302",
    );
    rejects(
        "a:=1;b:=2;left:{->first:&a;->second:&b};p:=&left;count:=0;'loop{p=&left;count=count+1;|count<2|'loop.restart()};b=3;value:*p.second",
        "E302",
    );
}

#[test]
pub(crate) fn header_shapes_keep_reference_free_union_paths_and_temporary_gate() {
    for source in [
        "value<int32><null>:null;p:=&value;count:=0;'loop{p=&value;count=count+1;|count<2|'loop.restart()};copy:*p",
        "value<int32><null>:null;cell:&value;p:=&cell;count:=0;'loop{p=&cell;count=count+1;|count<2|'loop.restart()};copy:**p",
        "value<int32><null>:null;holder:{->view:&value;->n:1};p:=&holder;count:=0;'loop{p=&holder;count=count+1;|count<2|'loop.restart()};copy:*p.view",
        "a:1;holder:{->view<&int32><null>:&a};p:=&holder;'loop{p=&holder;'loop.restart()}",
        "a:1;holder:{->view:&a;->tag<int32><null>:null};p:=&holder;'loop{p=&holder;'loop.restart()}",
    ] {
        accepts(source);
    }
    rejects("cell:&1;p:=&cell;'loop{p=&cell;'loop.restart()}", "B001");
}

#[test]
pub(crate) fn header_dereference_retains_public_input_bounds() {
    rejects(
        "id<& &int32>:(p<& &int32>,other<&string>){->p};a:1;cell:&a;other:=\"old\";p:=id(&cell,&other);count:=0;'loop{p=id(&cell,&other);count=count+1;|count<2|'loop.restart()};copy:*p;other=\"new\";value:*copy",
        "E302",
    );
    accepts(
        "id<& &int32>:(p<& &int32>,other<&string>){->p};a:1;cell:&a;other:=\"old\";p:=id(&cell,&other);count:=0;'loop{p=id(&cell,&other);count=count+1;|count<2|'loop.restart()};copy:*p;value:*copy;other=\"new\"",
    );
}

#[test]
pub(crate) fn fixed_point_comparison_keeps_component_identity() {
    use crate::borrow::{BTreeMap, Origin, Source, State, Step};
    use crate::flow::{Flow, TRUE};
    use crate::hir::{Field, Type};
    let scalar = Type::Int {
        bits: 32,
        signed: true,
    };
    let ty = Type::Reference(Box::new(Type::Record {
        primary: Box::new(Type::Null),
        fields: ["left", "right"]
            .into_iter()
            .map(|name| Field {
                name: name.into(),
                ty: Type::Reference(Box::new(scalar.clone())),
                mutable: false,
            })
            .collect(),
    }));
    let origin = |component, id| Origin {
        component,
        source: Source::Local {
            id,
            fields: Vec::new(),
        },
        guard: TRUE,
    };
    let left = vec![Step::Deref, Step::Slot(1)];
    let right = vec![Step::Deref, Step::Slot(2)];
    let a = State {
        origins: vec![
            origin(Vec::new(), 10),
            origin(left.clone(), 0),
            origin(left, 1),
            origin(right.clone(), 2),
        ],
        ..State::default()
    };
    let mut b = a.clone();
    b.origins[2].component = right;
    let mut flow = Flow::new();
    let span = crate::ast::Span::default();
    let shape = crate::borrow::header::Shape::new(&ty, &mut flow, span).unwrap();
    shape.validate(&a, true, &mut flow, span).unwrap();
    shape.validate(&b, true, &mut flow, span).unwrap();
    let mut missing = a.clone();
    missing.origins.pop();
    assert!(shape.validate(&missing, true, &mut flow, span).is_err());
    let a = BTreeMap::from([(1, BTreeMap::from([(20, a)]))]);
    let b = BTreeMap::from([(1, BTreeMap::from([(20, b)]))]);
    assert!(!crate::borrow::restart::same(&a, &b, &mut flow, span).unwrap());
}
