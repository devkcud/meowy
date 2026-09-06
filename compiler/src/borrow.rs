use std::collections::{BTreeMap, BTreeSet};

use crate::ast::Span;
pub(crate) use crate::borrow_value::{Origin, Path, Projection, Source, State, Step};
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow as Guards, Guard, TRUE};
use crate::hir::{
    Block, BlockId, CallId, EmitId, Expr, ExprKind, LocalId, Program, ReborrowId, Stmt, Type,
};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Tags = BTreeMap<((LocalId, Vec<String>), Type), Vec<(Type, Guard)>>;
pub(crate) const MAX_ORIGINS: usize = 4_096;
pub(crate) const MAX_FACT_ORIGINS: usize = 262_144;

#[derive(Default)]
pub(crate) struct Proofs {
    pub(crate) completions: BTreeMap<BlockId, Guard>,
    pub(crate) emissions: BTreeMap<EmitId, Guard>,
    pub(crate) conditions: BTreeMap<(usize, usize), Guard>,
    pub(crate) bindings: BTreeMap<LocalId, Guard>,
    pub(crate) tags: Tags,
    pub(crate) mutable: BTreeSet<LocalId>,
    pub(crate) calls: BTreeMap<CallId, Guard>,
}

pub(crate) fn slot<'a>(ty: &'a Type, field: &Option<String>) -> Option<(Path, &'a Type)> {
    match (ty, field) {
        (Type::Record { primary, .. }, None) => Some((vec![Step::Slot(0)], primary)),
        (Type::Record { fields, .. }, Some(name)) => fields
            .iter()
            .enumerate()
            .find(|(_, (field, _))| field == name)
            .map(|(index, (_, ty))| (vec![Step::Slot(index + 1)], ty)),
        (ty, None) => Some((Vec::new(), ty)),
        _ => None,
    }
}

#[derive(Default)]
pub(crate) struct Facts {
    pub(crate) locals: BTreeMap<LocalId, State>,
    pub(crate) blocks: BTreeMap<BlockId, State>,
    pub(crate) calls: BTreeMap<CallId, State>,
    pub(crate) reborrows: BTreeMap<ReborrowId, State>,
}

#[derive(Clone)]
pub(crate) struct Storage {
    pub(crate) block: BlockId,
    pub(crate) state: State,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Exit {
    Leave(BlockId),
    Restart(BlockId),
}

pub(crate) struct Flow {
    pub(crate) next: bool,
    pub(crate) exits: BTreeSet<Exit>,
}

impl Flow {
    pub(crate) fn new() -> Self {
        Self {
            next: true,
            exits: BTreeSet::new(),
        }
    }

    pub(crate) fn append(&mut self, next: Self) {
        self.next = next.next;
        self.exits.extend(next.exits);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.next |= other.next;
        self.exits.extend(other.exits);
    }
}

pub(crate) struct Value {
    pub(crate) state: State,
    pub(crate) flow: Flow,
}

pub(crate) struct Checker<'a> {
    pub(crate) program: &'a Program,
    pub(crate) guards: &'a mut Guards,
    pub(crate) proofs: &'a Proofs,
    pub(crate) locals: BTreeMap<LocalId, Storage>,
    pub(crate) blocks: Vec<BlockId>,
    pub(crate) types: BTreeMap<BlockId, Type>,
    pub(crate) results: BTreeMap<BlockId, State>,
    pub(crate) writes: BTreeMap<(BlockId, Option<String>), Guard>,
    pub(crate) scopes: Vec<Vec<LocalId>>,
    pub(crate) facts: Facts,
    pub(crate) origins: usize,
    pub(crate) assumed: Guard,
    pub(crate) assumed_scopes: Vec<Guard>,
    pub(crate) inputs: BTreeSet<LocalId>,
}

