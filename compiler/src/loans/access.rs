use super::{Graph, LocalId, Node, Path, Place, Result, Span, Step, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Read,
    Tag,
    Borrow,
    Write,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    Storage { place: Place, view: LocalId },
    Pointee(usize),
    Missing,
}

#[derive(Clone, Debug)]
pub(crate) struct Access {
    pub(crate) kind: Kind,
    pub(crate) target: Target,
    pub(crate) path: Path,
    pub(crate) span: Span,
    pub(crate) regions: Vec<super::Origin>,
    pub(crate) unresolved: super::Guard,
}

impl Access {
    pub(crate) fn canonical_region(mut origin: super::Origin) -> super::Origin {
        let fields = match &mut origin.source {
            super::Source::Local { fields, .. }
            | super::Source::Slot { fields, .. }
            | super::Source::Temporary { fields, .. }
            | super::Source::Input { fields, .. } => fields,
            super::Source::Expired { .. } => return origin,
        };
        let count = origin
            .component
            .iter()
            .take_while(|step| matches!(step, Step::Slot(index) if *index != 0))
            .count();
        for step in origin.component.drain(..count) {
            let Step::Slot(index) = step else {
                unreachable!()
            };
            fields.push(super::Projection::Field(index - 1));
        }
        origin
    }

    pub(crate) fn weight(&self) -> usize {
        self.path.len()
            + match &self.target {
                Target::Storage { place, .. } => place.fields.len() + 3,
                Target::Pointee(_) | Target::Missing => 2,
            }
    }

    pub(crate) fn write(&self) -> Option<(&Place, Span)> {
        match &self.target {
            Target::Storage { place, .. } if self.kind == Kind::Write => Some((place, self.span)),
            _ => None,
        }
    }
}

