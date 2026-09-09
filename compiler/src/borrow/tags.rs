use super::{Checker, Expr, ExprKind, Result, State, Step, Type};

impl Checker<'_> {
    pub(crate) fn inspect_tags(&mut self, expr: &Expr, value: &Expr) -> Result<()> {
        let Some(state) = self.tagged_state(value)? else {
            return Ok(());
        };
        self.reserve_origins(3, expr.span)?;
        self.assumed = self.guards.and(self.assumed, state.proof);
        self.facts
            .inspections
            .insert((expr.span.start, expr.span.end), state.proof);
        Ok(())
    }

    pub(self) fn tagged_state(&mut self, expr: &Expr) -> Result<Option<State>> {
        if !self.guards.spend(1) {
            return Err(State::budget(expr.span));
        }
        let mut state = match &expr.kind {
            ExprKind::Local(id)
                if self.proofs.variable(*id)
                    && self.proofs.versioned(self.program, *id)
                    && self.program.locals[*id].fixed_borrowed_value() =>
            {
                let state = &self
                    .locals
                    .get(id)
                    .ok_or_else(|| Self::unsupported(expr.span))?
                    .state;
                if !self.guards.spend(state.weight() + 1) {
                    return Err(State::budget(expr.span));
                }
                state.clone()
            }
            ExprKind::Field { value, index } => {
                let Some(state) = self.tagged_state(value)? else {
                    return Ok(None);
                };
                state.select(&[Step::Slot(index + 1)], self.guards)
            }
            ExprKind::Primary(value) => {
                let Some(state) = self.tagged_state(value)? else {
                    return Ok(None);
                };
                state.select(&[Step::Slot(0)], self.guards)
            }
            ExprKind::Coerce { value } => {
                let Some(state) = self.tagged_state(value)? else {
                    return Ok(None);
                };
                state.convert(&value.ty, &expr.ty, self.guards, expr.span)?
            }
            _ => return Ok(None),
        };
        if let Type::Union(members) = &expr.ty
            && !matches!(expr.kind, ExprKind::Coerce { .. })
            && let Some(tags) = self
                .proofs
                .observations
                .get(&(expr.span.start, expr.span.end))
        {
            if tags.len() != members.len()
                || !self
                    .guards
                    .spend(tags.len().saturating_mul(state.weight() + 1))
            {
                return Err(State::budget(expr.span));
            }
            for (index, (ty, tag)) in tags.iter().enumerate() {
                if ty != &members[index] {
                    return Err(Self::unsupported(expr.span));
                }
                let actual = state.member(&[], index, self.guards);
                let both = self.guards.and(*tag, actual);
                let neither = self
                    .guards
                    .and(self.guards.not(*tag), self.guards.not(actual));
                let equal = self.guards.or(both, neither);
                let relation = self.guards.or(self.guards.not(state.present), equal);
                state.proof = self.guards.and(state.proof, relation);
            }
        }
        Ok(Some(state))
    }
}
