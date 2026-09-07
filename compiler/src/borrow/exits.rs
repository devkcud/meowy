use super::branches::{Arm, Values};
use super::{BlockId, Checker, FALSE, Flow, Result, Span, State};

pub(crate) struct Target {
    pub(crate) incoming: Values,
    pub(crate) exits: Vec<Arm>,
}

impl Checker<'_> {
    pub(crate) fn enter_target(&mut self, id: BlockId, span: Span) -> Result<()> {
        if !self
            .guards
            .spend(self.targets.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let incoming = self.versions(span)?;
        self.reserve_origins(
            incoming
                .iter()
                .map(|(_, state)| state.weight() + 1)
                .sum::<usize>()
                + 1,
            span,
        )?;
        self.targets.insert(
            id,
            Target {
                incoming,
                exits: Vec::new(),
            },
        );
        Ok(())
    }

    pub(crate) fn leave_target(&mut self, id: BlockId, span: Span) -> Result<()> {
        let lookup = self.targets.len().checked_ilog2().unwrap_or(0) as usize + 1;
        if !self.guards.spend(lookup) {
            return Err(State::budget(span));
        }
        let target = self
            .targets
            .remove(&id)
            .ok_or_else(|| Self::unsupported(span))?;
        let normal = self.assumed;
        let values = self.capture_versions(&target.incoming, normal, span)?;
        self.reserve_origins(
            values
                .iter()
                .map(|(_, state)| state.weight() + 1)
                .sum::<usize>()
                + 1,
            span,
        )?;
        let mut target = target;
        target.exits.push(Arm {
            values,
            normal,
            flow: Flow::new(),
        });
        self.targets.insert(id, target);
        Ok(())
    }

    pub(crate) fn complete_target(
        &mut self,
        id: BlockId,
        returning: bool,
        span: Span,
    ) -> Result<()> {
        if !self
            .guards
            .spend(self.targets.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let mut target = self
            .targets
            .remove(&id)
            .ok_or_else(|| Self::unsupported(span))?;
        let normal = if returning { self.assumed } else { FALSE };
        let values = self.capture_versions(&target.incoming, normal, span)?;
        target.exits.push(Arm {
            values,
            normal,
            flow: Flow::new(),
        });
        self.merge_versions(&target.incoming, &target.exits, span)?;
        Ok(())
    }
}
