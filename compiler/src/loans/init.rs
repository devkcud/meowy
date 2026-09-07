use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::storage::{EventKind, Key};
use super::{Diagnostic, FALSE, Graph, Guard, MAX_LIVE, Result, TRUE};

pub(crate) type Values = BTreeMap<Key, Availability>;
pub(crate) type Needs = BTreeSet<Key>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Availability {
    pub(crate) ready: Guard,
    pub(crate) moved: Guard,
    pub(crate) empty: Guard,
    pub(crate) ended: Guard,
}

impl Availability {
    pub(crate) fn ended(guard: Guard) -> Self {
        Self {
            ended: guard,
            ..Self::default()
        }
    }
}

impl Graph<'_> {
    pub(crate) fn storage_needs(&mut self, reach: &[Guard]) -> Result<Vec<Needs>> {
        self.charge(self.nodes.len())?;
        let mut before = vec![Vec::new(); self.nodes.len()];
        for (id, node) in self.nodes.iter().enumerate() {
            for edge in &node.next {
                before[edge.target].push(id);
            }
        }
        let mut needed = vec![Needs::new(); self.nodes.len()];
        let mut queue: VecDeque<_> = (0..self.nodes.len()).rev().collect();
        let mut queued = vec![true; self.nodes.len()];
        let mut entries = 0usize;
        while let Some(id) = queue.pop_front() {
            queued[id] = false;
            if reach[id] == FALSE {
                continue;
            }
            self.charge(self.nodes[id].next.len() + self.nodes[id].events.len() + 1)?;
            let mut next = Needs::new();
            let edges = self.nodes[id].next.clone();
            for edge in edges {
                if edge.guard != FALSE && reach[edge.target] != FALSE {
                    let work = needed[edge.target]
                        .len()
                        .saturating_mul(next.len().checked_ilog2().unwrap_or(0) as usize + 1);
                    self.charge(work)?;
                    if needed[edge.target]
                        .len()
                        .saturating_add(next.len())
                        .saturating_add(entries)
                        > MAX_LIVE
                    {
                        return Err(Self::budget());
                    }
                    next.extend(needed[edge.target].iter().copied());
                    if next.len().saturating_add(entries) > MAX_LIVE {
                        return Err(Self::budget());
                    }
                }
            }
            self.charge(next.len().saturating_mul(self.nodes[id].events.len() + 1))?;
            for event in self.nodes[id].events.iter().rev() {
                match event.kind {
                    EventKind::Enter(scope) => next.retain(|key| !self.owns(scope, key)),
                    EventKind::End(scope) => {
                        next.retain(|key| !self.owns(scope, key));
                        if self.scopes[scope.0].cells != 0 {
                            next.insert(Key::Scope(scope));
                        }
                    }
                    EventKind::Init(cell) => {
                        next.remove(&Key::Cell(cell));
                        let store = self.stores.get(&cell).ok_or_else(Self::budget)?;
                        next.insert(Key::Scope(store.scope));
                    }
                    EventKind::Use { id, .. } => {
                        next.insert(Key::Cell(id));
                    }
                }
            }
            if next != needed[id] {
                entries = entries - needed[id].len() + next.len();
                if entries > MAX_LIVE {
                    return Err(Self::budget());
                }
                needed[id] = next;
                for prior in &before[id] {
                    if !queued[*prior] {
                        queued[*prior] = true;
                        queue.push_back(*prior);
                    }
                }
            }
        }
        Ok(needed)
    }

    pub(crate) fn require_ready(
        &mut self,
        state: Availability,
        entered: Guard,
        span: super::Span,
    ) -> Result<()> {
        self.charge(1)?;
        let other = self.guards.or(state.empty, state.ended);
        let bad = self.guards.or(state.moved, other);
        if !self.guards.overlap(entered, bad) && self.guards.implies(entered, state.ready) {
            return Ok(());
        }
        let possible = self.guards.or(state.ready, other);
        if self.guards.implies(entered, state.moved) && !self.guards.overlap(entered, possible) {
            return Err(Diagnostic::new(
                "E301",
                "storage was moved before this use",
                span,
            ));
        }
        Err(Diagnostic::new(
            "E309",
            "storage may be uninitialized or outside its lifetime",
            span,
        ))
    }

    pub(crate) fn storage_output(
        &mut self,
        id: usize,
        entered: Guard,
        needed: &[Needs],
        validate: bool,
    ) -> Result<Values> {
        let work = self.availability[id].len().saturating_mul(5);
        self.charge(work + self.nodes[id].events.len() + self.nodes[id].next.len() + 1)?;
        if work > MAX_LIVE {
            return Err(Self::budget());
        }
        let mut values = self.availability[id].clone();
        let mut wanted = Needs::new();
        let edges = self.nodes[id].next.clone();
        for edge in edges {
            if edge.guard != FALSE {
                self.charge(
                    needed[edge.target]
                        .len()
                        .saturating_mul(wanted.len().checked_ilog2().unwrap_or(0) as usize + 1),
                )?;
                if wanted.len().saturating_add(needed[edge.target].len()) > MAX_LIVE {
                    return Err(Self::budget());
                }
                wanted.extend(needed[edge.target].iter().copied());
                if wanted.len() > MAX_LIVE {
                    return Err(Self::budget());
                }
            }
        }
        self.charge(wanted.len() + values.len() + self.nodes[id].events.len())?;
        if wanted
            .len()
            .saturating_add(values.len())
            .saturating_add(self.nodes[id].events.len())
            > MAX_LIVE
        {
            return Err(Self::budget());
        }
        let mut touched = wanted.clone();
        touched.extend(values.keys().copied());
        for event in &self.nodes[id].events {
            match event.kind {
                EventKind::Use { id, .. } => {
                    touched.insert(Key::Cell(id));
                }
                EventKind::Init(id) => {
                    touched.insert(Key::Scope(self.stores[&id].scope));
                }
                EventKind::End(scope) => {
                    if self.scopes[scope.0].cells != 0 {
                        touched.insert(Key::Scope(scope));
                    }
                }
                EventKind::Enter(_) => {}
            }
        }
        self.charge(
            touched
                .len()
                .saturating_mul(self.nodes[id].events.len() + 1),
        )?;
        let events = self.nodes[id].events.clone();
        for event in events {
            match event.kind {
                EventKind::Enter(scope) => {
                    for key in &touched {
                        if self.owns(scope, key) {
                            let state = match key {
                                Key::Scope(_) => Availability {
                                    ready: entered,
                                    ..Availability::default()
                                },
                                Key::Cell(_) => Availability {
                                    empty: entered,
                                    ..Availability::default()
                                },
                            };
                            values.insert(*key, state);
                        }
                    }
                }
                EventKind::End(scope) => {
                    if validate && self.scopes[scope.0].cells != 0 {
                        self.require_ready(
                            values
                                .get(&Key::Scope(scope))
                                .copied()
                                .unwrap_or_else(|| Availability::ended(entered)),
                            entered,
                            event.span,
                        )?;
                    }
                    for key in &touched {
                        if self.owns(scope, key) {
                            values.insert(*key, Availability::ended(entered));
                        }
                    }
                }
                EventKind::Init(cell) => {
                    if validate {
                        let owner =
                            Key::Scope(self.stores.get(&cell).ok_or_else(Self::budget)?.scope);
                        self.require_ready(
                            values
                                .get(&owner)
                                .copied()
                                .unwrap_or_else(|| Availability::ended(entered)),
                            entered,
                            event.span,
                        )?;
                    }
                    if touched.contains(&Key::Cell(cell)) {
                        values.insert(
                            Key::Cell(cell),
                            Availability {
                                ready: entered,
                                ..Availability::default()
                            },
                        );
                    }
                }
                EventKind::Use { id, take } => {
                    let state = values
                        .get(&Key::Cell(id))
                        .copied()
                        .unwrap_or_else(|| Availability::ended(entered));
                    if validate {
                        self.require_ready(state, entered, event.span)?
                    }
                    if take && !self.stores.get(&id).ok_or_else(Self::budget)?.copy {
                        values.insert(
                            Key::Cell(id),
                            Availability {
                                moved: entered,
                                ..Availability::default()
                            },
                        );
                    }
                }
            }
        }
        for key in &wanted {
            values
                .entry(*key)
                .or_insert_with(|| Availability::ended(entered));
        }
        values.retain(|key, _| wanted.contains(key));
        Ok(values)
    }

    pub(crate) fn solve_init(&mut self, reach: &[Guard]) -> Result<()> {
        if !self.availability.is_empty() || reach.len() != self.nodes.len() {
            return Err(Self::budget());
        }
        let needed = self.storage_needs(reach)?;
        let base = needed.iter().map(Needs::len).sum::<usize>();
        self.charge(self.nodes.len() + base)?;
        self.availability.resize_with(self.nodes.len(), Values::new);
        let mut seen = vec![FALSE; self.nodes.len()];
        seen[0] = TRUE;
        for key in &needed[0] {
            self.availability[0].insert(*key, Availability::ended(TRUE));
        }
        let mut entries = self.availability[0].len().saturating_mul(5) + base;
        if entries > MAX_LIVE {
            return Err(Self::budget());
        }
        let mut queue = VecDeque::from([0usize]);
        let mut queued = vec![false; self.nodes.len()];
        queued[0] = true;
        while let Some(id) = queue.pop_front() {
            queued[id] = false;
            self.charge(1)?;
            let output = self.storage_output(id, seen[id], &needed, false)?;
            if output.len().saturating_mul(5).saturating_add(entries) > MAX_LIVE {
                return Err(Self::budget());
            }
            let edges = self.nodes[id].next.clone();
            for edge in edges {
                if edge.guard == FALSE || reach[edge.target] == FALSE {
                    continue;
                }
                let entered = if edge.reset {
                    TRUE
                } else {
                    self.guards.and(seen[id], edge.guard)
                };
                if entered == FALSE {
                    continue;
                }
                let merged = self.guards.or(seen[edge.target], entered);
                let mut changed = merged != seen[edge.target];
                seen[edge.target] = merged;
                for key in &needed[edge.target] {
                    self.charge(
                        self.availability[edge.target]
                            .len()
                            .checked_ilog2()
                            .unwrap_or(0) as usize
                            + 5,
                    )?;
                    let source = output
                        .get(key)
                        .copied()
                        .unwrap_or_else(|| Availability::ended(seen[id]));
                    let prior = self.availability[edge.target].get(key).copied();
                    let mut next = prior.unwrap_or_default();
                    for (source, target) in [
                        (source.ready, &mut next.ready),
                        (source.moved, &mut next.moved),
                        (source.empty, &mut next.empty),
                        (source.ended, &mut next.ended),
                    ] {
                        let guard = if edge.reset {
                            if source == FALSE { FALSE } else { TRUE }
                        } else {
                            self.guards.and(source, edge.guard)
                        };
                        *target = self.guards.or(*target, guard);
                    }
                    if prior != Some(next) {
                        if prior.is_none() {
                            entries += 5;
                            if entries > MAX_LIVE {
                                return Err(Self::budget());
                            }
                        }
                        self.availability[edge.target].insert(*key, next);
                        changed = true;
                    }
                }
                if changed && !queued[edge.target] {
                    queued[edge.target] = true;
                    queue.push_back(edge.target);
                }
            }
        }
        for (id, entered) in seen.iter().enumerate() {
            if *entered != reach[id] {
                return Err(Self::budget());
            }
            if *entered != FALSE {
                self.storage_output(id, *entered, &needed, true)?;
            }
        }
        Ok(())
    }
}
