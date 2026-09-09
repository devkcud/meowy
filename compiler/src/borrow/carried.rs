use super::{BTreeMap, BlockId, Diagnostic, Guards, LocalId, Proofs, Result, Span, State, Type};

pub(crate) type Key = (BlockId, Option<String>);
pub(crate) type Slots = BTreeMap<Key, Slot>;
pub(crate) const MAX_SLOTS: usize = 64;
pub(crate) const MAX_PARTS: usize = 256;
pub(crate) const MAX_DEPTH: usize = 32;

pub(crate) struct Slot {
    pub(crate) ty: Type,
    pub(crate) span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    Plain,
    List,
}

pub(crate) fn scalar(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Bool | Type::Int { .. } | Type::Float { .. } | Type::String
    )
}

pub(crate) fn eligible(ty: &Type, guards: &mut Guards, span: Span) -> Result<bool> {
    Ok(shape(ty, guards, span)?.is_some())
}

pub(crate) fn shape(ty: &Type, guards: &mut Guards, span: Span) -> Result<Option<Shape>> {
    let mut pending = vec![(ty, 1)];
    let mut parts = 0;
    let mut result = Shape::Plain;
    while let Some((ty, depth)) = pending.pop() {
        parts += 1;
        if parts > MAX_PARTS || depth > MAX_DEPTH || !guards.spend(1) {
            return Err(State::budget(span));
        }
        if scalar(ty) || (depth > 1 && *ty == Type::Null) {
            continue;
        }
        if let Type::List { element, capacity } = ty {
            if *capacity > crate::list::MAX_CAPACITY {
                return Ok(None);
            }
            if parts + pending.len() >= MAX_PARTS || !guards.spend(1) {
                return Err(State::budget(span));
            }
            result = Shape::List;
            pending.push((element, depth + 1));
            continue;
        }
        let Type::Record { primary, fields } = ty else {
            return Ok(None);
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
    Ok(Some(result))
}

pub(crate) fn storage(proofs: &Proofs, id: LocalId, guards: &mut Guards, span: Span) -> Result<()> {
    if proofs.carried.is_empty() {
        return Ok(());
    }
    if !guards.spend(proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 1) {
        return Err(State::budget(span));
    }
    let Some(alias) = proofs.aliases.get(&id) else {
        return Ok(());
    };
    if !guards.spend(
        (alias.field.len() + 1)
            .saturating_mul(proofs.carried.len().checked_ilog2().unwrap_or(0) as usize + 1),
    ) {
        return Err(State::budget(span));
    }
    if let Some(slot) = proofs
        .carried
        .get(&(alias.target, Some(alias.field.clone())))
        && shape(&slot.ty, guards, span)? != Some(Shape::Plain)
    {
        return Err(Diagnostic::unsupported(
            "borrows and indexed writes of carried list storage",
            span,
        ));
    }
    Ok(())
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
                "carried publication outside scalar and reference-free record/list shapes",
                slot.span,
            ));
        }
    }
    Ok(())
}
