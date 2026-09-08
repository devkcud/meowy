use super::{accepts, rejects};

#[test]
pub(crate) fn carried_scalar_boolean_refinement_rejects_impossible_paths() {
    use crate::loans::emission_value::{Op, Value};
    use std::collections::BTreeMap;

    let mut values = BTreeMap::new();
    let both = Value::Binary(
        Op::And,
        Box::new(Value::Local(0)),
        Box::new(Value::Local(0).negated()),
    );
    assert!(!both.refine(&mut values, true));
    values.clear();
    let either = Value::Binary(
        Op::Or,
        Box::new(Value::Local(0)),
        Box::new(Value::Local(0).negated()),
    );
    assert!(!either.refine(&mut values, false));
    accepts(
        "<R>:<{n<int32>}>;run<null>:(flag<boolean>){first:=true;i:=0;r<R>:'out{'loop{|first&&(flag||!flag)|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}}",
    );
}

#[test]
pub(crate) fn carried_scalar_proof_rejects_missing_initialization_duplicates_and_budget_exhaustion()
{
    use crate::ast::Span;
    use crate::borrow::carried::{Slot, Slots};
    use crate::borrow::{Facts, Proofs};
    use crate::flow::Flow;
    use crate::hir::{Block, Field, Program, Type};
    use crate::loans::emission_init::Event;
    use crate::loans::storage::{EventKind, ScopeKind};
    use crate::loans::{Edge, Graph, Node};

    let int = Type::Int {
        bits: 32,
        signed: true,
    };
    let program = Program {
        body: Block {
            id: 0,
            stmts: Vec::new(),
            ty: Type::Record {
                primary: Box::new(Type::Null),
                fields: vec![Field {
                    name: "n".into(),
                    ty: int.clone(),
                    mutable: false,
                }],
            },
        },
        functions: Vec::new(),
        locals: Vec::new(),
    };
    let key = (0, Some("n".into()));
    let proofs = Proofs {
        carried: Slots::from([(
            key.clone(),
            Slot {
                ty: int,
                span: Span::default(),
            },
        )]),
        ..Proofs::default()
    };
    let facts = Facts::default();
    let mut flow = Flow::default();
    let mut graph = Graph::new(&program, &facts, &proofs, &mut flow);
    let scope = graph.new_scope(ScopeKind::Block(0)).unwrap();
    let mut node = Node::default();
    graph
        .event(&mut node, EventKind::Enter(scope), Span::default())
        .unwrap();
    node.emissions.push(Event::Complete(0));
    graph.nodes[0] = node;
    assert_eq!(graph.emission_states().unwrap_err().code, "B001");
    graph.nodes[0].emissions.insert(0, Event::Emit(key.clone()));
    graph.emission_states().unwrap();
    graph.nodes[0].emissions.insert(0, Event::Emit(key.clone()));
    assert_eq!(graph.emission_states().unwrap_err().code, "B001");
    graph.nodes[0].emissions = vec![
        Event::Acquire(key.clone(), Span::default()),
        Event::Emit(key.clone()),
        Event::Complete(0),
    ];
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
    graph.guards.spend(usize::MAX);
    assert_eq!(graph.emission_states().unwrap_err().code, "B001");
}

#[test]
pub(crate) fn carried_scalar_slot_count_is_bounded_before_bitset_construction() {
    let fields = (0..65)
        .map(|id| format!("n{id}<int32>"))
        .collect::<Vec<_>>()
        .join(";");
    let emits = (0..65)
        .map(|id| format!("'out->n{id}:1;"))
        .collect::<String>();
    rejects(
        &format!(
            "<R>:<{{{fields}}}>;first:=true;i:=0;r<R>:'out{{'loop{{|first|{{{emits}first=false}};i=i+1;|i<2|'loop.restart()}}}}"
        ),
        "B001",
    );
}
