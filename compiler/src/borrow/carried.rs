use super::{BTreeMap, BlockId, Diagnostic, Guards, Proofs, Result, Span, State, Type};

pub(crate) type Key = (BlockId, Option<String>);
pub(crate) type Slots = BTreeMap<Key, Slot>;
pub(crate) const MAX_SLOTS: usize = 64;

pub(crate) struct Slot {
    pub(crate) ty: Type,
    pub(crate) span: Span,
}

pub(crate) fn scalar(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Bool | Type::Int { .. } | Type::Float { .. } | Type::String
    )
}

pub(crate) fn validate(proofs: &Proofs, guards: &mut Guards) -> Result<()> {
    if proofs.carried.is_empty() {
        return Ok(());
    }
    let lookup = proofs.carried.len().checked_ilog2().unwrap_or(0) as usize + 1;
    if proofs.carried.len() > MAX_SLOTS
        || !guards.spend(proofs.aliases.len().saturating_mul(lookup) + proofs.carried.len() + 1)
    {
        return Err(State::budget(Span::default()));
    }
    for slot in proofs.carried.values() {
        if !scalar(&slot.ty) {
            return Err(Diagnostic::unsupported(
                "non-scalar carried publication",
                slot.span,
            ));
        }
    }
    for alias in proofs.aliases.values() {
        if !guards.spend((alias.field.len() + 1).saturating_mul(lookup)) {
            return Err(State::budget(alias.span));
        }
        if proofs
            .carried
            .contains_key(&(alias.target, Some(alias.field.clone())))
            && let Some(span) = alias.exclusive
        {
            return Err(Diagnostic::unsupported(
                "exclusively borrowing carried publication storage",
                span,
            ));
        }
    }
    Ok(())
}
