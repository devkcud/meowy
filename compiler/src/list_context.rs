use std::collections::BTreeSet;

use crate::ast::{self, ExprKind, Span, StmtKind};
use crate::check::{Checker, Constant, Result, Scope, Value};
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Guard, TRUE};
use crate::hir::{self, Type};

pub(crate) const MAX_CONTEXTS: usize = 256;
pub(crate) const MAX_SCALAR_NODES: usize = 4096;

pub(crate) struct Scalar {
    pub(crate) checker: Checker,
    pub(crate) nodes: usize,
    pub(crate) bytes: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fit {
    Yes,
    No,
    Unknown,
}

impl Fit {
    pub(crate) fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::No, _) | (_, Self::No) => Self::No,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            _ => Self::Yes,
        }
    }
}

impl Checker {
    pub(crate) fn list_union(
        &mut self,
        values: &[ast::Expr],
        contexts: &[&Type],
        span: Span,
    ) -> Result<hir::Expr> {
        if contexts.len() > MAX_CONTEXTS {
            return Err(Diagnostic::unsupported(
                "list candidate budget exhausted",
                span,
            ));
        }
        let mut choices: Vec<_> = contexts
            .iter()
            .copied()
            .filter(|ty| matches!(ty, Type::List { capacity, .. } if *capacity >= values.len()))
            .collect();
        if choices.is_empty() {
            return Err(Self::error(
                "E103",
                format!(
                    "{} elements exceed every expected list capacity",
                    values.len()
                ),
                span,
            ));
        }
        if choices.len() == 1 {
            return self.list_literal(values, Some(choices[0]), span);
        }
        let mut viable = Vec::new();
        for choice in choices {
            let Type::List { element, .. } = choice else {
                unreachable!()
            };
            let mut fit = Fit::Yes;
            for value in values {
                fit = fit.and(self.list_probe(value, element, FALSE)?);
            }
            if fit != Fit::No {
                viable.push(choice);
            }
        }
        choices = viable;
        if choices.is_empty() {
            return Err(Self::error(
                "E207",
                "list literal fits no expected list type",
                span,
            ));
        }
        if choices.len() == 1 {
            return self.list_literal(values, Some(choices[0]), span);
        }
        let mut items = vec![None; values.len()];
        let mut deferred = Vec::new();
        for (index, value) in values.iter().enumerate() {
            if choices.len() > 1 && self.list_deferred(value)? {
                choices = self.list_filter(value, choices, self.reach)?;
                if choices.len() > 1 {
                    deferred.push((index, self.reach));
                    continue;
                }
            }
            let elements: Vec<_> = choices
                .iter()
                .map(|ty| {
                    let Type::List { element, .. } = ty else {
                        unreachable!()
                    };
                    element.as_ref()
                })
                .collect();
            let common = elements.iter().all(|ty| *ty == elements[0]);
            let item = if !common
                && let Some(item) = self.list_effect_block(
                    value,
                    &mut choices,
                    !deferred.is_empty() || index + 1 < values.len(),
                )? {
                item
            } else if common {
                self.expr(value, Some(elements[0]))?
            } else if self.list_independent(value) {
                self.expr(value, None)?
            } else {
                return Err(Diagnostic::unsupported(
                    "list element needs a concrete contextual type before its effects can be checked",
                    value.span,
                ));
            };
            let mut matching = Vec::new();
            for ty in choices {
                let Type::List { element, .. } = ty else {
                    unreachable!()
                };
                crate::borrow_contract::type_weight(&item.ty, &mut self.flow, value.span)?;
                if Self::list_assigns(element, &item.ty) {
                    matching.push(ty);
                }
            }
            choices = matching;
            if choices.is_empty() {
                return Err(Self::error(
                    "E207",
                    "list element fits no expected list type",
                    value.span,
                ));
            }
            items[index] = Some(item);
        }
        for (index, reach) in &deferred {
            if choices.len() == 1 {
                break;
            }
            choices = self.list_filter(&values[*index], choices, *reach)?;
        }
        if choices.len() != 1 {
            for choice in &choices {
                let Type::List { element, .. } = choice else {
                    unreachable!()
                };
                for (index, reach) in &deferred {
                    if self.list_probe(&values[*index], element, *reach)? == Fit::Unknown {
                        return Err(Diagnostic::unsupported(
                            "list candidates need unresolved nested contextual inference",
                            values[*index].span,
                        ));
                    }
                }
            }
            return Err(Self::error(
                "E207",
                "list literal fits multiple expected list types",
                span,
            ));
        }
        let list = choices[0].clone();
        let Type::List { element, .. } = &list else {
            unreachable!()
        };
        for (index, reach) in deferred {
            let after = std::mem::replace(&mut self.reach, reach);
            let result = self.expr(&values[index], Some(element));
            self.reach = after;
            items[index] = Some(result?);
        }
        let values = items
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                Self::expected_value(
                    value.expect("checked list element"),
                    element,
                    values[index].span,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let ty = if values.iter().any(|value| value.ty == Type::Never) {
            Type::Never
        } else {
            list.clone()
        };
        Ok(hir::Expr {
            kind: hir::ExprKind::List { values, list },
            ty,
            span,
        })
    }

    pub(crate) fn list_effect_block(
        &mut self,
        value: &ast::Expr,
        choices: &mut Vec<&Type>,
        unresolved: bool,
    ) -> Result<Option<hir::Expr>> {
        let mut form = value;
        while let ExprKind::Group(value) = &form.kind {
            form = value;
        }
        let ExprKind::Block(block) = &form.kind else {
            return Ok(None);
        };
        if block.label.is_some() {
            return Ok(None);
        }
        let mut start = None;
        for (index, stmt) in block.stmts.iter().enumerate() {
            if !self.flow.spend(1) {
                return Err(Diagnostic::unsupported(
                    "effectful list block budget exhausted",
                    stmt.span,
                ));
            }
            match &stmt.kind {
                StmtKind::Emit {
                    label: None,
                    ty: None,
                    mutable: false,
                    ..
                } => {
                    start.get_or_insert(index);
                }
                StmtKind::Bind { .. } | StmtKind::Assign { .. } | StmtKind::Expr(_)
                    if start.is_none() => {}
                _ => return Ok(None),
            }
        }
        let Some(start) = start.filter(|start| *start > 0) else {
            return Ok(None);
        };
        let mut stmts = self.block_start(block, None, None, false)?;
        for stmt in &block.stmts[..start] {
            stmts.extend(self.stmt(stmt)?);
        }
        let mut collision = None;
        let mut nodes = 1usize;
        for stmt in &block.stmts[start..] {
            let StmtKind::Emit { value, .. } = &stmt.kind else {
                unreachable!()
            };
            let Some(scalar) = self.list_pure(value, true)? else {
                return Err(Diagnostic::unsupported(
                    "effectful list result needs unresolved lexical or effect context",
                    value.span,
                ));
            };
            nodes = nodes.saturating_add(scalar.nodes).saturating_add(1);
            if nodes > MAX_SCALAR_NODES {
                return Err(Diagnostic::unsupported(
                    "effectful list suffix budget exhausted",
                    stmt.span,
                ));
            }
            if let StmtKind::Emit {
                name: Some(name), ..
            } = &stmt.kind
                && self
                    .scopes
                    .last()
                    .expect("block scope")
                    .values
                    .contains_key(name)
            {
                collision = Some((name, stmt.span));
            }
        }
        if !self.flow.spend(
            block
                .span
                .end
                .saturating_sub(block.stmts[start].span.start)
                .saturating_add(1),
        ) {
            return Err(Diagnostic::unsupported(
                "effectful list suffix budget exhausted",
                block.span,
            ));
        }
        let suffix = ast::Expr {
            kind: ExprKind::Block(ast::Block {
                label: None,
                stmts: block.stmts[start..].to_vec(),
                span: block.span,
            }),
            span: form.span,
        };
        let mut matching = Vec::new();
        let mut unknown = false;
        let mut errors = Vec::new();
        for ty in choices.iter().copied() {
            let Type::List { element, .. } = ty else {
                unreachable!()
            };
            let Some(scalar) = self.list_pure(&suffix, true)? else {
                return Err(Diagnostic::unsupported(
                    "effectful list result needs unresolved lexical or effect context",
                    form.span,
                ));
            };
            let fit = match self.list_pure_probe(scalar, &suffix, element, self.reach) {
                Err(error) if ["E203", "E205", "E206"].contains(&error.code) => {
                    errors.push(error);
                    Fit::No
                }
                result => result?,
            };
            if fit != Fit::No {
                matching.push(ty);
                unknown |= fit == Fit::Unknown;
            }
        }
        if matching.is_empty() {
            if errors.len() == choices.len()
                && errors.iter().all(|error| error.code == errors[0].code)
            {
                return Err(errors.remove(0));
            }
            return Err(Self::error(
                "E207",
                "list element fits no expected list type",
                form.span,
            ));
        }
        if matching.len() > 1 {
            if let Some((name, span)) = collision {
                return Err(Self::error(
                    "E203",
                    format!("value `{name}` is already declared in this scope"),
                    span,
                ));
            }
            return Err(if unknown || unresolved {
                Diagnostic::unsupported(
                    "effectful list result has unresolved candidate constraints",
                    form.span,
                )
            } else {
                Self::error(
                    "E207",
                    "list literal fits multiple expected list types",
                    form.span,
                )
            });
        }
        let Type::List { element, .. } = matching[0] else {
            unreachable!()
        };
        self.frames.last_mut().expect("block frame").expected = Some(*element.clone());
        for stmt in &block.stmts[start..] {
            stmts.extend(self.stmt(stmt)?);
        }
        let block = self.block_end(block, stmts)?;
        let ty = block.ty.clone();
        *choices = matching;
        Ok(Some(hir::Expr {
            kind: hir::ExprKind::Block(block),
            ty,
            span: value.span,
        }))
    }

    pub(crate) fn list_filter<'a>(
        &mut self,
        value: &ast::Expr,
        choices: Vec<&'a Type>,
        reach: Guard,
    ) -> Result<Vec<&'a Type>> {
        let mut matching = Vec::new();
        for ty in choices {
            let Type::List { element, .. } = ty else {
                unreachable!()
            };
            if self.list_probe(value, element, reach)? != Fit::No {
                matching.push(ty);
            }
        }
        if matching.is_empty() {
            return Err(Self::error(
                "E207",
                "list element fits no expected list type",
                value.span,
            ));
        }
        Ok(matching)
    }

    pub(crate) fn list_assigns(expected: &Type, actual: &Type) -> bool {
        expected.accepts(actual)
            || matches!(actual, Type::Record { primary, .. }
            if !matches!(expected, Type::Record { .. }) && expected.accepts(primary))
    }

    pub(crate) fn list_independent(&mut self, value: &ast::Expr) -> bool {
        match &value.kind {
            ExprKind::Name(_)
            | ExprKind::Call { .. }
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Ascribe { .. }
            | ExprKind::Dispatch { .. } => true,
            ExprKind::Group(value) => self.list_independent(value),
            ExprKind::Unary { op, .. } if ["&", "*", "!"].contains(&op.as_str()) => true,
            ExprKind::Binary { .. } | ExprKind::Unary { .. } => self.hint(value).is_some(),
            _ => false,
        }
    }

    pub(crate) fn list_deferred(&mut self, value: &ast::Expr) -> Result<bool> {
        if !self.flow.spend(
            value
                .span
                .end
                .saturating_sub(value.span.start)
                .saturating_add(1),
        ) {
            return Err(Diagnostic::unsupported(
                "list deferral budget exhausted",
                value.span,
            ));
        }
        if self.scalar_literal(value) {
            return Ok(true);
        }
        match &value.kind {
            ExprKind::Group(value) => self.list_deferred(value),
            ExprKind::List(values) => {
                for value in values {
                    if !self.list_deferred(value)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ExprKind::Block(block) if block.label.is_none() => {
                for stmt in &block.stmts {
                    let StmtKind::Emit {
                        label: None,
                        ty: None,
                        mutable: false,
                        value,
                        ..
                    } = &stmt.kind
                    else {
                        return Ok(false);
                    };
                    if !self.list_deferred(value)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(self.list_scalar(value)?.is_some()),
        }
    }

    pub(crate) fn list_scalar(&mut self, value: &ast::Expr) -> Result<Option<Scalar>> {
        self.list_pure(value, false)
    }

    pub(crate) fn list_pure(
        &mut self,
        value: &ast::Expr,
        aggregate: bool,
    ) -> Result<Option<Scalar>> {
        let mut scalar = Scalar {
            checker: Self::new(),
            nodes: 0,
            bytes: 0,
        };
        scalar.checker.scopes[0].values.clear();
        let mut pending = vec![value];
        let mut emitted = BTreeSet::new();
        let mut used = BTreeSet::new();
        while let Some(value) = pending.pop() {
            scalar.nodes += 1;
            if scalar.nodes > MAX_SCALAR_NODES
                || pending.len().saturating_add(2) > MAX_SCALAR_NODES
                || !self.flow.spend(1)
            {
                return Err(Diagnostic::unsupported(
                    "scalar list probe budget exhausted",
                    value.span,
                ));
            }
            let bytes = match &value.kind {
                ExprKind::List(values) if aggregate => {
                    for value in values {
                        if pending.len() == MAX_SCALAR_NODES || !self.flow.spend(1) {
                            return Err(Diagnostic::unsupported(
                                "pure list suffix budget exhausted",
                                value.span,
                            ));
                        }
                        pending.push(value);
                    }
                    0
                }
                ExprKind::Block(block) if aggregate && block.label.is_none() => {
                    let mut bytes = 0usize;
                    for stmt in &block.stmts {
                        let StmtKind::Emit {
                            label: None,
                            name,
                            ty: None,
                            mutable: false,
                            value,
                        } = &stmt.kind
                        else {
                            return Ok(None);
                        };
                        if pending.len() == MAX_SCALAR_NODES
                            || !self
                                .flow
                                .spend(name.as_ref().map_or(1, |name| name.len() + 1))
                        {
                            return Err(Diagnostic::unsupported(
                                "pure list suffix budget exhausted",
                                value.span,
                            ));
                        }
                        if let Some(name) = name {
                            emitted.insert(name.as_str());
                            bytes = bytes.saturating_add(name.len());
                        }
                        pending.push(value);
                    }
                    bytes
                }
                ExprKind::Int(text) | ExprKind::Float(text) => text.len(),
                ExprKind::String(parts) => {
                    let mut bytes = parts.len();
                    for part in parts {
                        let ast::StringPart::Text(text) = part else {
                            return Ok(None);
                        };
                        bytes = bytes.saturating_add(text.len());
                    }
                    bytes
                }
                ExprKind::Group(value) => {
                    pending.push(value);
                    0
                }
                ExprKind::Unary { op, value } if ["-", "!", "~"].contains(&op.as_str()) => {
                    pending.push(value);
                    0
                }
                ExprKind::Binary { op, left, right }
                    if [
                        "+", "-", "*", "/", "%", "&", "|", "^", "&&", "||", "==", "!=", "<", ">",
                        "<=", ">=",
                    ]
                    .contains(&op.as_str()) =>
                {
                    pending.push(right);
                    pending.push(left);
                    0
                }
                ExprKind::Name(name) => {
                    if aggregate {
                        if !self.flow.spend(name.len() + 1) {
                            return Err(Diagnostic::unsupported(
                                "pure list suffix lookup budget exhausted",
                                value.span,
                            ));
                        }
                        used.insert(name.as_str());
                    }
                    if scalar.checker.scopes[0].values.contains_key(name) {
                        continue;
                    }
                    let lookups = self.scopes.iter().fold(0usize, |work, scope| {
                        work.saturating_add(
                            scope.values.len().checked_ilog2().unwrap_or(0) as usize + 1,
                        )
                    });
                    if !self.flow.spend(lookups.saturating_mul(name.len() + 1)) {
                        return Err(Diagnostic::unsupported(
                            "scalar list lookup budget exhausted",
                            value.span,
                        ));
                    }
                    let Some(symbol) = Self::list_symbol(&self.scopes, name) else {
                        return Ok(None);
                    };
                    let constant = match symbol {
                        Value::Constant(value) => value,
                        Value::Local {
                            ty,
                            mutable: false,
                            owner,
                            constant: Some(value),
                            ..
                        } if *owner == self.owner
                            && matches!(
                                ty,
                                Type::Null
                                    | Type::Bool
                                    | Type::Int { .. }
                                    | Type::Float { .. }
                                    | Type::String
                            ) =>
                        {
                            value
                        }
                        _ => return Ok(None),
                    };
                    let bytes = name.len().saturating_add(match constant {
                        Constant::String(value) => value.len(),
                        _ => 0,
                    });
                    if !self.flow.spend(bytes) {
                        return Err(Diagnostic::unsupported(
                            "scalar list constant budget exhausted",
                            value.span,
                        ));
                    }
                    let symbol = match symbol {
                        Value::Local { ty, .. } => {
                            let id = scalar.checker.locals.len();
                            scalar.checker.locals.push(ty.clone());
                            Value::Local {
                                id,
                                ty: ty.clone(),
                                mutable: false,
                                owner: 0,
                                constant: Some(constant.clone()),
                            }
                        }
                        _ => Value::Constant(constant.clone()),
                    };
                    scalar.checker.scopes[0].values.insert(name.clone(), symbol);
                    bytes
                }
                _ => return Ok(None),
            };
            if pending.len() > MAX_SCALAR_NODES || !self.flow.spend(bytes) {
                return Err(Diagnostic::unsupported(
                    "scalar list probe budget exhausted",
                    value.span,
                ));
            }
            scalar.bytes = scalar.bytes.saturating_add(bytes);
        }
        if used.iter().any(|name| emitted.contains(name)) {
            return Ok(None);
        }
        Ok(Some(scalar))
    }

    pub(crate) fn list_scalar_probe(
        &mut self,
        value: &ast::Expr,
        expected: &Type,
        reach: Guard,
    ) -> Result<Option<Fit>> {
        let Some(scalar) = self.list_scalar(value)? else {
            return Ok(None);
        };
        self.list_pure_probe(scalar, value, expected, reach)
            .map(Some)
    }

    pub(crate) fn list_pure_probe(
        &mut self,
        mut scalar: Scalar,
        value: &ast::Expr,
        expected: &Type,
        reach: Guard,
    ) -> Result<Fit> {
        let weight = crate::borrow_contract::type_weight(expected, &mut self.flow, value.span)?;
        let names = scalar.checker.scopes[0].values.len();
        let work = scalar.nodes.saturating_mul(scalar.nodes).saturating_mul(
            names
                .saturating_add(weight)
                .saturating_add(scalar.bytes)
                .saturating_add(scalar.nodes)
                .saturating_add(1),
        );
        if !self.flow.spend(work) {
            return Err(Diagnostic::unsupported(
                "scalar list checking budget exhausted",
                value.span,
            ));
        }
        scalar.checker.reach = if reach == FALSE { FALSE } else { TRUE };
        let result = scalar.checker.expr(value, Some(expected));
        if scalar.checker.flow.exceeded() || !self.flow.spend(scalar.checker.flow.work) {
            return Err(Diagnostic::unsupported(
                "scalar list checking budget exhausted",
                value.span,
            ));
        }
        Ok(match result {
            Ok(_) => Fit::Yes,
            Err(error) if ["B001", "E203", "E205", "E206"].contains(&error.code) => {
                return Err(error);
            }
            Err(error)
                if error.code == "E207"
                    && (error.message.contains("multiple possible expected types")
                        || error.message.contains("fits multiple expected list types")) =>
            {
                Fit::Unknown
            }
            Err(_) => Fit::No,
        })
    }

    pub(crate) fn list_symbol<'a>(scopes: &'a [Scope], name: &str) -> Option<&'a Value> {
        scopes.iter().rev().find_map(|scope| scope.values.get(name))
    }

    pub(crate) fn list_source<'a>(
        scopes: &'a [Scope],
        value: &ast::Expr,
    ) -> (Option<&'a Type>, usize) {
        match &value.kind {
            ExprKind::Name(name) => (
                match Self::list_symbol(scopes, name) {
                    Some(Value::Local { ty, .. }) => Some(ty),
                    _ => None,
                },
                1,
            ),
            ExprKind::Call { callee, .. } => {
                let ExprKind::Name(name) = &callee.kind else {
                    return (None, 1);
                };
                (
                    match Self::list_symbol(scopes, name) {
                        Some(Value::Function { result, .. }) => result.as_ref(),
                        _ => None,
                    },
                    1,
                )
            }
            ExprKind::Group(value) => Self::list_source(scopes, value),
            ExprKind::Unary { op, value } if op == "*" => {
                let (ty, work) = Self::list_source(scopes, value);
                (
                    if let Some(Type::Reference(ty)) = ty {
                        Some(ty.as_ref())
                    } else {
                        None
                    },
                    work.saturating_add(1),
                )
            }
            ExprKind::Field { value, name } => {
                let (ty, work) = Self::list_source(scopes, value);
                let ty = if let Some(Type::Reference(ty)) = ty {
                    Some(ty.as_ref())
                } else {
                    ty
                };
                let Some(Type::Record { fields, .. }) = ty else {
                    return (None, work.saturating_add(1));
                };
                if let Some((index, (_, ty))) = fields
                    .iter()
                    .enumerate()
                    .find(|(_, (field, _))| field == name)
                {
                    (Some(ty), work.saturating_add(index + 1))
                } else {
                    (None, work.saturating_add(fields.len()))
                }
            }
            ExprKind::Index { value, .. } => {
                let (ty, work) = Self::list_source(scopes, value);
                let ty = if let Some(Type::Reference(ty)) = ty {
                    Some(ty.as_ref())
                } else {
                    ty
                };
                (
                    if let Some(Type::List { element, .. }) = ty {
                        Some(element.as_ref())
                    } else {
                        None
                    },
                    work.saturating_add(1),
                )
            }
            _ => (None, 1),
        }
    }

    pub(crate) fn list_probe(
        &mut self,
        value: &ast::Expr,
        expected: &Type,
        reach: Guard,
    ) -> Result<Fit> {
        crate::borrow_contract::type_weight(expected, &mut self.flow, value.span)?;
        if !self
            .flow
            .spend(value.span.end.saturating_sub(value.span.start) + 1)
        {
            return Err(Diagnostic::unsupported(
                "list candidate work budget exhausted",
                value.span,
            ));
        }
        let scalar = match &value.kind {
            ExprKind::Int(text) => Some(self.integer(text, false, Some(expected), value.span)),
            ExprKind::Float(text) => Some(Self::floating(text, Some(expected), value.span)),
            ExprKind::Unary { op, value: inner } if op == "-" => match &inner.kind {
                ExprKind::Int(text) => Some(self.integer(text, true, Some(expected), value.span)),
                ExprKind::Float(text) => Some(Self::floating(text, Some(expected), value.span)),
                _ => None,
            },
            ExprKind::String(parts)
                if parts
                    .iter()
                    .all(|part| matches!(part, ast::StringPart::Text(_))) =>
            {
                return Ok(if expected.accepts(&Type::String) {
                    Fit::Yes
                } else {
                    Fit::No
                });
            }
            _ => None,
        };
        if let Some(result) = scalar {
            return Ok(match result {
                Ok(value) if expected.accepts(&value.ty) => Fit::Yes,
                Err(error) if error.code == "E207" => Fit::Unknown,
                _ => Fit::No,
            });
        }
        if matches!(value.kind, ExprKind::Unary { .. } | ExprKind::Binary { .. })
            && let Some(fit) = self.list_scalar_probe(value, expected, reach)?
        {
            return Ok(fit);
        }
        match &value.kind {
            ExprKind::Group(value) => return self.list_probe(value, expected, reach),
            ExprKind::List(values) => {
                let mut matches = Vec::new();
                for ty in expected.members() {
                    let Type::List { element, capacity } = ty else {
                        continue;
                    };
                    if values.len() > *capacity {
                        continue;
                    }
                    let mut fit = Fit::Yes;
                    for value in values {
                        fit = fit.and(self.list_probe(value, element, reach)?);
                    }
                    if fit != Fit::No {
                        matches.push(fit);
                    }
                }
                return Ok(match matches.as_slice() {
                    [] => Fit::No,
                    [fit] => *fit,
                    _ => Fit::Unknown,
                });
            }
            ExprKind::Block(block) => return self.list_block_probe(block, expected, reach),
            _ => {}
        }
        if let ExprKind::Name(name) = &value.kind
            && matches!(
                Self::list_symbol(&self.scopes, name),
                Some(Value::Constant(_))
            )
            && let Some(ty) = self.literal_default(value)
        {
            return Ok(if expected.accepts(&ty) {
                Fit::Yes
            } else {
                Fit::No
            });
        }
        let (ty, work) = Self::list_source(&self.scopes, value);
        if !self.flow.spend(work) {
            return Err(Diagnostic::unsupported(
                "list source lookup budget exhausted",
                value.span,
            ));
        }
        if let Some(ty) = ty {
            crate::borrow_contract::type_weight(ty, &mut self.flow, value.span)?;
            return Ok(if Self::list_assigns(expected, ty) {
                Fit::Yes
            } else if ty
                .members()
                .iter()
                .any(|ty| Self::list_assigns(expected, ty))
            {
                Fit::Unknown
            } else {
                Fit::No
            });
        }
        Ok(Fit::Unknown)
    }

    pub(crate) fn list_block_probe(
        &mut self,
        block: &ast::Block,
        expected: &Type,
        reach: Guard,
    ) -> Result<Fit> {
        if block.label.is_some() {
            return Ok(Fit::Unknown);
        }
        let mut fields = Vec::new();
        for stmt in &block.stmts {
            let StmtKind::Emit {
                label: None,
                name,
                ty: None,
                mutable: false,
                value,
            } = &stmt.kind
            else {
                return Ok(Fit::Unknown);
            };
            if !self.flow.spend(fields.len() + 1) {
                return Err(Diagnostic::unsupported(
                    "list record-shape budget exhausted",
                    stmt.span,
                ));
            }
            if fields.iter().any(|(field, _)| field == &name.as_deref()) {
                return Ok(Fit::Unknown);
            }
            if name.is_none()
                && matches!(
                    value.kind,
                    ExprKind::Block(_) | ExprKind::Name(_) | ExprKind::Call { .. }
                )
            {
                return Ok(Fit::Unknown);
            }
            fields.push((name.as_deref(), value));
        }
        let mut choices = Vec::new();
        for ty in expected.members() {
            let mut fit = Fit::Yes;
            if let Type::Record {
                primary,
                fields: members,
            } = ty
            {
                if !self
                    .flow
                    .spend(fields.len().saturating_mul(members.len()).saturating_mul(2))
                {
                    return Err(Diagnostic::unsupported(
                        "list record-shape budget exhausted",
                        block.span,
                    ));
                }
                for (name, value) in &fields {
                    let slot = if let Some(name) = name {
                        members
                            .iter()
                            .find_map(|(field, ty)| (field == name).then_some(ty))
                    } else {
                        Some(primary.as_ref())
                    };
                    fit = fit.and(if let Some(slot) = slot {
                        self.list_probe(value, slot, reach)?
                    } else {
                        Fit::No
                    });
                }
                if !fields.iter().any(|(name, _)| name.is_none()) && !primary.accepts(&Type::Null) {
                    fit = Fit::No;
                }
                if members.iter().any(|(name, ty)| {
                    !ty.accepts(&Type::Null)
                        && !fields
                            .iter()
                            .any(|(field, _)| *field == Some(name.as_str()))
                }) {
                    fit = Fit::No;
                }
            } else if fields.iter().any(|(name, _)| name.is_some()) {
                fit = Fit::No;
            } else if let Some((_, value)) = fields.first() {
                fit = self.list_probe(value, ty, reach)?;
            } else if !ty.accepts(&Type::Null) {
                fit = Fit::No;
            }
            if fit != Fit::No {
                choices.push(fit);
            }
        }
        Ok(match choices.as_slice() {
            [] => Fit::No,
            [fit] => *fit,
            _ => Fit::Unknown,
        })
    }
}

