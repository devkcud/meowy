use crate::ast::{self, ExprKind};
use crate::check::{Checker, Result, Value, inputs::Input};
use crate::diagnostic::Diagnostic;
use crate::hir::Type;

impl Checker {
    pub(crate) fn required_field(&mut self, expr: &ast::Expr) -> Result<(Type, Input)> {
        let ExprKind::Field { value, name } = &expr.kind else {
            unreachable!()
        };
        let mut root = value.as_ref();
        while let ExprKind::Group(value) = &root.kind {
            root = value;
        }
        let ExprKind::Name(root_name) = &root.kind else {
            return Err(Diagnostic::unsupported(
                "computed field roots outside named records",
                expr.span,
            ));
        };
        let Value::Local {
            id,
            ty: Type::Record { fields, .. },
            mutable: false,
            ..
        } = self.required_value(root_name, root.span)?
        else {
            return Err(Diagnostic::unsupported(
                "computed fields outside immutable local records",
                expr.span,
            ));
        };
        if !self.flow.spend(fields.len() + name.len() + 1) {
            return Err(super::Work::budget(expr.span));
        }
        let (index, field) = fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == *name)
            .ok_or_else(|| {
                Self::error("E201", format!("unknown record field `{name}`"), expr.span)
            })?;
        let input = self.field_input(id, index).ok_or_else(|| {
            Self::error(
                "E211",
                "record initializer is unavailable during required type evaluation",
                expr.span,
            )
        })?;
        Ok((field.ty.clone(), input))
    }
}
