use super::{Bundle, CallId, FALSE, Graph, Node, Result, Span, Type};

impl Graph<'_> {
    pub(crate) fn returned(
        &mut self,
        site: CallId,
        inputs: &[Bundle],
        result: &Bundle,
        ty: &Type,
        span: Span,
    ) -> Result<()> {
        let Some(transfers) = self.facts.returns.get(&site) else {
            let node = self.append(Node::default())?;
            self.missing_calls.push((node, span));
            return Ok(());
        };
        self.charge(transfers.len() * 2 + 1)?;
        let transfers = transfers.clone();
        let Some(root) = result.get(&Vec::new()).copied() else {
            return Ok(());
        };
        let weight = self.values[root]
            .origins
            .iter()
            .map(super::Origin::weight)
            .sum::<usize>();
        self.charge(weight)?;
        let parent = self.value(self.values[root].origins.clone())?;
        let mut node = Node {
            defs: vec![parent, root],
            ..Node::default()
        };
        let mut covered = FALSE;
        let mut missing = false;
        for transfer in transfers {
            if transfer.guard == FALSE {
                continue;
            }
            let Some(source) = inputs
                .get(transfer.input)
                .and_then(|value| value.get(&Vec::new()))
                .copied()
            else {
                missing = true;
                continue;
            };
            self.copy_link(&mut node, parent, source, transfer.guard)?;
            covered = self.guards.or(covered, transfer.guard);
        }
        let state = self.facts.calls.get(&site).ok_or_else(Self::budget)?;
        let normal = self.guards.and(state.present, state.proof);
        missing |= !self.guards.implies(normal, covered);
        let node = self.append(node)?;
        if missing {
            self.missing_calls.push((node, span));
        }
        self.grant_mode(
            node,
            root,
            Some(parent),
            ty.reference_mode().ok_or_else(Self::budget)?,
        )?;
        Ok(())
    }
}
