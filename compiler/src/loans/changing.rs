use super::{BlockId, Bundle, FALSE, Graph, Guard, Node, Result, Span, TRUE};

impl Graph<'_> {
    pub(crate) fn changing_header(
        &mut self,
        id: BlockId,
        incoming: &super::branches::Versions,
        header: &super::branches::Versions,
    ) -> Result<Guard> {
        let Some(ids) = self.facts.changing_published.get(&id) else {
            return Ok(FALSE);
        };
        self.charge(ids.len().saturating_mul(incoming.len() + header.len() + 1) + 1)?;
        for local in ids {
            self.charge(
                self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize
                    + self.proofs.mutable.len().checked_ilog2().unwrap_or(0) as usize
                    + self.blocks.len().checked_ilog2().unwrap_or(0) as usize
                    + 3,
            )?;
            if !incoming.contains_key(local) || !header.contains_key(local) {
                return Ok(TRUE);
            }
            let Some(alias) = self.proofs.aliases.get(local) else {
                return Ok(TRUE);
            };
            let Some(scope) = self.blocks.get(&alias.target) else {
                return Ok(TRUE);
            };
            let Some(ty) = self.program.locals.get(*local) else {
                return Ok(TRUE);
            };
            crate::borrow_contract::type_weight(&scope.ty, self.guards, Span::default())?;
            if !alias.mutable
                || alias.backing != Some(crate::borrow::Backing::Result)
                || !self.proofs.mutable.contains(local)
                || !ty.has_reference()
                || !ty.fixed_borrowed_value()
                || crate::borrow::aliases::result_slot(&scope.ty, &alias.field, ty).is_none()
            {
                return Ok(TRUE);
            }
        }
        Ok(FALSE)
    }

    pub(crate) fn retained_alias(&mut self, source: &Bundle, target: &Bundle) -> Result<Node> {
        self.charge(source.len() + target.len() + 1)?;
        let mut node = Node {
            defs: target.values().copied().collect(),
            ..Node::default()
        };
        for (path, target) in target {
            self.charge(
                (path.len() + 1)
                    .saturating_mul(source.len().checked_ilog2().unwrap_or(0) as usize + 1),
            )?;
            if let Some(source) = source.get(path) {
                node.transfers.push((*target, *source, TRUE));
                self.copy_link(&mut node, *target, *source, TRUE)?;
            }
        }
        Ok(node)
    }

    pub(crate) fn refresh_published(&mut self, id: BlockId) -> Result<()> {
        if self.current.is_empty() {
            return Ok(());
        }
        self.charge(
            self.facts
                .refresh_published
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize
                + 1,
        )?;
        let Some(ids) = self.facts.refresh_published.get(&id) else {
            return Ok(());
        };
        self.charge(ids.len() + 1)?;
        let ids = ids.iter().copied().collect::<Vec<_>>();
        for local in ids {
            self.charge(self.locals.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
            if !self.locals.contains_key(&local) {
                continue;
            }
            if let Some((source, target)) = self.alias_values(local, &[], Span::default())? {
                let node = self.retained_alias(&source, &target)?;
                self.append(node)?;
            }
        }
        Ok(())
    }
}
