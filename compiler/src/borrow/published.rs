use super::{
    BTreeMap, BTreeSet, BlockId, Checker, FALSE, Guard, Guards, Result, Span, State, Step, Type,
};

pub(crate) type Slots = BTreeMap<(BlockId, usize), Slot>;
pub(crate) type Fixed = BTreeMap<BlockId, BTreeSet<BlockId>>;

pub(crate) fn fixed_weight(fixed: &Fixed) -> usize {
    fixed.values().map(|targets| targets.len() + 1).sum()
}

pub(crate) struct Slot {
    pub(crate) ty: Type,
    pub(crate) state: State,
}

pub(crate) struct Snapshot {
    pub(crate) slots: Slots,
    pub(crate) entered: Guard,
}

pub(crate) fn unchanged(
    initial: &Snapshot,
    current: &Snapshot,
    targets: &BTreeSet<BlockId>,
    guards: &mut Guards,
    span: Span,
) -> Result<Guard> {
    let count = initial.slots.len().saturating_add(current.slots.len());
    let lookup = count.checked_ilog2().unwrap_or(0) as usize
        + targets.len().checked_ilog2().unwrap_or(0) as usize
        + 3;
    if !guards.spend(count.saturating_mul(lookup) + 1) {
        return Err(State::budget(span));
    }
    if current.entered == FALSE {
        return Ok(FALSE);
    }
    if initial.entered == FALSE {
        return Ok(current.entered);
    }
    let keys = initial
        .slots
        .keys()
        .chain(current.slots.keys())
        .filter(|(owner, _)| targets.contains(owner))
        .copied()
        .collect::<BTreeSet<_>>();
    let mask = guards.and(initial.entered, current.entered);
    for key in keys {
        let (Some(left), Some(right)) = (initial.slots.get(&key), current.slots.get(&key)) else {
            return Ok(current.entered);
        };
        let left_size = crate::borrow_contract::type_weight(&left.ty, guards, span)?;
        let right_size = crate::borrow_contract::type_weight(&right.ty, guards, span)?;
        if !guards.spend(left_size + right_size + left.state.weight() + right.state.weight() + 1) {
            return Err(State::budget(span));
        }
        if left.ty != right.ty {
            return Ok(current.entered);
        }
        let shape = super::header::Shape::new(&left.ty, guards, span)?;
        shape.validate(&left.state, false, guards, span)?;
        shape.validate(&right.state, false, guards, span)?;
        let left = left.state.clone().under(mask, guards);
        let right = right.state.clone().under(mask, guards);
        if left.present != right.present
            || left.proof != right.proof
            || !left
                .origins
                .iter()
                .map(|origin| (&origin.component, &origin.source, origin.guard))
                .eq(right
                    .origins
                    .iter()
                    .map(|origin| (&origin.component, &origin.source, origin.guard)))
            || !left
                .bounds
                .iter()
                .map(|origin| (&origin.component, &origin.source, origin.guard))
                .eq(right
                    .bounds
                    .iter()
                    .map(|origin| (&origin.component, &origin.source, origin.guard)))
            || !left
                .active
                .iter()
                .map(|active| (&active.component, active.member, active.guard))
                .eq(right
                    .active
                    .iter()
                    .map(|active| (&active.component, active.member, active.guard)))
        {
            return Ok(current.entered);
        }
    }
    Ok(FALSE)
}

pub(crate) fn capture(
    slots: &Slots,
    owners: &[BlockId],
    entered: Guard,
    guards: &mut Guards,
    span: Span,
) -> Result<(Snapshot, usize)> {
    if !guards.spend(slots.len().saturating_mul(owners.len() + 1) + 1) {
        return Err(State::budget(span));
    }
    let mut result = Slots::new();
    let mut weight = 1usize;
    if entered != FALSE {
        for (key, slot) in slots {
            if !owners.contains(&key.0) {
                continue;
            }
            let size = crate::borrow_contract::type_weight(&slot.ty, guards, span)?;
            let size = size.saturating_add(slot.state.weight() + 3);
            weight = weight.saturating_add(size);
            if weight > super::MAX_FACT_ORIGINS || !guards.spend(size) {
                return Err(State::budget(span));
            }
            let state = slot.state.clone().under(entered, guards);
            let shape = super::header::Shape::new(&slot.ty, guards, span)?;
            shape.validate(&state, false, guards, span)?;
            result.insert(
                *key,
                Slot {
                    ty: slot.ty.clone(),
                    state,
                },
            );
        }
    }
    Ok((
        Snapshot {
            slots: result,
            entered,
        },
        weight,
    ))
}

