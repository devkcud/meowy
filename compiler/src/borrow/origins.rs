use super::{
    BTreeMap, BTreeSet, Checker, Diagnostic, FALSE, Guard, LocalId, MAX_FACT_ORIGINS, MAX_ORIGINS,
    Path, Result, Source, Span, State, Step, Storage, TRUE, Type,
};

impl Checker<'_> {
    pub(crate) fn unsupported(span: Span) -> Diagnostic {
        Diagnostic::unsupported("borrow origins outside immutable local storage", span)
    }

    pub(crate) fn close_scope(&mut self) {
        for id in self.scopes.pop().expect("storage scope") {
            self.locals.remove(&id);
        }
        self.assumed = self.assumed_scopes.pop().expect("assumption scope");
    }

    pub(crate) fn reserve_origins(&mut self, count: usize, span: Span) -> Result<()> {
        if self.origins + count > MAX_FACT_ORIGINS {
            return Err(Diagnostic::unsupported(
                "borrow-origin fact budget exhausted",
                span,
            ));
        }
        self.origins += count;
        Ok(())
    }

    pub(crate) fn assumptions(&mut self) -> Guard {
        self.assumed
    }

    pub(crate) fn bind(&mut self, id: LocalId, mut state: State, span: Span) -> Result<()> {
        let ty = self
            .program
            .locals
            .get(id)
            .ok_or_else(|| Self::unsupported(span))?
            .clone();
        if self.locals.contains_key(&id) {
            return Err(Self::unsupported(span));
        }
        self.complete(&ty, &state, span)?;
        if !self.proofs.mutable.contains(&id) {
            self.link_tags(id, &ty, &mut state, span)?;
        } else {
            state = State::unknown(&ty, self.guards, span)?;
        }
        self.reserve_origins(state.weight() + 1, span)?;
        self.assumed = self.guards.and(self.assumed, state.proof);
        self.locals.insert(
            id,
            Storage {
                block: *self.blocks.last().expect("storage block"),
                state: state.clone(),
            },
        );
        if state.size() != 0 || state.proof != TRUE {
            self.facts.locals.insert(id, state);
        }
        self.scopes.last_mut().expect("storage scope").push(id);
        Ok(())
    }

    pub(crate) fn live(
        &mut self,
        source: &Source,
        span: Span,
    ) -> Result<Option<crate::hir::BlockId>> {
        let work = match source {
            Source::Local { fields, .. } => {
                fields.len() + self.locals.len().checked_ilog2().unwrap_or(0) as usize + 1
            }
            Source::Slot { fields, .. } => {
                fields.len() + self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 3
            }
            Source::Input {
                component, fields, ..
            } => {
                component.len()
                    + fields.len()
                    + self.inputs.len().checked_ilog2().unwrap_or(0) as usize
                    + 1
            }
        };
        if !self.guards.spend(work) {
            return Err(State::budget(span));
        }
        match source {
            Source::Local { id, .. } => self
                .locals
                .get(id)
                .map(|storage| Some(storage.block))
                .ok_or_else(|| {
                    Diagnostic::new("E303", "borrowed storage has ended before this use", span)
                }),
            Source::Slot {
                target,
                root,
                view,
                fields,
            } => {
                let alias = self
                    .proofs
                    .aliases
                    .get(view)
                    .filter(|alias| {
                        alias.target == *target && alias.root == *root && alias.backing.is_some()
                    })
                    .ok_or_else(|| Self::unsupported(span))?;
                let local = self
                    .program
                    .locals
                    .get(*view)
                    .ok_or_else(|| Self::unsupported(span))?;
                if crate::borrow_contract::projected_type(local, fields).is_none() {
                    return Err(Self::unsupported(span));
                }
                if !self.types.contains_key(&alias.target) {
                    return Err(Diagnostic::new(
                        "E303",
                        "borrowed result slot has ended before this use",
                        span,
                    ));
                }
                Ok(Some(alias.target))
            }
            Source::Input {
                id,
                component,
                fields,
            } if self.inputs.contains(id)
                && crate::borrow_contract::component_type(&self.program.locals[*id], component)
                    .is_some_and(|ty| match ty {
                        Type::Reference(ty) => {
                            crate::borrow_contract::projected_type(ty, fields).is_some()
                        }
                        _ => false,
                    }) =>
            {
                Ok(None)
            }
            _ => Err(Self::unsupported(span)),
        }
    }

    pub(crate) fn link_tags(
        &mut self,
        id: LocalId,
        ty: &Type,
        state: &mut State,
        span: Span,
    ) -> Result<()> {
        let entered = self
            .proofs
            .bindings
            .get(&id)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let present = self.guards.and(entered, state.present);
        let mut pending = vec![(ty, Path::new(), Vec::<String>::new(), present)];
        while let Some((ty, path, names, present)) = pending.pop() {
            if !self.guards.spend(path.len() + names.len() + 1) {
                return Err(State::budget(span));
            }
            match ty {
                Type::Record { primary, fields } => {
                    for (index, ty) in std::iter::once(primary.as_ref())
                        .chain(fields.iter().map(|field| &field.ty))
                        .enumerate()
                    {
                        let mut nested = path.clone();
                        nested.push(Step::Slot(index));
                        let mut names = names.clone();
                        if index != 0 {
                            names.push(fields[index - 1].name.clone());
                        }
                        pending.push((ty, nested, names, present));
                        if pending.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                }
                Type::Union(members) => {
                    let tags = self.proofs.tags.get(&((id, names.clone()), ty.clone()));
                    for (index, member) in members.iter().enumerate() {
                        let actual = state.member(&path, index, self.guards);
                        if let Some(tag) = tags.and_then(|tags| {
                            tags.iter()
                                .find(|(ty, _)| ty == member)
                                .map(|(_, guard)| *guard)
                        }) {
                            let both = self.guards.and(tag, actual);
                            let neither = self
                                .guards
                                .and(self.guards.not(tag), self.guards.not(actual));
                            let equal = self.guards.or(both, neither);
                            let relation = self.guards.or(self.guards.not(present), equal);
                            state.proof = self.guards.and(state.proof, relation);
                        }
                        let nested_guard = self.guards.and(present, actual);
                        let mut nested = path.clone();
                        nested.push(Step::Variant(index));
                        pending.push((member, nested, names.clone(), nested_guard));
                        if pending.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn complete(&mut self, ty: &Type, state: &State, span: Span) -> Result<()> {
        let assumptions = self.assumptions();
        let proof = self.guards.and(state.proof, assumptions);
        let present = self.guards.and(state.present, proof);
        if present == FALSE {
            return Ok(());
        }
        if !self.guards.spend(state.weight()) {
            return Err(State::budget(span));
        }
        let mut origins = BTreeMap::new();
        for origin in &state.origins {
            let prior = origins.get(&origin.component).copied().unwrap_or(FALSE);
            origins.insert(
                origin.component.clone(),
                self.guards.or(prior, origin.guard),
            );
        }
        let mut valid = BTreeSet::new();
        let mut unions = BTreeMap::new();
        let mut pending = vec![(ty, Path::new(), present)];
        while let Some((ty, path, present)) = pending.pop() {
            if !self.guards.spend(path.len() + 1) {
                return Err(State::budget(span));
            }
            match ty {
                Type::Reference(ty) => {
                    valid.insert(path.clone());
                    if !self
                        .guards
                        .implies(present, origins.get(&path).copied().unwrap_or(FALSE))
                    {
                        return Err(Self::unsupported(span));
                    }
                    if ty.has_reference() {
                        if pending.len() + valid.len() + unions.len() >= MAX_ORIGINS
                            || !self.guards.spend(path.len() + 1)
                        {
                            return Err(State::budget(span));
                        }
                        let mut path = path;
                        path.push(Step::Deref);
                        pending.push((ty, path, present));
                    }
                }
                Type::Record { primary, fields } => {
                    for (index, ty) in std::iter::once(primary.as_ref())
                        .chain(fields.iter().map(|field| &field.ty))
                        .enumerate()
                    {
                        let mut path = path.clone();
                        path.push(Step::Slot(index));
                        pending.push((ty, path, present));
                        if pending.len() + valid.len() + unions.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                }
                Type::Union(members) => {
                    unions.insert(path.clone(), members.len());
                    let mut covered = FALSE;
                    for (index, ty) in members.iter().enumerate() {
                        let active = state.member(&path, index, self.guards);
                        let guard = self.guards.and(present, active);
                        if self.guards.overlap(guard, covered) {
                            return Err(Self::unsupported(span));
                        }
                        covered = self.guards.or(covered, active);
                        let mut nested = path.clone();
                        nested.push(Step::Variant(index));
                        pending.push((ty, nested, guard));
                        if pending.len() + valid.len() + unions.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                    if !self.guards.implies(present, covered) {
                        return Err(Self::unsupported(span));
                    }
                }
                _ => {}
            }
        }
        if state
            .origins
            .iter()
            .chain(&state.bounds)
            .any(|origin| !valid.contains(&origin.component))
            || state.active.iter().any(|active| {
                unions
                    .get(&active.component)
                    .is_none_or(|count| active.member >= *count)
            })
        {
            return Err(Self::unsupported(span));
        }
        Ok(())
    }
}