#[cfg(test)]
mod tests {
    pub(crate) fn rejects(source: &str, code: &str) {
        let errors = crate::compile(source).unwrap_err();
        assert_eq!(
            errors[0].code,
            code,
            "{}-byte source: {errors:?}",
            source.len()
        );
    }

    #[test]
    pub(crate) fn literals_use_the_same_sign_and_width_rules_as_single_contexts() {
        for source in [
            "values<int8[1]><uint8[1]>:[255]",
            "values<int8[1]><uint8[1]>:[-128]",
            "values<int8[1]><uint8[1]>:[-0]",
            "values<float32[1]><float64[1]>:[1e39]",
            "values<int32[1]><float32[1]>:[1]",
        ] {
            assert!(crate::compile(source).is_ok(), "{source}");
        }
        rejects("values<int8[1]><uint8[1]>:[-(128)]", "E207");
        rejects("values<uint8[1]><string[1]>:[-0]", "E207");
        rejects("values<float32[1]><float64[1]>:[1.0]", "E207");
        rejects("values<uint8[2]><string[1]>:[256,1]", "E216");
    }

    #[test]
    pub(crate) fn pure_compounds_reuse_contextual_operator_rules() {
        for source in [
            "values<int8[1]><int16[1]>:[-(128)]",
            "values<int8[1]><int16[1]>:[-(-128)]",
            "values<int8[1]><int16[1]>:[(127+1)-1]",
            "values<int8[1]><uint8[1]>:[~128]",
            "values<float32[1]><float64[1]>:[1e39-1e39]",
            "values<boolean[1]><int32[1]>:[!(false||true)&&false]",
            "values<boolean[1]><int32[1]>:[false&&(1/0==1)]",
            "values<boolean[1]><int32[1]>:[true||(1/0==1)]",
            "values<int8[1]><string[1]>:[(-128)%-1]",
            "values<int8[1][1]><int16[1][1]>:[[(127+1)-1]]",
            "<A>:<{value<int8>}>;<B>:<{value<int16>}>;values<A[1]><B[1]>:[{->value:127+1}]",
        ] {
            assert!(crate::compile(source).is_ok(), "{source}");
        }
        for source in [
            "values<int8[1]><uint8[1]>:[~1]",
            "values<float32[1]><float64[1]>:[3e38+3e38]",
            "values<int8[1]><uint8[1]>:[256-256]",
            "values<int8[1]><int16[1]>:[1/0]",
        ] {
            rejects(source, "E207");
        }
        rejects("values<int8[2]><string[1]>:[127+1,0]", "E107");
        rejects("values<boolean[1]><string[1]>:[true&&(1/0==1)]", "E107");
    }

