pub(crate) mod control;
pub(crate) mod graph;
pub(crate) mod solve;
pub(crate) mod state;
pub(crate) mod values;
use std::collections::{BTreeMap, VecDeque};

use crate::ast::Span;
use crate::borrow::{Facts, Origin, Path, Projection, Proofs, Source, Step};
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow, Guard, TRUE};
use crate::hir::{Block, BlockId, CallId, Expr, ExprKind, LocalId, Place, Program, Stmt, Type};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Live = BTreeMap<usize, Guard>;
pub(crate) type Bundle = BTreeMap<Path, usize>;
pub(crate) const MAX_NODES: usize = 65_536;
pub(crate) const MAX_VALUES: usize = 65_536;
pub(crate) const MAX_WORK: usize = 1_048_576;
pub(crate) const MAX_LIVE: usize = 262_144;
pub(crate) const MAX_ORIGINS: usize = 262_144;

pub(crate) use state::{Edge, Graph, Node, Scope};

pub(crate) fn check(
    program: &Program,
    facts: &Facts,
    proofs: &Proofs,
    guards: &mut Flow,
) -> std::result::Result<(), Vec<Diagnostic>> {
    let result = (|| {
        Graph::new(program, facts, proofs, guards).check(&program.body, &[])?;
        for function in &program.functions {
            Graph::new(program, facts, proofs, guards).check(&function.body, &function.params)?;
        }
        Ok(())
    })();
    if guards.exceeded() {
        Err(vec![Diagnostic::unsupported(
            "control-flow proof budget exhausted",
            Span { start: 0, end: 0 },
        )])
    } else {
        result.map_err(|error| vec![error])
    }
}

#[cfg(test)]
mod tests;
