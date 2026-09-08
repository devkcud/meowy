use super::{BTreeSet, BlockId, Checker, FALSE, LocalId, Result, Span, State};
use crate::borrow::Backing;

pub(crate) type Views = super::BTreeMap<BlockId, BTreeSet<LocalId>>;

pub(crate) fn weight(views: &Views) -> usize {
    views.values().map(|ids| ids.len() + 1).sum()
}

impl Checker<'_> {
    pub(crate) fn changing_header(
        &mut self,
        id: BlockId,
        input: &super::state::Predecessor,
        span: Span,
    ) -> Result<()> {
        let Some(ids) = self.facts.changing_published.get(&id) else {
            return Ok(());
        };
        let lookup = input.values.len().checked_ilog2().unwrap_or(0) as usize
            + 3 * self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize
            + self.proofs.mutable.len().checked_ilog2().unwrap_or(0) as usize
            + self.types.len().checked_ilog2().unwrap_or(0) as usize
            + self
                .facts
                .published_inputs
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize
            + 8;
        if !self.guards.spend(ids.len().saturating_mul(lookup) + 1) {
            return Err(State::budget(span));
        }
        let mut slots = BTreeSet::new();
        for local in ids {
            let alias = self
                .proofs
                .aliases
                .get(local)
                .ok_or_else(|| Self::unsupported(span))?;
            let state = input.values.get(local).ok_or_else(|| {
                super::Diagnostic::unsupported(
                    "published alias must be initialized before restart entry",
                    alias.span,
                )
            })?;
            let ty = self
                .program
                .locals
                .get(*local)
                .ok_or_else(|| Self::unsupported(span))?;
            if !alias.mutable
                || alias.backing != Some(Backing::Result)
                || !self.proofs.mutable.contains(local)
                || !ty.has_reference()
                || !ty.fixed_borrowed_value()
            {
                return Err(Self::unsupported(alias.span));
            }
            let target = self
                .types
                .get(&alias.target)
                .ok_or_else(|| Self::unsupported(span))?;
            crate::borrow_contract::type_weight(target, self.guards, span)?;
            let (prefix, _) = super::aliases::result_slot(target, &alias.field, ty)
                .ok_or_else(|| Self::unsupported(alias.span))?;
            let Some(super::Step::Slot(index)) = prefix.first() else {
                return Err(Self::unsupported(span));
            };
            if !self.guards.spend(state.weight() + slots.len() + 1)
                || !slots.insert((alias.target, *index))
            {
                return Err(State::budget(span));
            }
            let present = self.guards.and(state.present, state.proof);
            if !self.guards.implies(input.entered, present)
                || self
                    .facts
                    .published_inputs
                    .get(&id)
                    .and_then(|input| input.slots.get(&(alias.target, *index)))
                    .is_none()
            {
                return Err(Self::unsupported(alias.span));
            }
        }
        Ok(())
    }

    pub(crate) fn refresh_published(&mut self, id: BlockId, span: Span) -> Result<()> {
        if self.assumed == FALSE {
            return Ok(());
        }
        if !self.guards.spend(
            self.facts
                .refresh_published
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize
                + 1,
        ) {
            return Err(State::budget(span));
        }
        let Some(ids) = self.facts.refresh_published.get(&id) else {
            return Ok(());
        };
        if !self.guards.spend(ids.len() + 1) {
            return Err(State::budget(span));
        }
        let ids = ids.iter().copied().collect::<Vec<_>>();
        for local in ids {
            if !self
                .guards
                .spend(self.locals.len().checked_ilog2().unwrap_or(0) as usize + 1)
            {
                return Err(State::budget(span));
            }
            let Some(storage) = self.locals.get(&local) else {
                continue;
            };
            if !self.guards.spend(storage.state.weight() + 1) {
                return Err(State::budget(span));
            }
            let state = storage.state.clone();
            if let Some(value) = self.alias_value(local, &[], &state, span)? {
                self.write_alias(value, span)?;
            }
        }
        Ok(())
    }

    pub(crate) fn finish_published(
        &mut self,
        id: BlockId,
        mut state: State,
        span: Span,
    ) -> Result<State> {
        if !self.guards.spend(
            self.facts
                .refresh_published
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize
                + self.blocks.len()
                + 2,
        ) {
            return Err(State::budget(span));
        }
        let Some(ids) = self.facts.refresh_published.get(&id) else {
            return Ok(state);
        };
        let lookup = self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 1;
        if !self.guards.spend(ids.len().saturating_mul(lookup) + 1) {
            return Err(State::budget(span));
        }
        if !ids.iter().any(|local| {
            self.proofs
                .aliases
                .get(local)
                .is_some_and(|alias| alias.target == id)
        }) {
            return Ok(state);
        }
        let target = self
            .blocks
            .iter()
            .position(|block| *block == id)
            .ok_or_else(|| Self::unsupported(span))?;
        let active = self.guards.and(state.present, state.proof);
        state.origins = self.retained(state.origins, active, target, span)?;
        state.bounds = self.retained(state.bounds, active, target, span)?;
        Ok(state)
    }
}