    #[test]
    pub(crate) fn pure_constant_leaves_retain_types_and_lexical_identity() {
        for source in [
            "byte<uint8>:254;values<uint8[1]><uint16[1]>:[byte+1]",
            "flag:false;values<boolean[1]><int32[1]>:[!flag]",
            "f<null>:(){true:false;values<boolean[1]><int32[1]>:[true&&(1/0==0)]}",
        ] {
            assert!(crate::compile(source).is_ok(), "{source}");
        }
        rejects(
            "byte<uint8>:255;values<uint8[1]><uint16[1]>:[byte+1]",
            "E107",
        );
        rejects(
            "byte<uint8>:1;values<uint16[1]><string[1]>:[byte+1]",
            "E207",
        );
        rejects("byte<uint8>:1;values:[byte,1+1]", "E207");
        rejects(
            "byte<uint8>:1;f<null>:(){values<uint8[1]><uint16[1]>:[byte+1]}",
            "B001",
        );
    }

    #[test]
    pub(crate) fn compound_candidates_use_their_original_evaluation_reach() {
        for source in [
            "d:@\"debug\";values<int8[2]><uint8[2]>:[127+1,{d.print(1);->1}]",
            "d:@\"debug\";stop<never>:(){d.panic(\"stop\")};values<int8[2]><uint8[2]>:[127+1,stop()]",
            "d:@\"debug\";stop<never>:(){d.panic(\"stop\")};values<int8[2]><string[2]>:[stop(),127+1]",
        ] {
            assert!(crate::compile(source).is_ok(), "{source}");
        }
        rejects(
            "d:@\"debug\";stop<never>:(){d.panic(\"stop\")};values<int8[2]><uint8[2]>:[stop(),127+1]",
            "E207",
        );
        rejects(
            "d:@\"debug\";stop<never>:(){d.panic(\"stop\")};values<int8[2]><uint8[2]>:[stop(),256-256]",
            "E207",
        );
        rejects(
            "d:@\"debug\";stop<never>:(){d.panic(\"stop\")};values<int8[2]><string[2]>:[127+1,stop()]",
            "E107",
        );
    }

