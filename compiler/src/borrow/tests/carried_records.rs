use super::{accepts, rejects};
use crate::ast::Span;
use crate::borrow::carried::{MAX_DEPTH, MAX_PARTS, eligible};
use crate::flow::Flow;
use crate::hir::{Field, FoundationType, Type};

#[test]
pub(crate) fn carried_records_initialize_nested_scalar_and_unit_members_once() {
    accepts(
        r#"<Inner>:<{n<int32>;unit<null>}>;<Row>:<{inner<Inner>;ok<boolean>;text<string>}>;<R>:<{row<Row>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->row:{->inner:{->n:7;->unit:null};->ok:true;->text:"ready"};first=false};i=i+1;|i<3|'loop.restart()}};v:r.row.inner.n"#,
    );
    accepts(
        "<Row>:<{-><int32>;n<uint8>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->5;->n:255};first=false;'loop.restart()}}};v<int32>:r.row;w:r.row.n",
    );
}

#[test]
pub(crate) fn carried_records_preserve_copies_field_writes_and_whole_replacement() {
    accepts(
        r#"<Row>:<{n<int32>:=;text<string>}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7;->text:"old"};old:row;row.n=8;row={->n:=9;->text:"new"};v:old.n;first=false;'loop.restart()}}};v:r.row.n"#,
    );
}

#[test]
pub(crate) fn carried_records_keep_owner_resets_and_leave_completion() {
    accepts(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;i:=0;r<R>:'out{first:=true;'loop{|first|{'out->row:{->n:i};first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()};v:r.row.n",
    );
    accepts(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};first=false;'loop.restart()};'out.leave()}};v:r.row.n",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->row:{->n:i};first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_records_reject_incomplete_duplicate_and_mistyped_results() {
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->row:{->n:7};first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;i:=0;r<R>:'out{'loop{'out->row:{->n:7};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};'out->row:{->n:8};first=false;'loop.restart()}}}",
        "E205",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{};first=false;'loop.restart()}}}",
        "E204",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:true};first=false;'loop.restart()}}}",
        "E207",
    );
}

#[test]
pub(crate) fn carried_records_keep_wider_shapes_and_storage_borrows_gated() {
    for (row, value) in [
        ("<{n<int32><null>}>", "{->n:7}"),
        ("<{n<int32[2]>}>", "{->n:[1,2]}"),
        ("<{n<&int32>}>", "{->n:&x}"),
    ] {
        rejects(
            &format!(
                "<Row>:{row};<R>:<{{row<Row>}}>;x:1;first:=true;r<R>:'out{{'loop{{|first|{{'out->row:{value};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    rejects(
        "<Row>:<{n<int32>}>;first:=true;r:'out{'loop{|first|{'out->row:{->n:7};first=false;'loop.restart()}}}",
        "B001",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row><null>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};first=false;'loop.restart()}}}",
        "B001",
    );
    rejects(
        "<Row>:<{n<int32>:=}>; <R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7};p:&!row;v:p.n;first=false;'loop.restart()}}}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_record_copies_and_scalar_siblings_keep_existing_borrow_rules() {
    accepts(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};copy:row;p:&(copy.n);v:*p;first=false;'loop.restart()}}};p:&(r.row.n);v:*p",
    );
    accepts(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>;count<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};'out->count:=1;p:&!count;*p=9;first=false;'loop.restart()}}};v:r.row.n",
    );
}

#[test]
pub(crate) fn carried_records_preserve_initializer_effects_and_partial_panics() {
    accepts(
        r#"d:@"debug";<Row>:<{n<int32>}>;<R>:<{row<Row>}>;run<int32>:(stop<boolean>){first:=true;builds:=0;r<R>:'out{'loop{|first|{'out->row:{builds=builds+1;|stop|d.panic("halt");->n:7};first=false;'loop.restart()}}};->r.row.n}"#,
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;run<null>:(stop<boolean>){first:=true;r<R>:'out{'loop{|first|{'out->row:{|stop|'out.leave();->n:7};first=false;'loop.restart()}}}}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_record_shape_checks_reject_non_plain_members() {
    let int = Type::Int {
        bits: 32,
        signed: true,
    };
    for ty in [
        Type::Reference(Box::new(int.clone())),
        Type::Exclusive(Box::new(int.clone())),
        Type::List {
            element: Box::new(int.clone()),
            capacity: 2,
        },
        Type::Union(vec![Type::Null, int]),
        Type::Foundation(FoundationType::Allocator),
        Type::Never,
    ] {
        let row = Type::Record {
            primary: Box::new(Type::Null),
            fields: vec![Field {
                name: "n".into(),
                ty,
                mutable: false,
            }],
        };
        assert!(!eligible(&row, &mut Flow::new(), Span::default()).unwrap());
    }
    assert!(!eligible(&Type::Null, &mut Flow::new(), Span::default()).unwrap());
}

#[test]
pub(crate) fn carried_record_shape_depth_size_and_work_are_bounded() {
    for count in [MAX_PARTS - 2, MAX_PARTS - 1] {
        let row = Type::Record {
            primary: Box::new(Type::Null),
            fields: (0..count)
                .map(|n| Field {
                    name: format!("n{n}"),
                    ty: Type::Bool,
                    mutable: false,
                })
                .collect(),
        };
        let result = eligible(&row, &mut Flow::new(), Span::default());
        if count == MAX_PARTS - 2 {
            assert!(result.unwrap());
        } else {
            assert_eq!(result.unwrap_err().code, "B001");
        }
    }
    let mut row = Type::Bool;
    for _ in 0..MAX_DEPTH {
        row = Type::Record {
            primary: Box::new(Type::Null),
            fields: vec![Field {
                name: "next".into(),
                ty: row,
                mutable: false,
            }],
        };
    }
    assert_eq!(
        eligible(&row, &mut Flow::new(), Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
    let mut guards = Flow::new();
    assert!(!guards.spend(usize::MAX));
    assert_eq!(
        eligible(&Type::Bool, &mut guards, Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
}
