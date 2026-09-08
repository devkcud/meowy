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
    if proofs.carried.len() > MAX_SLOTS || !guards.spend(proofs.carried.len() + 1) {
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
    Ok(())
}
