use super::{
    Checker, Expr, ExprKind, FALSE, Flow, Guard, LocalId, Result, Span, State, TRUE, Type,
};

pub(crate) type Values = Vec<(LocalId, State)>;

pub(crate) struct Arm {
    pub(crate) values: Values,
    pub(crate) normal: Guard,
    pub(crate) flow: Flow,
}

impl Checker<'_> {
    pub(crate) fn condition(&self, expr: &Expr) -> Result<Guard> {
        match expr.kind {
            ExprKind::Bool(value) => Ok(if value { TRUE } else { FALSE }),
            _ => self
                .proofs
                .conditions
                .get(&(expr.span.start, expr.span.end))
                .copied()
                .ok_or_else(|| Self::unsupported(expr.span)),
        }
    }

    pub(crate) fn versions(&mut self, span: Span) -> Result<Values> {
        let lookup = self.proofs.mutable.len().checked_ilog2().unwrap_or(0) as usize + 1;
        let work = self.locals.len().saturating_mul(lookup);
        if !self.guards.spend(work) {
            return Err(State::budget(span));
        }
        let mut values = Vec::new();
        for (id, value) in &self.locals {
            if matches!(self.program.locals[*id], Type::Reference(_))
                && self.proofs.mutable.contains(id)
            {
                if !self.guards.spend(value.state.weight() + 1) {
                    return Err(State::budget(span));
                }
                values.push((*id, value.state.clone()));
            }
        }
        Ok(values)
    }

    pub(crate) fn restore_versions(&mut self, values: &Values, span: Span) -> Result<()> {
        let lookup = self.locals.len().checked_ilog2().unwrap_or(0) as usize + 1;
        let work = values
            .iter()
            .map(|(_, state)| state.weight() + lookup)
            .sum();
        if !self.guards.spend(work) {
            return Err(State::budget(span));
        }
        for (id, state) in values {
            self.locals
                .get_mut(id)
                .ok_or_else(|| Self::unsupported(span))?
                .state = state.clone();
        }
        Ok(())
    }

    pub(crate) fn arm(
        &mut self,
        values: &Values,
        assumed: Guard,
        guard: Guard,
        span: Span,
        run: impl FnOnce(&mut Self) -> Result<Flow>,
    ) -> Result<Arm> {
        self.restore_versions(values, span)?;
        self.assumed = self.guards.and(assumed, guard);
        if self.assumed == FALSE {
            return Ok(Arm {
                values: Vec::new(),
                normal: FALSE,
                flow: Flow {
                    next: false,
                    exits: Default::default(),
                },
            });
        }
        self.scopes.push(Vec::new());
        self.assumed_scopes.push(assumed);
        let flow = run(self)?;
        let normal = if flow.next { self.assumed } else { FALSE };
        let result = self.capture_versions(values, normal, span)?;
        self.close_scope();
        Ok(Arm {
            values: result,
            normal,
            flow,
        })
    }

    pub(crate) fn capture_versions(
        &mut self,
        values: &Values,
        normal: Guard,
        span: Span,
    ) -> Result<Values> {
        let mut result = Vec::new();
        if normal != FALSE {
            let lookup = self.locals.len().checked_ilog2().unwrap_or(0) as usize + 1;
            for (id, _) in values {
                if !self.guards.spend(lookup) {
                    return Err(State::budget(span));
                }
                let state = &self
                    .locals
                    .get(id)
                    .ok_or_else(|| Self::unsupported(span))?
                    .state;
                if !self.guards.spend(state.weight() + 1) {
                    return Err(State::budget(span));
                }
                result.push((*id, state.clone().under(normal, self.guards)));
            }
        }
        Ok(result)
    }

    pub(crate) fn merge_versions(
        &mut self,
        incoming: &Values,
        arms: &[Arm],
        span: Span,
    ) -> Result<Guard> {
        self.restore_versions(incoming, span)?;
        if !self.guards.spend(arms.len()) {
            return Err(State::budget(span));
        }
        let mut normal = FALSE;
        for arm in arms {
            normal = self.guards.or(normal, arm.normal);
        }
        for (index, (id, _)) in incoming.iter().enumerate() {
            let mut state = State::absent();
            for arm in arms {
                if !self.guards.spend(1) {
                    return Err(State::budget(span));
                }
                if arm.normal != FALSE {
                    let (source, value) = arm
                        .values
                        .get(index)
                        .ok_or_else(|| Self::unsupported(span))?;
                    if source != id || !self.guards.spend(value.weight()) {
                        return Err(State::budget(span));
                    }
                    state.merge(value.clone(), self.guards, span)?;
                }
            }
            if normal != FALSE {
                self.reserve_origins(state.weight() + 1, span)?;
                self.locals
                    .get_mut(id)
                    .ok_or_else(|| Self::unsupported(span))?
                    .state = state;
            }
        }
        self.assumed = normal;
        Ok(normal)
    }

    pub(crate) fn conditional(
        &mut self,
        guard: Guard,
        span: Span,
        yes: impl FnOnce(&mut Self) -> Result<Flow>,
        no: impl FnOnce(&mut Self) -> Result<Flow>,
    ) -> Result<Flow> {
        let incoming = self.versions(span)?;
        let assumed = self.assumed;
        let yes = self.arm(&incoming, assumed, guard, span, yes)?;
        let no = self.arm(&incoming, assumed, self.guards.not(guard), span, no)?;
        let arms = [yes, no];
        let normal = self.merge_versions(&incoming, &arms, span)?;
        let [yes, no] = arms;
        let mut flow = yes.flow;
        flow.merge(no.flow);
        flow.next = normal != FALSE;
        Ok(flow)
    }
}
