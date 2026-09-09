use super::{BTreeMap, BlockId, Diagnostic, Guards, Proofs, Result, Span, State, Type};

pub(crate) type Key = (BlockId, Option<String>);
pub(crate) type Slots = BTreeMap<Key, Slot>;
pub(crate) const MAX_SLOTS: usize = 64;
pub(crate) const MAX_PARTS: usize = 256;
pub(crate) const MAX_DEPTH: usize = 32;

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

pub(crate) fn eligible(ty: &Type, guards: &mut Guards, span: Span) -> Result<bool> {
    let mut pending = vec![(ty, 1)];
    let mut parts = 0;
    while let Some((ty, depth)) = pending.pop() {
        parts += 1;
        if parts > MAX_PARTS || depth > MAX_DEPTH || !guards.spend(1) {
            return Err(State::budget(span));
        }
        if scalar(ty) || (depth > 1 && *ty == Type::Null) {
            continue;
        }
        let Type::Record { primary, fields } = ty else {
            return Ok(false);
        };
        if fields.len() > MAX_PARTS - parts - pending.len()
            || parts + pending.len() + fields.len() + 1 > MAX_PARTS
            || !guards.spend(fields.len() + 1)
        {
            return Err(State::budget(span));
        }
        pending.push((primary, depth + 1));
        for field in fields {
            if !guards.spend(field.name.len() + 1) {
                return Err(State::budget(span));
            }
            pending.push((&field.ty, depth + 1));
        }
    }
    Ok(true)
}

pub(crate) fn validate(proofs: &Proofs, guards: &mut Guards) -> Result<()> {
    if proofs.carried.is_empty() {
        return Ok(());
    }
    if proofs.carried.len() > MAX_SLOTS || !guards.spend(proofs.carried.len() + 1) {
        return Err(State::budget(Span::default()));
    }
    for slot in proofs.carried.values() {
        if !eligible(&slot.ty, guards, slot.span)? {
            return Err(Diagnostic::unsupported(
                "carried publication outside scalar and reference-free record shapes",
                slot.span,
            ));
        }
    }
    Ok(())
}
