use super::{BlockId, Diagnostic, EmitId, Guard, Guards, LocalId, Proofs, Result, Span, State};

pub(crate) struct Frontier {
    pub(crate) target: BlockId,
    pub(crate) first: EmitId,
    pub(crate) entered: Guard,
}

pub(crate) fn late(
    proofs: &Proofs,
    target: BlockId,
    local: LocalId,
    guards: &mut Guards,
    span: Span,
) -> Result<bool> {
    let lookup = proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize
        + proofs.bindings.len().checked_ilog2().unwrap_or(0) as usize
        + proofs.emissions.len().checked_ilog2().unwrap_or(0) as usize
        + 4;
    if !guards.spend(lookup + proofs.frontiers.len() + 1) {
        return Err(State::budget(span));
    }
    let missing = || Diagnostic::unsupported("missing published initialization frontier", span);
    let alias = proofs.aliases.get(&local).ok_or_else(missing)?;
    if alias.target == target {
        return Err(missing());
    }
    let emitted = proofs
        .emissions
        .get(&alias.emission)
        .copied()
        .ok_or_else(missing)?;
    let bound = proofs.bindings.get(&local).copied().ok_or_else(missing)?;
    if !guards.implies(emitted, bound) {
        return Err(missing());
    }
    let mut first = None;
    for frontier in proofs
        .frontiers
        .values()
        .filter(|frontier| frontier.target == target)
    {
        if !guards.spend(3) {
            return Err(State::budget(span));
        }
        if first.is_some_and(|first| first != frontier.first) {
            return Err(missing());
        }
        first = Some(frontier.first);
        if alias.emission >= frontier.first && guards.overlap(emitted, frontier.entered) {
            return Err(Diagnostic::unsupported(
                "published alias initialization reaches a restart edge",
                alias.span,
            ));
        }
    }
    let first = first.ok_or_else(missing)?;
    Ok(alias.emission >= first)
}
