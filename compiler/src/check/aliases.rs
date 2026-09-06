use super::{Checker, Result};
use crate::ast::Span;
use crate::borrow::Alias;
use crate::diagnostic::Diagnostic;
use crate::flow::FALSE;
use crate::hir::{self, Type};

impl Checker {
    pub(crate) fn slot_alias(
        &mut self,
        id: usize,
        target: usize,
        field: &str,
        emission: usize,
        span: Span,
    ) -> Result<hir::Stmt> {
        if self.proofs.aliases.len() >= 65_536
            || !self
                .flow
                .spend((self.proofs.aliases.len() + 3).saturating_mul(field.len() + 1))
        {
            return Err(Diagnostic::unsupported(
                "result alias budget exhausted",
                span,
            ));
        }
        let root = self
            .proofs
            .aliases
            .iter()
            .find_map(|(id, alias)| (alias.target == target && alias.field == field).then_some(*id))
            .unwrap_or(id);
        self.proofs.aliases.insert(
            id,
            Alias {
                target,
                field: field.into(),
                emission,
                root,
                span,
            },
        );
        self.proofs.mutable.insert(id);
        Ok(hir::Stmt::SlotAlias {
            id,
            target,
            field: field.into(),
        })
    }

    pub(crate) fn validate_aliases(&mut self, target: usize, ty: &Type) -> Result<()> {
        if !self.flow.spend(self.proofs.aliases.len() + 1) {
            return Err(Diagnostic::unsupported(
                "result alias validation budget exhausted",
                Span::default(),
            ));
        }
        for (id, alias) in &self.proofs.aliases {
            if alias.target != target {
                continue;
            }
            let local = &self.locals[*id];
            crate::borrow_contract::type_weight(local, &mut self.flow, alias.span)?;
            crate::borrow_contract::type_weight(ty, &mut self.flow, alias.span)?;
            if let Type::Record { fields, .. } = ty
                && !self
                    .flow
                    .spend(fields.len().saturating_mul(alias.field.len() + 1))
            {
                return Err(Diagnostic::unsupported(
                    "result alias field lookup budget exhausted",
                    alias.span,
                ));
            }
            if let Type::Record { fields, .. } = ty
                && fields.iter().any(|field| {
                    field.name == alias.field && field.mutable && field.ty.accepts(local)
                })
            {
                continue;
            }
            let emitted = self
                .proofs
                .emissions
                .get(&alias.emission)
                .copied()
                .ok_or_else(|| {
                    Diagnostic::unsupported("missing result alias initialization proof", alias.span)
                })?;
            if self.flow.and(emitted, self.reach) != FALSE {
                return Err(Diagnostic::unsupported(
                    "result alias needs a compatible concrete completed field",
                    alias.span,
                ));
            }
        }
        Ok(())
    }
}