    #[test]
    pub(crate) fn scalar_probe_work_includes_referenced_constant_bytes() {
        let source = format!(
            "text:\"{}\";values<boolean[1]><string[1]>:[text==text]",
            "a".repeat(500_000)
        );
        rejects(&source, "B001");
        let expression = (0..64).map(|_| "1").collect::<Vec<_>>().join("+");
        let types = (1..=64)
            .map(|capacity| format!("<int32[{capacity}]>"))
            .collect::<String>();
        rejects(&format!("values{types}:[{expression}]"), "B001");
        let values = vec!["1"; super::MAX_SCALAR_NODES].join(",");
        rejects(
            &format!(
                "d:@\"debug\";values<int8[4096][1]><int16[4096][1]>:[{{d.print(1);->[{values}]}}]"
            ),
            "B001",
        );
    }

    #[test]
    pub(crate) fn unresolved_effectful_contexts_are_explicit_and_can_be_annotated() {
        rejects(
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1}]",
            "E207",
        );
        assert!(
            crate::compile(
                "d:@\"debug\";byte<uint8>:1;values<uint8[2]><uint16[2]>:[{d.print(1);->1},byte]"
            )
            .is_ok()
        );
        rejects(
            "<Inner>:<int32[1]><int32[2]>;values<Inner[1]><string[1]>:[[1]]",
            "E207",
        );
        rejects(
            "<Inner>:<int32[1]><int32[2]>;values<Inner[1]><int32[2][1]>:[[1]]",
            "B001",
        );
    }

    #[test]
    pub(crate) fn effect_prefixes_choose_context_after_once_only_checking() {
        for source in [
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->300}]",
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->255+1}]",
            "d:@\"debug\";values<uint8[1]><int32[1]>:[{x:1;d.print(1);->x}]",
            "d:@\"debug\";x<uint8>:1;values<uint8[1]><string[1]>:[{x:\"x\";d.print(1);->x}]",
            "d:@\"debug\";values<uint8[1][1]><uint16[1][1]>:[{d.print(1);->[300]}]",
            "d:@\"debug\";<A>:<{n<uint8>}>;<B>:<{n<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->n:300}]",
            "d:@\"debug\";<A>:<{n<uint8>}>;<B>:<{n<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->{->n:300}}]",
            "d:@\"debug\";<R>:<{n<int32>}>;<S>:<{-><R><int32>;n<int32>}>;<U>:<R><S>;values<R[1]><U[1]>:[{d.print(1);->{->n:1};->n:2}]",
            "d:@\"debug\";byte<uint8>:1;values<uint8[2]><uint16[2]>:[{d.print(1);->1},byte]",
        ] {
            let result = crate::compile(source);
            assert!(result.is_ok(), "{source}: {result:?}");
        }
        rejects(
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1}]",
            "E207",
        );
        rejects(
            "d:@\"debug\";values<uint8[2]><uint16[2]>:[{d.print(1);->1},{byte<uint8>:2;->byte}]",
            "B001",
        );
        rejects(
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1;d.print(2)}]",
            "B001",
        );
        rejects(
            "d:@\"debug\";values<int32[1]><string[1]>:[{x:=1;d.print(1);->x}]",
            "B001",
        );
        rejects(
            "d:@\"debug\";x<uint8>:1;<A>:<{x<uint16>;y<uint8>}>;<B>:<{x<uint16>;y<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->x:300;->y:x}]",
            "B001",
        );
    }

    #[test]
    pub(crate) fn effect_prefix_reach_and_source_errors_precede_suffix_selection() {
        for (source, code) in [
            (
                "d:@\"debug\";values<int8[1]><uint8[1]>:[{d.panic(\"stop\");->127+1}]",
                "E207",
            ),
            (
                "d:@\"debug\";values<uint8[1]><uint16[1]>:[{unknown();->300}]",
                "E201",
            ),
            (
                "d:@\"debug\";<A>:<{x<uint8>}>;<B>:<{x<uint16>}>;values<A[1]><B[1]>:[{x:1;d.print(1);->x:2}]",
                "E203",
            ),
            (
                "d:@\"debug\";values<int8[1]><int16[1]>:[{d.print(1);->1;->2}]",
                "E205",
            ),
            (
                "d:@\"debug\";values<int8[1]><uint8[1]>:[{d.panic(\"stop\");a<int32[1/0]>:[];->1}]",
                "E107",
            ),
            (
                "d:@\"debug\";a:=1;r:&a;values<uint8[1]><uint16[1]>:[{a=2;->300}];x:*r",
                "E302",
            ),
        ] {
            rejects(source, code);
        }
        for source in [
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.panic(\"stop\");->300}]",
            "d:@\"debug\";<A>:<{n<uint8>;missing<string>}>;<B>:<{n<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->n:1}]",
            "d:@\"debug\";values<int8[1]><uint8[1]>:[{d.panic(\"stop\");->-128;->-128}]",
        ] {
            let result = crate::compile(source);
            assert!(result.is_ok(), "{source}: {result:?}");
        }
    }

    #[test]
    pub(crate) fn candidate_count_and_probe_work_are_bounded() {
        let types = (0..=super::MAX_CONTEXTS)
            .map(|capacity| format!("<int32[{capacity}]>"))
            .collect::<String>();
        rejects(&format!("values{types}:[]"), "B001");
        let types = (2..=super::MAX_CONTEXTS + 1)
            .map(|capacity| format!("<int32[{capacity}]>"))
            .collect::<String>();
        let source = format!("values{types}:[{}]", "0".repeat(20_000));
        rejects(&source, "B001");
        let fields = (0..4095)
            .map(|index| format!(";field{index}<null>"))
            .collect::<String>();
        let source = format!(
            "<Row>:<{{-><int32>{fields}}}>;use<null>:(row<Row>){{values{types}:[row,row]}}"
        );
        rejects(&source, "B001");
    }
}
