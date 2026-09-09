use super::{accepts, rejects};

#[test]
pub(crate) fn mixed_writes_check_each_field_and_selected_list_type() {
    for source in [
        "r:={->items:=[1,2];->other:=3};r.items[1]=4",
        "r:{->items:=[1,2]};r.items[1]=3",
        "r:=[{->n:=1},{->n:=2}];r[2].n=3",
        "r:={->rows:=[{->items:=[1,2]},{->items:=[3,4]}]};r.rows[2].items[1]=5",
        "r:={->rows:=[{->n:=1}]};(r.rows[1]).n=2",
    ] {
        accepts(source);
    }
    for (source, code) in [
        ("r:={->items:[1,2]};r.items[1]=3", "E305"),
        ("r:=[{->n:1}];r[1].n=2", "E305"),
        ("r:={->items:=[1,2]};p:&r;p.items[1]=3", "B001"),
        ("r:={->items:=[1,2]};r.items[0]=3", "E101"),
        (
            "<R>:<{items<uint8[1]>:=}>;r<R>:={->items:=[1]};r.items[1]=256",
            "E216",
        ),
    ] {
        rejects(source, code);
    }
    let source = "r:={->rows:=[{->items:=[1,2]}]};r.rows[1].items[2]=3";
    let program = crate::compile(source).unwrap();
    let Some(crate::hir::Stmt::SetPath { path, .. }) = program.body.stmts.last() else {
        panic!("write path")
    };
    let spans = path
        .iter()
        .filter_map(|step| {
            let crate::hir::WriteStep::Index(step) = step else {
                return None;
            };
            Some(&source[step.span.start..step.span.end])
        })
        .collect::<Vec<_>>();
    assert_eq!(spans, ["r.rows[1]", "r.rows[1].items[2]"]);
}