impl Graph<'_> {
    pub(crate) fn normalize_accesses(
        &mut self,
        reach: &[super::Guard],
        restart: bool,
    ) -> Result<()> {
        for (id, entered) in reach.iter().enumerate() {
            self.charge(1)?;
            if *entered == super::FALSE {
                continue;
            }
            let Some(mut access) = self.nodes[id].access.take() else {
                continue;
            };
            match &access.target {
                Target::Storage { place, view } => {
                    self.reserve_authority(place.fields.len() + access.path.len() + 5)?;
                    let source = self.proofs.source(&Place {
                        root: *view,
                        fields: place.fields.clone(),
                    });
                    access.regions.push(Access::canonical_region(super::Origin {
                        component: access.path.clone(),
                        source,
                        guard: *entered,
                    }));
                }
                Target::Pointee(value) => {
                    self.charge(self.values[*value].len() + 1)?;
                    let mut covered = super::FALSE;
                    for index in 0..self.values[*value].origins.len() {
                        let origin = &self.values[*value].origins[index];
                        let active = self.guards.and(*entered, origin.guard);
                        if active == super::FALSE {
                            continue;
                        }
                        let weight = origin.weight() + access.path.len();
                        self.reserve_authority(weight)?;
                        let source = self.values[*value].origins[index].source.clone();
                        access.regions.push(Access::canonical_region(super::Origin {
                            component: access.path.clone(),
                            source,
                            guard: active,
                        }));
                        covered = self.guards.or(covered, active);
                    }
                    let missing = self.guards.and(*entered, self.guards.not(covered));
                    if missing != super::FALSE
                        && (!restart
                            || !self.guards.implies(missing, self.authority[*value].opaque))
                    {
                        return Err(super::Diagnostic::unsupported(
                            "missing physical access origin",
                            access.span,
                        ));
                    }
                    access.unresolved = missing;
                    let mut known = self.authority[*value].opaque;
                    self.charge(self.authority[*value].weight())?;
                    for guard in self.authority[*value].loans.values() {
                        known = self.guards.or(known, *guard);
                    }
                    let missing = self.guards.and(*entered, self.guards.not(known));
                    self.authority[*value].opaque =
                        self.guards.or(self.authority[*value].opaque, missing);
                }
                Target::Missing => {
                    return Err(super::Diagnostic::unsupported(
                        "missing pointee access evidence",
                        access.span,
                    ));
                }
            }
            self.nodes[id].access = Some(access);
        }
        Ok(())
    }

    pub(crate) fn has_tag(&mut self, ty: &Type, path: &[Step]) -> Result<bool> {
        self.charge(path.len() + 1)?;
        let ty = crate::borrow_contract::component_type(ty, path).ok_or_else(Self::budget)?;
        Ok(matches!(ty, Type::Union(_)))
    }

    pub(crate) fn access_evidence(&mut self, reach: &[super::Guard]) -> Result<()> {
        self.charge(reach.len())?;
        for (id, reachable) in reach.iter().enumerate() {
            if *reachable != super::FALSE
                && let Some(access) = &self.nodes[id].access
                && access.target == Target::Missing
            {
                return Err(super::Diagnostic::unsupported(
                    "missing pointee access evidence",
                    access.span,
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn access(
        &mut self,
        kind: Kind,
        target: Target,
        path: &[Step],
        span: Span,
    ) -> Result<Access> {
        let mut access = Access {
            kind,
            target,
            path: Path::new(),
            span,
            regions: Vec::new(),
            unresolved: super::FALSE,
        };
        let weight = access.weight() + path.len();
        self.charge(weight)?;
        if self.origins.saturating_add(weight) > super::MAX_ORIGINS {
            return Err(Self::budget());
        }
        access.path.extend_from_slice(path);
        Ok(access)
    }

    pub(crate) fn storage(&self, view: LocalId, fields: Vec<usize>) -> Target {
        Target::Storage {
            place: Place {
                root: self
                    .proofs
                    .aliases
                    .get(&view)
                    .map_or(view, |alias| alias.root),
                fields,
            },
            view,
        }
    }

    pub(crate) fn read_local(
        &mut self,
        id: LocalId,
        path: &[Step],
        kind: Kind,
        span: Span,
    ) -> Result<()> {
        if self.current.is_empty() {
            return Ok(());
        }
        let target = self.storage(id, Vec::new());
        let access = self.access(kind, target, path, span)?;
        self.append(Node {
            access: Some(access),
            ..Node::default()
        })?;
        Ok(())
    }

    pub(crate) fn pointee_access(
        &mut self,
        value: &super::Bundle,
        path: &[Step],
        kind: Kind,
        span: Span,
    ) -> Result<Access> {
        let target = value
            .get(&Path::new())
            .copied()
            .map_or(Target::Missing, Target::Pointee);
        self.access(kind, target, path, span)
    }

    pub(crate) fn check_access(&mut self, access: &Access) -> Result<()> {
        let weight = access.weight();
        self.charge(weight)?;
        if self.origins.saturating_add(weight) > super::MAX_ORIGINS {
            return Err(Self::budget());
        }
        match &access.target {
            Target::Storage { place, view } => {
                if self.program.locals.get(*view).is_none()
                    || self
                        .proofs
                        .aliases
                        .get(view)
                        .map_or(*view, |alias| alias.root)
                        != place.root
                {
                    return Err(Self::budget());
                }
            }
            Target::Pointee(id) if self.values.get(*id).is_none() => {
                return Err(Self::budget());
            }
            Target::Pointee(_) | Target::Missing => {}
        }
        self.origins += weight;
        Ok(())
    }

    pub(crate) fn inspection_path(
        &mut self,
        from: &Type,
        to: &Type,
        path: &[Step],
    ) -> Result<Path> {
        self.charge(from.members().len() + to.members().len() + path.len() + 1)?;
        if from == to {
            return Ok(path.to_vec());
        }
        if let Type::Union(sources) = from {
            let (member, tail) = if let Type::Union(targets) = to {
                let Some((Step::Variant(index), tail)) = path.split_first() else {
                    return Ok(path.to_vec());
                };
                (targets.get(*index).ok_or_else(Self::budget)?, tail)
            } else {
                (to, path)
            };
            let index = sources
                .iter()
                .position(|ty| ty == member)
                .ok_or_else(Self::budget)?;
            return Ok(std::iter::once(Step::Variant(index))
                .chain(tail.iter().copied())
                .collect());
        }
        if let Type::Union(targets) = to {
            if let Some((Step::Variant(index), tail)) = path.split_first() {
                if targets.get(*index) != Some(from) {
                    return Err(Self::budget());
                }
                return Ok(tail.to_vec());
            }
            return Ok(path.to_vec());
        }
        Err(Self::budget())
    }
}
