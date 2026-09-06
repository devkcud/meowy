use super::{Checker, Place, Result, Value};
use crate::ast::{self, ExprKind, Span};
use crate::diagnostic::Diagnostic;
use crate::hir::{self, Type};

impl Checker {
    pub(crate) fn address_root<'a>(expr: &'a ast::Expr, fields: &mut Vec<String>) -> &'a ast::Expr {
        match &expr.kind {
            ExprKind::Group(value) => Self::address_root(value, fields),
            ExprKind::Field { value, name } => {
                let root = Self::address_root(value, fields);
                fields.push(name.clone());
                root
            }
            _ => expr,
        }
    }

    pub(crate) fn address_hint(&mut self, expr: &ast::Expr) -> Option<Type> {
        if let Ok((_, ty)) = self.address(expr) {
            return Some(ty);
        }
        match &expr.kind {
            ExprKind::Group(value) => self.address_hint(value),
            ExprKind::Unary { op, value } if op == "*" => match self.hint(value)? {
                Type::Reference(ty) => Some(*ty),
                _ => None,
            },
            ExprKind::Field { .. } => self.hint(expr),
            ExprKind::Index { value, .. } => {
                let ty = self.address_hint(value).or_else(|| self.hint(value))?;
                let ty = if let Type::Reference(ty) = ty {
                    *ty
                } else {
                    ty
                };
                match ty {
                    Type::List { element, .. } => Some(*element),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(crate) fn borrowed(&mut self, expr: &ast::Expr, span: Span) -> Result<hir::Expr> {
        match &expr.kind {
            ExprKind::Group(value) => return self.borrowed(value, span),
            ExprKind::Index { value, index } => return self.element_borrow(value, index, span),
            _ => {}
        }
        let error = match self.address(expr) {
            Ok((place, ty)) => {
                if let Some(alias) = self.proofs.aliases.get_mut(&place.root) {
                    alias.borrowed.get_or_insert(span);
                }
                return Ok(hir::Expr {
                    kind: hir::ExprKind::Borrow(place),
                    ty: Type::Reference(Box::new(ty)),
                    span,
                });
            }
            Err(error) => error,
        };
        let mut names = Vec::new();
        let root = Self::address_root(expr, &mut names);
        let mut value = if matches!(root.kind, ExprKind::Index { .. }) {
            self.borrowed(root, root.span)?
        } else if let ExprKind::Unary { op, value } = &root.kind
            && op == "*"
        {
            self.expr(value, None)?
        } else if !names.is_empty() {
            self.expr(root, None)?
        } else {
            return Err(error);
        };
        if value.ty == Type::Never {
            return Ok(value);
        }
        let mut names = names.into_iter().peekable();
        while !matches!(value.ty, Type::Reference(_)) {
            let Some(name) = names.next() else {
                return Err(error);
            };
            let Type::Record { fields, .. } = &value.ty else {
                return Err(error);
            };
            let (index, ty) = fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == name)
                .map(|(index, field)| (index, field.ty.clone()))
                .ok_or_else(|| {
                    Self::error("E201", format!("unknown record field `{name}`"), span)
                })?;
            value = self.narrow(hir::Expr {
                kind: hir::ExprKind::Field {
                    value: Box::new(value),
                    index,
                },
                ty,
                span: expr.span,
            });
            if names.peek().is_none() {
                return Err(error);
            }
        }
        let Type::Reference(target) = &value.ty else {
            unreachable!()
        };
        if target.has_reference() {
            return Err(Diagnostic::unsupported(
                "reborrowing reference-carrying referents",
                span,
            ));
        }
        let mut ty = target.as_ref();
        let mut path = Vec::new();
        for name in names {
            let Type::Record { fields, .. } = ty else {
                return Err(Diagnostic::unsupported(
                    "reborrow projection outside concrete record storage",
                    span,
                ));
            };
            let (index, field) = fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == name)
                .ok_or_else(|| {
                    Self::error("E201", format!("unknown record field `{name}`"), span)
                })?;
            path.push(index);
            ty = &field.ty;
        }
        let ty = Type::Reference(Box::new(ty.clone()));
        let site = self.reborrows;
        self.reborrows += 1;
        Ok(hir::Expr {
            kind: hir::ExprKind::Reborrow {
                site,
                value: Box::new(value),
                fields: path,
            },
            ty,
            span,
        })
    }

    pub(crate) fn address(&self, expr: &ast::Expr) -> Result<(hir::Place, Type)> {
        match &expr.kind {
            ExprKind::Group(value) => self.address(value),
            ExprKind::Name(name) => {
                let Value::Local { id, ty, .. } = self.value(name, expr.span)? else {
                    return Err(Diagnostic::unsupported(
                        "borrowing temporary or intrinsic values",
                        expr.span,
                    ));
                };
                if !self.places.contains(&id) && !self.proofs.aliases.contains_key(&id) {
                    return Err(Diagnostic::unsupported(
                        "borrowing emitted storage",
                        expr.span,
                    ));
                }
                if ty.has_reference() {
                    return Err(Diagnostic::unsupported(
                        "borrowing reference-carrying storage",
                        expr.span,
                    ));
                }
                Ok((
                    hir::Place {
                        root: id,
                        fields: Vec::new(),
                    },
                    ty,
                ))
            }
            ExprKind::Field { value, name } => {
                let (mut place, ty) = self.address(value)?;
                let Type::Record { fields, .. } = ty else {
                    return Err(Diagnostic::unsupported(
                        "borrowing fields outside concrete record storage",
                        expr.span,
                    ));
                };
                let (index, field) = fields
                    .into_iter()
                    .enumerate()
                    .find(|(_, field)| &field.name == name)
                    .ok_or_else(|| {
                        Self::error("E201", format!("unknown record field `{name}`"), expr.span)
                    })?;
                place.fields.push(index);
                Ok((place, field.ty))
            }
            _ => Err(Diagnostic::unsupported(
                "borrowing temporary or projected storage",
                expr.span,
            )),
        }
    }

    pub(crate) fn ast_place(&self, expr: &ast::Expr) -> Option<Place> {
        match &expr.kind {
            ExprKind::Name(name) => match self.value(name, expr.span).ok()? {
                Value::Local { id, .. } => Some((id, Vec::new())),
                _ => None,
            },
            ExprKind::Group(value) | ExprKind::Ascribe { value, .. } => self.ast_place(value),
            ExprKind::Field { value, name } => {
                let mut place = self.ast_place(value)?;
                place.1.push(name.clone());
                Some(place)
            }
            _ => None,
        }
    }
}
