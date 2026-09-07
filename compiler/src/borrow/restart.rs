use super::{
    BTreeMap, BTreeSet, BlockId, Checker, Diagnostic, Guards, LocalId, Origin, Result, Source,
    Span, State, TRUE,
};

pub(crate) type Header = BTreeMap<LocalId, State>;
pub(crate) type Headers = BTreeMap<BlockId, Header>;

pub(crate) fn weight(headers: &Headers) -> usize {
    headers
        .values()
        .map(|header| {
            header
                .values()
                .map(|state| state.weight() + 1)
                .sum::<usize>()
                + 1
        })
        .sum()
}

pub(crate) fn same(a: &Headers, b: &Headers, guards: &mut Guards, span: Span) -> Result<bool> {
    if !guards.spend(weight(a) + weight(b) + 1) {
        return Err(State::budget(span));
    }
    if !a.keys().eq(b.keys()) {
        return Ok(false);
    }
    for (id, left) in a {
        if !guards.spend(b.len().checked_ilog2().unwrap_or(0) as usize + 1) {
            return Err(State::budget(span));
        }
        let right = &b[id];
        if !left.keys().eq(right.keys()) {
            return Ok(false);
        }
        for (id, left) in left {
            if !guards.spend(right.len().checked_ilog2().unwrap_or(0) as usize + 1) {
                return Err(State::budget(span));
            }
            let right = &right[id];
            if !left
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
            {
                return Ok(false);
            }
            if !left
                .active
                .iter()
                .map(|active| (&active.component, active.member, active.guard))
                .eq(right
                    .active
                    .iter()
                    .map(|active| (&active.component, active.member, active.guard)))
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

impl Checker<'_> {
    pub(crate) fn restart_source(
        &mut self,
        source: &Source,
        target: BlockId,
        span: Span,
    ) -> Result<Source> {
        if !self.guards.spend(self.blocks.len().saturating_mul(2) + 1) {
            return Err(State::budget(span));
        }
        let position = self
            .blocks
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Self::unsupported(span))?;
        match self.live(source, span) {
            Ok(None) => Ok(source.clone()),
            Ok(Some(owner)) if self.blocks[..position].contains(&owner) => Ok(source.clone()),
            Ok(Some(_)) => source.expired().ok_or_else(|| Self::unsupported(span)),
            Err(error) if error.code == "E303" => {
                source.expired().ok_or_else(|| Self::unsupported(span))
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn widen_header(
        &mut self,
        id: BlockId,
        values: &Header,
        span: Span,
    ) -> Result<Header> {
        if !self
            .guards
            .spend(self.headers.len().checked_ilog2().unwrap_or(0) as usize + values.len() + 1)
        {
            return Err(State::budget(span));
        }
        let mut header = self.headers.remove(&id).unwrap_or_default();
        if !header.is_empty() && !header.keys().eq(values.keys()) {
            return Err(Self::unsupported(span));
        }
        for (local, state) in values {
            let shape = super::header::Shape::new(&self.program.locals[*local], self.guards, span)?;
            let activity = shape.inspect(state, false, self.guards, span)?;
            if !self
                .guards
                .spend(state.weight() + header.len().checked_ilog2().unwrap_or(0) as usize + 1)
            {
                return Err(State::budget(span));
            }
            let effective = self.guards.and(state.present, state.proof);
            let effective = self.guards.and(effective, self.assumed);
            let current = header.remove(local).unwrap_or_default();
            let mut origins = BTreeSet::new();
            let mut bounds = BTreeSet::new();
            let mut seen = BTreeSet::new();
            for active in &current.active {
                if !self.guards.spend(active.component.len() + 1) {
                    return Err(State::budget(span));
                }
                seen.insert((active.component.clone(), active.member));
            }
            for (input, output) in [
                (&current.origins, &mut origins),
                (&current.bounds, &mut bounds),
            ] {
                for origin in input {
                    let lookup = output.len().checked_ilog2().unwrap_or(0) as usize + 1;
                    if !self
                        .guards
                        .spend((origin.weight() + 1).saturating_mul(lookup))
                    {
                        return Err(State::budget(span));
                    }
                    output.insert((origin.component.clone(), origin.source.clone()));
                }
            }
            let mut count = origins.len() + bounds.len() + seen.len();
            for (key, member) in &activity.members {
                let lookup = seen.len().checked_ilog2().unwrap_or(0) as usize + 1;
                if !self.guards.spend((key.0.len() + 1).saturating_mul(lookup)) {
                    return Err(State::budget(span));
                }
                let parent = activity
                    .parents
                    .get(&key.0)
                    .copied()
                    .ok_or_else(|| Self::unsupported(span))?;
                let active = self.guards.and(parent, *member);
                if self.guards.overlap(active, effective) && !seen.contains(key) {
                    if count >= super::MAX_ORIGINS {
                        return Err(State::budget(span));
                    }
                    self.reserve_origins(key.0.len() + 1, span)?;
                    seen.insert(key.clone());
                    count += 1;
                }
            }
            for (input, output) in [(&state.origins, &mut origins), (&state.bounds, &mut bounds)] {
                for origin in input {
                    let lookup = output.len().checked_ilog2().unwrap_or(0) as usize + 1;
                    if !self.guards.spend(
                        (origin.weight() + 1)
                            .saturating_mul(lookup)
                            .saturating_mul(2),
                    ) {
                        return Err(State::budget(span));
                    }
                    let active = activity
                        .paths
                        .get(&origin.component)
                        .copied()
                        .ok_or_else(|| Self::unsupported(span))?;
                    let present = self.guards.and(effective, active);
                    if !self.guards.overlap(origin.guard, present) {
                        continue;
                    }
                    let source = self.restart_source(&origin.source, id, span)?;
                    let key = (origin.component.clone(), source);
                    if !output.contains(&key) {
                        if count >= super::MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                        self.reserve_origins(origin.weight() + 1, span)?;
                        output.insert(key);
                        count += 1;
                    }
                    if output.len() > super::MAX_ORIGINS {
                        return Err(State::budget(span));
                    }
                }
            }
            if count > super::MAX_ORIGINS || origins.is_empty() {
                return Err(State::budget(span));
            }
            let mut state = State {
                active: self.header_activity(id, *local, &shape, seen, span)?,
                ..State::default()
            };
            let activation =
                super::activity::Activity::new(&shape, &state, true, self.guards, span)?;
            for (keys, output) in [(origins, &mut state.origins), (bounds, &mut state.bounds)] {
                for (component, source) in keys {
                    if !self.guards.spend(
                        component.len()
                            + activation.paths.len().checked_ilog2().unwrap_or(0) as usize
                            + 1,
                    ) {
                        return Err(State::budget(span));
                    }
                    let guard = activation
                        .paths
                        .get(&component)
                        .copied()
                        .ok_or_else(|| Self::unsupported(span))?;
                    if guard == super::FALSE {
                        return Err(Diagnostic::unsupported(
                            "restart source outside observed activity",
                            span,
                        ));
                    }
                    output.push(Origin {
                        component,
                        source,
                        guard,
                    });
                }
            }
            shape.validate(&state, true, self.guards, span)?;
            header.insert(*local, state);
        }
        Ok(header)
    }

    pub(crate) fn enter_restart(&mut self, id: BlockId, span: Span) -> Result<()> {
        if !self
            .guards
            .spend(self.targets.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let target = self
            .targets
            .remove(&id)
            .ok_or_else(|| Self::unsupported(span))?;
        let input = self.predecessor(&target.incoming, span)?;
        let header = self.widen_header(id, &input.values, span)?;
        self.facts.header_inputs.insert(id, input);
        for (local, state) in &header {
            if !self
                .guards
                .spend(state.weight() + self.locals.len().checked_ilog2().unwrap_or(0) as usize + 1)
            {
                return Err(State::budget(span));
            }
            self.locals
                .get_mut(local)
                .ok_or_else(|| Self::unsupported(span))?
                .state = state.clone();
        }
        self.headers.insert(id, header);
        self.targets.insert(id, target);
        self.assumed = TRUE;
        Ok(())
    }

    pub(crate) fn restart_target(
        &mut self,
        id: BlockId,
        site: crate::hir::RestartId,
        span: Span,
    ) -> Result<()> {
        if !self
            .guards
            .spend(self.targets.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let target = self
            .targets
            .remove(&id)
            .ok_or_else(|| Self::unsupported(span))?;
        let input = self.predecessor(&target.incoming, span)?;
        let header = self.widen_header(id, &input.values, span)?;
        self.facts.restart_inputs.insert(site, input);
        self.headers.insert(id, header);
        self.targets.insert(id, target);
        Ok(())
    }

    pub(crate) fn predecessor(
        &mut self,
        incoming: &super::branches::Values,
        span: Span,
    ) -> Result<super::state::Predecessor> {
        let entered = self.assumed;
        let values = self.capture_versions(incoming, entered, span)?;
        let weight = values
            .iter()
            .map(|(_, state)| state.weight() + 1)
            .sum::<usize>()
            + 1;
        self.reserve_origins(weight, span)?;
        let lookup = values.len().checked_ilog2().unwrap_or(0) as usize + 1;
        if !self.guards.spend(values.len().saturating_mul(lookup) + 1) {
            return Err(State::budget(span));
        }
        Ok(super::state::Predecessor {
            values: values.into_iter().collect(),
            entered,
        })
    }
}
