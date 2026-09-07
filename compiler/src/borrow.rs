pub(crate) mod control;
pub(crate) mod emissions;
pub(crate) mod origins;
pub(crate) mod pointee;
pub(crate) mod state;
pub(crate) mod value;
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::Span;
pub(crate) use crate::borrow_value::{Origin, Path, Projection, Source, State, Step};
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow as Guards, Guard, TRUE};
use crate::hir::{
    Block, BlockId, CallId, EmitId, Expr, ExprKind, LocalId, Program, ReborrowId, Stmt, Type,
};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Tags = BTreeMap<((LocalId, Vec<String>), Type), Vec<(Type, Guard)>>;
pub(crate) const MAX_ORIGINS: usize = 4_096;
pub(crate) const MAX_FACT_ORIGINS: usize = 262_144;

pub(crate) use emissions::slot;
pub(crate) use state::{Alias, Backing, Checker, Exit, Facts, Flow, Proofs, Storage, Value};

pub(crate) fn check(
    program: &Program,
    guards: &mut Guards,
    proofs: &Proofs,
) -> std::result::Result<Facts, Vec<Diagnostic>> {
    let mut checker = Checker {
        program,
        guards,
        proofs,
        locals: BTreeMap::new(),
        blocks: Vec::new(),
        types: BTreeMap::new(),
        results: BTreeMap::new(),
        writes: BTreeMap::new(),
        scopes: Vec::new(),
        facts: Facts::default(),
        origins: 0,
        assumed: TRUE,
        assumed_scopes: Vec::new(),
        inputs: BTreeSet::new(),
        statements: BTreeMap::new(),
    };
    let result = (|| {
        checker.block(&program.body)?;
        for function in &program.functions {
            checker.locals.clear();
            checker.assumed = TRUE;
            checker.inputs = function.params.iter().copied().collect();
            for id in &function.params {
                let span = Span { start: 0, end: 0 };
                let ty = &program.locals[*id];
                let mut state = crate::borrow_contract::input(*id, ty, checker.guards, span)?;
                checker.link_tags(*id, ty, &mut state, span)?;
                checker.complete(ty, &state, span)?;
                checker.reserve_origins(state.weight() + 1, span)?;
                checker.assumed = checker.guards.and(checker.assumed, state.proof);
                checker.facts.locals.insert(*id, state.clone());
                checker.locals.insert(
                    *id,
                    Storage {
                        block: function.body.id,
                        state,
                    },
                );
            }
            checker.block(&function.body)?;
        }
        Ok(())
    })();
    if checker.guards.exceeded() {
        Err(vec![Diagnostic::unsupported(
            "control-flow proof budget exhausted",
            Span { start: 0, end: 0 },
        )])
    } else {
        result.map(|()| checker.facts).map_err(|error| vec![error])
    }
}

#[cfg(test)]
mod tests;
