use super::{
    BlockId, Checker, Diagnostic, EmitId, FALSE, Guard, Origin, Path, Result, Span, State, Step,
    Type,
};

pub(crate) fn slot<'a>(ty: &'a Type, field: &Option<String>) -> Option<(Path, &'a Type)> {
    match (ty, field) {
        (Type::Record { primary, .. }, None) => Some((vec![Step::Slot(0)], primary)),
        (Type::Record { fields, .. }, Some(name)) => fields
            .iter()
            .enumerate()
            .find(|(_, field)| &field.name == name)
            .map(|(index, field)| (vec![Step::Slot(index + 1)], &field.ty)),
        (ty, None) => Some((Vec::new(), ty)),
        _ => None,
    }
}

impl Checker<'_> {
    pub(crate) fn emit(
        &mut self,
        id: EmitId,
        target: BlockId,
        field: &Option<String>,
        value: State,
        ty: &Type,
        span: Span,
    ) -> Result<()> {
        let written = self
            .proofs
            .emissions
            .get(&id)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let complete = self
            .proofs
            .completions
            .get(&target)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let retained = self.guards.and(written, complete);
        self.publish(target, field, &value, ty, span)?;
        if retained == FALSE {
            return Ok(());
        }
        let target_type = self
            .types
            .get(&target)
            .ok_or_else(|| Self::unsupported(span))?
            .clone();
        let Some((prefix, destination)) = slot(&target_type, field) else {
            return Ok(());
        };
        if !destination.accepts(ty) {
            return Ok(());
        }
        let mut state = value.convert(ty, destination, self.guards, span)?;
        let assumptions = self.assumptions();
        state.proof = self.guards.and(state.proof, assumptions);
        state = state.under(retained, self.guards);
        self.complete(destination, &state, span)?;
        let key = (target, field.clone());
        let prior = self.writes.get(&key).copied().unwrap_or(FALSE);
        self.writes
            .insert(key, self.guards.or(prior, state.present));
        let target_index = self
            .blocks
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Self::unsupported(span))?;
        let effective = self.guards.and(state.present, state.proof);
        state.origins = self.retained(state.origins, effective, target_index, span)?;
        state.bounds = self.retained(state.bounds, effective, target_index, span)?;
        self.guards
            .spend(state.weight() + prefix.len() * state.size());
        self.results.get_mut(&target).expect("result state").merge(
            state.prefix(&prefix),
            self.guards,
            span,
        )
    }

    pub(crate) fn retained(
        &mut self,
        origins: Vec<Origin>,
        effective: Guard,
        target: usize,
        span: Span,
    ) -> Result<Vec<Origin>> {
        let mut result = Vec::new();
        for origin in origins {
            if !self.guards.spend(origin.weight()) {
                return Err(State::budget(span));
            }
            if !self.guards.overlap(origin.guard, effective) {
                continue;
            }
            if let Some(owner) = self.live(&origin.source, span)? {
                let source = self
                    .blocks
                    .iter()
                    .position(|id| *id == owner)
                    .ok_or_else(|| Self::unsupported(span))?;
                if source >= target {
                    return Err(Diagnostic::new(
                        "E303",
                        "emitted borrow outlives its local storage",
                        span,
                    ));
                }
            }
            result.push(origin);
        }
        Ok(result)
    }
}
