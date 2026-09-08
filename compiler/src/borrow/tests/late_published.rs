use super::{accepts, rejects};
use crate::borrow::{Facts, Proofs};
use crate::flow::Flow;
use crate::hir::Program;

pub(crate) fn lower(source: &str) -> (Program, Proofs, Flow) {
    let tree = crate::parser::parse(source).unwrap();
    let mut checker = crate::check::Checker::new();
    let body = checker.block(&tree, None, None).unwrap();
    checker.proofs.conditions = checker.guards;
    checker.proofs.tags = checker.tags;
    (
        Program {
            body,
            functions: checker.functions.into_iter().flatten().collect(),
            locals: checker.locals,
        },
        checker.proofs,
        checker.flow,
    )
}

pub(crate) fn analyze(source: &str) -> (Facts, Proofs) {
    let (program, proofs, mut flow) = lower(source);
    let facts = crate::borrow::check(&program, &mut flow, &proofs)
        .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
    crate::loans::check(&program, &facts, &proofs, &mut flow)
        .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
    (facts, proofs)
}

#[test]
pub(crate) fn late_published_aliases_initialize_only_on_completing_iterations() {
    accepts("x:=1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y}};x=3;v:*r.p");
    accepts(
        "x:1;y:2;n:=3;r:'out{'loop{n=n-1;|n>1|'loop.restart();|n>0|'loop.restart();'out->p:=&x;p=&y}};v:*r.p",
    );
    accepts("x:1;y:2;n:=0;r:'out{'loop{n=n+1;|n<2|'loop.restart();{'out->p:=&x;p=&y}}};v:*r.p");
}

#[test]
pub(crate) fn late_published_aliases_keep_nullable_widened_and_record_backing() {
    accepts(
        "<R>:<{p<&int32><null><string>:=}>;x:1;n:=2;r<R>:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p<&int32><null>:=null;p=&x}};copy:r",
    );
    accepts(
        "x:1;y:2;flag:=true;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();|flag|{'out->p:=&x;p=&y}}};copy:r",
    );
    accepts(
        "x:1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->c:={->p:=&x;->q:=&y;->n:=0};c.p={c.q=&x;->&y};c.n=3}};copy:r",
    );
}

#[test]
pub(crate) fn late_published_rhs_leave_and_cell_lifetimes_keep_their_owner() {
    accepts(
        "x:1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p={p=&y;'out.leave();->&x}}};v:*r.p",
    );
    accepts(
        "x:1;y:2;n:=2;r:'out{cell:'cell{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y;'cell->&p}};v:**cell};v:*r.p",
    );
    accepts("x:1;y:2;n:=2;r:'out{'loop{'out->p:={n=n-1;|n>0|'loop.restart();->&x};p=&y}};v:*r.p");
    accepts("x:1;y:2;n:=2;r:'out{'loop{'out->p:={->&x;n=n-1;|n>0|'loop.restart()};p=&y}};v:*r.p");
    rejects(
        "x:1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y;'out->cell:&p}}",
        "E303",
    );
}

#[test]
pub(crate) fn late_published_aliases_preserve_old_copies_loans_and_bounds() {
    rejects(
        "x:=1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;old:p;p=&y;x=3;v:*old}}",
        "E302",
    );
    rejects(
        "x:1;y:=2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y}};y=3;v:*r.p",
        "E302",
    );
    rejects(
        "x:1;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;local:2;p=&local}};v:*r.p",
        "E303",
    );
    rejects(
        "first<&int32>:(p<&int32>,s<&string>){->p};x:1;s:=\"a\";n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=first(&x,&s)}};s=\"b\";v:*r.p",
        "E302",
    );
}

#[test]
pub(crate) fn late_published_frontiers_keep_duplicate_and_cross_edge_initialization_gates() {
    rejects(
        "x:1;y:2;n:=2;r:'out{'loop{'out->p:=&x;p=&y;n=n-1;|n>0|'loop.restart()}}",
        "B001",
    );
    rejects(
        "x:1;y:2;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:=&x;p=&y;'out->p:=&x}}",
        "E205",
    );
    rejects(
        "x:1;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();'out->p:&x;p=&x}}",
        "E305",
    );
}

