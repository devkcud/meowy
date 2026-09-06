use std::collections::{BTreeMap, BTreeSet};

use crate::ast::Span;
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow as Guards, Guard, TRUE};
use crate::hir::{Block, BlockId, EmitId, Expr, ExprKind, LocalId, Place, Program, Stmt, Type};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) const MAX_ORIGINS: usize = 4_096;
pub(crate) const MAX_FACT_ORIGINS: usize = 262_144;

#[derive(Default)]
pub(crate) struct Proofs {
    pub(crate) completions: BTreeMap<BlockId, Guard>,
    pub(crate) emissions: BTreeMap<EmitId, Guard>,
    pub(crate) conditions: BTreeMap<(usize, usize), Guard>,
}

#[derive(Clone)]
pub(crate) struct Origin {
    pub(crate) place: Place,
    pub(crate) guard: Guard,
}

#[derive(Default)]
pub(crate) struct Facts {
    pub(crate) locals: BTreeMap<LocalId, Vec<Origin>>,
    pub(crate) blocks: BTreeMap<BlockId, Vec<Origin>>,
}

#[derive(Clone)]
pub(crate) struct Storage {
    pub(crate) block: BlockId,
    pub(crate) origins: Vec<Origin>,
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
    pub(crate) origins: Vec<Origin>,
    pub(crate) flow: Flow,
}

