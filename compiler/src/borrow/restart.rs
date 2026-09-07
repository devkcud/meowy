use super::branches::Values;
use super::{
    BTreeMap, BTreeSet, BlockId, Checker, Diagnostic, Guards, LocalId, Origin, Path, Result,
    Source, Span, State, TRUE,
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
                .map(|origin| (&origin.component, &origin.source))
                .eq(right
                    .origins
                    .iter()
                    .map(|origin| (&origin.component, &origin.source)))
                || !left
                    .bounds
                    .iter()
                    .map(|origin| (&origin.component, &origin.source))
                    .eq(right
                        .bounds
                        .iter()
                        .map(|origin| (&origin.component, &origin.source)))
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
    ) -> Result<()> {
        if !self.guards.spend(self.blocks.len().saturating_mul(2) + 1) {
            return Err(State::budget(span));
        }
        if matches!(source, Source::Temporary { .. }) {
            return Err(Diagnostic::unsupported(
                "temporary sources in restart headers",
                span,
            ));
        }
        let position = self
            .blocks
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Self::unsupported(span))?;
        match self.live(source, span) {
            Ok(None) => Ok(()),
            Ok(Some(owner)) if self.blocks[..position].contains(&owner) => Ok(()),
            _ => Err(Diagnostic::unsupported(
                "restart-carried sources outside surviving storage",
                span,
            )),
        }
    }

    pub(crate) fn widen_header(
        &mut self,
        id: BlockId,
        values: &Values,
        span: Span,
    ) -> Result<Header> {
        if !self
            .guards
            .spend(self.headers.len().checked_ilog2().unwrap_or(0) as usize + values.len() + 1)
        {
            return Err(State::budget(span));
        }
        let mut header = self.headers.remove(&id).unwrap_or_default();
        if !header.is_empty() && !header.keys().copied().eq(values.iter().map(|(id, _)| *id)) {
            return Err(Self::unsupported(span));
        }
        for (local, state) in values {
            let shape = super::header::Shape::new(&self.program.locals[*local], self.guards, span)?;
            shape.validate(state, false, self.guards, span)?;
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
            let mut count = origins.len() + bounds.len();
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
                    if !self.guards.overlap(origin.guard, effective) {
                        continue;
                    }
                    self.restart_source(&origin.source, id, span)?;
                    let key = (origin.component.clone(), origin.source.clone());
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
            if origins.len() + bounds.len() > super::MAX_ORIGINS || origins.is_empty() {
                return Err(State::budget(span));
            }
            let canonical = |sources: BTreeSet<(Path, Source)>| {
                sources
                    .into_iter()
                    .map(|(component, source)| Origin {
                        component,
                        source,
                        guard: TRUE,
                    })
                    .collect()
            };
            let state = State {
                origins: canonical(origins),
                bounds: canonical(bounds),
                ..State::default()
            };
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
        let header = self.widen_header(id, &target.incoming, span)?;
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

    pub(crate) fn restart_target(&mut self, id: BlockId, span: Span) -> Result<()> {
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
        let values = self.capture_versions(&target.incoming, self.assumed, span)?;
        let header = self.widen_header(id, &values, span)?;
        self.headers.insert(id, header);
        self.targets.insert(id, target);
        Ok(())
    }
}
