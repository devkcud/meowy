use super::{Graph, LocalId, Result, Span, Step};
use crate::borrow::Backing;
use crate::hir::WriteStep;

impl Graph<'_> {
    pub(crate) fn sync_alias(&mut self, id: LocalId, path: &[WriteStep], span: Span) -> Result<()> {
        self.charge(self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
        let Some(alias) = self.proofs.aliases.get(&id) else {
            return Ok(());
        };
        let ty = &self.program.locals[id];
        if !alias.mutable || !ty.has_reference() || !ty.fixed_borrowed_value() {
            return Err(Self::budget());
        }
        if alias.backing == Some(Backing::Discarded) {
            return Ok(());
        }
        let scope = self.blocks.get(&alias.target).ok_or_else(Self::budget)?;
        if !self.guards.spend(alias.field.len() + path.len() + 1) {
            return Err(Self::budget());
        }
        crate::borrow_contract::type_weight(&scope.ty, self.guards, span)?;
        let (mut prefix, backing) =
            crate::borrow::slot(&scope.ty, &Some(alias.field.clone())).ok_or_else(Self::budget)?;
        if alias.backing != Some(Backing::Result) || backing != ty {
            return Err(Self::budget());
        }
        let path = path
            .iter()
            .map(|step| match step {
                WriteStep::Field(index) => Ok(Step::Slot(index + 1)),
                WriteStep::Index(_) => Err(Self::budget()),
            })
            .collect::<Result<Vec<_>>>()?;
        let work = scope
            .result
            .keys()
            .map(|path| path.len() + 1)
            .sum::<usize>();
        if !self.guards.spend(path.len() + prefix.len() + work + 1) {
            return Err(Self::budget());
        }
        prefix.extend_from_slice(&path);
        let target = Self::select(scope.result.clone(), &prefix);
        let source = Self::select(self.local(id)?, &path);
        let node = self.copied(&source, &target)?;
        self.append(node)?;
        Ok(())
    }
}