#[test]
pub(crate) fn late_published_aliases_stay_out_of_initial_and_backedge_headers() {
    let (facts, proofs) = analyze(
        "x:1;y:2;n:=3;r:'out{'loop{n=n-1;|n>1|'loop.restart();|n>0|'loop.restart();'out->p:=&x;p=&y}};v:*r.p",
    );
    assert_eq!(facts.late_published.len(), 1);
    let (target, ids) = facts.late_published.iter().next().unwrap();
    for id in ids {
        assert!(!facts.headers[target].contains_key(id));
        assert!(!facts.header_inputs[target].values.contains_key(id));
        for (site, frontier) in &proofs.frontiers {
            if frontier.target == *target {
                assert!(!facts.restart_inputs[site].values.contains_key(id));
            }
        }
    }
}

#[test]
pub(crate) fn late_published_aliases_can_be_preinitialized_for_nested_restarts() {
    let (facts, _) = analyze(
        "x:1;y:2;n:=2;m:=0;r:'out{'outer{n=n-1;|n>0|'outer.restart();'out->p:=&x;'inner{p=&y;m=m+1;|m<2|'inner.restart()}}};v:*r.p",
    );
    assert_eq!(facts.changing_published.len(), 2);
    assert_eq!(facts.late_published.len(), 1);
    for (target, ids) in &facts.changing_published {
        for id in ids {
            assert_eq!(
                facts.headers[target].contains_key(id),
                !facts.late_published.contains_key(target)
            );
        }
    }
}

#[test]
pub(crate) fn late_published_markers_count_dead_emissions_separately_from_live_writes() {
    let (facts, _) = analyze(
        "x:1;n:=2;r:{|false|->skip:0;->p:=&x;'loop{p=&x;n=n-1;|n>0|'loop.restart()}};v:*r.p",
    );
    assert!(facts.late_published.is_empty());
    accepts(
        "x:1;n:=2;r:'out{'loop{n=n-1;|n>0|'loop.restart();|false|'out->skip:0;'out->p:=&x;p=&x}};v:*r.p",
    );
}

#[test]
pub(crate) fn late_published_planning_requires_every_actual_restart_certificate() {
    let (program, mut proofs, mut flow) = lower(
        "x:1;y:2;n:=3;r:'out{'loop{n=n-1;|n>1|'loop.restart();|n>0|'loop.restart();'out->p:=&x;p=&y}};v:*r.p",
    );
    assert_eq!(proofs.frontiers.len(), 2);
    let site = *proofs.frontiers.keys().next().unwrap();
    proofs.frontiers.remove(&site);
    let errors = crate::borrow::check(&program, &mut flow, &proofs)
        .err()
        .unwrap();
    assert_eq!(errors[0].code, "B001");
    assert!(errors[0].message.contains("frontier"));
}

#[test]
pub(crate) fn late_published_certificates_check_all_edges_indices_and_work_limits() {
    use crate::ast::Span;
    use crate::borrow::frontier::{Frontier, late};
    use crate::borrow::{Alias, Backing};
    use crate::flow::{FALSE, TRUE};
    use std::collections::BTreeMap;

    let mut flow = Flow::default();
    let guard = flow.fresh();
    let mut proofs = Proofs::default();
    proofs.aliases.insert(
        7,
        Alias {
            target: 1,
            field: "p".into(),
            mutable: true,
            emission: 4,
            root: 7,
            span: Span::default(),
            backing: Some(Backing::Result),
            borrowed: None,
            exclusive: None,
        },
    );
    proofs.bindings.insert(7, guard);
    proofs.emissions.insert(4, guard);
    proofs.frontiers = BTreeMap::from([
        (
            0,
            Frontier {
                target: 2,
                first: 3,
                entered: flow.not(guard),
            },
        ),
        (
            1,
            Frontier {
                target: 2,
                first: 3,
                entered: FALSE,
            },
        ),
    ]);
    assert!(late(&proofs, 2, 7, &mut flow, Span::default()).unwrap());
    proofs.frontiers.get_mut(&1).unwrap().entered = guard;
    assert_eq!(
        late(&proofs, 2, 7, &mut flow, Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
    proofs.frontiers.get_mut(&1).unwrap().entered = FALSE;
    proofs.frontiers.get_mut(&1).unwrap().first = 2;
    assert_eq!(
        late(&proofs, 2, 7, &mut flow, Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
    for frontier in proofs.frontiers.values_mut() {
        frontier.first = 5;
        frontier.entered = TRUE;
    }
    assert!(!late(&proofs, 2, 7, &mut flow, Span::default()).unwrap());
    proofs.frontiers.clear();
    assert_eq!(
        late(&proofs, 2, 7, &mut flow, Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
    flow.spend(usize::MAX);
    assert_eq!(
        late(&proofs, 2, 7, &mut flow, Span::default())
            .unwrap_err()
            .code,
        "B001"
    );
}
