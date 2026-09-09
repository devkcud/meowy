use super::emission_value::Value;
use super::storage::{EventKind, ScopeKind};
use super::{
    BTreeMap, BlockId, Diagnostic, Expr, FALSE, Graph, LocalId, Node, Result, Source, Span,
};
use crate::borrow::carried::Key;
use std::collections::{BTreeSet, VecDeque};

pub(crate) const MAX_STATES: usize = 16_384;
pub(crate) const MAX_BOOLS: usize = 512;

#[derive(Clone)]
pub(crate) enum Event {
    Set(LocalId, Value),
    Forget,
    Emit(Key),
    Acquire(Key, Span),
    Complete(BlockId),
}

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct State {
    pub(crate) values: BTreeMap<LocalId, bool>,
    pub(crate) active: u64,
    pub(crate) initialized: u64,
}

impl Graph<'_> {
    pub(crate) fn emission_set(&mut self, node: &mut Node, id: LocalId, expr: &Expr) -> Result<()> {
        if let Some(value) = self.emission_bool(expr)? {
            node.emissions.push(Event::Set(id, value));
        }
        Ok(())
    }

    pub(crate) fn emission_finish(&mut self, node: &mut Node, id: BlockId) -> Result<()> {
        if !self.proofs.carried.is_empty() {
            self.charge(self.proofs.carried.len() + 1)?;
            if self.proofs.carried.keys().any(|(owner, _)| *owner == id) {
                node.emissions.push(Event::Complete(id));
            }
        }
        Ok(())
    }

    pub(crate) fn emission_write(
        &mut self,
        target: BlockId,
        field: &Option<String>,
        ty: &crate::hir::Type,
        span: Span,
    ) -> Result<Option<Event>> {
        if self.proofs.carried.is_empty() {
            return Ok(None);
        }
        let size = field.as_ref().map_or(0, String::len) + 1;
        self.charge(
            size.saturating_mul(
                self.proofs.carried.len().checked_ilog2().unwrap_or(0) as usize + 1,
            ),
        )?;
        let key = (target, field.clone());
        if let Some(slot) = self.proofs.carried.get(&key) {
            if slot.ty != *ty {
                return Err(Diagnostic::unsupported(
                    "carried publication type mismatch",
                    span,
                ));
            }
            return Ok(Some(Event::Emit(key)));
        }
        Ok(None)
    }

    pub(crate) fn emission_acquire(
        &mut self,
        source: &Source,
        span: Span,
    ) -> Result<Option<Event>> {
        if self.proofs.carried.is_empty() {
            return Ok(None);
        }
        let Source::Slot { target, view, .. } = source else {
            return Ok(None);
        };
        crate::borrow::carried::storage(self.proofs, *view, self.guards, span)?;
        self.charge(self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
        let alias = self.proofs.aliases.get(view).ok_or_else(Self::budget)?;
        self.charge(
            (alias.field.len() + 1).saturating_mul(
                self.proofs.carried.len().checked_ilog2().unwrap_or(0) as usize + 1,
            ),
        )?;
        let key = (*target, Some(alias.field.clone()));
        Ok(self
            .proofs
            .carried
            .contains_key(&key)
            .then_some(Event::Acquire(key, span)))
    }

    pub(crate) fn emission_states(&mut self) -> Result<()> {
        if self.proofs.carried.is_empty() {
            return Ok(());
        }
        crate::borrow::carried::validate(self.proofs, self.guards)?;
        let mut slots = BTreeMap::new();
        let mut owners = BTreeMap::<BlockId, u64>::new();
        for (index, (key, slot)) in self.proofs.carried.iter().enumerate() {
            self.charge(key.1.as_ref().map_or(0, String::len) + 3)?;
            let bit = 1u64 << index;
            slots.insert(key.clone(), (bit, slot.span));
            *owners.entry(key.0).or_default() |= bit;
        }
        let mut seen = BTreeSet::new();
        let mut pending = VecDeque::new();
        self.emission_queue(&mut seen, &mut pending, 0, State::default())?;
        while let Some((id, mut state)) = pending.pop_front() {
            let node = &self.nodes[id];
            let work = node.events.len()
                + node
                    .emissions
                    .iter()
                    .map(|event| match event {
                        Event::Set(_, value) => value.weight(),
                        Event::Acquire(key, _) => key.1.as_ref().map_or(0, String::len) + 1,
                        _ => 1,
                    })
                    .sum::<usize>()
                + node.condition.as_ref().map_or(0, Value::weight)
                + node.next.len()
                + 1;
            self.charge(work.saturating_mul(state.values.len() + 2))?;
            for event in &self.nodes[id].events {
                if let EventKind::Enter(scope) | EventKind::End(scope) = event.kind
                    && let ScopeKind::Block(owner) = self.scopes[scope.0].kind
                {
                    let bits = owners.get(&owner).copied().unwrap_or(0);
                    state.initialized &= !bits;
                    if matches!(event.kind, EventKind::Enter(_)) {
                        state.active |= bits;
                    } else {
                        state.active &= !bits;
                    }
                }
            }
            for event in &self.nodes[id].emissions {
                match event {
                    Event::Set(local, value) => {
                        if let Some(value) = value.get(&state.values) {
                            state.values.insert(*local, value);
                        } else {
                            state.values.remove(local);
                        }
                    }
                    Event::Forget => state.values.clear(),
                    Event::Emit(key) => {
                        let (bit, span) = slots.get(key).ok_or_else(Self::budget)?;
                        if state.active & bit == 0 || state.initialized & bit != 0 {
                            return Err(Diagnostic::unsupported(
                                "carried result may be initialized more than once",
                                *span,
                            ));
                        }
                        state.initialized |= bit;
                    }
                    Event::Acquire(key, span) => {
                        let (bit, _) = slots.get(key).ok_or_else(Self::budget)?;
                        if state.active & bit == 0 || state.initialized & bit == 0 {
                            return Err(Diagnostic::unsupported(
                                "carried result is not proved initialized before borrowing",
                                *span,
                            ));
                        }
                    }
                    Event::Complete(owner) => {
                        let bits = owners.get(owner).copied().ok_or_else(Self::budget)?;
                        if state.active & bits != bits || state.initialized & bits != bits {
                            let span = slots
                                .iter()
                                .find(|(key, _)| key.0 == *owner)
                                .map(|(_, (_, span))| *span)
                                .unwrap_or_default();
                            return Err(Diagnostic::unsupported(
                                "carried result is not proved initialized on every completing path",
                                span,
                            ));
                        }
                        state.active &= !bits;
                    }
                }
            }
            let node = &self.nodes[id];
            if node.condition.is_some() && node.next.len() != 2 {
                return Err(Self::budget());
            }
            let edges = node.next.clone();
            let condition = node.condition.clone();
            for (index, edge) in edges.into_iter().enumerate() {
                if edge.guard == FALSE {
                    continue;
                }
                self.charge(state.values.len() + 3)?;
                let mut next = state.clone();
                if let Some(condition) = &condition
                    && !condition.refine(&mut next.values, index == 0)
                {
                    continue;
                }
                self.emission_queue(&mut seen, &mut pending, edge.target, next)?;
            }
        }
        Ok(())
    }

    pub(crate) fn emission_queue(
        &mut self,
        seen: &mut BTreeSet<(usize, State)>,
        pending: &mut VecDeque<(usize, State)>,
        id: usize,
        state: State,
    ) -> Result<()> {
        let work = (state.values.len() + 3)
            .saturating_mul(seen.len().checked_ilog2().unwrap_or(0) as usize + 2);
        self.charge(work)?;
        if id >= self.nodes.len() || state.values.len() > MAX_BOOLS {
            return Err(Self::budget());
        }
        let key = (id, state);
        if !seen.contains(&key) {
            if seen.len() >= MAX_STATES {
                return Err(Self::budget());
            }
            seen.insert(key.clone());
            pending.push_back(key);
        }
        Ok(())
    }
}
