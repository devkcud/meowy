use super::{Checker, Result, Value};
use crate::ast::{self, ExprKind};
use crate::diagnostic::Diagnostic;
use crate::hir::{self, Type};

impl Checker {
    pub(crate) fn set_field(&mut self, target: &ast::Expr, value: &ast::Expr) -> Result<hir::Stmt> {
        let mut root = target;
        let mut fields = Vec::new();
        loop {
            if !self.flow.spend(1) {
                return Err(Diagnostic::unsupported(
                    "field assignment path budget exhausted",
                    target.span,
                ));
            }
            match &root.kind {
                ExprKind::Group(value) => root = value,
                ExprKind::Field { value, name } => {
                    if fields.len() == 256 {
                        return Err(Diagnostic::unsupported(
                            "field assignment path budget exhausted",
                            target.span,
                        ));
                    }
                    fields.push(name);
                    root = value;
                }
                _ => break,
            }
        }
        let ExprKind::Name(name) = &root.kind else {
            return Err(Diagnostic::unsupported(
                "field assignment outside ordinary local storage",
                target.span,
            ));
        };
        let Value::Local {
            id, ty, mutable, ..
        } = self.value(name, root.span)?
        else {
            return Err(Diagnostic::unsupported(
                "field assignment outside ordinary local storage",
                target.span,
            ));
        };
        if !self.places.contains(&id) || !matches!(ty, Type::Record { .. }) || ty.has_reference() {
            return Err(Diagnostic::unsupported(
                "field assignment requires ordinary reference-free record storage",
                target.span,
            ));
        }
        if !mutable {
            return Err(Self::error(
                "E305",
                format!("binding `{name}` is immutable"),
                root.span,
            ));
        }
        crate::borrow_contract::type_weight(&ty, &mut self.flow, target.span)?;
        let mut ty = ty;
        let mut path = Vec::new();
        let mut names = Vec::new();
        for name in fields.into_iter().rev() {
            let Type::Record { fields, .. } = ty else {
                return Err(Diagnostic::unsupported(
                    "field assignment requires concrete record paths",
                    target.span,
                ));
            };
            if !self.flow.spend(fields.len().saturating_mul(name.len() + 1)) {
                return Err(Diagnostic::unsupported(
                    "field assignment lookup budget exhausted",
                    target.span,
                ));
            }
            let (index, field) = fields
                .into_iter()
                .enumerate()
                .find(|(_, field)| &field.name == name)
                .ok_or_else(|| {
                    Self::error(
                        "E201",
                        format!("unknown record field `{name}`"),
                        target.span,
                    )
                })?;
            if !field.mutable {
                return Err(Self::error(
                    "E305",
                    format!("field `{name}` is immutable"),
                    target.span,
                ));
            }
            path.push(index);
            names.push(field.name);
            ty = field.ty;
        }
        let value = self.expr(value, Some(&ty))?;
        self.forget_field(id, &names, target.span)?;
        Ok(hir::Stmt::SetField {
            place: hir::Place {
                root: id,
                fields: path,
            },
            value,
            span: target.span,
        })
    }
}
