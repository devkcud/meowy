use super::branches::Versions;
use super::{BlockId, Diagnostic, FALSE, Graph, Guard, Node, Result, Span, TRUE};
use crate::borrow::header::Shape;
use crate::borrow::state::Predecessor;
use crate::hir::RestartId;

impl Graph<'_> {
    pub(crate) fn restart_header(
        &mut self,
        id: BlockId,
        incoming: &Versions,
    ) -> Result<Option<Versions>> {
        let Some(states) = self.facts.headers.get(&id) else {
            return Ok(None);
        };
        self.charge(states.len() + 1)?;
        let mut header = Versions::new();
        for (local, state) in states {
            self.charge(state.weight() + 1)?;
            let ty = self.program.locals.get(*local).ok_or_else(Self::budget)?;
            if !self.proofs.variable(*local) {
                return Err(Diagnostic::unsupported(
                    "restart header outside mutable reference storage",
                    Span::default(),
                ));
            }
            let shape = Shape::new(ty, self.guards, Span::default())?;
            shape.validate(state, true, self.guards, Span::default())?;
            header.insert(*local, self.bundle(super::values::Value::from(state), ty)?);
        }
        let proof = self.facts.header_inputs.get(&id);
        let (mut node, missing) = self.header_transfer(incoming, &header, proof)?;
        let fixed = self.fixed_published(id, self.facts.published_inputs.get(&id))?;
        let missing = self.guards.or(missing, fixed);
        let changing = self.changing_header(id, incoming, &header)?;
        let missing = self.guards.or(missing, changing);
        for value in header.values().flat_map(|value| value.values()) {
            self.opaque_value(&mut node, *value)?;
        }
        let node = self.append(node)?;
        if missing != FALSE {
            self.missing_headers.push((node, missing));
        }
        Ok(Some(header))
    }

    pub(crate) fn header_transfer(
        &mut self,
        source: &Versions,
        target: &Versions,
        proof: Option<&Predecessor>,
    ) -> Result<(Node, Guard)> {
        self.charge(source.len() + target.len() + 1)?;
        let mut node = Node::default();
        if !self.proofs.carried.is_empty() {
            self.reserve_authority(2)?;
            node.header = Some(super::restart_headers::Header {
                required: super::BTreeMap::new(),
                missing: TRUE,
            });
        }
        for value in target.values() {
            self.charge(value.len())?;
            node.defs.extend(value.values().copied());
            if let Some(header) = &mut node.header {
                self.reserve_authority(value.len().saturating_mul(2) + 1)?;
                for id in value.values() {
                    header.required.insert(*id, FALSE);
                }
            }
        }
        let Some(proof) = proof else {
            return Ok((node, TRUE));
        };
        if proof.entered == FALSE {
            if let Some(header) = &mut node.header {
                header.missing = FALSE;
            }
            return Ok((node, FALSE));
        }
        self.charge(proof.values.len() + 1)?;
        let mut missing =
            if source.keys().eq(target.keys()) && proof.values.keys().eq(target.keys()) {
                FALSE
            } else {
                proof.entered
            };
        for (id, value) in target {
            let Some(state) = proof.values.get(id) else {
                continue;
            };
            let ty = self.program.locals.get(*id).ok_or_else(Self::budget)?;
            let shape = Shape::new(ty, self.guards, Span::default())?;
            let activity = shape.inspect(state, false, self.guards, Span::default())?;
            let valid = self.guards.and(state.proof, proof.entered);
            let current = source.get(id);
            self.charge(value.len() + current.map_or(0, |value| value.len()) + 1)?;
            if value.keys().any(|path| !activity.paths.contains_key(path))
                || current.is_some_and(|value| {
                    value.keys().any(|path| !activity.paths.contains_key(path))
                })
            {
                missing = self.guards.or(missing, proof.entered);
            }
            for (path, active) in activity.paths {
                self.charge(path.len() + 1)?;
                let active = self.guards.and(active, valid);
                if active == FALSE {
                    continue;
                }
                if let Some(header) = &mut node.header
                    && let Some(target) = value.get(&path)
                {
                    self.charge(header.required.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
                    let required = header.required.get_mut(target).ok_or_else(Self::budget)?;
                    *required = self.guards.or(*required, active);
                }
                match (value.get(&path), current.and_then(|value| value.get(&path))) {
                    (Some(target), Some(source)) => {
                        node.transfers.push((*target, *source, active));
                    }
                    _ => missing = self.guards.or(missing, active),
                }
            }
        }
        if let Some(header) = &mut node.header {
            header.missing = missing;
        }
        Ok((node, missing))
    }

    pub(crate) fn restart(&mut self, id: BlockId, site: RestartId) -> Result<()> {
        let scope = self.blocks.get(&id).expect("restart target");
        let start = scope.start;
        if !scope.restarted {
            let node = self.append(Node::default())?;
            self.missing_headers.push((node, TRUE));
            self.connect(start, TRUE, true);
            return Ok(());
        }
        let work = scope
            .incoming
            .values()
            .flat_map(|value| value.keys())
            .map(|path| path.len() + 1)
            .sum::<usize>()
            + scope.incoming.len()
            + 1;
        self.charge(work)?;
        let header = self.blocks[&id].incoming.clone();
        let ids = header.keys().copied().collect::<Vec<_>>();
        let arm = self.surviving_arm(&ids)?;
        let proof = self.facts.restart_inputs.get(&site);
        let (node, missing) = self.header_transfer(&arm.versions, &header, proof)?;
        let fixed = self.fixed_published(id, self.facts.published_restarts.get(&site))?;
        let missing = self.guards.or(missing, fixed);
        self.current = arm.ends;
        let node = self.append(node)?;
        if missing != FALSE {
            self.missing_headers.push((node, missing));
        }
        self.connect(start, TRUE, true);
        Ok(())
    }

    pub(crate) fn fixed_published(
        &mut self,
        id: BlockId,
        current: Option<&crate::borrow::published::Snapshot>,
    ) -> Result<Guard> {
        self.charge(
            self.facts
                .fixed_published
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize
                + 1,
        )?;
        let Some(targets) = self.facts.fixed_published.get(&id) else {
            return Ok(FALSE);
        };
        let (Some(initial), Some(current)) = (self.facts.published_inputs.get(&id), current) else {
            return Ok(TRUE);
        };
        if current.entered == FALSE {
            return Ok(FALSE);
        }
        for input in [initial, current] {
            self.charge(input.slots.len().saturating_mul(targets.len() + 1) + 1)?;
            for ((owner, index), slot) in &input.slots {
                if !targets.contains(owner) {
                    continue;
                }
                let size =
                    crate::borrow_contract::type_weight(&slot.ty, self.guards, Span::default())?;
                self.charge(size + 1)?;
                let Some(scope) = self.blocks.get(owner) else {
                    return Ok(current.entered);
                };
                let super::Type::Record { fields, .. } = &scope.ty else {
                    return Ok(current.entered);
                };
                let Some(field) = index.checked_sub(1).and_then(|index| fields.get(index)) else {
                    return Ok(current.entered);
                };
                if (!field.mutable && !field.ty.has_mutable_fields()) || field.ty != slot.ty {
                    return Ok(current.entered);
                }
            }
        }
        crate::borrow::published::unchanged(initial, current, targets, self.guards, Span::default())
    }
}
