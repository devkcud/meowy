use super::branches::Versions;
use super::{BlockId, Diagnostic, Graph, Node, Result, Span, TRUE, Type};

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
            let valid =
                self.program.locals.get(*local).is_some_and(
                    |ty| matches!(ty, Type::Reference(target) if !target.has_reference()),
                );
            if !valid
                || !self.proofs.mutable.contains(local)
                || state.present != TRUE
                || state.proof != TRUE
                || !state.active.is_empty()
                || state
                    .origins
                    .iter()
                    .chain(&state.bounds)
                    .any(|origin| !origin.component.is_empty() || origin.guard != TRUE)
            {
                return Err(Diagnostic::unsupported(
                    "restart header outside canonical reference-free pointees",
                    Span::default(),
                ));
            }
            let origins = state.origins.iter().chain(&state.bounds).cloned().collect();
            header.insert(*local, self.bundle(origins)?);
        }
        let (node, missing) = self.header_transfer(incoming, &header)?;
        let node = self.append(node)?;
        if missing {
            self.missing_headers.push(node);
        }
        Ok(Some(header))
    }

    pub(crate) fn header_transfer(
        &mut self,
        source: &Versions,
        target: &Versions,
    ) -> Result<(Node, bool)> {
        self.charge(source.len() + target.len() + 1)?;
        let mut missing = source.len() != target.len();
        let mut node = Node::default();
        for (id, value) in target {
            for (path, target) in value {
                self.charge(path.len() + 1)?;
                node.defs.push(*target);
                if let Some(source) = source.get(id).and_then(|source| source.get(path)) {
                    node.transfers.push((*target, *source));
                } else {
                    missing = true;
                }
            }
            if value.is_empty() {
                missing = true;
            }
        }
        Ok((node, missing))
    }

    pub(crate) fn restart(&mut self, id: BlockId) -> Result<()> {
        let scope = self.blocks.get(&id).expect("restart target");
        let start = scope.start;
        if !scope.restarted {
            let node = self.append(Node::default())?;
            self.missing_headers.push(node);
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
        let (node, missing) = self.header_transfer(&arm.versions, &header)?;
        self.current = arm.ends;
        let node = self.append(node)?;
        if missing {
            self.missing_headers.push(node);
        }
        self.connect(start, TRUE, true);
        Ok(())
    }
}
