use super::{Backing, Checker, FALSE, LocalId, Path, Result, Span, State, Step, Type};

pub(crate) fn result_path(ty: &Type, name: &str, local: &Type) -> Option<Path> {
    let Type::Record { fields, .. } = ty else {
        return None;
    };
    let (index, field) = fields
        .iter()
        .enumerate()
        .find(|(_, field)| field.name == name && field.mutable)?;
    let mut path = vec![Step::Slot(index + 1)];
    if field.ty != *local {
        let Type::Union(members) = &field.ty else {
            return None;
        };
        let member = members.iter().position(|member| member == local)?;
        path.push(Step::Variant(member));
    }
    Some(path)
}

impl Checker<'_> {
    pub(crate) fn sync_alias(
        &mut self,
        id: LocalId,
        path: &[Step],
        value: &State,
        span: Span,
    ) -> Result<()> {
        if !self
            .guards
            .spend(self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let Some(alias) = self.proofs.aliases.get(&id) else {
            return Ok(());
        };
        let ty = &self.program.locals[id];
        if !alias.mutable || !ty.has_reference() || !ty.fixed_borrowed_value() {
            return Err(Self::unsupported(span));
        }
        if alias.backing == Some(Backing::Discarded) {
            return Ok(());
        }
        let target = alias.target;
        if !self.guards.spend(alias.field.len() + path.len() + 1) {
            return Err(State::budget(span));
        }
        let target_type = self
            .types
            .get(&target)
            .ok_or_else(|| Self::unsupported(span))?;
        crate::borrow_contract::type_weight(target_type, self.guards, span)?;
        if alias.backing != Some(Backing::Result) {
            return Err(Self::unsupported(span));
        }
        let mut prefix =
            result_path(target_type, &alias.field, ty).ok_or_else(|| Self::unsupported(span))?;
        let complete = self
            .proofs
            .completions
            .get(&target)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let guard = self.guards.and(self.assumed, complete);
        if guard == FALSE {
            return Ok(());
        }
        let mut value = value.select(path, self.guards).under(guard, self.guards);
        let target_index = self
            .blocks
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Self::unsupported(span))?;
        let active = self.guards.and(value.present, value.proof);
        value.origins = self.retained(value.origins, active, target_index, span)?;
        value.bounds = self.retained(value.bounds, active, target_index, span)?;
        prefix.extend_from_slice(path);
        let current = self
            .results
            .get(&target)
            .ok_or_else(|| Self::unsupported(span))?;
        if !self
            .guards
            .spend(current.weight().saturating_mul(2) + prefix.len() + 1)
        {
            return Err(State::budget(span));
        }
        let mut outside = current.clone().under(self.guards.not(guard), self.guards);
        let inside = current.clone().under(guard, self.guards).replaced(
            &prefix,
            value,
            self.guards,
            span,
        )?;
        outside.merge(inside, self.guards, span)?;
        self.reserve_origins(outside.weight() + 1, span)?;
        self.results.insert(target, outside);
        Ok(())
    }
}