pub(crate) fn check(
    program: &Program,
    guards: &mut Guards,
    proofs: &Proofs,
) -> std::result::Result<Facts, Vec<Diagnostic>> {
    let mut checker = Checker {
        program,
        guards,
        proofs,
        locals: BTreeMap::new(),
        blocks: Vec::new(),
        types: BTreeMap::new(),
        results: BTreeMap::new(),
        writes: BTreeMap::new(),
        scopes: Vec::new(),
        facts: Facts::default(),
        origins: 0,
        assumed: TRUE,
        assumed_scopes: Vec::new(),
        inputs: BTreeSet::new(),
    };
    let result = (|| {
        checker.block(&program.body)?;
        for function in &program.functions {
            checker.locals.clear();
            checker.assumed = TRUE;
            checker.inputs = function.params.iter().copied().collect();
            for id in &function.params {
                let span = Span { start: 0, end: 0 };
                let ty = &program.locals[*id];
                let mut state = crate::borrow_contract::input(*id, ty, checker.guards, span)?;
                checker.link_tags(*id, ty, &mut state, span)?;
                checker.complete(ty, &state, span)?;
                checker.reserve_origins(state.weight() + 1, span)?;
                checker.assumed = checker.guards.and(checker.assumed, state.proof);
                checker.facts.locals.insert(*id, state.clone());
                checker.locals.insert(
                    *id,
                    Storage {
                        block: function.body.id,
                        state,
                    },
                );
            }
            checker.block(&function.body)?;
        }
        Ok(())
    })();
    if checker.guards.exceeded() {
        Err(vec![Diagnostic::unsupported(
            "control-flow proof budget exhausted",
            Span { start: 0, end: 0 },
        )])
    } else {
        result.map(|()| checker.facts).map_err(|error| vec![error])
    }
}

