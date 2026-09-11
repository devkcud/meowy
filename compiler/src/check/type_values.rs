use super::{Checker, Result, Value};
use crate::ast::{self, ExprKind, Span};
use crate::diagnostic::Diagnostic;
use crate::hir::Type;

pub(crate) const MAX_WORK: usize = 4096;
pub(crate) const MAX_DEPTH: usize = 64;
pub(crate) const MAX_NODES: usize = 16384;

#[derive(Default)]
pub(crate) struct Work {
    pub(crate) visits: usize,
    pub(crate) depth: usize,
    pub(crate) nodes: usize,
}

impl Work {
    pub(crate) fn budget(span: Span) -> Diagnostic {
        Diagnostic::unsupported("computed type bootstrap budget exhausted", span)
    }

    pub(crate) fn spend(&mut self, span: Span) -> Result<()> {
        self.visits += 1;
        if self.visits > MAX_WORK {
            return Err(Self::budget(span));
        }
        Ok(())
    }

    pub(crate) fn enter(&mut self, span: Span) -> Result<()> {
        self.spend(span)?;
        if self.depth == MAX_DEPTH {
            return Err(Self::budget(span));
        }
        self.depth += 1;
        Ok(())
    }

    pub(crate) fn materialize(&mut self, ty: &Type, span: Span) -> Result<()> {
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            self.nodes += 1;
            if self.nodes > MAX_NODES || pending.len() > MAX_NODES {
                return Err(Self::budget(span));
            }
            match ty {
                Type::Reference(ty) | Type::Exclusive(ty) | Type::List { element: ty, .. } => {
                    pending.push(ty);
                }
                Type::Record { primary, fields } => {
                    pending.push(primary);
                    pending.extend(fields.iter().map(|field| &field.ty));
                }
                Type::Union(members) => pending.extend(members),
                _ => {}
            }
        }
        Ok(())
    }
}

impl Checker {
    pub(crate) fn type_value(&mut self, expr: &ast::Expr) -> Result<Type> {
        let root = self.type_work.is_none();
        if root {
            self.type_work = Some(Work::default());
        }
        let result = (|| {
            self.type_work.as_mut().unwrap().enter(expr.span)?;
            let result = self.type_value_inner(expr);
            let work = self.type_work.as_mut().unwrap();
            work.depth -= 1;
            let ty = result?;
            work.materialize(&ty, expr.span)?;
            Ok(ty)
        })();
        if root {
            self.type_work = None;
        }
        result
    }

    pub(crate) fn type_value_inner(&mut self, expr: &ast::Expr) -> Result<Type> {
        match &expr.kind {
            ExprKind::TypeValue(ty) => self.ty(ty),
            ExprKind::TypeQuery(value) => {
                if let Some(ty) = self.hint(value) {
                    return Ok(ty);
                }
                match &value.kind {
                    ExprKind::Int(_) => Ok(Type::Int {
                        bits: 32,
                        signed: true,
                    }),
                    ExprKind::Float(_) => Ok(Type::Float { bits: 64 }),
                    ExprKind::String(_) => Ok(Type::String),
                    _ => Err(Diagnostic::unsupported(
                        "type queries requiring expression evaluation",
                        expr.span,
                    )),
                }
            }
            ExprKind::Name(name) => match self.value(name, expr.span)? {
                Value::Type(ty) => Ok(ty),
                _ => Err(Self::error(
                    "E211",
                    "computed annotation does not produce a compile-time type",
                    expr.span,
                )),
            },
            ExprKind::Group(value) => self.type_value(value),
            _ => Err(Diagnostic::unsupported(
                "computed type evaluation",
                expr.span,
            )),
        }
    }
}

#[cfg(test)]
mod tests;
