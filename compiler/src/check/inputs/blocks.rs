use super::{Checker, Input};
use crate::hir;

impl Checker {
    pub(crate) fn input_block(
        &mut self,
        block: &hir::Block,
        depth: usize,
        count: &mut usize,
    ) -> Option<Input> {
        let mut result = Input {
            work: 0,
            error: None,
            value: None,
        };
        let mut emitted = false;
        for stmt in &block.stmts {
            *count += 1;
            if *count > super::super::type_values::MAX_WORK
                || depth >= super::super::type_values::MAX_DEPTH
                || !self.flow.spend(1)
            {
                return None;
            }
            result.work = result.work.saturating_add(1);
            match stmt {
                hir::Stmt::Bind { id, .. } if !self.proofs.mutable.contains(id) => {
                    result.add(self.inputs.get(id)?);
                }
                hir::Stmt::Emit {
                    target,
                    field: None,
                    value,
                    ..
                } if *target == block.id && !emitted => {
                    let source = self.input_expr(value, depth, count)?;
                    result.add(&source);
                    result.value = source.value;
                    emitted = true;
                }
                _ => return None,
            }
        }
        emitted.then_some(result)
    }
}
