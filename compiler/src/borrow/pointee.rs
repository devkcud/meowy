use super::{Checker, Expr, ExprKind, Result, Span, State, Step, Value};

impl Checker<'_> {
    pub(crate) fn live_value(&mut self, state: &State, span: Span) -> Result<()> {
        if !self.guards.spend(state.weight()) {
            return Err(State::budget(span));
        }
        let assumptions = self.assumptions();
        let proof = self.guards.and(state.proof, assumptions);
        let proof = self.guards.and(proof, state.present);
        for origin in state
            .origins
            .iter()
            .chain(&state.bounds)
            .filter(|origin| !origin.component.contains(&Step::Deref))
        {
            if self.guards.overlap(origin.guard, proof) {
                self.live(&origin.source, span)?;
            }
        }
        Ok(())
    }

    pub(crate) fn read(&mut self, expr: &Expr, path: &[Step]) -> Result<Value> {
        if !self.guards.spend(path.len() + 1) {
            return Err(State::budget(expr.span));
        }
        match &expr.kind {
            ExprKind::Field { value, index } => {
                let path = std::iter::once(Step::Slot(index + 1))
                    .chain(path.iter().copied())
                    .collect::<Vec<_>>();
                self.read(value, &path)
            }
            ExprKind::Primary(value) => {
                let path = std::iter::once(Step::Slot(0))
                    .chain(path.iter().copied())
                    .collect::<Vec<_>>();
                self.read(value, &path)
            }
            ExprKind::Local(id) if !self.proofs.mutable.contains(id) => {
                let state = self
                    .locals
                    .get(id)
                    .ok_or_else(|| Self::unsupported(expr.span))?
                    .state
                    .select(path, self.guards);
                self.live_value(&state, expr.span)?;
                Ok(Value {
                    state,
                    flow: super::Flow::new(),
                })
            }
            ExprKind::Deref(value) => {
                let result = self.expression(value)?;
                if result.flow.next {
                    self.live_value(&result.state, expr.span)?;
                }
                let state = result
                    .state
                    .dereferenced(&expr.ty, self.guards, expr.span)?
                    .select(path, self.guards);
                if result.flow.next {
                    self.live_value(&state, expr.span)?;
                }
                Ok(Value {
                    state,
                    flow: result.flow,
                })
            }
            _ => {
                let result = self.expression(expr)?;
                Ok(Value {
                    state: result.state.select(path, self.guards),
                    flow: result.flow,
                })
            }
        }
    }
}
