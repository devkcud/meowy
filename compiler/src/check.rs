use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{self, ExprKind, Span, StmtKind, TypeKind};
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow, Guard, TRUE};
use crate::hir::{self, Type};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Slots = BTreeMap<Option<String>, Vec<Slot>>;
pub(crate) type Place = (usize, Vec<String>);

#[derive(Clone)]
pub(crate) enum Constant {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    String(String),
}

#[derive(Clone)]
pub(crate) enum Value {
    Local {
        id: usize,
        ty: Type,
        mutable: bool,
        owner: usize,
        constant: Option<Constant>,
    },
    Constant(Constant),
    Module(String),
    Function {
        id: usize,
        params: Vec<Type>,
        result: Option<Type>,
    },
    Print,
    Panic,
    Control {
        target: usize,
        restart: bool,
        owner: usize,
    },
    Type(Type),
}

#[derive(Clone)]
pub(crate) enum Spec {
    Data(Type),
    Function { params: Vec<Type>, result: Type },
}

#[derive(Default)]
pub(crate) struct Scope {
    pub(crate) values: BTreeMap<String, Value>,
    pub(crate) types: BTreeMap<String, Spec>,
    pub(crate) labels: BTreeMap<String, usize>,
}

#[derive(Clone)]
pub(crate) struct Slot {
    pub(crate) ty: Type,
    pub(crate) mutable: bool,
    pub(crate) guard: Guard,
    pub(crate) order: usize,
}

pub(crate) struct Frame {
    pub(crate) id: usize,
    pub(crate) expected: Option<Type>,
    pub(crate) leaves: Guard,
    pub(crate) slots: Slots,
    pub(crate) start: usize,
    pub(crate) partial: bool,
    pub(crate) owner: usize,
}

pub(crate) struct Checker {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) flow: Flow,
    pub(crate) reach: Guard,
    pub(crate) tags: BTreeMap<(Place, Type), Vec<(Type, Guard)>>,
    pub(crate) bools: BTreeMap<Place, Guard>,
    pub(crate) guards: BTreeMap<(usize, usize), Guard>,
    pub(crate) writes: usize,
    pub(crate) functions: Vec<Option<hir::Function>>,
    pub(crate) locals: Vec<Type>,
    pub(crate) places: BTreeSet<usize>,
    pub(crate) proofs: crate::borrow::Proofs,
    pub(crate) constants: BTreeMap<usize, Constant>,
    pub(crate) block: usize,
    pub(crate) owner: usize,
    pub(crate) calls: usize,
}

pub fn check(block: &ast::Block) -> std::result::Result<hir::Program, Vec<Diagnostic>> {
    let mut checker = Checker::new();
    match checker.block(block, None, None) {
        Ok(body) => {
            let program = hir::Program {
                body,
                functions: checker.functions.into_iter().flatten().collect(),
                locals: checker.locals,
            };
            checker.proofs.conditions = checker.guards;
            checker.proofs.tags = checker.tags;
            let facts = crate::borrow::check(&program, &mut checker.flow, &checker.proofs)?;
            crate::loans::check(&program, &facts, &checker.proofs, &mut checker.flow)?;
            Ok(program)
        }
        Err(error) => Err(vec![if checker.flow.exceeded() {
            Diagnostic::unsupported("control-flow proof budget exhausted", block.span)
        } else {
            error
        }]),
    }
}

impl Checker {
    pub(crate) fn new() -> Self {
        let mut prelude = Scope::default();
        prelude
            .values
            .insert("true".into(), Value::Constant(Constant::Bool(true)));
        prelude
            .values
            .insert("false".into(), Value::Constant(Constant::Bool(false)));
        prelude
            .values
            .insert("null".into(), Value::Constant(Constant::Null));
        for name in [
            "null", "never", "boolean", "int8", "int16", "int32", "int64", "uint8", "uint16",
            "uint32", "uint64", "isize", "usize", "float32", "float64", "string",
        ] {
            if let Some(ty) = Self::primitive(name) {
                prelude.types.insert(name.into(), Spec::Data(ty));
            }
        }
        Self {
            scopes: vec![prelude],
            frames: Vec::new(),
            flow: Flow::new(),
            reach: TRUE,
            tags: BTreeMap::new(),
            bools: BTreeMap::new(),
            guards: BTreeMap::new(),
            writes: 0,
            functions: Vec::new(),
            locals: Vec::new(),
            places: BTreeSet::new(),
            proofs: crate::borrow::Proofs::default(),
            constants: BTreeMap::new(),
            block: 0,
            owner: 0,
            calls: 0,
        }
    }

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

