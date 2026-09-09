use super::{Backing, Checker, FALSE, LocalId, Path, Result, Span, State, Step, Type};

pub(crate) fn result_slot<'a>(ty: &'a Type, name: &str, local: &Type) -> Option<(Path, &'a Type)> {
    let Type::Record { fields, .. } = ty else {
        return None;
    };
    let (index, field) = fields
        .iter()
        .enumerate()
        .find(|(_, field)| field.name == name)?;
    let mut path = vec![Step::Slot(index + 1)];
    if field.ty != *local {
        let Type::Union(members) = &field.ty else {
            return None;
        };
        if let Some(member) = members.iter().position(|member| member == local) {
            path.push(Step::Variant(member));
            return Some((path, &members[member]));
        }
        if !matches!(local, Type::Union(_)) || !field.ty.accepts(local) {
            return None;
        }
    }
    Some((path, &field.ty))
}

pub(crate) struct Publication {
    pub(crate) target: super::BlockId,
    pub(crate) prefix: Path,
    pub(crate) state: State,
    pub(crate) guard: super::Guard,
}

impl Checker<'_> {
    pub(crate) fn sync_alias(
        &mut self,
        id: LocalId,
        path: &[Step],
        value: &State,
        span: Span,
    ) -> Result<()> {
        let Some(mut value) = self.alias_value(id, path, value, span)? else {
            return Ok(());
        };
        let target = self
            .blocks
            .iter()
            .position(|id| *id == value.target)
            .ok_or_else(|| Self::unsupported(span))?;
        let active = self.guards.and(value.state.present, value.state.proof);
        value.state.origins = self.retained(value.state.origins, active, target, span)?;
        value.state.bounds = self.retained(value.state.bounds, active, target, span)?;
        self.write_alias(value, span)
    }

    pub(crate) fn alias_value(
        &mut self,
        id: LocalId,
        path: &[Step],
        value: &State,
        span: Span,
    ) -> Result<Option<Publication>> {
        if !self
            .guards
            .spend(self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 1)
        {
            return Err(State::budget(span));
        }
        let Some(alias) = self.proofs.aliases.get(&id) else {
            return Ok(None);
        };
        let ty = &self.program.locals[id];
        if !self.proofs.variable(id) || !ty.has_borrowed() || !ty.fixed_borrowed_value() {
            return Err(Self::unsupported(span));
        }
        if alias.backing == Some(Backing::Discarded) {
            return Ok(None);
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
        let (mut prefix, backing) =
            result_slot(target_type, &alias.field, ty).ok_or_else(|| Self::unsupported(span))?;
        let value = if backing != ty {
            if !path.is_empty() || !self.guards.spend(value.weight() + 1) {
                return Err(Self::unsupported(span));
            }
            value.clone().convert(ty, backing, self.guards, span)?
        } else {
            value.select(path, self.guards)
        };
        prefix.extend_from_slice(path);
        let entered = self
            .proofs
            .bindings
            .get(&id)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let guard = self.guards.and(self.assumed, entered);
        self.update_published(target, &prefix, &value, guard, span)?;
        let complete = self
            .proofs
            .completions
            .get(&target)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let guard = self.guards.and(guard, complete);
        if guard == FALSE {
            return Ok(None);
        }
        Ok(Some(Publication {
            target,
            prefix,
            state: value.under(guard, self.guards),
            guard,
        }))
    }

    pub(crate) fn write_alias(&mut self, value: Publication, span: Span) -> Result<()> {
        let Publication {
            target,
            prefix,
            state: value,
            guard,
        } = value;
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
