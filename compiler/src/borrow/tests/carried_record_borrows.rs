use super::{accepts, rejects};

#[test]
pub(crate) fn carried_record_borrows_acquire_whole_and_nested_storage() {
    accepts(
        r#"<Inner>:<{n<uint8>;unit<null>}>;<Row>:<{inner<Inner>;name<string>;ok<boolean>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->inner:{->n:255;->unit:null};->name:"ready";->ok:true};p:&row;q:&row.inner.n;s:&row.name;b:&row.ok;v:p.inner.n;w:*q;t:*s;ok:*b;same:q==&p.inner.n;first=false;'loop.restart()}}}"#,
    );
}

#[test]
pub(crate) fn carried_record_borrows_retain_alias_storage_and_old_reference_copies() {
    accepts(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;x<Row>:{->n:1};p:=&x;old:p;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->row:{->n:7};p=&row;first=false};v:p.n;w:old.n;i=i+1;|i<3|'loop.restart()};v:p.n};p=&x;v:p.n",
    );
}

#[test]
pub(crate) fn carried_record_borrows_retain_projected_reborrows() {
    accepts(
        "<Inner>:<{n<int32>}>;<Row>:<{inner<Inner>}>;<R>:<{row<Row>}>;x:1;p:=&x;q:=&x;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->row:{->inner:{->n:7}};whole:&row;p=&row.inner.n;q=&whole.inner.n;first=false};v:*p;w:*q;same:p==q;i=i+1;|i<3|'loop.restart()};v:*q};p=&x;q=&x;v:*p;w:*q",
    );
}

#[test]
pub(crate) fn carried_record_borrows_expire_on_owner_completion_and_reset() {
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;x<Row>:{->n:1};p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};p=&row;first=false;'loop.restart()}}};v:p.n",
        "E303",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;x:1;p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};p=&row.n;first=false;'loop.restart()}}};v:*p",
        "E303",
    );
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;x:1;p:=&x;i:=0;r<R>:'out{v:*p;first:=true;'loop{|first|{'out->row:{->n:i};p=&row.n;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
        "E303",
    );
    accepts(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;x:1;p:=&x;i:=0;r<R>:'out{p=&x;first:=true;'loop{|first|{'out->row:{->n:i};p=&row.n;first=false;'loop.restart()};v:*p};i=i+1;|i<2|'out.restart()};p=&x;v:*p",
    );
}

#[test]
pub(crate) fn carried_record_borrows_preserve_field_conflicts_and_final_use() {
    let prefix = "<Row>:<{n<int32>:=;other<int32>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7;->other:=1};";
    let suffix = "first=false;'loop.restart()}}}";
    accepts(&format!(
        "{prefix}p:&row.n;row.other=9;v:*p;row.n=8;q:&row;w:q.n;row={{->n:=10;->other:=2}};{suffix}"
    ));
    accepts(&format!("{prefix}old:row;p:&old.n;row.n=8;v:*p;{suffix}"));
    rejects(&format!("{prefix}p:&row.n;row.n=8;v:*p;{suffix}"), "E302");
    rejects(
        &format!("{prefix}p:&row;row={{->n:=8;->other:=2}};v:p.n;{suffix}"),
        "E302",
    );
}

#[test]
pub(crate) fn carried_record_borrows_keep_call_bounds_and_leave_expiry() {
    let source = "<Inner>:<{n<int32>}>;<Row>:<{inner<Inner>}>;<R>:<{row<Row>}>;keep<&int32>:(value<&int32>){->value};x:1;p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->row:{->inner:{->n:7}};p=keep(&row.inner.n);first=false;'loop.restart()};v:*p;'out.leave()}};";
    accepts(&format!("{source}p=&x;v:*p"));
    rejects(&format!("{source}v:*p"), "E303");
}

#[test]
pub(crate) fn carried_record_borrows_keep_unproved_and_exclusive_storage_gated() {
    rejects(
        "<Row>:<{n<int32>}>;<R>:<{row<Row>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->row:{->n:7};p:&row.n;v:*p;first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<Row>:<{n<int32>:=}>;<R>:<{row<Row>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->row:={->n:=7};p:&!row;v:p.n;first=false;'loop.restart()}}}",
        "B001",
    );
    rejects(
        "<Row>:<{n<int32><null>}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->n:7};p:&row;v:p.n;first=false;'loop.restart()}}}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_record_borrows_require_whole_slot_initialization_at_acquisition() {
    use crate::ast::Span;
    use crate::borrow::carried::{Slot, Slots};
    use crate::borrow::{Facts, Proofs};
    use crate::flow::Flow;
    use crate::hir::{Block, Field, Program, Type};
    use crate::loans::emission_init::Event;
    use crate::loans::storage::{EventKind, ScopeKind};
    use crate::loans::{Edge, Graph, Node};

    let row = Type::Record {
        primary: Box::new(Type::Null),
        fields: vec![Field {
            name: "n".into(),
            ty: Type::Int {
                bits: 32,
                signed: true,
            },
            mutable: false,
        }],
    };
    let program = Program {
        body: Block {
            id: 0,
            stmts: Vec::new(),
            ty: Type::Record {
                primary: Box::new(Type::Null),
                fields: vec![Field {
                    name: "row".into(),
                    ty: row.clone(),
                    mutable: false,
                }],
            },
        },
        functions: Vec::new(),
        locals: Vec::new(),
    };
    let key = (0, Some("row".into()));
    let proofs = Proofs {
        carried: Slots::from([(
            key.clone(),
            Slot {
                ty: row,
                span: Span::default(),
            },
        )]),
        ..Proofs::default()
    };
    let facts = Facts::default();
    let mut flow = Flow::new();
    let mut graph = Graph::new(&program, &facts, &proofs, &mut flow);
    let scope = graph.new_scope(ScopeKind::Block(0)).unwrap();
    let mut node = Node::default();
    graph
        .event(&mut node, EventKind::Enter(scope), Span::default())
        .unwrap();
    node.emissions = vec![
        Event::Acquire(key.clone(), Span::default()),
        Event::Emit(key.clone()),
        Event::Complete(0),
    ];
    graph.nodes[0] = node;
    assert_eq!(graph.emission_states().unwrap_err().code, "B001");
    graph.nodes[0].emissions.swap(0, 1);
    graph.emission_states().unwrap();
    graph.nodes[0].emissions.swap(1, 2);
    assert_eq!(graph.emission_states().unwrap_err().code, "B001");
    graph.nodes[0].emissions = vec![Event::Emit(key.clone())];
    let mut reset = Node::default();
    graph
        .event(&mut reset, EventKind::End(scope), Span::default())
        .unwrap();
    graph
        .event(&mut reset, EventKind::Enter(scope), Span::default())
        .unwrap();
    reset
        .emissions
        .push(Event::Acquire(key.clone(), Span::default()));
    graph.nodes.push(reset);
    graph.nodes[0].next.push(Edge {
        target: 1,
        guard: crate::flow::TRUE,
        reset: true,
    });
    assert_eq!(graph.emission_states().unwrap_err().code, "B001");
    graph.nodes[1].emissions.insert(0, Event::Emit(key));
    graph.nodes[1].emissions.push(Event::Complete(0));
    graph.emission_states().unwrap();
}
