use std::collections::BTreeMap;

use crate::ast::{self, ExprKind, Span, StmtKind, TypeKind};
use crate::diagnostic::Diagnostic;
use crate::hir::{self, Type};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Slots = BTreeMap<Option<String>, Slot>;
pub(crate) type Path = BTreeMap<usize, Slots>;

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
}

pub(crate) struct Frame {
    pub(crate) id: usize,
    pub(crate) expected: Option<Type>,
    pub(crate) leaves: Vec<Path>,
    pub(crate) owner: usize,
}

pub(crate) struct Checker {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) paths: Vec<Path>,
    pub(crate) functions: Vec<Option<hir::Function>>,
    pub(crate) locals: Vec<Type>,
    pub(crate) constants: BTreeMap<usize, Constant>,
    pub(crate) block: usize,
    pub(crate) owner: usize,
}

pub fn check(block: &ast::Block) -> std::result::Result<hir::Program, Vec<Diagnostic>> {
    let mut checker = Checker::new();
    match checker.block(block, None, None) {
        Ok(body) => Ok(hir::Program {
            body,
            functions: checker.functions.into_iter().flatten().collect(),
            locals: checker.locals,
        }),
        Err(error) => Err(vec![error]),
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
            paths: vec![Path::new()],
            functions: Vec::new(),
            locals: Vec::new(),
            constants: BTreeMap::new(),
            block: 0,
            owner: 0,
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
            TypeKind::Function { params, result } => Ok(Spec::Function {
                params: params.iter().map(|ty| self.ty(ty)).collect::<Result<_>>()?,
                result: self.ty(result)?,
            }),
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
                Ok(Spec::Data(Type::Record {
                    primary: Box::new(primary),
                    fields: result.into_iter().collect(),
                }))
            }
            TypeKind::Computed(value) => Ok(Spec::Data(self.type_value(value)?)),
            TypeKind::Union(types) => {
                let mut result = None;
                for ty in types {
                    let ty = self.ty(ty)?;
                    if result.as_ref().is_some_and(|value| *value != ty) {
                        return Err(Diagnostic::unsupported("union types", expr.span));
                    }
                    result = Some(ty);
                }
                Ok(Spec::Data(result.unwrap_or(Type::Never)))
            }
            TypeKind::List { .. } => {
                Err(Diagnostic::unsupported("list and slice types", expr.span))
            }
            TypeKind::Reference { .. } => {
                Err(Diagnostic::unsupported("reference types", expr.span))
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

    pub(crate) fn block(
        &mut self,
        block: &ast::Block,
        expected: Option<Type>,
        receiver: Option<hir::Expr>,
    ) -> Result<hir::Block> {
        let id = self.block;
        self.block += 1;
        self.scopes.push(Scope::default());
        if let Some(label) = &block.label {
            self.scopes
                .last_mut()
                .expect("scope")
                .labels
                .insert(label.clone(), id);
        }
        self.frames.push(Frame {
            id,
            expected: expected.clone(),
            leaves: Vec::new(),
            owner: self.owner,
        });
        for path in &mut self.paths {
            path.insert(id, Slots::new());
        }
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
        self.paths.extend(frame.leaves);
        let ty = self.block_type(id, expected.as_ref(), block.span)?;
        for path in &mut self.paths {
            path.remove(&id);
        }
        self.scopes.pop();
        Ok(hir::Block { id, ty, stmts })
    }

    pub(crate) fn block_type(
        &self,
        id: usize,
        expected: Option<&Type>,
        span: Span,
    ) -> Result<Type> {
        if self.paths.is_empty() {
            return Ok(Type::Never);
        }
        let mut slots: Slots = Slots::new();
        for path in &self.paths {
            if let Some(fields) = path.get(&id) {
                for (name, slot) in fields {
                    if let Some(prior) = slots.get(name) {
                        if prior.mutable != slot.mutable {
                            return Err(Self::error(
                                "E206",
                                "result field mutability differs between paths",
                                span,
                            ));
                        }
                        if prior.ty != slot.ty {
                            return Err(Diagnostic::unsupported(
                                "result slots with union types",
                                span,
                            ));
                        }
                    } else {
                        slots.insert(name.clone(), slot.clone());
                    }
                }
            }
        }
        if let Some(expected) = expected {
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
                if ty != Type::Null
                    && self
                        .paths
                        .iter()
                        .any(|path| !path.get(&id).is_some_and(|slots| slots.contains_key(&name)))
                {
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
            }
        }
        for (name, slot) in &slots {
            if slot.ty != Type::Null
                && self.paths.iter().any(|path| {
                    !path
                        .get(&id)
                        .is_some_and(|fields| fields.contains_key(name))
                })
            {
                return Err(Diagnostic::unsupported(
                    "inferred nullable result slots",
                    span,
                ));
            }
        }
        let primary = slots
            .remove(&None)
            .map(|slot| slot.ty)
            .unwrap_or(Type::Null);
        let fields: Vec<_> = slots
            .into_iter()
            .filter_map(|(name, slot)| name.map(|name| (name, slot.ty)))
            .collect();
        let actual = if fields.is_empty() {
            primary
        } else {
            Type::Record {
                primary: Box::new(primary),
                fields,
            }
        };
        if let Some(expected) = expected {
            if let Type::Record { primary, fields } = expected {
                let actual_fields = match &actual {
                    Type::Record { fields, .. } => fields.clone(),
                    _ => Vec::new(),
                };
                let actual_primary = Self::primary_type(&actual);
                if actual_primary != **primary
                    || actual_fields.iter().any(|field| !fields.contains(field))
                {
                    return Err(Self::error(
                        "E207",
                        format!("block has type {actual:?}, expected {expected:?}"),
                        span,
                    ));
                }
                return Ok(expected.clone());
            }
            if actual != *expected {
                return Err(Self::error(
                    "E207",
                    format!("block has type {actual:?}, expected {expected:?}"),
                    span,
                ));
            }
        }
        Ok(actual)
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
        let paths = std::mem::replace(&mut self.paths, vec![Path::new()]);
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
        self.paths = paths;
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
                let id = self.local(ty.clone());
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
                Ok(vec![hir::Stmt::Assign {
                    id,
                    value: self.expr(value, Some(&ty))?,
                }])
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
                    let constant = match self.constant(&condition) {
                        Some(Constant::Bool(value)) => Some(value),
                        _ => None,
                    };
                    let before = self.paths.clone();
                    if constant == Some(false) {
                        self.paths.clear();
                    }
                    self.scopes.push(Scope::default());
                    let then = self.stmt(body)?;
                    self.scopes.pop();
                    if constant != Some(true) {
                        self.paths.extend(before);
                    }
                    if self.paths.len() > 4096 {
                        return Err(Diagnostic::unsupported(
                            "control-flow graphs with more than 4096 active paths",
                            stmt.span,
                        ));
                    }
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
                    let frame = self
                        .frames
                        .iter_mut()
                        .find(|frame| frame.id == target)
                        .ok_or_else(|| {
                            Self::error(
                                "E201",
                                "scope operation escaped its target lifetime",
                                value.span,
                            )
                        })?;
                    if !restart {
                        frame.leaves.append(&mut self.paths);
                    } else {
                        self.paths.clear();
                    }
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
        let expected = match (&frame.expected, name) {
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
            && expected != annotated
        {
            return Err(Self::error(
                "E207",
                "emission annotation differs from the required slot type",
                span,
            ));
        }
        let value = self.expr(value, annotated.as_ref().or(expected.as_ref()))?;
        if value.ty == Type::Never {
            return Ok(vec![hir::Stmt::Expr(value)]);
        }
        let mut stmts = Vec::new();
        if name.is_none() && matches!(value.ty, Type::Record { .. }) {
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
            stmts.push(hir::Stmt::Emit {
                target,
                field: None,
                value: hir::Expr {
                    kind: hir::ExprKind::Primary(Box::new(local.clone())),
                    ty: *primary,
                    span,
                },
            });
            for (index, (field, ty)) in fields.into_iter().enumerate() {
                self.write_slot(target, Some(field.clone()), ty.clone(), mutable, span)?;
                stmts.push(hir::Stmt::Emit {
                    target,
                    field: Some(field),
                    value: hir::Expr {
                        kind: hir::ExprKind::Field {
                            value: Box::new(local.clone()),
                            index,
                        },
                        ty,
                        span,
                    },
                });
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
            stmts.push(hir::Stmt::Emit {
                target,
                field: Some(name.into()),
                value: hir::Expr {
                    kind: hir::ExprKind::Local(id),
                    ty,
                    span,
                },
            });
        } else {
            self.write_slot(target, None, value.ty.clone(), mutable, span)?;
            stmts.push(hir::Stmt::Emit {
                target,
                field: None,
                value,
            });
        }
        Ok(stmts)
    }

    pub(crate) fn write_slot(
        &mut self,
        target: usize,
        name: Option<String>,
        ty: Type,
        mutable: bool,
        span: Span,
    ) -> Result<()> {
        for path in &mut self.paths {
            let slots = path.entry(target).or_default();
            if slots.contains_key(&name) {
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
            slots.insert(
                name.clone(),
                Slot {
                    ty: ty.clone(),
                    mutable,
                },
            );
        }
        Ok(())
    }

    pub(crate) fn expr(&mut self, expr: &ast::Expr, expected: Option<&Type>) -> Result<hir::Expr> {
        let value = self.expression(expr, expected)?;
        if value.ty == Type::Never {
            self.paths.clear();
            return Ok(value);
        }
        if let Some(expected) = expected
            && value.ty != *expected
        {
            if let Type::Record { primary, .. } = &value.ty
                && **primary == *expected
                && !matches!(expected, Type::Record { .. })
            {
                return Ok(hir::Expr {
                    ty: expected.clone(),
                    span: expr.span,
                    kind: hir::ExprKind::Primary(Box::new(value)),
                });
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
        let (kind, ty) = match &expr.kind {
            ExprKind::Int(text) => return self.integer(text, false, expected, expr.span),
            ExprKind::Float(text) => {
                let ty = match expected {
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
                Value::Local { id, ty, .. } => (hir::ExprKind::Local(id), ty),
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
                if ["&", "&!", "*", ">>", "<<"].contains(&op.as_str()) {
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
                    && !self.paths.is_empty()
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
                let value = self.expr(value, None)?;
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
                (
                    hir::ExprKind::Field {
                        value: Box::new(value),
                        index,
                    },
                    ty,
                )
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
                    let id = self.next_block();
                    let constant = Constant::Bool(value.ty == ty);
                    self.constants.insert(id, constant.clone());
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::Block(hir::Block {
                            id,
                            ty: Type::Bool,
                            stmts: vec![
                                hir::Stmt::Expr(value),
                                hir::Stmt::Emit {
                                    target: id,
                                    field: None,
                                    value: Self::constant_expr(constant, expr.span),
                                },
                            ],
                        }),
                        ty: Type::Bool,
                        span: expr.span,
                    });
                }
                if value.ty != ty && value.ty != Type::Never {
                    return Err(Self::error(
                        "E208",
                        format!(
                            "value of type {:?} is not proven to have type {ty:?}",
                            value.ty
                        ),
                        expr.span,
                    ));
                }
                return Ok(value);
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

    pub(crate) fn next_block(&mut self) -> usize {
        let id = self.block;
        self.block += 1;
        id
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
        let ty = match expected {
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

    pub(crate) fn hint(&self, expr: &ast::Expr) -> Option<Type> {
        match &expr.kind {
            ExprKind::Name(name) => match self.value(name, expr.span).ok()? {
                Value::Local { ty, .. } => Some(ty),
                Value::Constant(value) => Some(Self::constant_expr(value, expr.span).ty),
                _ => None,
            },
            ExprKind::Group(value) | ExprKind::Unary { value, .. } => self.hint(value),
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
                if let Some(Type::Record { fields, .. }) = self.hint(value) {
                    fields
                        .into_iter()
                        .find(|(field, _)| field == name)
                        .map(|(_, ty)| ty)
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
        let compare = ["==", "!=", "<", ">", "<=", ">="].contains(&op);
        let context = if boolean {
            Some(Type::Bool)
        } else {
            self.hint(left)
                .or_else(|| self.hint(right))
                .map(|ty| Self::primary_type(&ty))
                .or_else(|| {
                    if compare {
                        None
                    } else {
                        expected.map(Self::primary_type)
                    }
                })
        };
        let mut left = self.expression(left, context.as_ref())?;
        let before = self.paths.clone();
        let constant = self.constant(&left);
        let skipped = matches!(
            (&constant, op),
            (Some(Constant::Bool(false)), "&&") | (Some(Constant::Bool(true)), "||")
        );
        if skipped {
            self.paths.clear();
        }
        let right_context = Self::primary_type(&left.ty);
        let mut right = self.expression(right, Some(&right_context))?;
        if boolean && (skipped || !matches!(constant, Some(Constant::Bool(_)))) {
            self.paths.extend(before);
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
        if left.ty != right.ty {
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
            "&&" | "||" => left.ty == Type::Bool,
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
        if !self.paths.is_empty()
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
                (hir::ExprKind::Call { id, args: values }, result)
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
            _ => parts.push(self.expr(expr, None)?),
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
    pub(crate) fn path_enumeration_stops_at_the_bootstrap_budget() {
        let source = format!("flag:=true;{}", "|flag|x:1;".repeat(13));
        rejects(&source, "B001");
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
        rejects("x<int32><null>:1", "B001");
        rejects("x:[1,2]", "B001");
        rejects("x:1;f<int32>:(){->x}", "B001");
        rejects("x:1;text:\"{x}\"", "B001");
        rejects("f<int32>:(){->1};x:f==f", "E222");
    }
}
