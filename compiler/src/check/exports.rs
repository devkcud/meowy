use super::{Checker, Result, Value};
use crate::ast::{self, ExprKind, Span};
use crate::diagnostic::Diagnostic;
use crate::hir::{self, Type};
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Module {
    pub(crate) block: usize,
    pub(crate) values: BTreeMap<String, Value>,
}

impl Checker {
    pub(crate) fn module_value(
        &mut self,
        value: &ast::Expr,
        expected: Option<&Type>,
    ) -> Result<(hir::Expr, Module)> {
        if !matches!(value.kind, ExprKind::Block(_)) {
            return Err(Diagnostic::unsupported(
                "missing file-module initializer",
                value.span,
            ));
        }
        let saved = std::mem::replace(
            &mut self.module,
            Module {
                block: self.block,
                values: BTreeMap::new(),
            },
        );
        let result = self.expr(value, expected);
        let module = std::mem::replace(&mut self.module, saved);
        Ok((result?, module))
    }

    pub(crate) fn export_function(
        &mut self,
        label: Option<&str>,
        name: Option<&str>,
        annotation: Option<&ast::TypeExpr>,
        mutable: bool,
        value: &ast::Expr,
        span: Span,
    ) -> Result<bool> {
        if label.is_some()
            || self.owner != 0
            || self
                .frames
                .last()
                .is_none_or(|frame| frame.id != self.module.block)
        {
            return Ok(false);
        }
        let Some(name) = name else { return Ok(false) };
        if !self.flow.spend(name.len() + self.module.values.len() + 1) {
            return Err(Diagnostic::unsupported(
                "module export budget exhausted",
                span,
            ));
        }
        if self.module.values.contains_key(name) {
            return Err(Self::error(
                "E205",
                format!("module export `{name}` is already emitted"),
                span,
            ));
        }
        let ExprKind::Function { params, body } = &value.kind else {
            return Ok(false);
        };
        if self
            .frames
            .last()
            .unwrap()
            .slots
            .contains_key(&Some(name.into()))
        {
            return Err(Self::error(
                "E205",
                format!("module export `{name}` is already emitted"),
                span,
            ));
        }
        if mutable {
            return Err(Diagnostic::unsupported("mutable function exports", span));
        }
        let annotation = annotation.ok_or_else(|| {
            Self::error(
                "E214",
                "exported function requires an explicit result annotation",
                span,
            )
        })?;
        self.declare_function(name, Some(annotation), params, body, span)?;
        self.module
            .values
            .insert(name.into(), self.value(name, span)?);
        Ok(true)
    }

    pub(crate) fn module_member(
        &mut self,
        id: usize,
        ty: &Type,
        name: &str,
        span: Span,
    ) -> Result<Option<Value>> {
        if !self.flow.spend(name.len() + self.exports.len() + 1) {
            return Err(Diagnostic::unsupported(
                "module member lookup budget exhausted",
                span,
            ));
        }
        if let Some(value) = self.exports.get(&id).and_then(|values| values.get(name)) {
            return Ok(Some(value.clone()));
        }
        if let Type::Record { fields, .. } = ty {
            if !self.flow.spend(fields.len().saturating_mul(name.len() + 1)) {
                return Err(Diagnostic::unsupported(
                    "module member lookup budget exhausted",
                    span,
                ));
            }
            if fields.iter().any(|field| field.name == name) {
                return Ok(None);
            }
        }
        Err(Self::error(
            "E201",
            format!("module has no exported value `{name}`"),
            span,
        ))
    }
}