pub(crate) struct Checker<'a> {
    pub(crate) program: &'a Program,
    pub(crate) guards: &'a mut Guards,
    pub(crate) proofs: &'a Proofs,
    pub(crate) locals: BTreeMap<LocalId, Storage>,
    pub(crate) blocks: Vec<BlockId>,
    pub(crate) types: BTreeMap<BlockId, Type>,
    pub(crate) results: BTreeMap<BlockId, Vec<Origin>>,
    pub(crate) scopes: Vec<Vec<LocalId>>,
    pub(crate) facts: Facts,
    pub(crate) origins: usize,
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
        scopes: Vec::new(),
        facts: Facts::default(),
        origins: 0,
    };
    let result = (|| {
        checker.block(&program.body)?;
        for function in &program.functions {
            checker.locals.clear();
            for id in &function.params {
                checker.locals.insert(
                    *id,
                    Storage {
                        block: function.body.id,
                        origins: Vec::new(),
                    },
                );
            }
            checker.block(&function.body)?;
            if function.result.has_reference()
                || function
                    .params
                    .iter()
                    .any(|id| program.locals[*id].has_reference())
            {
                return Err(Diagnostic::unsupported(
                    "function borrow contracts",
                    Span { start: 0, end: 0 },
                ));
            }
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

    pub(crate) fn bind(&mut self, id: LocalId, origins: Vec<Origin>, span: Span) -> Result<()> {
        let ty = self
            .program
            .locals
            .get(id)
            .ok_or_else(|| Self::unsupported(span))?;
        if self.locals.contains_key(&id)
            || (ty.has_reference() && (!matches!(ty, Type::Reference(_)) || origins.is_empty()))
        {
            return Err(Self::unsupported(span));
        }
        self.reserve_origins(origins.len(), span)?;
        self.locals.insert(
            id,
            Storage {
                block: *self.blocks.last().expect("storage block"),
                origins: origins.clone(),
            },
        );
        if !origins.is_empty() {
            self.facts.locals.insert(id, origins);
        }
        self.scopes.last_mut().expect("storage scope").push(id);
        Ok(())
    }

    pub(crate) fn live(&self, place: &Place, span: Span) -> Result<&Storage> {
        self.locals.get(&place.root).ok_or_else(|| {
            Diagnostic::new("E303", "borrowed storage has ended before this use", span)
        })
    }

    pub(crate) fn emit(
        &mut self,
        id: EmitId,
        target: BlockId,
        field: &Option<String>,
        origins: Vec<Origin>,
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
        if field.is_some()
            || !matches!(self.types.get(&target), Some(Type::Reference(_)))
            || origins.is_empty()
        {
            return Err(Self::unsupported(span));
        }
        let target_index = self
            .blocks
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Self::unsupported(span))?;
        let mut covered = FALSE;
        for mut origin in origins {
            origin.guard = self.guards.and(origin.guard, retained);
            if origin.guard == FALSE {
                continue;
            }
            covered = self.guards.or(covered, origin.guard);
            let storage = self.live(&origin.place, span)?;
            let source = self
                .blocks
                .iter()
                .position(|id| *id == storage.block)
                .ok_or_else(|| Self::unsupported(span))?;
            if source >= target_index {
                return Err(Diagnostic::new(
                    "E303",
                    "emitted borrow outlives its local storage",
                    span,
                ));
            }
            let result = self.results.get_mut(&target).expect("result origins");
            if let Some(existing) = result.iter_mut().find(|item| item.place == origin.place) {
                existing.guard = self.guards.or(existing.guard, origin.guard);
            } else if result.len() == MAX_ORIGINS {
                return Err(Diagnostic::unsupported(
                    "borrow-origin budget exhausted",
                    span,
                ));
            } else {
                result.push(origin);
            }
        }
        if !self.guards.implies(retained, covered) {
            return Err(Self::unsupported(span));
        }
        Ok(())
    }

    pub(crate) fn block(&mut self, block: &Block) -> Result<Value> {
        self.blocks.push(block.id);
        self.types.insert(block.id, block.ty.clone());
        self.results.insert(block.id, Vec::new());
        self.scopes.push(Vec::new());
        let mut flow = self.statements(&block.stmts)?;
        let complete = self
            .proofs
            .completions
            .get(&block.id)
            .copied()
            .ok_or_else(|| Self::unsupported(Span { start: 0, end: 0 }))?;
        let origins = self.results.remove(&block.id).expect("result origins");
        if !origins.is_empty() {
            self.reserve_origins(origins.len(), Span { start: 0, end: 0 })?;
            self.facts.blocks.insert(block.id, origins.clone());
        }
        self.close_scope();
        self.blocks.pop();
        self.types.remove(&block.id);
        flow.next = complete != FALSE;
        flow.exits.remove(&Exit::Leave(block.id));
        flow.exits.remove(&Exit::Restart(block.id));
        Ok(Value { origins, flow })
    }

    pub(crate) fn branch(&mut self, stmts: &[Stmt]) -> Result<Flow> {
        self.scopes.push(Vec::new());
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
                        self.bind(*id, result.origins, value.span)?;
                    }
                    result.flow
                }
                Stmt::Assign { id, value } => {
                    let result = self.expression(value)?;
                    if result.flow.next
                        && (!result.origins.is_empty() || self.program.locals[*id].has_reference())
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
                    if result.flow.next && value.ty.has_reference() {
                        self.emit(*id, *target, field, result.origins, value.span)?;
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
        let origins = match &expr.kind {
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
                vec![Origin {
                    place: place.clone(),
                    guard: TRUE,
                }]
            }
            ExprKind::Local(id) if expr.ty.has_reference() => {
                let origins = self
                    .locals
                    .get(id)
                    .map(|storage| storage.origins.clone())
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                for origin in &origins {
                    self.live(&origin.place, expr.span)?;
                }
                origins
            }
            ExprKind::Coerce { value } => {
                let value = self.expression(value)?;
                flow = value.flow;
                value.origins
            }
            ExprKind::Deref(value) => {
                let value = self.expression(value)?;
                flow = value.flow;
                if flow.next {
                    if value.origins.is_empty() {
                        return Err(Self::unsupported(expr.span));
                    }
                    for origin in &value.origins {
                        self.live(&origin.place, expr.span)?;
                    }
                }
                Vec::new()
            }
            ExprKind::Unary { value, .. }
            | ExprKind::Field { value, .. }
            | ExprKind::Primary(value)
            | ExprKind::StringSize(value)
            | ExprKind::TypeTest { value, .. } => {
                flow = self.expression(value)?.flow;
                Vec::new()
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
                Vec::new()
            }
            ExprKind::Call { args, .. } => {
                for arg in args {
                    if !flow.next {
                        break;
                    }
                    let value = self.expression(arg)?;
                    if value.flow.next && arg.ty.has_reference() {
                        return Err(Diagnostic::unsupported(
                            "function borrow contracts",
                            arg.span,
                        ));
                    }
                    flow.append(value.flow);
                }
                Vec::new()
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
                Vec::new()
            }
            ExprKind::Block(block) => {
                let value = self.block(block)?;
                flow = value.flow;
                value.origins
            }
            ExprKind::Null
            | ExprKind::Bool(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Local(_) => Vec::new(),
        };
        if flow.next
            && expr.ty.has_reference()
            && (!matches!(expr.ty, Type::Reference(_)) || origins.is_empty())
        {
            return Err(Self::unsupported(expr.span));
        }
        if expr.ty == Type::Never {
            flow.next = false;
        }
        Ok(Value { origins, flow })
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
    pub(crate) fn reference_aggregates_mutation_and_function_contracts_remain_explicit() {
        for source in [
            "a:1;r:{->field:&a}",
            "a:1;flag:=true;r:{|flag|->&a}",
            "a:1;r:=&a",
            "a:=1;r:&!a",
            "r:&(1+2)",
            "a:1;r:{->&a};s:&r",
            "f<int32>:(r<&int32>){->*r}",
            "f<&int32>:(){a:1;->&a}",
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
}
