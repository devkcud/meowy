use crate::check::Checker;
use crate::hir::{BlockId, ExprKind, RestartId, Stmt};
use std::collections::BTreeSet;

pub(crate) fn sites(stmts: &[Stmt], found: &mut Vec<(BlockId, RestartId)>) {
    for stmt in stmts {
        match stmt {
            Stmt::Restart { target, site } => found.push((*target, *site)),
            Stmt::Statement { stmts, .. } => sites(stmts, found),
            Stmt::If {
                then, otherwise, ..
            } => {
                sites(then, found);
                sites(otherwise, found);
            }
            Stmt::Expr(value) => {
                if let ExprKind::Block(block) = &value.kind {
                    sites(&block.stmts, found);
                }
            }
            _ => {}
        }
    }
}

#[test]
pub(crate) fn restart_sites_are_unique_across_targets_functions_and_aliases() {
    let source = "'outer{|false|'outer.restart();'inner{again:'outer.restart;|false|again();|false|'inner.restart()}};f<null>:()'loop{|false|'loop.restart()};g<null>:()'loop{again:'loop.restart;|false|again()}";
    let program = crate::compile(source).unwrap();
    let mut found = Vec::new();
    sites(&program.body.stmts, &mut found);
    for function in &program.functions {
        sites(&function.body.stmts, &mut found);
    }
    assert_eq!(found.len(), 5);
    assert_eq!(found[0].0, found[1].0);
    assert_ne!(found[0].0, found[2].0);
    assert_eq!(
        found
            .iter()
            .map(|(_, site)| *site)
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );
}

#[test]
pub(crate) fn restart_site_exhaustion_does_not_reuse_an_id() {
    let tree =
        crate::parser::parse("'loop{|false|'loop.restart();|false|'loop.restart()}").unwrap();
    let mut checker = Checker::new();
    checker.restarts = 65_535;
    let error = checker.block(&tree, None, None).unwrap_err();
    assert_eq!(error.code, "B001");
    assert!(error.message.contains("restart site budget"));
    assert_eq!(checker.restarts, 65_536);
}
