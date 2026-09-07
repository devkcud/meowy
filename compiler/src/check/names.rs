use super::{Checker, Constant, Result, Spec, Value};
use crate::ast::{self, ExprKind, Span, TypeKind};
use crate::diagnostic::Diagnostic;
use crate::hir::{self, Type};
use std::collections::BTreeMap;

impl Checker {
    pub(crate) fn primitive(name: &str) -> Option<Type> {
        Some(match name {
            "null" => Type::Null,
            "never" => Type::Never,
            "boolean" => Type::Bool,
            "string" => Type::String,
            "isize" => Type::Int {
                bits: 64,
                signed: true,
            },
            "usize" => Type::Int {
                bits: 64,
                signed: false,
            },
            "float32" => Type::Float { bits: 32 },
            "float64" => Type::Float { bits: 64 },
            value => {
                let (digits, signed) = if let Some(digits) = value.strip_prefix("uint") {
                    (digits, false)
                } else {
                    let digits = value.strip_prefix("int")?;
                    (digits, true)
                };
                let bits = digits.parse().ok()?;
                if ![8, 16, 32, 64].contains(&bits) {
                    return None;
                }
                Type::Int { bits, signed }
            }
        })
    }

    pub(crate) fn value(&self, name: &str, span: Span) -> Result<Value> {
        let value = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.values.get(name))
            .cloned()
            .ok_or_else(|| Self::error("E201", format!("unknown value `{name}`"), span))?;
        match &value {
            Value::Local { .. } if self.required => Ok(value),
            Value::Local { owner, .. } | Value::Control { owner, .. } if *owner != self.owner => {
                Err(Diagnostic::unsupported(
                    "capturing a value from an enclosing function or module",
                    span,
                ))
            }
            _ => Ok(value),
        }
    }

    pub(crate) fn declare(&mut self, name: &str, value: Value, span: Span) -> Result<()> {
        let scope = self.scopes.last_mut().expect("scope");
        if scope.values.contains_key(name) {
            return Err(Self::error(
                "E203",
                format!("value `{name}` is already declared in this scope"),
                span,
            ));
        }
        scope.values.insert(name.into(), value);
        Ok(())
    }

    pub(crate) fn local(&mut self, ty: Type) -> usize {
        let id = self.locals.len();
        self.locals.push(ty);
        self.proofs.bindings.insert(id, self.reach);
        id
    }

    pub(crate) fn spec(&mut self, expr: &ast::TypeExpr) -> Result<Spec> {
        match &expr.kind {
            TypeKind::Name(name) => {
                if let Some((module, member)) = name.split_once('.') {
                    if let Value::Module(module) = self.value(module, expr.span)?
                        && module == "core"
                    {
                        return Self::primitive(member).map(Spec::Data).ok_or_else(|| {
                            if ["int128", "uint128", "Type", "error", "any"].contains(&member) {
                                Diagnostic::unsupported(format!("type `core.{member}`"), expr.span)
                            } else {
                                Self::error(
                                    "E202",
                                    format!("unknown core type `{member}`"),
                                    expr.span,
                                )
                            }
                        });
                    }
                    return Err(Self::error(
                        "E202",
                        format!("unknown type `{name}`"),
                        expr.span,
                    ));
                }
                self.scopes
                    .iter()
                    .rev()
                    .find_map(|scope| scope.types.get(name))
                    .cloned()
                    .ok_or_else(|| {
                        if ["int128", "uint128", "Type", "error", "any"].contains(&name.as_str()) {
                            Diagnostic::unsupported(format!("type `{name}`"), expr.span)
                        } else {
                            Self::error("E202", format!("unknown type `{name}`"), expr.span)
                        }
                    })
            }
            TypeKind::Function { params, result } => {
                let params = params
                    .iter()
                    .map(|ty| self.ty(ty))
                    .collect::<Result<Vec<_>>>()?;
                let result = self.ty(result)?;
                Ok(Spec::Function { params, result })
            }
            TypeKind::Record { primary, fields } => {
                let primary = primary
                    .as_ref()
                    .map(|ty| self.ty(ty))
                    .transpose()?
                    .unwrap_or(Type::Null);
                let mut result = BTreeMap::new();
                for (name, ty, mutable) in fields {
                    let ty = self.ty(ty)?;
                    if *mutable && ty.has_reference() {
                        return Err(Diagnostic::unsupported(
                            "mutable reference-bearing record fields",
                            expr.span,
                        ));
                    }
                    if result
                        .insert(
                            name.clone(),
                            hir::Field {
                                name: name.clone(),
                                ty,
                                mutable: *mutable,
                            },
                        )
                        .is_some()
                    {
                        return Err(Self::error(
                            "E206",
                            format!("duplicate record field `{name}`"),
                            expr.span,
                        ));
                    }
                }
                if matches!(primary, Type::Record { .. }) {
                    return Err(Diagnostic::unsupported(
                        "composed primary record types",
                        expr.span,
                    ));
                }
                let ty = Type::Record {
                    primary: Box::new(primary),
                    fields: result.into_values().collect(),
                };
                Ok(Spec::Data(ty))
            }
            TypeKind::Computed(value) => Ok(Spec::Data(self.type_value(value)?)),
            TypeKind::Union(types) => {
                let types = types
                    .iter()
                    .map(|ty| self.ty(ty))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Spec::Data(Type::union(types)))
            }
            TypeKind::List { element, size } => {
                let Some(size) = size else {
                    return Err(Diagnostic::unsupported("borrowed slice types", expr.span));
                };
                let element = self.ty(element)?;
                let capacity = self.list_extent(size)?;
                Ok(Spec::Data(self.list_type(element, capacity, expr.span)?))
            }
            TypeKind::Reference { value, mutable } => {
                if *mutable {
                    let ty = self.ty(value)?;
                    return Ok(Spec::Data(self.exclusive_type(ty, expr.span)?));
                }
                let ty = self.ty(value)?;
                Ok(Spec::Data(self.reference_type(ty, expr.span)?))
            }
            TypeKind::Unsupported(feature) => Err(Diagnostic::unsupported(feature, expr.span)),
        }
    }

    pub(crate) fn ty(&mut self, expr: &ast::TypeExpr) -> Result<Type> {
        match self.spec(expr)? {
            Spec::Data(ty) => Ok(ty),
            Spec::Function { .. } => Err(Diagnostic::unsupported(
                "stored function pointers",
                expr.span,
            )),
        }
    }

    pub(crate) fn type_value(&mut self, expr: &ast::Expr) -> Result<Type> {
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

    pub(crate) fn label(&self, name: &str, span: Span) -> Result<usize> {
        let id = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.labels.get(name))
            .copied()
            .ok_or_else(|| {
                Self::error("E201", format!("unknown enclosing label `'{name}`"), span)
            })?;
        if !self
            .frames
            .iter()
            .any(|frame| frame.id == id && frame.owner == self.owner)
        {
            return Err(Self::error(
                "E201",
                format!("label `'{name}` is outside this function"),
                span,
            ));
        }
        Ok(id)
    }

    pub(crate) fn symbol(&mut self, expr: &ast::Expr) -> Result<Option<Value>> {
        match &expr.kind {
            ExprKind::Name(name) => Ok(Some(self.value(name, expr.span)?)),
            ExprKind::Group(value) => self.symbol(value),
            ExprKind::Import(name) => {
                if !["core", "debug"].contains(&name.as_str()) {
                    return Err(Diagnostic::unsupported(
                        format!("module import `@\"{name}\"`"),
                        expr.span,
                    ));
                }
                Ok(Some(Value::Module(name.clone())))
            }
            ExprKind::TypeValue(ty) => Ok(Some(Value::Type(self.ty(ty)?))),
            ExprKind::TypeQuery(_) => Ok(Some(Value::Type(self.type_value(expr)?))),
            ExprKind::Field { value, name } => {
                if let ExprKind::Label(label) = &value.kind {
                    let target = self.label(label, expr.span)?;
                    return match name.as_str() {
                        "leave" | "restart" => Ok(Some(Value::Control {
                            target,
                            restart: name == "restart",
                            owner: self.owner,
                        })),
                        _ => Err(Self::error(
                            "E201",
                            format!("unknown label operation `{name}`"),
                            expr.span,
                        )),
                    };
                }
                if let Some(Value::Module(module)) = self.symbol(value)? {
                    let result = match (module.as_str(), name.as_str()) {
                        ("core", "true") => Value::Constant(Constant::Bool(true)),
                        ("core", "false") => Value::Constant(Constant::Bool(false)),
                        ("core", "null") => Value::Constant(Constant::Null),
                        ("debug", "print") => Value::Print,
                        ("debug", "panic") => Value::Panic,
                        ("core", name) if Self::primitive(name).is_some() => {
                            Value::Type(Self::primitive(name).expect("primitive"))
                        }
                        _ => {
                            return Err(Self::error(
                                "E201",
                                format!("module `{module}` has no supported member `{name}`"),
                                expr.span,
                            ));
                        }
                    };
                    return Ok(Some(result));
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