impl Checker<'_> {
    pub(crate) fn publish(
        &mut self,
        target: BlockId,
        field: &Option<String>,
        value: &State,
        ty: &Type,
        span: Span,
    ) -> Result<()> {
        if !self.merging {
            return Ok(());
        }
        if !self
            .guards
            .spend(self.types.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let Some(Type::Record { fields, .. }) = self.types.get(&target) else {
            return Ok(());
        };
        if !self
            .guards
            .spend(fields.len() + field.as_ref().map_or(0, String::len) + 1)
        {
            return Err(State::budget(span));
        }
        let Some((index, field)) = fields.iter().enumerate().find(|(_, slot)| {
            field.as_ref() == Some(&slot.name) && (slot.mutable || slot.ty.has_mutable_fields())
        }) else {
            return Ok(());
        };
        let destination = &field.ty;
        if (!destination.has_reference() && !destination.has_mutable_fields())
            || !destination.fixed_borrowed_value()
            || !destination.accepts(ty)
        {
            return Ok(());
        }
        let size = crate::borrow_contract::type_weight(destination, self.guards, span)?;
        if !self.guards.spend(size + value.weight() + 1) {
            return Err(State::budget(span));
        }
        let state = value.clone().convert(ty, destination, self.guards, span)?;
        let ty = destination.clone();
        let key = (target, index + 1);
        let prior = self
            .published
            .remove(&key)
            .map(|slot| slot.state)
            .unwrap_or_else(State::absent);
        if !self.guards.spend(prior.weight() + 1) {
            return Err(State::budget(span));
        }
        let mut prior = prior.under(self.guards.not(self.assumed), self.guards);
        prior.merge(state.under(self.assumed, self.guards), self.guards, span)?;
        self.reserve_origins(size + prior.weight() + 3, span)?;
        self.published.insert(key, Slot { ty, state: prior });
        Ok(())
    }

    pub(crate) fn update_published(
        &mut self,
        target: BlockId,
        prefix: &[Step],
        value: &State,
        guard: Guard,
        span: Span,
    ) -> Result<()> {
        let [Step::Slot(index), path @ ..] = prefix else {
            return Err(Self::unsupported(span));
        };
        if !self
            .guards
            .spend(self.published.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let key = (target, *index);
        let Some(slot) = self.published.remove(&key) else {
            return Ok(());
        };
        if !self
            .guards
            .spend(slot.state.weight().saturating_mul(2) + value.weight() + path.len() + 1)
        {
            return Err(State::budget(span));
        }
        let mut outside = slot
            .state
            .clone()
            .under(self.guards.not(guard), self.guards);
        let value = value.clone().under(guard, self.guards);
        let inside = if path.is_empty() {
            value
        } else {
            slot.state
                .under(guard, self.guards)
                .replaced(path, value, self.guards, span)?
        };
        outside.merge(inside, self.guards, span)?;
        self.reserve_origins(outside.weight() + 3, span)?;
        self.published.insert(
            key,
            Slot {
                ty: slot.ty,
                state: outside,
            },
        );
        Ok(())
    }

    pub(crate) fn capture_published(&mut self, target: BlockId, span: Span) -> Result<Snapshot> {
        if !self.guards.spend(self.blocks.len() + 1) {
            return Err(State::budget(span));
        }
        let position = self
            .blocks
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Self::unsupported(span))?;
        let (snapshot, weight) = capture(
            &self.published,
            &self.blocks[..position],
            self.assumed,
            self.guards,
            span,
        )?;
        self.reserve_origins(weight, span)?;
        Ok(snapshot)
    }

    pub(crate) fn end_published(&mut self, target: BlockId, span: Span) -> Result<()> {
        if !self.guards.spend(self.published.len() + 1) {
            return Err(State::budget(span));
        }
        self.published.retain(|(id, _), _| *id != target);
        Ok(())
    }
}