impl Checker<'_> {
    pub(crate) fn unsupported(span: Span) -> Diagnostic {
        Diagnostic::unsupported("borrow origins outside immutable local storage", span)
    }

    pub(crate) fn close_scope(&mut self) {
        for id in self.scopes.pop().expect("storage scope") {
            self.locals.remove(&id);
        }
        self.assumed = self.assumed_scopes.pop().expect("assumption scope");
    }

    pub(crate) fn reserve_origins(&mut self, count: usize, span: Span) -> Result<()> {
        if self.origins + count > MAX_FACT_ORIGINS {
            return Err(Diagnostic::unsupported(
                "borrow-origin fact budget exhausted",
                span,
            ));
        }
        self.origins += count;
        Ok(())
    }

    pub(crate) fn assumptions(&mut self) -> Guard {
        self.assumed
    }

    pub(crate) fn bind(&mut self, id: LocalId, mut state: State, span: Span) -> Result<()> {
        let ty = self
            .program
            .locals
            .get(id)
            .ok_or_else(|| Self::unsupported(span))?
            .clone();
        if self.locals.contains_key(&id) {
            return Err(Self::unsupported(span));
        }
        self.complete(&ty, &state, span)?;
        if !self.proofs.mutable.contains(&id) {
            self.link_tags(id, &ty, &mut state, span)?;
        } else {
            state = State::unknown(&ty, self.guards, span)?;
        }
        self.reserve_origins(state.weight() + 1, span)?;
        self.assumed = self.guards.and(self.assumed, state.proof);
        self.locals.insert(
            id,
            Storage {
                block: *self.blocks.last().expect("storage block"),
                state: state.clone(),
            },
        );
        if state.size() != 0 || state.proof != TRUE {
            self.facts.locals.insert(id, state);
        }
        self.scopes.last_mut().expect("storage scope").push(id);
        Ok(())
    }

    pub(crate) fn live(&self, source: &Source, span: Span) -> Result<Option<&Storage>> {
        match source {
            Source::Local { id, .. } => self.locals.get(id).map(Some).ok_or_else(|| {
                Diagnostic::new("E303", "borrowed storage has ended before this use", span)
            }),
            Source::Input {
                id,
                component,
                fields,
            } if self.inputs.contains(id)
                && crate::borrow_contract::component_type(&self.program.locals[*id], component)
                    .is_some_and(|ty| match ty {
                        Type::Reference(ty) => {
                            crate::borrow_contract::projected_type(ty, fields).is_some()
                        }
                        _ => false,
                    }) =>
            {
                Ok(None)
            }
            _ => Err(Self::unsupported(span)),
        }
    }

    pub(crate) fn link_tags(
        &mut self,
        id: LocalId,
        ty: &Type,
        state: &mut State,
        span: Span,
    ) -> Result<()> {
        let entered = self
            .proofs
            .bindings
            .get(&id)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let present = self.guards.and(entered, state.present);
        let mut pending = vec![(ty, Path::new(), Vec::<String>::new(), present)];
        while let Some((ty, path, names, present)) = pending.pop() {
            if !self.guards.spend(path.len() + names.len() + 1) {
                return Err(State::budget(span));
            }
            match ty {
                Type::Record { primary, fields } => {
                    for (index, ty) in std::iter::once(primary.as_ref())
                        .chain(fields.iter().map(|(_, ty)| ty))
                        .enumerate()
                    {
                        let mut nested = path.clone();
                        nested.push(Step::Slot(index));
                        let mut names = names.clone();
                        if index != 0 {
                            names.push(fields[index - 1].0.clone());
                        }
                        pending.push((ty, nested, names, present));
                        if pending.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                }
                Type::Union(members) => {
                    let tags = self.proofs.tags.get(&((id, names.clone()), ty.clone()));
                    for (index, member) in members.iter().enumerate() {
                        let actual = state.member(&path, index, self.guards);
                        if let Some(tag) = tags.and_then(|tags| {
                            tags.iter()
                                .find(|(ty, _)| ty == member)
                                .map(|(_, guard)| *guard)
                        }) {
                            let both = self.guards.and(tag, actual);
                            let neither = self
                                .guards
                                .and(self.guards.not(tag), self.guards.not(actual));
                            let equal = self.guards.or(both, neither);
                            let relation = self.guards.or(self.guards.not(present), equal);
                            state.proof = self.guards.and(state.proof, relation);
                        }
                        let nested_guard = self.guards.and(present, actual);
                        let mut nested = path.clone();
                        nested.push(Step::Variant(index));
                        pending.push((member, nested, names.clone(), nested_guard));
                        if pending.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn complete(&mut self, ty: &Type, state: &State, span: Span) -> Result<()> {
        let assumptions = self.assumptions();
        let proof = self.guards.and(state.proof, assumptions);
        let present = self.guards.and(state.present, proof);
        if present == FALSE {
            return Ok(());
        }
        if !self.guards.spend(state.weight()) {
            return Err(State::budget(span));
        }
        let mut origins = BTreeMap::new();
        for origin in &state.origins {
            let prior = origins.get(&origin.component).copied().unwrap_or(FALSE);
            origins.insert(
                origin.component.clone(),
                self.guards.or(prior, origin.guard),
            );
        }
        let mut valid = BTreeSet::new();
        let mut unions = BTreeMap::new();
        let mut pending = vec![(ty, Path::new(), present)];
        while let Some((ty, path, present)) = pending.pop() {
            if !self.guards.spend(path.len() + 1) {
                return Err(State::budget(span));
            }
            match ty {
                Type::Reference(_) => {
                    valid.insert(path.clone());
                    if !self
                        .guards
                        .implies(present, origins.get(&path).copied().unwrap_or(FALSE))
                    {
                        return Err(Self::unsupported(span));
                    }
                }
                Type::Record { primary, fields } => {
                    for (index, ty) in std::iter::once(primary.as_ref())
                        .chain(fields.iter().map(|(_, ty)| ty))
                        .enumerate()
                    {
                        let mut path = path.clone();
                        path.push(Step::Slot(index));
                        pending.push((ty, path, present));
                        if pending.len() + valid.len() + unions.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                }
                Type::Union(members) => {
                    unions.insert(path.clone(), members.len());
                    let mut covered = FALSE;
                    for (index, ty) in members.iter().enumerate() {
                        let active = state.member(&path, index, self.guards);
                        let guard = self.guards.and(present, active);
                        if self.guards.overlap(guard, covered) {
                            return Err(Self::unsupported(span));
                        }
                        covered = self.guards.or(covered, active);
                        let mut nested = path.clone();
                        nested.push(Step::Variant(index));
                        pending.push((ty, nested, guard));
                        if pending.len() + valid.len() + unions.len() > MAX_ORIGINS {
                            return Err(State::budget(span));
                        }
                    }
                    if !self.guards.implies(present, covered) {
                        return Err(Self::unsupported(span));
                    }
                }
                _ => {}
            }
        }
        if state
            .origins
            .iter()
            .chain(&state.bounds)
            .any(|origin| !valid.contains(&origin.component))
            || state.active.iter().any(|active| {
                unions
                    .get(&active.component)
                    .is_none_or(|count| active.member >= *count)
            })
        {
            return Err(Self::unsupported(span));
        }
        Ok(())
    }

    pub(crate) fn emit(
        &mut self,
        id: EmitId,
        target: BlockId,
        field: &Option<String>,
        value: State,
        ty: &Type,
        span: Span,
    ) -> Result<()> {
        let written = self
            .proofs
            .emissions
            .get(&id)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let complete = self
            .proofs
            .completions
            .get(&target)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let retained = self.guards.and(written, complete);
        if retained == FALSE {
            return Ok(());
        }
        let target_type = self
            .types
            .get(&target)
            .ok_or_else(|| Self::unsupported(span))?
            .clone();
        let Some((prefix, destination)) = slot(&target_type, field) else {
            return Ok(());
        };
        if !destination.accepts(ty) {
            return Ok(());
        }
        let mut state = value
            .convert(ty, destination, self.guards, span)?
            .under(retained, self.guards);
        let assumptions = self.assumptions();
        state.proof = self.guards.and(state.proof, assumptions);
        self.complete(destination, &state, span)?;
        let key = (target, field.clone());
        let prior = self.writes.get(&key).copied().unwrap_or(FALSE);
        self.writes
            .insert(key, self.guards.or(prior, state.present));
        let target_index = self
            .blocks
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Self::unsupported(span))?;
        let effective = self.guards.and(state.present, state.proof);
        state.origins = self.retained(state.origins, effective, target_index, span)?;
        state.bounds = self.retained(state.bounds, effective, target_index, span)?;
        self.guards
            .spend(state.weight() + prefix.len() * state.size());
        self.results.get_mut(&target).expect("result state").merge(
            state.prefix(&prefix),
            self.guards,
            span,
        )
    }

    pub(crate) fn retained(
        &mut self,
        origins: Vec<Origin>,
        effective: Guard,
        target: usize,
        span: Span,
    ) -> Result<Vec<Origin>> {
        let mut result = Vec::new();
        for origin in origins {
            if !self.guards.spend(origin.weight()) {
                return Err(State::budget(span));
            }
            if !self.guards.overlap(origin.guard, effective) {
                continue;
            }
            if let Some(storage) = self.live(&origin.source, span)? {
                let source = self
                    .blocks
                    .iter()
                    .position(|id| *id == storage.block)
                    .ok_or_else(|| Self::unsupported(span))?;
                if source >= target {
                    return Err(Diagnostic::new(
                        "E303",
                        "emitted borrow outlives its local storage",
                        span,
                    ));
                }
            }
            result.push(origin);
        }
        Ok(result)
    }

    pub(crate) fn block(&mut self, block: &Block) -> Result<Value> {
        self.blocks.push(block.id);
        self.types.insert(block.id, block.ty.clone());
        self.results.insert(block.id, State::absent());
        self.scopes.push(Vec::new());
        self.assumed_scopes.push(self.assumed);
        let mut flow = self.statements(&block.stmts)?;
        let span = Span { start: 0, end: 0 };
        let complete = self
            .proofs
            .completions
            .get(&block.id)
            .copied()
            .ok_or_else(|| Self::unsupported(span))?;
        let mut state = self.results.remove(&block.id).expect("result state");
        let slots: Vec<_> = match &block.ty {
            Type::Record { primary, fields } => std::iter::once((None, primary.as_ref()))
                .chain(fields.iter().map(|(name, ty)| (Some(name.clone()), ty)))
                .collect(),
            ty => vec![(None, ty)],
        };
        for (field, ty) in slots {
            let written = self
                .writes
                .remove(&(block.id, field.clone()))
                .unwrap_or(FALSE);
            let missing = self.guards.and(complete, self.guards.not(written));
            if missing != FALSE && ty.accepts(&Type::Null) {
                let prefix = slot(&block.ty, &field).expect("result slot").0;
                let default = State::default()
                    .convert(&Type::Null, ty, self.guards, span)?
                    .under(missing, self.guards)
                    .prefix(&prefix);
                state.merge(default, self.guards, span)?;
            }
        }
        state.present = complete;
        let assumptions = self.assumptions();
        state.proof = self.guards.and(state.proof, assumptions);
        self.complete(&block.ty, &state, span)?;
        if state.size() != 0 || state.proof != TRUE {
            self.reserve_origins(state.weight() + 1, span)?;
            self.facts.blocks.insert(block.id, state.clone());
        }
        self.close_scope();
        self.blocks.pop();
        self.types.remove(&block.id);
        let effective = self.guards.and(state.present, state.proof);
        flow.next = effective != FALSE;
        flow.exits.remove(&Exit::Leave(block.id));
        flow.exits.remove(&Exit::Restart(block.id));
        Ok(Value { state, flow })
    }

    pub(crate) fn branch(&mut self, stmts: &[Stmt]) -> Result<Flow> {
        self.scopes.push(Vec::new());
        self.assumed_scopes.push(self.assumed);
        let flow = self.statements(stmts)?;
        self.close_scope();
        Ok(flow)
    }

    pub(crate) fn statements(&mut self, stmts: &[Stmt]) -> Result<Flow> {
        let mut flow = Flow::new();
        for stmt in stmts {
            if !flow.next {
                break;
            }
            let next = match stmt {
                Stmt::Bind { id, value } => {
                    let result = self.expression(value)?;
                    if result.flow.next {
                        self.bind(*id, result.state, value.span)?;
                    }
                    result.flow
                }
                Stmt::Assign { id, value } => {
                    let result = self.expression(value)?;
                    if result.flow.next
                        && (!result.state.origins.is_empty()
                            || self.program.locals[*id].has_reference())
                    {
                        return Err(Self::unsupported(value.span));
                    }
                    result.flow
                }
                Stmt::Emit {
                    id,
                    target,
                    field,
                    value,
                } => {
                    let result = self.expression(value)?;
                    if result.flow.next {
                        self.emit(*id, *target, field, result.state, &value.ty, value.span)?;
                    }
                    result.flow
                }
                Stmt::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    let mut result = self.expression(condition)?.flow;
                    if result.next {
                        let next = match &condition.kind {
                            ExprKind::Bool(true) => self.branch(then)?,
                            ExprKind::Bool(false) => self.branch(otherwise)?,
                            _ => {
                                let mut next = self.branch(then)?;
                                next.merge(self.branch(otherwise)?);
                                next
                            }
                        };
                        result.append(next);
                    }
                    result
                }
                Stmt::Leave(id) | Stmt::Restart(id) => Flow {
                    next: false,
                    exits: BTreeSet::from([if matches!(stmt, Stmt::Leave(_)) {
                        Exit::Leave(*id)
                    } else {
                        Exit::Restart(*id)
                    }]),
                },
                Stmt::Expr(value) => self.expression(value)?.flow,
            };
            flow.append(next);
        }
        Ok(flow)
    }

    pub(crate) fn expression(&mut self, expr: &Expr) -> Result<Value> {
        let mut flow = Flow::new();
        let state = match &expr.kind {
            ExprKind::Borrow(place) => {
                if !self.locals.contains_key(&place.root) {
                    return Err(Self::unsupported(expr.span));
                }
                let mut ty = self
                    .program
                    .locals
                    .get(place.root)
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                for index in &place.fields {
                    let Type::Record { fields, .. } = ty else {
                        return Err(Self::unsupported(expr.span));
                    };
                    ty = &fields
                        .get(*index)
                        .ok_or_else(|| Self::unsupported(expr.span))?
                        .1;
                }
                if ty.has_reference() || expr.ty != Type::Reference(Box::new(ty.clone())) {
                    return Err(Self::unsupported(expr.span));
                }
                State {
                    origins: vec![Origin {
                        component: Vec::new(),
                        source: Source::local(place),
                        guard: TRUE,
                    }],
                    ..State::default()
                }
            }
            ExprKind::Reborrow { site, value, .. }
            | ExprKind::ElementBorrow { site, value, .. } => {
                let result = self.expression(value)?;
                flow = result.flow;
                let fields = match &expr.kind {
                    ExprKind::Reborrow { fields, .. } => {
                        if !self.guards.spend(fields.len() + 1) {
                            return Err(State::budget(expr.span));
                        }
                        fields
                            .iter()
                            .copied()
                            .map(Projection::Field)
                            .collect::<Vec<_>>()
                    }
                    ExprKind::ElementBorrow { index, .. } => {
                        if flow.next {
                            flow.append(self.expression(index)?.flow);
                        }
                        vec![Projection::Element]
                    }
                    _ => unreachable!(),
                };
                let Type::Reference(ty) = &value.ty else {
                    return Err(Self::unsupported(expr.span));
                };
                let target = crate::borrow_contract::projected_type(ty, &fields)
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                if ty.has_reference()
                    || (expr.ty != Type::Never
                        && expr.ty != Type::Reference(Box::new(target.clone())))
                {
                    return Err(Self::unsupported(expr.span));
                }
                let mut state = result.state;
                if !self
                    .guards
                    .spend(state.weight() + fields.len().saturating_mul(state.origins.len()))
                {
                    return Err(State::budget(expr.span));
                }
                for origin in &mut state.origins {
                    origin.source = origin.source.project(&fields);
                }
                self.reserve_origins(state.weight() + 1, expr.span)?;
                self.facts.reborrows.insert(*site, state.clone());
                state
            }
            ExprKind::Local(id) => {
                if self.proofs.mutable.contains(id) {
                    State::unknown(&expr.ty, self.guards, expr.span)?
                } else {
                    let state = self
                        .locals
                        .get(id)
                        .map(|storage| storage.state.clone())
                        .ok_or_else(|| Self::unsupported(expr.span))?;
                    let assumptions = self.assumptions();
                    let proof = self.guards.and(state.proof, assumptions);
                    for origin in state.origins.iter().chain(&state.bounds) {
                        if self.guards.overlap(origin.guard, proof) {
                            self.live(&origin.source, expr.span)?;
                        }
                    }
                    state
                }
            }
            ExprKind::Coerce { value } => {
                let result = self.expression(value)?;
                flow = result.flow;
                result
                    .state
                    .convert(&value.ty, &expr.ty, self.guards, expr.span)?
            }
            ExprKind::Deref(value) => {
                let result = self.expression(value)?;
                flow = result.flow;
                if flow.next {
                    if result.state.origins.is_empty() {
                        return Err(Self::unsupported(expr.span));
                    }
                    let proof = self.guards.and(result.state.present, result.state.proof);
                    for origin in result.state.origins.iter().chain(&result.state.bounds) {
                        if self.guards.overlap(origin.guard, proof) {
                            self.live(&origin.source, expr.span)?;
                        }
                    }
                }
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::Field { value, index } => {
                let value = self.expression(value)?;
                flow = value.flow;
                value.state.select(&[Step::Slot(index + 1)], self.guards)
            }
            ExprKind::Primary(value) => {
                let value = self.expression(value)?;
                flow = value.flow;
                value.state.select(&[Step::Slot(0)], self.guards)
            }
            ExprKind::Unary { value, .. }
            | ExprKind::StringSize(value)
            | ExprKind::ListSize(value)
            | ExprKind::TypeTest { value, .. } => {
                flow = self.expression(value)?.flow;
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::List { values, .. } => {
                for value in values {
                    if !flow.next {
                        break;
                    }
                    flow.append(self.expression(value)?.flow);
                }
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::ListIndex { value, index } | ExprKind::ListAdd { value, item: index } => {
                flow = self.expression(value)?.flow;
                if flow.next {
                    flow.append(self.expression(index)?.flow);
                }
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::Binary { op, left, right } => {
                flow = self.expression(left)?.flow;
                if flow.next {
                    let skip = match (&left.kind, op.as_str()) {
                        (ExprKind::Bool(value), "&&") => Some(!value),
                        (ExprKind::Bool(value), "||") => Some(*value),
                        _ => None,
                    };
                    if skip != Some(true) {
                        let mut next = self.expression(right)?.flow;
                        if skip.is_none() && ["&&", "||"].contains(&op.as_str()) {
                            next.next = true;
                        }
                        flow.append(next);
                    }
                }
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::Call { site, args, .. } => {
                let mut inputs = Vec::new();
                for arg in args {
                    if !flow.next {
                        break;
                    }
                    let value = self.expression(arg)?;
                    inputs.push((&arg.ty, value.state));
                    flow.append(value.flow);
                }
                let state = if flow.next {
                    crate::borrow_contract::call(&expr.ty, &inputs, self.guards, expr.span)?
                } else {
                    State::absent()
                };
                let entered = self
                    .proofs
                    .calls
                    .get(site)
                    .copied()
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                let normal = self.guards.and(state.present, state.proof);
                let post = self.guards.or(self.guards.not(entered), normal);
                self.assumed = self.guards.and(self.assumed, post);
                self.reserve_origins(state.weight() + 1, expr.span)?;
                self.facts.calls.insert(*site, state.clone());
                state
            }
            ExprKind::Print { parts, .. } | ExprKind::Panic { parts } => {
                for part in parts {
                    if !flow.next {
                        break;
                    }
                    flow.append(self.expression(part)?.flow);
                }
                if matches!(expr.kind, ExprKind::Panic { .. }) {
                    flow.next = false;
                }
                State::default()
            }
            ExprKind::Block(block) => {
                let value = self.block(block)?;
                flow = value.flow;
                value.state
            }
            ExprKind::Null
            | ExprKind::Bool(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_) => State::default(),
        };
        let assumptions = self.assumptions();
        let proof = self.guards.and(state.proof, assumptions);
        if self.guards.and(state.present, proof) == FALSE {
            flow.next = false;
        }
        if flow.next {
            self.complete(&expr.ty, &state, expr.span)?;
        }
        if expr.ty == Type::Never {
            flow.next = false;
        }
        Ok(Value { state, flow })
    }
}

#[cfg(test)]
mod tests {
    pub(crate) fn accepts(source: &str) {
        let result = crate::compile(source);
        assert!(result.is_ok(), "{source}: {result:?}");
    }

    pub(crate) fn rejects(source: &str, code: &str) {
        let errors = crate::compile(source).unwrap_err();
        assert_eq!(errors[0].code, code, "{source}: {errors:?}");
    }

    #[test]
    pub(crate) fn aliases_preserve_origins_through_nested_block_results() {
        accepts("a:1;view:&a;{copy:view;value:*copy};value:*view");
        accepts("a:1;view:{alias:{->&a};->alias};same:view==&a;value:*view");
        accepts("a:{->x:1;->nested:{->y:2}};view:{->{->&a.nested.y}};value:*view");
        accepts("f<int32>:(){a:9;view:{->&a};->*view};value:f()");
    }

    #[test]
    pub(crate) fn complementary_conditions_preserve_every_origin() {
        accepts("f<int32>:(flag<boolean>){a:11;b:22;r:{|flag|->&a;|!flag|->&b};->*r}");
        accepts(
            "f<int32>:(flag<boolean>){a:11;b:22;r:'result {|flag|{'result->&a;'result.leave()};->&b};->*r}",
        );
        rejects(
            "f<int32>:(flag<boolean>){a:11;r:'result {|flag|{b:22;'result->&b;'result.leave()};->&a};->*r}",
            "E303",
        );
        rejects(
            "a:11;flag:=false;r:'outer {b:22;view:{|flag|->&a;|!flag|->&b};->view}",
            "E303",
        );
        rejects(
            "a:11;flag:=false;r:'outer {b:22;view:{|flag|->&b;|!flag|->&a};->view}",
            "E303",
        );
    }

    #[test]
    pub(crate) fn named_outer_emissions_preserve_origin_and_leave_target() {
        accepts("a:1;r:'result {'inner {'result->&a;'result.leave()}};v:*r");
        rejects(
            "r:'result {'inner {a:1;'result->&a;'result.leave()}}",
            "E303",
        );
        rejects(
            "a:1;r:'outer {'inner {'outer->&a;'inner.restart()}}",
            "B001",
        );
    }

    #[test]
    pub(crate) fn completing_local_results_never_extend_storage_lifetimes() {
        for source in [
            "value:{a:1;->&a}",
            "make:(){a:1;view:&a;alias:view;->alias}",
            "a:1;value:{a:2;view:{->&a};->view}",
            "a:1;r:'scope {b:2;|true|->&b}",
            "a:1;r:'scope {b:2;flag:true;|flag|->&b}",
            "condition:(x<boolean>){->x};|condition(false)|{x:1;->&x}",
        ] {
            rejects(source, "E303");
        }
    }

    #[test]
    pub(crate) fn discarded_reference_emissions_do_not_escape() {
        for source in [
            "'loop {x:1;->&x;'loop.restart()}",
            "debug:@\"debug\";{x:1;->&x;debug.panic(\"stop\")}",
            "'outer {{x:1;->&x;'outer.leave()}}",
            "flag:false;'scope {x:1;|flag|->&x}",
            "'scope {x:1;|false|->&x}",
            "owner:42;again:=true;r:'result {|again|{local:1;'result->&local;again=false;'result.restart()};->&owner};value:*r",
            "owner:42;again:=true;r:'result {|again|{local:1;'result->&local;again=false;'result.restart()};'nested {'result->&owner;'result.leave()}};value:*r",
            "debug:@\"debug\";f<int32>:(flag<boolean>){a:1;r:'result {|flag|{b:2;'result->&b;debug.panic(\"stop\")};->&a};->*r}",
        ] {
            accepts(source);
        }
    }

    #[test]
    pub(crate) fn scoped_transfers_skip_unreachable_reference_uses() {
        accepts("'outer {{'outer.leave()};x:1;->&x}");
        accepts("'outer {{'outer.restart()};x:1;->&x}");
        accepts("debug:@\"debug\";'scope {debug.panic(\"stop\");x:1;->&x}");
        accepts("a:1;r:'scope {|true|{'scope->&a;'scope.leave()};x:2;->&x};value:*r");
    }

    #[test]
    pub(crate) fn reference_mutation_and_exclusive_contracts_remain_explicit() {
        for source in [
            "a:1;r:={->field:&a}",
            "a:1;flag:=true;r:={|flag|->&a}",
            "a:1;r:=&a",
            "a:=1;r:&!a",
            "r:&(1+2)",
            "a:1;r:{->&a};s:&r",
            "f<int32>:(r<&!int32>){->*r}",
        ] {
            rejects(source, "B001");
        }
    }

    #[test]
    pub(crate) fn separate_function_frames_keep_local_borrows_independent() {
        accepts("f<int32>:(){a:1;r:{->&a};->*r};g<int32>:(){a:2;r:{->&a};->*r};x:f()+g()");
        rejects(
            "f<int32>:(){a:1;r:{->&a};->*r};g:(){a:2;r:{->&a};->r}",
            "E303",
        );
    }

    #[test]
    pub(crate) fn record_origins_preserve_primary_named_and_nested_components() {
        accepts(
            "a:1;b:2;r<{left<&int32>;right<&int32>}>:{->left:&a;->right:&b};copy:r;x:*copy.left;y:*copy.right",
        );
        accepts("a:1;b:2;r:{->&a;->ref:&b};copy:{->r};view<&int32>:copy;x:*view;y:*copy.ref");
        accepts("a:1;r:{->outer:{->inner:&a}};copy:r.outer;view:copy.inner;x:*view");
        accepts("a:1;r:{b:2;pair:{->safe:&a;->bad:&b};copy:pair;->copy.safe};x:*r");
        rejects("a:1;r:{b:2;pair:{->safe:&a;->bad:&b};->pair}", "E303");
        rejects("r:{a:1;->outer:{->inner:&a}}", "E303");
    }

    #[test]
    pub(crate) fn each_record_component_retains_its_own_guarded_origins() {
        accepts(
            "f<int32>:(flag<boolean>){a:1;b:2;r:{|flag|->left:&a;|!flag|->left:&b;->right:&a};->*r.left+*r.right}",
        );
        rejects(
            "a:1;flag:=true;r:'out {b:2;|flag|'out->field:&a;|!flag|'out->field:&b}",
            "E303",
        );
        accepts(
            "a:1;again:=true;r:'out {|again|{b:2;'out->field:&b;again=false;'out.restart()};->field:&a};x:*r.field",
        );
        accepts(
            "a:1;d:@\"debug\";f<int32>:(flag<boolean>){b:2;r:'out {|flag|{c:3;'out->nested:{->view:&c};d.panic(\"stop\")};->nested:{->view:&b}};->*r.nested.view}",
        );
    }

    #[test]
    pub(crate) fn carrier_borrows_and_mutation_remain_explicit_boundaries() {
        for source in [
            "a:1;r<{view<&int32>}><null>:=null",
            "a:1;flag:=true;r:={|flag|->view:&a}",
            "a:1;r:{->view:&a};alias:&r",
            "a:1;r:{->view:&a};alias:&r.view",
            "a:1;r:={->view:&a}",
            "<R>:<{view<&int32>}>;f<null>:(r<&R>){x:r}",
            "a:1;r:{->&a;->count:3};d:@\"debug\";d.print(r)",
        ] {
            rejects(source, "B001");
        }
    }

    #[test]
    pub(crate) fn nullable_references_preserve_active_and_absent_origins() {
        accepts("a:1;u<&int32><null>:&a;copy:u;|copy<&int32>|{r<&int32>:copy;x:*r}");
        accepts("u<&int32><null>:null;|u<&int32>|{r<&int32>:u;x:*r}");
        accepts("a:1;flag:=true;u<&int32><null>:{|flag|->&a};|u<&int32>|x:*u<&int32>");
        accepts(
            "a:1;r<{view<&int32><null>;count<int32>}>:{->count:1};|r.view<&int32>|x:*r.view<&int32>",
        );
        rejects("u<&int32><null>:{a:1;->&a}", "E303");
        rejects("flag:=true;u:{a:1;|flag|->view:&a}", "E303");
    }

    #[test]
    pub(crate) fn retagging_maps_reference_origins_by_member_type() {
        accepts(
            "a:1;u<&int32><null>:&a;wide<boolean><&int32><null>:u;|wide<&int32>|{r<&int32>:wide;x:*r}",
        );
        accepts(
            "a:1;b<int64>:2;flag:=true;u<&int32><&int64><null>:{|flag|->&a;|!flag|->&b};|u<&int32>|x:*u<&int32>;|u<&int64>|y:*u<&int64>",
        );
        accepts(
            "a:1;flag:=true;u<&int32><null>:{inner<&int32><null>:{|flag|->&a};|inner<&int32>|->inner<&int32>};copy:u;|copy<&int32>|x:*copy<&int32>",
        );
    }

    #[test]
    pub(crate) fn nested_union_fields_preserve_variant_specific_activity() {
        accepts(
            "<A>:<{kind<boolean>;view<&int32><null>}>;<B>:<{kind<int32>;view<&int32><null>}>;f<null>:(flag<boolean>){a:1;left<A>:{->kind:true;->view:&a};right<B>:{->kind:2};u<A><B>:{|flag|->left;|!flag|->right};|u<A>|{|u.view<&int32>|x:*u.view<&int32>};|u<B>|{|u.view<null>|{}}}",
        );
        rejects(
            "<A>:<{view<&int32>}>;u<A><null>:{a:1;r<A>:{->view:&a};->r}",
            "E303",
        );
        accepts(
            "a:1;again:=true;u<{view<&int32><null>}>:'out {|again|{b:2;'out->view:&b;again=false;'out.restart()}};|u.view<&int32>|x:*u.view<&int32>",
        );
    }
}