    pub(crate) fn error(code: &'static str, message: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic::new(code, message, span)
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
                    if *mutable {
                        return Err(Diagnostic::unsupported("mutable record fields", expr.span));
                    }
                    if result.insert(name.clone(), self.ty(ty)?).is_some() {
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
                    fields: result.into_iter().collect(),
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
            TypeKind::List { .. } => {
                Err(Diagnostic::unsupported("list and slice types", expr.span))
            }
            TypeKind::Reference { value, mutable } => {
                if *mutable {
                    return Err(Diagnostic::unsupported(
                        "exclusive reference types",
                        expr.span,
                    ));
                }
                let ty = self.ty(value)?;
                if ty.has_reference() {
                    return Err(Diagnostic::unsupported("nested reference types", expr.span));
                }
                Ok(Spec::Data(Type::Reference(Box::new(ty))))
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

    pub(crate) fn forget(&mut self, id: usize) {
        self.tags.retain(|((root, _), _), _| *root != id);
        self.bools.retain(|(root, _), _| *root != id);
    }

    pub(crate) fn forget_mutable(&mut self) {
        let ids: Vec<_> = self
            .scopes
            .iter()
            .flat_map(|scope| scope.values.values())
            .filter_map(|value| match value {
                Value::Local {
                    id, mutable: true, ..
                } => Some(*id),
                _ => None,
            })
            .collect();
        for id in ids {
            self.forget(id);
        }
    }

    pub(crate) fn place(value: &hir::Expr) -> Option<Place> {
        match &value.kind {
            hir::ExprKind::Local(id) => Some((*id, Vec::new())),
            hir::ExprKind::Coerce { value } => Self::place(value),
            hir::ExprKind::Field { value, index } => {
                let Type::Record { fields, .. } = &value.ty else {
                    return None;
                };
                let mut place = Self::place(value)?;
                place.1.push(fields[*index].0.clone());
                Some(place)
            }
            _ => None,
        }
    }

    pub(crate) fn variants(&mut self, place: Place, ty: &Type) -> Vec<(Type, Guard)> {
        let key = (place, ty.clone());
        if let Some(tags) = self.tags.get(&key) {
            return tags.clone();
        }
        let mut rest = TRUE;
        let mut tags = Vec::new();
        let count = ty.members().len();
        for (index, ty) in ty.members().iter().enumerate() {
            let guard = if index + 1 == count {
                rest
            } else {
                let tag = self.flow.fresh();
                let guard = self.flow.and(rest, tag);
                let absent = self.flow.not(tag);
                rest = self.flow.and(rest, absent);
                guard
            };
            tags.push((ty.clone(), guard));
        }
        self.tags.insert(key, tags.clone());
        tags
    }

    pub(crate) fn refined(&mut self, place: Place, ty: &Type) -> Type {
        if self.reach == FALSE || !matches!(ty, Type::Union(_)) {
            return ty.clone();
        }
        let types = self
            .variants(place, ty)
            .into_iter()
            .filter_map(|(ty, guard)| self.flow.overlap(self.reach, guard).then_some(ty))
            .collect::<Vec<_>>();
        Type::union(types)
    }

    pub(crate) fn coerce(value: hir::Expr, ty: Type) -> hir::Expr {
        if value.ty == ty {
            value
        } else {
            hir::Expr {
                span: value.span,
                ty,
                kind: hir::ExprKind::Coerce {
                    value: Box::new(value),
                },
            }
        }
    }

    pub(crate) fn narrow(&mut self, value: hir::Expr) -> hir::Expr {
        if let Some(place) = Self::place(&value) {
            let ty = self.refined(place, &value.ty);
            Self::coerce(value, ty)
        } else {
            value
        }
    }

    pub(crate) fn storage_type(value: &hir::Expr) -> &Type {
        match &value.kind {
            hir::ExprKind::Coerce { value } => Self::storage_type(value),
            _ => &value.ty,
        }
    }

    pub(crate) fn guard(&mut self, expr: &hir::Expr) -> Guard {
        let key = (expr.span.start, expr.span.end);
        if let Some(guard) = self.guards.get(&key) {
            return *guard;
        }
        let guard = if let Some(Constant::Bool(value)) = self.constant(expr) {
            if value { TRUE } else { FALSE }
        } else {
            match &expr.kind {
                hir::ExprKind::Unary { op, value } if op == "!" => {
                    let guard = self.guard(value);
                    self.flow.not(guard)
                }
                hir::ExprKind::Binary { op, left, right } if op == "&&" || op == "||" => {
                    let left = self.guard(left);
                    let right = self.guard(right);
                    if op == "&&" {
                        self.flow.and(left, right)
                    } else {
                        self.flow.or(left, right)
                    }
                }
                hir::ExprKind::TypeTest { value, ty } => {
                    if ty.accepts(&value.ty) {
                        TRUE
                    } else if ty.intersection(&value.ty) == Type::Never {
                        FALSE
                    } else if let Some(place) = Self::place(value) {
                        let tags = self.variants(place, Self::storage_type(value));
                        let mut guard = FALSE;
                        for (variant, tag) in tags {
                            if ty.accepts(&variant) {
                                guard = self.flow.or(guard, tag);
                            }
                        }
                        guard
                    } else {
                        self.flow.fresh()
                    }
                }
                _ => {
                    if let Some(place) = Self::place(expr) {
                        if let Some(guard) = self.bools.get(&place) {
                            *guard
                        } else {
                            let guard = self.flow.fresh();
                            self.bools.insert(place, guard);
                            guard
                        }
                    } else {
                        self.flow.fresh()
                    }
                }
            }
        };
        self.guards.insert(key, guard);
        guard
    }

    pub(crate) fn block(
        &mut self,
        block: &ast::Block,
        expected: Option<Type>,
        receiver: Option<hir::Expr>,
    ) -> Result<hir::Block> {
        self.block_inner(block, expected, receiver, false)
    }

    pub(crate) fn block_inner(
        &mut self,
        block: &ast::Block,
        expected: Option<Type>,
        receiver: Option<hir::Expr>,
        partial: bool,
    ) -> Result<hir::Block> {
        let id = self.block;
        self.block += 1;
        self.scopes.push(Scope::default());
        if let Some(label) = &block.label {
            self.forget_mutable();
            self.scopes
                .last_mut()
                .expect("scope")
                .labels
                .insert(label.clone(), id);
        }
        self.frames.push(Frame {
            id,
            expected: expected.clone(),
            leaves: FALSE,
            slots: Slots::new(),
            start: self.writes,
            partial,
            owner: self.owner,
        });
        let mut stmts = Vec::new();
        if let Some(value) = receiver {
            let ty = value.ty.clone();
            let local = self.local(ty.clone());
            self.declare(
                "self",
                Value::Local {
                    id: local,
                    ty,
                    mutable: false,
                    owner: self.owner,
                    constant: self.constant(&value),
                },
                block.span,
            )?;
            stmts.push(hir::Stmt::Bind { id: local, value });
        }
        let mut index = 0;
        while index < block.stmts.len() {
            if matches!(block.stmts[index].kind, StmtKind::Forward { .. }) {
                index = self.forward(&block.stmts, index)?;
            } else {
                stmts.extend(self.stmt(&block.stmts[index])?);
                index += 1;
            }
        }
        let frame = self.frames.pop().expect("frame");
        self.reach = self.flow.or(self.reach, frame.leaves);
        let ty = self.block_type(&frame, expected.as_ref(), block.span)?;
        self.proofs.completions.insert(id, self.reach);
        if self.flow.exceeded() {
            return Err(Diagnostic::unsupported(
                "control-flow proof budget exhausted",
                block.span,
            ));
        }
        self.scopes.pop();
        Ok(hir::Block { id, ty, stmts })
    }

    pub(crate) fn block_type(
        &mut self,
        frame: &Frame,
        expected: Option<&Type>,
        span: Span,
    ) -> Result<Type> {
        if self.reach == FALSE {
            return Ok(Type::Never);
        }
        let mut slots = BTreeMap::new();
        for (name, writes) in &frame.slots {
            let mut types = Vec::new();
            let mut initialized = FALSE;
            let mut mutable = None;
            for slot in writes {
                if !self.flow.overlap(slot.guard, self.reach) {
                    continue;
                }
                if mutable.is_some_and(|value| value != slot.mutable) {
                    return Err(Self::error(
                        "E206",
                        "result field mutability differs between paths",
                        span,
                    ));
                }
                mutable = Some(slot.mutable);
                types.push(slot.ty.clone());
                initialized = self.flow.or(initialized, slot.guard);
            }
            if types.is_empty() {
                continue;
            }
            if !self.flow.implies(self.reach, initialized) {
                types.push(Type::Null);
            }
            slots.insert(name.clone(), (Type::union(types), initialized));
        }
        let constructor =
            expected.is_some_and(Self::record_union) && slots.keys().any(Option::is_some);
        if let Some(expected) = expected.filter(|_| !frame.partial && !constructor) {
            let required: Vec<(Option<String>, Type)> = match expected {
                Type::Record { primary, fields } => std::iter::once((None, *primary.clone()))
                    .chain(
                        fields
                            .iter()
                            .map(|(name, ty)| (Some(name.clone()), ty.clone())),
                    )
                    .collect(),
                ty => vec![(None, ty.clone())],
            };
            for (name, ty) in required {
                let initialized = slots.get(&name).map(|(_, guard)| *guard).unwrap_or(FALSE);
                if !ty.accepts(&Type::Null) && !self.flow.implies(self.reach, initialized) {
                    return Err(Self::error(
                        "E204",
                        format!(
                            "required result {} is uninitialized on a completing path",
                            name.as_ref()
                                .map(|name| format!("field `{name}`"))
                                .unwrap_or_else(|| "primary".into())
                        ),
                        span,
                    ));
                }
                let actual = slots.get(&name).map(|(ty, _)| ty).unwrap_or(&Type::Null);
                if !ty.accepts(actual) {
                    return Err(Self::error(
                        "E207",
                        format!("result slot has type {actual:?}, expected {ty:?}"),
                        span,
                    ));
                }
            }
            return Ok(expected.clone());
        }
        let primary = slots.remove(&None).map(|(ty, _)| ty).unwrap_or(Type::Null);
        let fields: Vec<_> = slots
            .into_iter()
            .filter_map(|(name, (ty, _))| name.map(|name| (name, ty)))
            .collect();
        let actual = if fields.is_empty() {
            primary
        } else {
            Type::Record {
                primary: Box::new(primary),
                fields,
            }
        };
        if constructor && !frame.partial {
            let expected = expected.expect("record union");
            let choices: Vec<_> = expected
                .members()
                .iter()
                .filter(|member| Self::record_fits(member, &actual))
                .collect();
            if choices.len() != 1 {
                return Err(Self::error(
                    "E207",
                    "record constructor does not select one compatible union member",
                    span,
                ));
            }
            return Ok(choices[0].clone());
        }
        Ok(actual)
    }

    pub(crate) fn record_union(ty: &Type) -> bool {
        matches!(ty, Type::Union(members) if members.iter().any(|ty| matches!(ty, Type::Record { .. })))
    }

    pub(crate) fn record_fits(expected: &Type, actual: &Type) -> bool {
        let (
            Type::Record { primary, fields },
            Type::Record {
                primary: value,
                fields: values,
            },
        ) = (expected, actual)
        else {
            return false;
        };
        primary.accepts(value)
            && values.iter().all(|(name, ty)| {
                fields
                    .iter()
                    .any(|(field, expected)| field == name && expected.accepts(ty))
            })
            && fields.iter().all(|(name, ty)| {
                values.iter().any(|(field, _)| field == name) || ty.accepts(&Type::Null)
            })
    }

    pub(crate) fn union_slot(ty: &Type, name: Option<&str>) -> Option<Type> {
        let mut types = Vec::new();
        if name.is_none() {
            types.extend(ty.members().iter().cloned());
        }
        for member in ty.members() {
            if let Type::Record { primary, fields } = member {
                if let Some(name) = name {
                    types.extend(
                        fields
                            .iter()
                            .filter(|(field, _)| field == name)
                            .map(|(_, ty)| ty.clone()),
                    );
                } else {
                    types.push(*primary.clone());
                }
            }
        }
        (!types.is_empty()).then(|| Type::union(types))
    }

    pub(crate) fn forward(&mut self, stmts: &[ast::Stmt], start: usize) -> Result<usize> {
        let mut index = start;
        let mut names = BTreeMap::new();
        while let Some(ast::Stmt {
            kind: StmtKind::Forward { name, ty },
            span,
        }) = stmts.get(index)
        {
            let Spec::Function { params, result } = self.spec(ty)? else {
                return Err(Self::error(
                    "E221",
                    "forward declarations require a concrete function signature",
                    *span,
                ));
            };
            let id = self.functions.len();
            self.functions.push(None);
            self.declare(
                name,
                Value::Function {
                    id,
                    params: params.clone(),
                    result: Some(result.clone()),
                },
                *span,
            )?;
            names.insert(name.clone(), (id, params, result));
            index += 1;
        }
        let count = names.len();
        for _ in 0..count {
            let stmt = stmts.get(index).ok_or_else(|| {
                Self::error(
                    "E221",
                    "forward function group is missing a definition",
                    stmts[start].span,
                )
            })?;
            let (name, ty, params, body) = match &stmt.kind {
                StmtKind::Bind {
                    name,
                    ty,
                    mutable: false,
                    value:
                        ast::Expr {
                            kind: ExprKind::Function { params, body },
                            ..
                        },
                } => (name, ty, params, body),
                _ => {
                    return Err(Self::error(
                        "E221",
                        "only the reserved function definitions may follow forward signatures",
                        stmt.span,
                    ));
                }
            };
            let (id, expected_params, expected_result) = names.remove(name).ok_or_else(|| {
                Self::error(
                    "E221",
                    format!("`{name}` is not a pending forward definition"),
                    stmt.span,
                )
            })?;
            let actual_params = params
                .iter()
                .map(|param| self.ty(&param.ty))
                .collect::<Result<Vec<_>>>()?;
            let actual_result = ty
                .as_ref()
                .map(|ty| self.ty(ty))
                .transpose()?
                .unwrap_or(expected_result.clone());
            if expected_params != actual_params || actual_result != expected_result {
                return Err(Self::error(
                    "E221",
                    format!("definition of `{name}` does not match its reserved signature"),
                    stmt.span,
                ));
            }
            self.function(id, name, params, body, Some(expected_result))
                .map_err(|error| {
                    if error.code == "B001" && error.message.contains("captur") {
                        Self::error(
                            "E221",
                            "forward functions cannot capture enclosing locals",
                            error.span,
                        )
                    } else {
                        error
                    }
                })?;
            index += 1;
        }
        Ok(index)
    }

    pub(crate) fn function(
        &mut self,
        id: usize,
        name: &str,
        params: &[ast::Param],
        body: &ast::Block,
        result: Option<Type>,
    ) -> Result<Type> {
        let reach = std::mem::replace(&mut self.reach, TRUE);
        let owner = self.owner;
        self.owner = id + 1;
        self.scopes.push(Scope::default());
        let mut ids = Vec::new();
        for param in params {
            let ty = self.ty(&param.ty)?;
            let id = self.local(ty.clone());
            self.declare(
                &param.name,
                Value::Local {
                    id,
                    ty,
                    mutable: false,
                    owner: self.owner,
                    constant: None,
                },
                param.span,
            )?;
            ids.push(id);
        }
        let block = self.block(body, result.clone(), None)?;
        let result = result.unwrap_or_else(|| block.ty.clone());
        self.functions[id] = Some(hir::Function {
            id,
            name: name.into(),
            params: ids,
            result: result.clone(),
            body: block,
        });
        self.scopes.pop();
        self.owner = owner;
        self.reach = reach;
        Ok(result)
    }

    pub(crate) fn stmt(&mut self, stmt: &ast::Stmt) -> Result<Vec<hir::Stmt>> {
        match &stmt.kind {
            StmtKind::Bind {
                name,
                ty,
                mutable,
                value,
            } => {
                if let ExprKind::Function { params, body } = &value.kind {
                    if *mutable {
                        return Err(Diagnostic::unsupported(
                            "mutable function bindings",
                            stmt.span,
                        ));
                    }
                    let result = ty.as_ref().map(|ty| self.ty(ty)).transpose()?;
                    let args = params
                        .iter()
                        .map(|param| self.ty(&param.ty))
                        .collect::<Result<_>>()?;
                    let id = self.functions.len();
                    self.functions.push(None);
                    self.declare(
                        name,
                        Value::Function {
                            id,
                            params: args,
                            result: result.clone(),
                        },
                        stmt.span,
                    )?;
                    let result = self.function(id, name, params, body, result)?;
                    if let Some(Value::Function { result: target, .. }) =
                        self.scopes.last_mut().expect("scope").values.get_mut(name)
                    {
                        *target = Some(result);
                    }
                    return Ok(Vec::new());
                }
                if let Some(symbol) = self.symbol(value)?
                    && !matches!(symbol, Value::Local { .. } | Value::Constant(_))
                {
                    if *mutable || ty.is_some() {
                        return Err(Diagnostic::unsupported(
                            "annotated or mutable compile-time identity bindings",
                            stmt.span,
                        ));
                    }
                    self.declare(name, symbol, stmt.span)?;
                    return Ok(Vec::new());
                }
                let expected = ty.as_ref().map(|ty| self.ty(ty)).transpose()?;
                let value = self.expr(value, expected.as_ref())?;
                let ty = expected.unwrap_or_else(|| value.ty.clone());
                if *mutable && ty.has_reference() {
                    return Err(Diagnostic::unsupported(
                        "mutable reference bindings",
                        stmt.span,
                    ));
                }
                let id = self.local(ty.clone());
                if *mutable {
                    self.proofs.mutable.insert(id);
                }
                self.places.insert(id);
                let constant = if *mutable {
                    None
                } else {
                    self.constant(&value)
                };
                self.declare(
                    name,
                    Value::Local {
                        id,
                        ty,
                        mutable: *mutable,
                        owner: self.owner,
                        constant,
                    },
                    stmt.span,
                )?;
                Ok(vec![hir::Stmt::Bind { id, value }])
            }
            StmtKind::TypeAlias { name, ty } => {
                let spec = self.spec(ty)?;
                let scope = self.scopes.last_mut().expect("scope");
                if scope.types.contains_key(name) {
                    return Err(Self::error(
                        "E203",
                        format!("type `{name}` is already declared in this scope"),
                        stmt.span,
                    ));
                }
                scope.types.insert(name.clone(), spec);
                Ok(Vec::new())
            }
            StmtKind::Assign { target, value } => {
                let ExprKind::Name(name) = &target.kind else {
                    return Err(Diagnostic::unsupported(
                        "assignment through fields or references",
                        target.span,
                    ));
                };
                let Value::Local {
                    id, ty, mutable, ..
                } = self.value(name, target.span)?
                else {
                    return Err(Self::error(
                        "E305",
                        "assignment requires a mutable local binding",
                        target.span,
                    ));
                };
                if !mutable {
                    return Err(Self::error(
                        "E305",
                        format!("binding `{name}` is immutable"),
                        target.span,
                    ));
                }
                let value = self.expr(value, Some(&ty))?;
                self.forget(id);
                Ok(vec![hir::Stmt::Assign { id, value }])
            }
            StmtKind::Emit {
                label,
                name,
                ty,
                mutable,
                value,
            } => self.emit(
                label.as_deref(),
                name.as_deref(),
                ty.as_ref(),
                *mutable,
                value,
                stmt.span,
            ),
            StmtKind::Match { arms } => {
                let mut stmts = Vec::new();
                for (condition, body) in arms {
                    let Some(condition) = condition else {
                        return Err(Diagnostic::unsupported("matcher fallback arms", stmt.span));
                    };
                    let condition = self.expr(condition, None)?;
                    if condition.ty != Type::Bool {
                        return Err(Self::error(
                            "E215",
                            format!("matcher requires boolean, found {:?}", condition.ty),
                            condition.span,
                        ));
                    }
                    let guard = self.guard(&condition);
                    let absent = self.flow.not(guard);
                    let skipped = self.flow.and(self.reach, absent);
                    self.reach = self.flow.and(self.reach, guard);
                    self.scopes.push(Scope::default());
                    let then = self.stmt(body)?;
                    self.scopes.pop();
                    self.reach = self.flow.or(self.reach, skipped);
                    stmts.push(hir::Stmt::If {
                        condition,
                        then,
                        otherwise: Vec::new(),
                    });
                }
                Ok(stmts)
            }
            StmtKind::Expr(value) => {
                if let ExprKind::Call { callee, args } = &value.kind
                    && let Some(Value::Control {
                        target,
                        restart,
                        owner,
                    }) = self.symbol(callee)?
                {
                    if !args.is_empty() {
                        return Err(Self::error(
                            "E212",
                            "scope operations take no arguments",
                            value.span,
                        ));
                    }
                    if owner != self.owner {
                        return Err(Self::error(
                            "E201",
                            "scope operation cannot cross a function",
                            value.span,
                        ));
                    }
                    let index = self
                        .frames
                        .iter()
                        .position(|frame| frame.id == target)
                        .ok_or_else(|| {
                            Self::error(
                                "E201",
                                "scope operation escaped its target lifetime",
                                value.span,
                            )
                        })?;
                    if restart {
                        let start = self.frames[index].start;
                        for frame in &self.frames[..index] {
                            for slot in frame.slots.values().flatten() {
                                if slot.order >= start && self.flow.overlap(self.reach, slot.guard)
                                {
                                    return Err(Diagnostic::unsupported(
                                        "restart after an emission into an enclosing scope",
                                        value.span,
                                    ));
                                }
                            }
                        }
                        self.forget_mutable();
                    } else {
                        let leaves = self.frames[index].leaves;
                        self.frames[index].leaves = self.flow.or(leaves, self.reach);
                    }
                    self.reach = FALSE;
                    return Ok(vec![if restart {
                        hir::Stmt::Restart(target)
                    } else {
                        hir::Stmt::Leave(target)
                    }]);
                }
                Ok(vec![hir::Stmt::Expr(self.expr(value, None)?)])
            }
            StmtKind::Forward { .. } => Err(Self::error(
                "E221",
                "forward signature is not in a definition group",
                stmt.span,
            )),
        }
    }

    pub(crate) fn emit(
        &mut self,
        label: Option<&str>,
        name: Option<&str>,
        annotation: Option<&ast::TypeExpr>,
        mutable: bool,
        value: &ast::Expr,
        span: Span,
    ) -> Result<Vec<hir::Stmt>> {
        if mutable {
            return Err(Diagnostic::unsupported("mutable emitted fields", span));
        }
        let target = match label {
            Some(name) => self.label(name, span)?,
            None => self.frames.last().expect("frame").id,
        };
        let frame = self
            .frames
            .iter()
            .find(|frame| frame.id == target)
            .expect("frame");
        let union = frame
            .expected
            .as_ref()
            .filter(|ty| Self::record_union(ty))
            .cloned();
        let record = if name.is_none() && matches!(frame.expected, Some(Type::Record { .. })) {
            frame.expected.clone()
        } else {
            None
        };
        let expected = match (&frame.expected, name) {
            (Some(ty), name) if Self::record_union(ty) => Self::union_slot(ty, name),
            (Some(Type::Record { primary, .. }), None) => Some(*primary.clone()),
            (Some(Type::Record { fields, .. }), Some(name)) => Some(
                fields
                    .iter()
                    .find(|(field, _)| field == name)
                    .map(|(_, ty)| ty.clone())
                    .ok_or_else(|| {
                        Self::error(
                            "E207",
                            format!("field `{name}` is not in the expected record"),
                            span,
                        )
                    })?,
            ),
            (Some(ty), None) => Some(ty.clone()),
            (Some(_), Some(name)) => {
                return Err(Self::error(
                    "E207",
                    format!("scalar result cannot contain field `{name}`"),
                    span,
                ));
            }
            (None, _) => None,
        };
        let annotated = annotation.map(|ty| self.ty(ty)).transpose()?;
        if let (Some(expected), Some(annotated)) = (&expected, &annotated)
            && !expected.accepts(annotated)
        {
            return Err(Self::error(
                "E207",
                "emission annotation differs from the required slot type",
                span,
            ));
        }
        let value = if let Some(record) = record {
            self.composed(value, record, annotated.as_ref().or(expected.as_ref()))?
        } else if union.is_some() {
            if let Some(annotated) = annotated.as_ref() {
                self.expr(value, Some(annotated))?
            } else {
                self.expression(value, expected.as_ref())?
            }
        } else {
            self.expr(value, annotated.as_ref().or(expected.as_ref()))?
        };
        if value.ty == Type::Never {
            return Ok(vec![hir::Stmt::Expr(value)]);
        }
        let mut stmts = Vec::new();
        if name.is_none()
            && matches!(value.ty, Type::Record { .. })
            && !union.as_ref().is_some_and(|ty| ty.accepts(&value.ty))
        {
            let ty = value.ty.clone();
            let id = self.local(ty.clone());
            let local = hir::Expr {
                kind: hir::ExprKind::Local(id),
                ty: ty.clone(),
                span,
            };
            stmts.push(hir::Stmt::Bind { id, value });
            let Type::Record { primary, fields } = ty else {
                unreachable!()
            };
            self.write_slot(target, None, *primary.clone(), mutable, span)?;
            stmts.push(self.emission(
                target,
                None,
                hir::Expr {
                    kind: hir::ExprKind::Primary(Box::new(local.clone())),
                    ty: *primary,
                    span,
                },
            ));
            for (index, (field, ty)) in fields.into_iter().enumerate() {
                self.write_slot(target, Some(field.clone()), ty.clone(), mutable, span)?;
                stmts.push(self.emission(
                    target,
                    Some(field),
                    hir::Expr {
                        kind: hir::ExprKind::Field {
                            value: Box::new(local.clone()),
                            index,
                        },
                        ty,
                        span,
                    },
                ));
            }
        } else if let Some(name) = name {
            let ty = value.ty.clone();
            let id = self.local(ty.clone());
            self.write_slot(target, Some(name.into()), ty.clone(), mutable, span)?;
            self.declare(
                name,
                Value::Local {
                    id,
                    ty: ty.clone(),
                    mutable: false,
                    owner: self.owner,
                    constant: self.constant(&value),
                },
                span,
            )?;
            stmts.push(hir::Stmt::Bind { id, value });
            stmts.push(self.emission(
                target,
                Some(name.into()),
                hir::Expr {
                    kind: hir::ExprKind::Local(id),
                    ty,
                    span,
                },
            ));
        } else {
            self.write_slot(target, None, value.ty.clone(), mutable, span)?;
            stmts.push(self.emission(target, None, value));
        }
        Ok(stmts)
    }

    pub(crate) fn emission(
        &mut self,
        target: hir::BlockId,
        field: Option<String>,
        value: hir::Expr,
    ) -> hir::Stmt {
        let id = self.proofs.emissions.len();
        self.proofs.emissions.insert(id, self.reach);
        hir::Stmt::Emit {
            id,
            target,
            field,
            value,
        }
    }

    pub(crate) fn write_slot(
        &mut self,
        target: usize,
        name: Option<String>,
        ty: Type,
        mutable: bool,
        span: Span,
    ) -> Result<()> {
        if self.reach == FALSE {
            return Ok(());
        }
        let frame = self
            .frames
            .iter()
            .find(|frame| frame.id == target)
            .expect("frame");
        let expected = match (&frame.expected, &name) {
            (Some(Type::Record { primary, .. }), None) => Some(primary.as_ref()),
            (Some(Type::Record { fields, .. }), Some(name)) => fields
                .iter()
                .find(|(field, _)| field == name)
                .map(|(_, ty)| ty),
            (Some(ty), None) => Some(ty),
            _ => None,
        };
        if let Some(expected) = expected
            && !expected.accepts(&ty)
            && !frame.expected.as_ref().is_some_and(Self::record_union)
        {
            return Err(Self::error(
                "E207",
                format!("result slot has type {ty:?}, expected {expected:?}"),
                span,
            ));
        }
        if frame.expected.is_some()
            && expected.is_none()
            && !frame.expected.as_ref().is_some_and(Self::record_union)
        {
            return Err(Self::error(
                "E207",
                "result field is absent from the expected type",
                span,
            ));
        }
        let slots = &mut self
            .frames
            .iter_mut()
            .find(|frame| frame.id == target)
            .expect("frame")
            .slots;
        let writes = slots.entry(name.clone()).or_default();
        if writes
            .iter()
            .any(|slot| self.flow.overlap(slot.guard, self.reach))
        {
            return Err(Self::error(
                "E205",
                format!(
                    "result {} may be emitted twice",
                    name.as_ref()
                        .map(|name| format!("field `{name}`"))
                        .unwrap_or_else(|| "primary".into())
                ),
                span,
            ));
        }
        writes.push(Slot {
            ty,
            mutable,
            guard: self.reach,
            order: self.writes,
        });
        self.writes += 1;
        Ok(())
    }

    pub(crate) fn composed(
        &mut self,
        value: &ast::Expr,
        record: Type,
        expected: Option<&Type>,
    ) -> Result<hir::Expr> {
        match &value.kind {
            ExprKind::Group(value) => self.composed(value, record, expected),
            ExprKind::Block(block) => {
                let block = self.block_inner(block, Some(record), None, true)?;
                Ok(hir::Expr {
                    ty: block.ty.clone(),
                    kind: hir::ExprKind::Block(block),
                    span: value.span,
                })
            }
            ExprKind::DispatchBlock {
                value: receiver,
                block,
            } => {
                let receiver = self.expr(receiver, None)?;
                let block = self.block_inner(block, Some(record), Some(receiver), true)?;
                Ok(hir::Expr {
                    ty: block.ty.clone(),
                    kind: hir::ExprKind::Block(block),
                    span: value.span,
                })
            }
            _ => {
                let value = self.expression(value, expected)?;
                if matches!(value.ty, Type::Record { .. }) {
                    Ok(value)
                } else if let Some(expected) = expected
                    && expected.accepts(&value.ty)
                {
                    Ok(Self::coerce(value, expected.clone()))
                } else {
                    Ok(value)
                }
            }
        }
    }

    pub(crate) fn expr(&mut self, expr: &ast::Expr, expected: Option<&Type>) -> Result<hir::Expr> {
        let value = self.expression(expr, expected)?;
        if value.ty == Type::Never {
            self.reach = FALSE;
            return Ok(value);
        }
        if let Some(expected) = expected
            && value.ty != *expected
        {
            if expected.accepts(&value.ty) {
                return Ok(Self::coerce(value, expected.clone()));
            }
            if let Type::Record { primary, .. } = &value.ty
                && expected.accepts(primary)
                && !matches!(expected, Type::Record { .. })
            {
                let ty = *primary.clone();
                return Ok(Self::coerce(
                    hir::Expr {
                        ty,
                        span: expr.span,
                        kind: hir::ExprKind::Primary(Box::new(value)),
                    },
                    expected.clone(),
                ));
            }
            return Err(Self::error(
                "E207",
                format!("expected {expected:?}, found {:?}", value.ty),
                expr.span,
            ));
        }
        Ok(value)
    }

    pub(crate) fn expression(
        &mut self,
        expr: &ast::Expr,
        expected: Option<&Type>,
    ) -> Result<hir::Expr> {
        let value = self.raw_expression(expr, expected)?;
        if value.ty == Type::Bool {
            self.guard(&value);
        }
        if value.ty == Type::Never {
            self.reach = FALSE;
        }
        Ok(value)
    }

    pub(crate) fn raw_expression(
        &mut self,
        expr: &ast::Expr,
        expected: Option<&Type>,
    ) -> Result<hir::Expr> {
        let (kind, ty) = match &expr.kind {
            ExprKind::Int(text) => return self.integer(text, false, expected, expr.span),
            ExprKind::Float(text) => {
                let context = Self::numeric_context(expected, false, expr.span)?;
                let ty = match context.as_ref() {
                    Some(Type::Float { bits }) => Type::Float { bits: *bits },
                    _ => Type::Float { bits: 64 },
                };
                let text = text.replace('_', "");
                let value: f64 = if ty == (Type::Float { bits: 32 }) {
                    text.parse::<f32>().map(f64::from)
                } else {
                    text.parse::<f64>()
                }
                .map_err(|_| {
                    Self::error("E216", "floating literal is not representable", expr.span)
                })?;
                if !value.is_finite() {
                    return Err(Self::error(
                        "E216",
                        format!("floating literal overflows {ty:?}"),
                        expr.span,
                    ));
                }
                (hir::ExprKind::Float(value), ty)
            }
            ExprKind::String(parts) => {
                let mut text = String::new();
                for part in parts {
                    match part {
                        ast::StringPart::Text(value) => text.push_str(value),
                        ast::StringPart::Value(_) => {
                            return Err(Diagnostic::unsupported(
                                "interpolated strings outside debug.print/debug.panic",
                                expr.span,
                            ));
                        }
                    }
                }
                (hir::ExprKind::String(text), Type::String)
            }
            ExprKind::Name(_)
            | ExprKind::Import(_)
            | ExprKind::TypeValue(_)
            | ExprKind::TypeQuery(_) => match self.symbol(expr)?.expect("symbol") {
                Value::Local { id, ty, .. } => {
                    return Ok(self.narrow(hir::Expr {
                        kind: hir::ExprKind::Local(id),
                        ty,
                        span: expr.span,
                    }));
                }
                Value::Constant(value) => return Ok(Self::constant_expr(value, expr.span)),
                Value::Function { .. } => {
                    return Err(Diagnostic::unsupported(
                        "first-class function values",
                        expr.span,
                    ));
                }
                Value::Type(_) => {
                    return Err(Diagnostic::unsupported(
                        "runtime use of type values",
                        expr.span,
                    ));
                }
                _ => {
                    return Err(Diagnostic::unsupported(
                        "runtime use of compile-time identities",
                        expr.span,
                    ));
                }
            },
            ExprKind::Group(value) => return self.expr(value, expected),
            ExprKind::Unary { op, value } => {
                if op == "-"
                    && let ExprKind::Int(text) = &value.kind
                {
                    return self.integer(text, true, expected, expr.span);
                }
                if op == "&" {
                    let (place, ty) = self.address(value)?;
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::Borrow(place),
                        ty: Type::Reference(Box::new(ty)),
                        span: expr.span,
                    });
                }
                if op == "*" {
                    let value = self.expr(value, None)?;
                    let Type::Reference(ty) = &value.ty else {
                        return Err(Self::error(
                            "E222",
                            "dereference requires a safe reference",
                            expr.span,
                        ));
                    };
                    return Ok(hir::Expr {
                        ty: *ty.clone(),
                        kind: hir::ExprKind::Deref(Box::new(value)),
                        span: expr.span,
                    });
                }
                if ["&!", ">>", "<<"].contains(&op.as_str()) {
                    return Err(Diagnostic::unsupported(format!("unary `{op}`"), expr.span));
                }
                let mut value = self.expr(value, expected)?;
                if matches!(value.ty, Type::Record { .. }) {
                    value = Self::project(value);
                }
                let valid = match op.as_str() {
                    "-" => matches!(
                        value.ty,
                        Type::Int { signed: true, .. } | Type::Float { .. }
                    ),
                    "!" => value.ty == Type::Bool,
                    "~" => matches!(value.ty, Type::Int { .. }),
                    _ => false,
                };
                if !valid {
                    return Err(Self::error(
                        "E222",
                        format!("operator `{op}` is not defined for {:?}", value.ty),
                        expr.span,
                    ));
                }
                if op == "-"
                    && self.reach != FALSE
                    && let Some(Constant::Int(number)) = self.constant(&value)
                    && number
                        .checked_neg()
                        .is_none_or(|number| !Self::in_range(number, &value.ty))
                {
                    return Err(Self::error(
                        "E107",
                        format!("negating {number} overflows {:?}", value.ty),
                        expr.span,
                    ));
                }
                let ty = value.ty.clone();
                (
                    hir::ExprKind::Unary {
                        op: op.clone(),
                        value: Box::new(value),
                    },
                    ty,
                )
            }
            ExprKind::Binary { op, left, right } => {
                return self.binary(op, left, right, expected, expr.span);
            }
            ExprKind::Call { callee, args } => return self.call(callee, args, None, expr.span),
            ExprKind::Dispatch {
                value,
                callee,
                args,
            } => return self.call(callee, args, Some(value), expr.span),
            ExprKind::DispatchBlock { value, block } => {
                let value = self.expr(value, None)?;
                if value.ty.has_reference() {
                    return Err(Diagnostic::unsupported(
                        "reference dispatch receivers",
                        expr.span,
                    ));
                }
                let block = self.block(block, expected.cloned(), Some(value))?;
                let ty = block.ty.clone();
                (hir::ExprKind::Block(block), ty)
            }
            ExprKind::Block(block) => {
                let block = self.block(block, expected.cloned(), None)?;
                let ty = block.ty.clone();
                (hir::ExprKind::Block(block), ty)
            }
            ExprKind::Field { value, name } => {
                if let Some(symbol) = self.symbol(expr)? {
                    return match symbol {
                        Value::Constant(value) => Ok(Self::constant_expr(value, expr.span)),
                        _ => Err(Diagnostic::unsupported(
                            "runtime use of intrinsic operation values",
                            expr.span,
                        )),
                    };
                }
                let mut value = self.expr(value, None)?;
                if let Type::Reference(ty) = &value.ty {
                    value = hir::Expr {
                        ty: *ty.clone(),
                        span: value.span,
                        kind: hir::ExprKind::Deref(Box::new(value)),
                    };
                }
                let Type::Record { fields, .. } = &value.ty else {
                    return Err(Self::error(
                        "E201",
                        format!("type {:?} has no field `{name}`", value.ty),
                        expr.span,
                    ));
                };
                let (index, (_, ty)) = fields
                    .iter()
                    .enumerate()
                    .find(|(_, (field, _))| field == name)
                    .ok_or_else(|| {
                        Self::error("E201", format!("unknown record field `{name}`"), expr.span)
                    })?;
                let ty = ty.clone();
                return Ok(self.narrow(hir::Expr {
                    kind: hir::ExprKind::Field {
                        value: Box::new(value),
                        index,
                    },
                    ty,
                    span: expr.span,
                }));
            }
            ExprKind::Ascribe {
                value,
                ty,
                predicate,
            } => {
                let value = self.expr(value, None)?;
                let ty = self.ty(ty)?;
                if value.ty == Type::Never {
                    return Ok(value);
                }
                if *predicate {
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::TypeTest {
                            value: Box::new(value),
                            ty,
                        },
                        ty: Type::Bool,
                        span: expr.span,
                    });
                }
                if !ty.accepts(&value.ty) {
                    return Err(Self::error(
                        "E208",
                        format!(
                            "value of type {:?} is not proven to have type {ty:?}",
                            value.ty
                        ),
                        expr.span,
                    ));
                }
                return Ok(Self::coerce(value, ty));
            }
            ExprKind::Function { .. } => {
                return Err(Diagnostic::unsupported(
                    "anonymous function values",
                    expr.span,
                ));
            }
            ExprKind::Index { .. } | ExprKind::List(_) => {
                return Err(Diagnostic::unsupported(
                    "collections and indexing",
                    expr.span,
                ));
            }
            ExprKind::Label(_) => {
                return Err(Self::error(
                    "E201",
                    "labels are control targets, not ordinary values",
                    expr.span,
                ));
            }
            ExprKind::Unsupported(feature) => {
                return Err(Diagnostic::unsupported(feature, expr.span));
            }
        };
        Ok(hir::Expr {
            kind,
            ty,
            span: expr.span,
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
                if !self.places.contains(&id) {
                    return Err(Diagnostic::unsupported(
                        "borrowing parameter, receiver or emitted storage",
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
                let (index, (_, ty)) = fields
                    .into_iter()
                    .enumerate()
                    .find(|(_, (field, _))| field == name)
                    .ok_or_else(|| {
                        Self::error("E201", format!("unknown record field `{name}`"), expr.span)
                    })?;
                place.fields.push(index);
                Ok((place, ty))
            }
            _ => Err(Diagnostic::unsupported(
                "borrowing temporary or projected storage",
                expr.span,
            )),
        }
    }

    pub(crate) fn numeric_context(
        expected: Option<&Type>,
        integer: bool,
        span: Span,
    ) -> Result<Option<Type>> {
        let types: Vec<_> = expected
            .into_iter()
            .flat_map(Type::members)
            .filter(|ty| {
                if integer {
                    matches!(ty, Type::Int { .. })
                } else {
                    matches!(ty, Type::Float { .. })
                }
            })
            .cloned()
            .collect();
        if types.len() > 1 {
            return Err(Self::error(
                "E207",
                "numeric literal has multiple possible expected types",
                span,
            ));
        }
        Ok(types.into_iter().next())
    }

    pub(crate) fn integer(
        &self,
        text: &str,
        negative: bool,
        expected: Option<&Type>,
        span: Span,
    ) -> Result<hir::Expr> {
        let text = text.replace('_', "");
        let (radix, digits) = if text.starts_with("0x") || text.starts_with("0X") {
            (16, &text[2..])
        } else if text.starts_with("0b") || text.starts_with("0B") {
            (2, &text[2..])
        } else {
            (10, text.as_str())
        };
        let context = Self::numeric_context(expected, true, span)?;
        let ty = match context.as_ref() {
            Some(Type::Int { bits, signed }) => Type::Int {
                bits: *bits,
                signed: *signed,
            },
            _ => Type::Int {
                bits: 32,
                signed: true,
            },
        };
        if negative && matches!(ty, Type::Int { signed: false, .. }) {
            return Err(Self::error(
                "E222",
                "unsigned integer negation is not defined",
                span,
            ));
        }
        let value = i128::from_str_radix(digits, radix)
            .ok()
            .and_then(|value| {
                if negative {
                    value.checked_neg()
                } else {
                    Some(value)
                }
            })
            .filter(|value| Self::in_range(*value, &ty))
            .ok_or_else(|| {
                Self::error(
                    "E216",
                    format!(
                        "literal `{}`{text} is not representable in {ty:?}",
                        if negative { "-" } else { "" }
                    ),
                    span,
                )
            })?;
        Ok(hir::Expr {
            kind: hir::ExprKind::Int(value),
            ty,
            span,
        })
    }

    pub(crate) fn in_range(value: i128, ty: &Type) -> bool {
        match ty {
            Type::Int { bits, signed: true } => {
                value >= -(1i128 << (bits - 1)) && value < (1i128 << (bits - 1))
            }
            Type::Int {
                bits,
                signed: false,
            } => value >= 0 && value < (1i128 << bits),
            _ => false,
        }
    }

    pub(crate) fn primary_type(ty: &Type) -> Type {
        match ty {
            Type::Record { primary, .. } => Self::primary_type(primary),
            ty => ty.clone(),
        }
    }

    pub(crate) fn project(value: hir::Expr) -> hir::Expr {
        let ty = Self::primary_type(&value.ty);
        if ty == value.ty {
            value
        } else {
            hir::Expr {
                ty,
                span: value.span,
                kind: hir::ExprKind::Primary(Box::new(value)),
            }
        }
    }

    pub(crate) fn hint(&mut self, expr: &ast::Expr) -> Option<Type> {
        match &expr.kind {
            ExprKind::Name(name) => match self.value(name, expr.span).ok()? {
                Value::Local { id, ty, .. } => Some(self.refined((id, Vec::new()), &ty)),
                Value::Constant(value) => Some(Self::constant_expr(value, expr.span).ty),
                _ => None,
            },
            ExprKind::Group(value) => self.hint(value),
            ExprKind::Unary { op, value } if op == "&" => self
                .address(value)
                .ok()
                .map(|(_, ty)| Type::Reference(Box::new(ty))),
            ExprKind::Unary { op, value } if op == "*" => match self.hint(value)? {
                Type::Reference(ty) => Some(*ty),
                _ => None,
            },
            ExprKind::Unary { value, .. } => self.hint(value),
            ExprKind::Binary { left, right, op } => {
                if ["==", "!=", "<", ">", "<=", ">=", "&&", "||"].contains(&op.as_str()) {
                    Some(Type::Bool)
                } else {
                    self.hint(left)
                        .or_else(|| self.hint(right))
                        .map(|ty| Self::primary_type(&ty))
                }
            }
            ExprKind::String(_) => Some(Type::String),
            ExprKind::Field { value, name } => {
                let ty = self.hint(value).map(|ty| match ty {
                    Type::Reference(ty) => *ty,
                    ty => ty,
                });
                if let Some(Type::Record { fields, .. }) = ty {
                    let ty = fields
                        .into_iter()
                        .find(|(field, _)| field == name)
                        .map(|(_, ty)| ty)?;
                    if let Some(place) = self.ast_place(expr) {
                        Some(self.refined(place, &ty))
                    } else {
                        Some(ty)
                    }
                } else {
                    None
                }
            }
            ExprKind::Call { callee, .. } => {
                if let ExprKind::Name(name) = &callee.kind {
                    match self.value(name, expr.span).ok()? {
                        Value::Function { result, .. } => result,
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
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

    pub(crate) fn binary(
        &mut self,
        op: &str,
        left: &ast::Expr,
        right: &ast::Expr,
        expected: Option<&Type>,
        span: Span,
    ) -> Result<hir::Expr> {
        if matches!(self.symbol(left)?, Some(Value::Function { .. }))
            || matches!(self.symbol(right)?, Some(Value::Function { .. }))
        {
            return Err(Self::error(
                "E222",
                format!("operator `{op}` is not defined for function values"),
                span,
            ));
        }
        let boolean = ["&&", "||"].contains(&op);
        let equality = ["==", "!="].contains(&op);
        let compare = ["==", "!=", "<", ">", "<=", ">="].contains(&op);
        let context = if boolean {
            Some(Type::Bool)
        } else {
            self.hint(left)
                .or_else(|| self.hint(right))
                .map(|ty| {
                    if equality {
                        ty
                    } else {
                        Self::primary_type(&ty)
                    }
                })
                .or_else(|| {
                    if compare {
                        None
                    } else {
                        expected.map(Self::primary_type)
                    }
                })
        };
        let mut left = if equality && matches!(context, Some(Type::Record { .. })) {
            let record = context.clone().expect("record context");
            let primary = Self::primary_type(&record);
            self.composed(left, record, Some(&primary))?
        } else {
            self.expression(left, context.as_ref())?
        };
        let skipped = if boolean {
            let guard = self.guard(&left);
            let guard = if op == "||" {
                self.flow.not(guard)
            } else {
                guard
            };
            let absent = self.flow.not(guard);
            let skipped = self.flow.and(self.reach, absent);
            self.reach = self.flow.and(self.reach, guard);
            skipped
        } else {
            FALSE
        };
        let right_context = if equality {
            left.ty.clone()
        } else {
            Self::primary_type(&left.ty)
        };
        let mut right = if equality && matches!(right_context, Type::Record { .. }) {
            let primary = Self::primary_type(&right_context);
            self.composed(right, right_context, Some(&primary))?
        } else {
            self.expression(right, Some(&right_context))?
        };
        if boolean {
            self.reach = self.flow.or(self.reach, skipped);
        }
        if !["==", "!="].contains(&op)
            || !matches!(
                (&left.ty, &right.ty),
                (Type::Record { .. }, Type::Record { .. })
            )
        {
            left = Self::project(left);
            right = Self::project(right);
        }
        if left.ty != right.ty
            && !(boolean
                && matches!(left.ty, Type::Bool | Type::Never)
                && matches!(right.ty, Type::Bool | Type::Never))
        {
            let numeric = matches!(left.ty, Type::Int { .. } | Type::Float { .. })
                && matches!(right.ty, Type::Int { .. } | Type::Float { .. });
            return Err(Self::error(
                if numeric { "E213" } else { "E222" },
                format!(
                    "operator `{op}` requires compatible operands, found {:?} and {:?}",
                    left.ty, right.ty
                ),
                span,
            ));
        }
        let valid = match op {
            "+" | "-" | "*" | "/" => matches!(left.ty, Type::Int { .. } | Type::Float { .. }),
            "%" | "&" | "|" | "^" => matches!(left.ty, Type::Int { .. }),
            "&&" | "||" => matches!(left.ty, Type::Bool | Type::Never),
            "<" | ">" | "<=" | ">=" => matches!(
                left.ty,
                Type::Int { .. } | Type::Float { .. } | Type::String
            ),
            "==" | "!=" => true,
            _ => false,
        };
        if !valid {
            return Err(Self::error(
                "E222",
                format!("operator `{op}` is not defined for {:?}", left.ty),
                span,
            ));
        }
        if self.reach != FALSE
            && matches!(left.ty, Type::Int { .. })
            && !compare
            && let (Some(Constant::Int(a)), Some(Constant::Int(b))) =
                (self.constant(&left), self.constant(&right))
        {
            let value = match op {
                "+" => a.checked_add(b),
                "-" => a.checked_sub(b),
                "*" => a.checked_mul(b),
                "/" => a.checked_div(b),
                "%" => a.checked_rem(b),
                "&" => Some(a & b),
                "|" => Some(a | b),
                "^" => Some(a ^ b),
                _ => Some(0),
            };
            if value.is_none_or(|value| !Self::in_range(value, &left.ty)) {
                return Err(Self::error(
                    "E107",
                    format!("{a} {op} {b} is invalid in {:?}", left.ty),
                    span,
                ));
            }
        }
        let ty = if boolean || compare {
            Type::Bool
        } else {
            left.ty.clone()
        };
        Ok(hir::Expr {
            kind: hir::ExprKind::Binary {
                op: op.into(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
            span,
        })
    }

    pub(crate) fn call(
        &mut self,
        callee: &ast::Expr,
        args: &[ast::Expr],
        receiver: Option<&ast::Expr>,
        span: Span,
    ) -> Result<hir::Expr> {
        if receiver.is_none()
            && let ExprKind::Field { value, name } = &callee.kind
            && name == "size"
            && self.hint(value) == Some(Type::String)
        {
            if !args.is_empty() {
                return Err(Self::error("E212", "string.size takes no arguments", span));
            }
            let value = self.expr(value, Some(&Type::String))?;
            return Ok(hir::Expr {
                kind: hir::ExprKind::StringSize(Box::new(value)),
                ty: Type::Int {
                    bits: 64,
                    signed: false,
                },
                span,
            });
        }
        let value = self.symbol(callee)?.ok_or_else(|| {
            Diagnostic::unsupported("indirect calls and callable fields", callee.span)
        })?;
        let args: Vec<_> = receiver.into_iter().chain(args.iter()).collect();
        let (kind, ty) = match value {
            Value::Print | Value::Panic => {
                if args.len() != 1 {
                    return Err(Self::error(
                        "E212",
                        "debug output operations take exactly one argument",
                        span,
                    ));
                }
                let mut parts = Vec::new();
                self.format_parts(args[0], &mut parts)?;
                if matches!(value, Value::Print) {
                    (
                        hir::ExprKind::Print {
                            parts,
                            newline: true,
                        },
                        Type::Null,
                    )
                } else {
                    (hir::ExprKind::Panic { parts }, Type::Never)
                }
            }
            Value::Function { id, params, result } => {
                if params.len() != args.len() {
                    return Err(Self::error(
                        "E212",
                        format!(
                            "function expects {} arguments, found {}",
                            params.len(),
                            args.len()
                        ),
                        span,
                    ));
                }
                let result = result.ok_or_else(|| {
                    Diagnostic::unsupported(
                        "recursive functions without an explicit result annotation",
                        span,
                    )
                })?;
                let mut values = Vec::new();
                for (arg, ty) in args.into_iter().zip(params) {
                    values.push(self.expr(arg, Some(&ty)).map_err(|error| {
                        if error.code == "E207" {
                            Self::error("E212", error.message, error.span)
                        } else {
                            error
                        }
                    })?);
                }
                let site = self.calls;
                self.calls += 1;
                self.proofs.calls.insert(site, self.reach);
                (
                    hir::ExprKind::Call {
                        id,
                        site,
                        args: values,
                    },
                    result,
                )
            }
            Value::Control { .. } => {
                return Err(Diagnostic::unsupported(
                    "scope control calls in value expressions",
                    span,
                ));
            }
            _ => return Err(Self::error("E212", "value is not callable", callee.span)),
        };
        Ok(hir::Expr { kind, ty, span })
    }

    pub(crate) fn format_parts(
        &mut self,
        expr: &ast::Expr,
        parts: &mut Vec<hir::Expr>,
    ) -> Result<()> {
        match &expr.kind {
            ExprKind::String(values) => {
                for value in values {
                    match value {
                        ast::StringPart::Text(text) => parts.push(hir::Expr {
                            kind: hir::ExprKind::String(text.clone()),
                            ty: Type::String,
                            span: expr.span,
                        }),
                        ast::StringPart::Value(value) => self.format_parts(value, parts)?,
                    }
                }
            }
            ExprKind::Group(value) => self.format_parts(value, parts)?,
            _ => {
                let value = Self::project(self.expr(expr, None)?);
                if value.ty.has_reference() {
                    return Err(Diagnostic::unsupported(
                        "reference formatting; dereference the copyable value",
                        expr.span,
                    ));
                }
                parts.push(value);
            }
        }
        Ok(())
    }

    pub(crate) fn constant_expr(value: Constant, span: Span) -> hir::Expr {
        let (kind, ty) = match value {
            Constant::Null => (hir::ExprKind::Null, Type::Null),
            Constant::Bool(value) => (hir::ExprKind::Bool(value), Type::Bool),
            Constant::Int(value) => (
                hir::ExprKind::Int(value),
                Type::Int {
                    bits: 32,
                    signed: true,
                },
            ),
            Constant::Float(value) => (hir::ExprKind::Float(value), Type::Float { bits: 64 }),
            Constant::String(value) => (hir::ExprKind::String(value), Type::String),
        };
        hir::Expr { kind, ty, span }
    }

    pub(crate) fn constant(&self, expr: &hir::Expr) -> Option<Constant> {
        match &expr.kind {
            hir::ExprKind::Null => Some(Constant::Null),
            hir::ExprKind::Bool(value) => Some(Constant::Bool(*value)),
            hir::ExprKind::Int(value) => Some(Constant::Int(*value)),
            hir::ExprKind::Float(value) => Some(Constant::Float(*value)),
            hir::ExprKind::String(value) => Some(Constant::String(value.clone())),
            hir::ExprKind::Local(id) => self
                .scopes
                .iter()
                .rev()
                .flat_map(|scope| scope.values.values())
                .find_map(|value| match value {
                    Value::Local {
                        id: local,
                        constant,
                        ..
                    } if id == local => constant.clone(),
                    _ => None,
                }),
            hir::ExprKind::Unary { op, value } => match (op.as_str(), self.constant(value)?) {
                ("!", Constant::Bool(value)) => Some(Constant::Bool(!value)),
                ("-", Constant::Int(value)) => value.checked_neg().map(Constant::Int),
                ("~", Constant::Int(value)) => Some(Constant::Int(match expr.ty {
                    Type::Int {
                        bits,
                        signed: false,
                    } => !value & ((1i128 << bits) - 1),
                    _ => !value,
                })),
                ("-", Constant::Float(value)) => Some(Constant::Float(-value)),
                _ => None,
            },
            hir::ExprKind::Binary { op, left, right } => {
                let a = self.constant(left)?;
                if let Constant::Bool(value) = a
                    && (op == "&&" && !value || op == "||" && value)
                {
                    return Some(Constant::Bool(value));
                }
                let b = self.constant(right)?;
                match (a, b) {
                    (Constant::Int(a), Constant::Int(b)) => match op.as_str() {
                        "+" => a.checked_add(b).map(Constant::Int),
                        "-" => a.checked_sub(b).map(Constant::Int),
                        "*" => a.checked_mul(b).map(Constant::Int),
                        "/" => a.checked_div(b).map(Constant::Int),
                        "%" => a.checked_rem(b).map(Constant::Int),
                        "&" => Some(Constant::Int(a & b)),
                        "|" => Some(Constant::Int(a | b)),
                        "^" => Some(Constant::Int(a ^ b)),
                        "==" => Some(Constant::Bool(a == b)),
                        "!=" => Some(Constant::Bool(a != b)),
                        "<" => Some(Constant::Bool(a < b)),
                        ">" => Some(Constant::Bool(a > b)),
                        "<=" => Some(Constant::Bool(a <= b)),
                        ">=" => Some(Constant::Bool(a >= b)),
                        _ => None,
                    },
                    (Constant::Bool(a), Constant::Bool(b)) => match op.as_str() {
                        "&&" => Some(Constant::Bool(a && b)),
                        "||" => Some(Constant::Bool(a || b)),
                        "==" => Some(Constant::Bool(a == b)),
                        "!=" => Some(Constant::Bool(a != b)),
                        _ => None,
                    },
                    (Constant::Null, Constant::Null) => match op.as_str() {
                        "==" => Some(Constant::Bool(true)),
                        "!=" => Some(Constant::Bool(false)),
                        _ => None,
                    },
                    (Constant::String(a), Constant::String(b)) => match op.as_str() {
                        "==" => Some(Constant::Bool(a == b)),
                        "!=" => Some(Constant::Bool(a != b)),
                        "<" => Some(Constant::Bool(a < b)),
                        ">" => Some(Constant::Bool(a > b)),
                        "<=" => Some(Constant::Bool(a <= b)),
                        ">=" => Some(Constant::Bool(a >= b)),
                        _ => None,
                    },
                    _ => None,
                }
            }
            hir::ExprKind::Block(block) => {
                self.constants
                    .get(&block.id)
                    .cloned()
                    .or_else(|| match block.stmts.as_slice() {
                        [
                            hir::Stmt::Emit {
                                target,
                                field: None,
                                value,
                                ..
                            },
                        ] if *target == block.id => self.constant(value),
                        _ => None,
                    })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    pub(crate) fn accepts(source: &str) {
        if let Err(errors) = crate::compile(source) {
            panic!("{source}\n{errors:?}");
        }
    }

    pub(crate) fn rejects(source: &str, code: &str) {
        let errors = crate::compile(source).expect_err(source);
        assert_eq!(errors[0].code, code, "{source}\n{errors:?}");
    }

    #[test]
    pub(crate) fn integer_literals_respect_expected_width_and_negation_boundary() {
        accepts("x<int8>:-128;y<uint64>:18446744073709551615;z<int16>:-32_768");
        rejects("x<int8>:128", "E216");
        rejects("x<int8>:-(128)", "E216");
        rejects("x<uint8>:-1", "E222");
        rejects("x:2147483648", "E216");
    }

    #[test]
    pub(crate) fn checked_arithmetic_propagates_immutable_constants() {
        rejects("x<int8>:127;y:x+1", "E107");
        rejects("x<int8>:-128;y:-x", "E107");
        rejects("x:1/0", "E107");
        accepts("x<int8>:=127;y:x+1");
        accepts("x:false&&(1/0==0)");
        accepts("x<int64>:-9223372036854775808;y:x%-1");
        rejects("x<int64>:-9223372036854775808;y:x/-1", "E107");
        accepts("flag:=true;value:'v {|flag|{'v->1;'v.leave()};->0};result:1/value");
    }

    #[test]
    pub(crate) fn float32_literals_round_once_from_decimal() {
        let program = crate::compile("x<float32>:1.0000000596046447753906250000000001").unwrap();
        let crate::hir::Stmt::Bind { value, .. } = &program.body.stmts[0] else {
            panic!("binding");
        };
        let crate::hir::ExprKind::Float(value) = value.kind else {
            panic!("float");
        };
        assert_eq!(value, f64::from(f32::from_bits(0x3f800001)));
        rejects("x<float32>:3.5e38", "E216");
        accepts("x<float32>:1.0e-99");
    }

    #[test]
    pub(crate) fn typed_numeric_values_never_widen_silently() {
        accepts("x<uint8>:2;y:1+x;z<uint8>:y");
        rejects("x<int8>:1;y<int16>:2;z:x+y", "E213");
        rejects("x<int8>:1;y<int16>:x", "E207");
    }

    #[test]
    pub(crate) fn scalar_assignment_and_matcher_domains_are_checked() {
        rejects("x:1;x=2", "E305");
        rejects("x:=1;x=\"two\"", "E207");
        rejects("|1|x:2", "E215");
        rejects("x:true+false", "E222");
        rejects("x<float32>:1.0;y<float64>:1.0;z:x+y", "E213");
    }

    #[test]
    pub(crate) fn lexical_aliases_preserve_intrinsic_identity() {
        accepts(
            "core:@\"core\";debug:@\"debug\";print:debug.print;{true:\"shadow\";x<core.boolean>:core.true;print(x)}",
        );
        accepts("<T>:<int8>;T:42;x<T>:1");
        rejects("x:1;x:2", "E203");
        rejects("x<missing>:1", "E202");
        rejects("x:missing", "E201");
    }

    #[test]
    pub(crate) fn blocks_construct_records_and_project_scalar_primaries() {
        accepts(
            "reading:{->24;->unit:\"celsius\"};answer<int32>:reading+1;copy<int32>:reading;unit:reading.unit",
        );
        accepts("outer:{->{->42;->unit:\"u\"};->ready:true};unit:outer.unit");
        rejects("x:{->1;->2}", "E205");
        rejects("x:{->name:1;->name:2}", "E205");
    }

    #[test]
    pub(crate) fn normal_paths_must_initialize_required_results() {
        accepts("f<int32>:(flag<boolean>) 'result {|flag|{'result->1;'result.leave()};->2}");
        rejects("f<int32>:(flag<boolean>){|flag|->1}", "E204");
        rejects("f<int32>:(flag<boolean>){|flag|->1;->2}", "E205");
        accepts("x:{|false|->1;->2}");
    }

    #[test]
    pub(crate) fn independent_matchers_share_flow_facts() {
        let source = format!("flag:=true;{}", "|flag|x:1;".repeat(1000));
        accepts(&source);
    }

    #[test]
    pub(crate) fn forward_groups_support_mutual_recursion_and_require_adjacency() {
        accepts(
            "even<(uint32)->boolean>;odd<(uint32)->boolean>;even<boolean>:(n<uint32>) 'r {|n==0|{'r->true;'r.leave()};->odd(n-1)};odd<boolean>:(n<uint32>) 'r {|n==0|{'r->false;'r.leave()};->even(n-1)};x:even(4)",
        );
        rejects("f<()->int32>;x:1;f<int32>:(){->1}", "E221");
        rejects("f<()->int32>;f<boolean>:(){->true}", "E221");
        rejects("f<()->int32>", "E221");
    }

    #[test]
    pub(crate) fn scoped_control_aliases_obey_lexical_lifetime() {
        accepts("x:'work {finish:'work.leave;->1;finish()}");
        accepts("n:=0;'work {n=n+1;|n<3|'work.restart()}");
        rejects("'work {}; 'work.leave()", "E201");
        rejects("x:'work {f:(){'work.leave()}}", "E201");
    }

    #[test]
    pub(crate) fn unsupported_features_have_capability_diagnostics() {
        rejects("x:[1,2]", "B001");
        rejects("x:1;f<int32>:(){->x}", "B001");
        rejects("x:1;text:\"{x}\"", "B001");
        rejects("f<int32>:(){->1};x:f==f", "E222");
    }

    #[test]
    pub(crate) fn shared_references_keep_storage_types_and_copy_values() {
        accepts("a:1;r<&int32>:&a;s:r;v:*s;same:r==s");
        accepts("a:{->x:1;->nested:{->y:true}};r:&a;v:r.x;s:&a.nested.y;t:*s");
        accepts("f<int32>:(){a:9;r:&a;->*r};v:f()");
        accepts("x<int32><null>:1;r:&x;v:*r");
        rejects("x:1;v:*x", "E222");
        rejects("x:1;r<&int64>:&x", "E207");
    }

    #[test]
    pub(crate) fn record_equality_preserves_aggregate_and_scalar_contexts() {
        accepts("a:1;b:2;left:{->view:&a};right:{->view:&b};same:left=={->right}");
        accepts("a:1;b:2;left:{->view:&a};same:left=={->view:&b}");
        accepts("a:1;n<int64>:7;packet:{->n;->view:&a};same:packet==7;reverse:7==packet");
        accepts("a:1;n<uint8>:7;packet:{->n;->view:&a};same:packet==7");
        accepts(
            "a:=1;n<int64>:7;packet:{->n;->view:&a};a=2;same:packet=={->7};reverse:{->7}==packet",
        );
        accepts(
            "f<boolean>:(flag<boolean>){a:1;n<int64>:7;packet:{->n;->view:&a};->packet=='result {|flag|{'result->7;'result.leave()};->7}}",
        );
        rejects(
            "a:1;n<uint8>:7;packet:{->n;->view:&a};same:packet==256",
            "E216",
        );
    }

    #[test]
    pub(crate) fn union_record_constructors_keep_member_context_and_defaults() {
        accepts(
            "<R>:<{view<&int32><null>}>;a:1;u<R><null>:{->view:&a};|u<R>|{|u.view<&int32>|x:*u.view<&int32>}",
        );
        accepts("<R>:<{view<&int32><null>;count<int64>}>;u<R><null>:{->count:7};|u<R>|x:u.count");
        accepts("<R>:<{-><int64>;tag<string>}>;u<R><null>:{->7;->tag:\"ok\"};|u<R>|x<int64>:u");
        accepts("<R>:<{value<uint8>;view<&int32>}>;a:1;u<R><null>:{->value:255;->view:&a}");
        rejects("<R>:<{value<uint8>}>;u<R><null>:{->value:256}", "E216");
        rejects(
            "<A>:<{value<int32>}>;<B>:<{value<int64>}>;u<A><B>:{->value:7}",
            "E207",
        );
        rejects(
            "<A>:<{value<int32>}>;<B>:<{value<int32><null>}>;u<A><B>:{->value:7}",
            "E207",
        );
        rejects(
            "<A>:<{view<&int32>}>;<B>:<{view<&string>}>;s:\"x\";u<A><B>:{->view<&int32>:&s}",
            "E207",
        );
    }

    #[test]
    pub(crate) fn union_equality_requires_the_same_normalized_union_type() {
        accepts("a:1;u<&int32><null>:&a;empty<&int32><null>:null;same:u==empty");
        rejects("a:1;u<&int32><null>:&a;same:u==null", "E222");
        rejects("a:1;u<&int32><null>:&a;same:null==u", "E222");
        rejects("a:1;u<&int32><null>:&a;same:u==&a", "E222");
        rejects("u<int8><null>:7;same:u==7", "E222");
        rejects("u<int8><null>:7;value<int32>:7;same:u==value", "E222");
    }

    #[test]
    pub(crate) fn reference_capability_boundaries_are_explicit() {
        for source in [
            "x:1;r:=&x",
            "x:=1;r:&!x",
            "r:&(1+2)",
            "x:1;r:&x;s:&r",
            "x:{->a:1;r:&a}",
            "f<int32>:(x<int32>){r:&x;->*r}",
            "f<int32>:(x<&!int32>){->*x}",
            "<R>:<{value<&int32>}>;f<null>:(x<&R>){->null}",
            "x:1;r:&x;r.{v:*self}",
            "x:1;r:&x;debug:@\"debug\";debug.print(r)",
            "x:1;r:&x;s:&*r",
            "<R>:<{x<int32>}>;record<R>:{->x:1};x<R><null>:record;|x<R>|{r:&x.x}",
        ] {
            rejects(source, "B001");
        }
    }

    #[test]
    pub(crate) fn union_members_keep_numeric_context_without_widening() {
        accepts("x<int8><null>:-128;y<float32><null>:1.5");
        accepts("x<{count<int8><null>}>:{->count<int8>:127}");
        accepts("<T>:<string><null><never><string>;x<T>:null;y<null><string>:x");
        rejects("x<int8><null>:128", "E216");
        rejects("x<int8>:1;y<int16><null>:x", "E207");
        rejects("x<int8><int32>:1", "E207");
        rejects("x<float32><float64>:1.5", "E207");
        rejects("x<string><null>:null;y<string>:x", "E207");
        rejects("x<string><null>:null;y:x<string>", "E208");
    }

    #[test]
    pub(crate) fn conditional_slots_join_only_completing_paths() {
        accepts(
            "make:(flag<boolean>){|flag|->name:\"hello\"};value<{name<string><null>}>:make(false)",
        );
        accepts("f<int32><null>:(flag<boolean>){|flag|->1}");
        accepts("x<{name<string><null>}>:{}");
        accepts("x<{name<string><null>}>:{->{->name:\"ok\"}}");
        accepts("x<{count<int8>;name<string><null>}>:{->{->count:127};->name:\"ok\"}");
        rejects(
            "f<{name<string>}>:(flag<boolean>){|flag|->name:\"x\"}",
            "E204",
        );
        rejects("x<{count<int8>}>:{->{->count:128}}", "E216");
        rejects("x<{name<string><null>}>:{->{->other:1}}", "E207");
        let program =
            crate::compile("n:=0;x:'loop {n=n+1;|n<2|{'loop->gone:1;'loop.restart()}}").unwrap();
        assert_eq!(program.locals.last(), Some(&crate::hir::Type::Null));
    }

    #[test]
    pub(crate) fn stable_predicates_prove_disjoint_emissions() {
        accepts("f<int32>:(v<string><null>){|v<null>|->1;|!(v<null>)|->2}");
        accepts("f<int32>:(v<string><int32><null>){|v<null>|->1;|v<string>|->2;|v<int32>|->3}");
        accepts("f<int32>:(flag<boolean>){|flag|->1;|!flag|->2}");
        rejects(
            "f<int32>:(v<string><null>){|v<null>|->1;|v<null>|->2}",
            "E205",
        );
        rejects("f<int32>:(v<int32>){|v>0|->1;|v>10|->2}", "E205");
        accepts(
            "f<int32>:(v<string><int32><null>){|!(v<int32>)&&v<string>|->1;|v<int32><null>|->2}",
        );
    }

    #[test]
    pub(crate) fn distinct_record_variants_keep_their_field_tag_domains() {
        accepts(
            "<A>:<{name<string><null>}>;<B>:<{name<int32><null>}>;f<null>:(x<A><B>){|x<A>|{|x.name<string>|y:x.name<string>};|x<B>|{|x.name<int32>|y:x.name+1}}",
        );
    }

    #[test]
    pub(crate) fn narrowing_tracks_short_circuit_and_leaving_paths() {
        accepts(
            "f<string>:(v<string><null>) 'r {|v<null>|{'r->\"fallback\";'r.leave()};->v<string>}",
        );
        accepts("f<null>:(v<string><null>){|v<string>&&v.size()>0|x:v<string>}");
        accepts("f<null>:(v<string><null>){|v<null>||v.size()==0|{}}");
        accepts("f<null>:(v<{name<string><null>}>){|!(v.name<null>)|x:v.name<string>}");
        rejects("f<null>:(v<string><null>){|v<null>|{};x:v<string>}", "E208");
        rejects(
            "f<null>:(v<string><null>){|v<string>||true|x:v<string>}",
            "E208",
        );
    }

    #[test]
    pub(crate) fn assignment_invalidates_scalar_and_field_proofs() {
        rejects(
            "v<string><null>:=\"x\";|v<string>|{v=null;x:v<string>}",
            "E208",
        );
        rejects(
            "v<{name<string><null>}>:={->name:\"x\"};|v.name<string>|{v={};x:v.name<string>}",
            "E208",
        );
        rejects(
            "v<string><null>:=\"x\";|v<null>|v=\"x\";x:v<string>",
            "E208",
        );
        accepts("v<string><null>:=\"x\";|v<string>|{x:v<string>;v=null}");
    }

    #[test]
    pub(crate) fn restart_drops_mutable_proofs_and_rejects_outer_slot_hazards() {
        rejects(
            "v<string><null>:=\"x\";|v<string>|'loop {x:v<string>;v=null;'loop.restart()}",
            "E208",
        );
        rejects("x:'outer {'inner {'outer->1;'inner.restart()}}", "B001");
        accepts("n:=0;x:'loop {n=n+1;|n<2|{'loop->1;'loop.restart()};->2}");
    }
}
