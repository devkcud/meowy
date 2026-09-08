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
            if !self.proofs.mutable.contains(local) {
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
        for value in target.values() {
            self.charge(value.len())?;
            node.defs.extend(value.values().copied());
        }
        let Some(proof) = proof else {
            return Ok((node, TRUE));
        };
        if proof.entered == FALSE {
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
                match (value.get(&path), current.and_then(|value| value.get(&path))) {
                    (Some(target), Some(source)) => {
                        node.transfers.push((*target, *source, active));
                    }
                    _ => missing = self.guards.or(missing, active),
                }
            }
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
        self.current = arm.ends;
        let node = self.append(node)?;
        if missing != FALSE {
            self.missing_headers.push((node, missing));
        }
        self.connect(start, TRUE, true);
        Ok(())
    }
}
