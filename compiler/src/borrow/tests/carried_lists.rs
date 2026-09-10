use super::{accepts, rejects};

#[test]
pub(crate) fn carried_lists_keep_copies_lengths_and_whole_replacement() {
    accepts(
        "<R>:<{items<int32[3]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[1,2];old:items;items=[9];items=items.add(10);v:old[2];first=false;'loop.restart()}}};n:r.items.size();v:r.items[2]",
    );
}

#[test]
pub(crate) fn carried_lists_initialize_nested_lists_records_and_primaries() {
    accepts(
        "<Row>:<{value<int32>;unit<null>}>;<R>:<{items<Row[3][2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[[{->value:7;->unit:null}],[]];first=false;'loop.restart()}}};v:r.items[1][1].value",
    );
    accepts(
        "<Row>:<{items<int32[3]>:=;other<int32>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->items:=[1,2];->other:=3};row.items=[9];row.other=4;first=false;'loop.restart()}}};v:r.row.items[1]",
    );
    accepts(
        "first:=true;r<int32[3]>:'out{'loop{|first|{'out->[1,2];first=false;'loop.restart()}}};v:r[2]",
    );
    accepts(
        "<R>:<{empty<int32[0]>;units<null[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->empty:[];'out->units:[null];first=false;'loop.restart()}}};a:r.empty.size();b:r.units.size()",
    );
}

#[test]
pub(crate) fn carried_lists_require_complete_once_only_initialization() {
    for source in [
        "<R>:<{items<int32[2]>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->items:[1];first=false};i=i+1;|i<2|'loop.restart()}}",
        "<R>:<{items<int32[2]>}>;i:=0;r<R>:'out{'loop{'out->items:[1];i=i+1;|i<2|'loop.restart()}}",
        "<R>:<{items<int32[0]>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->items:[];first=false};i=i+1;|i<2|'loop.restart()}}",
    ] {
        rejects(source, "B001");
    }
    rejects(
        "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[1];'out->items:[2];first=false;'loop.restart()}}}",
        "E205",
    );
    rejects(
        "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[true];first=false;'loop.restart()}}}",
        "E207",
    );
}

#[test]
pub(crate) fn carried_lists_preserve_owner_resets_and_leave_completion() {
    accepts(
        "<R>:<{items<int32[2]>}>;i:=0;r<R>:'out{first:=true;'loop{|first|{'out->items:[i];first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()};v:r.items[1]",
    );
    accepts(
        "<R>:<{items<int32[2]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[1];first=false;'loop.restart()};'out.leave()}};v:r.items[1]",
    );
    rejects(
        "<R>:<{items<int32[2]>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->items:[i];first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
        "B001",
    );
    rejects(
        "<R>:<{items<int32[2]>}>;run<null>:(stop<boolean>){first:=true;r<R>:'out{'loop{|first|{'out->items:[{ |stop|'out.leave();->1}];first=false;'loop.restart()}}}}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_lists_keep_whole_list_borrows_and_shared_writes_gated() {
    for body in ["p:&!items", "s:&items;s[1]=9"] {
        rejects(
            &format!(
                "<R>:<{{items<int32[2]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[1];{body};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    accepts(
        "<R>:<{items<int32[2]>:=}>;run<null>:(stop<boolean>){first:=true;r<R>:'out{'loop{|first|{'out->items:=[1];items[{|stop|'out.leave();->1}]=9;first=false;'loop.restart()}}}}",
    );
    for body in ["p:row.&!items", "s:row.&items;s[1]=9"] {
        rejects(
            &format!(
                "<Row>:<{{items<int32[2]>:=;n<int32>:=}}>; <R>:<{{row<Row>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->row:={{->items:=[1];->n:=2}};{body};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
}

#[test]
pub(crate) fn carried_lists_preserve_independent_copies_and_scalar_sibling_loans() {
    accepts(
        "<R>:<{items<int32[2]>:=;n<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[1];old:=items;p:&(old[1]);v:*p;old[1]=9;items=[2];'out->n:=3;q:&!n;*q=4;first=false;'loop.restart()}}};p:&(r.items[1]);v:*p",
    );
}

#[test]
pub(crate) fn carried_lists_keep_union_reference_and_inferred_shapes_gated() {
    for (ty, value) in [
        ("<int32[2]><null>", "[1]"),
        ("<U[2]>", "[1]"),
        ("<&int32[2]>", "&x"),
        ("<V[2]>", "[&n]"),
    ] {
        rejects(
            &format!(
                "<U>:<int32><null>;<V>:<&int32>;<R>:<{{items{ty}}}>;n:1;x:[1,2];first:=true;r<R>:'out{{'loop{{|first|{{'out->items:{value};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    rejects(
        "first:=true;r:'out{'loop{|first|{'out->items:[1];first=false;'loop.restart()}}}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_lists_shape_keeps_zero_capacity_elements_and_layout_limits() {
    use crate::ast::Span;
    use crate::borrow::carried::{Shape, shape};
    use crate::flow::Flow;
    use crate::hir::{FoundationType, Type};

    for ty in [
        Type::Reference(Box::new(Type::Bool)),
        Type::Exclusive(Box::new(Type::Bool)),
        Type::Union(vec![Type::Bool, Type::Null]),
        Type::Foundation(FoundationType::Allocator),
        Type::Never,
    ] {
        let list = Type::List {
            element: Box::new(ty),
            capacity: 0,
        };
        assert_eq!(
            shape(&list, &mut Flow::new(), Span::default()).unwrap(),
            None
        );
    }
    let list = Type::List {
        element: Box::new(Type::Null),
        capacity: 0,
    };
    assert_eq!(
        shape(&list, &mut Flow::new(), Span::default()).unwrap(),
        Some(Shape::List)
    );
    let list = Type::List {
        element: Box::new(Type::Bool),
        capacity: crate::list::MAX_CAPACITY + 1,
    };
    assert_eq!(
        shape(&list, &mut Flow::new(), Span::default()).unwrap(),
        None
    );
    for source in [
        "<R>:<{items<int32[65537]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[];first=false;'loop.restart()}}}",
        "<R>:<{items<string[65536]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[];first=false;'loop.restart()}}}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub(crate) fn carried_lists_shape_depth_parts_and_work_remain_bounded() {
    use crate::ast::Span;
    use crate::borrow::carried::{MAX_DEPTH, MAX_PARTS, eligible};
    use crate::flow::Flow;
    use crate::hir::{Field, Type};

    let mut list = Type::Bool;
    for _ in 1..MAX_DEPTH {
        list = Type::List {
            element: Box::new(list),
            capacity: 0,
        };
    }
    assert!(eligible(&list, &mut Flow::new(), Span::default()).unwrap());
    list = Type::List {
        element: Box::new(list),
        capacity: 0,
    };
    assert_eq!(
        eligible(&list, &mut Flow::new(), Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
    for count in [(MAX_PARTS - 2) / 2, MAX_PARTS / 2] {
        let row = Type::Record {
            primary: Box::new(Type::Null),
            fields: (0..count)
                .map(|n| Field {
                    name: format!("n{n}"),
                    ty: Type::List {
                        element: Box::new(Type::Bool),
                        capacity: 0,
                    },
                    mutable: false,
                })
                .collect(),
        };
        let result = eligible(&row, &mut Flow::new(), Span::default());
        if count == (MAX_PARTS - 2) / 2 {
            assert!(result.unwrap());
        } else {
            assert_eq!(result.unwrap_err().code, "B001");
        }
    }
    let mut flow = Flow::new();
    flow.spend(usize::MAX);
    assert_eq!(
        eligible(&list, &mut flow, Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
}
