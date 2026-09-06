use std::collections::{BTreeMap, VecDeque};

use crate::ast::Span;
use crate::borrow::{Facts, Origin, Path, Proofs, Source, Step};
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow, Guard, TRUE};
use crate::hir::{Block, BlockId, CallId, Expr, ExprKind, LocalId, Place, Program, Stmt, Type};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Live = BTreeMap<usize, Guard>;
pub(crate) type Bundle = BTreeMap<Path, usize>;
pub(crate) const MAX_NODES: usize = 65_536;
pub(crate) const MAX_VALUES: usize = 65_536;
pub(crate) const MAX_WORK: usize = 1_048_576;
pub(crate) const MAX_LIVE: usize = 262_144;
pub(crate) const MAX_ORIGINS: usize = 262_144;

#[derive(Clone)]
pub(crate) struct Edge {
    pub(crate) target: usize,
    pub(crate) guard: Guard,
    pub(crate) reset: bool,
}

#[derive(Default)]
pub(crate) struct Node {
    pub(crate) uses: Vec<usize>,
    pub(crate) defs: Vec<usize>,
    pub(crate) write: Option<(Place, Span)>,
    pub(crate) next: Vec<Edge>,
}

pub(crate) struct Scope {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) result: Bundle,
    pub(crate) ty: Type,
}

pub(crate) struct Graph<'a> {
    pub(crate) program: &'a Program,
    pub(crate) facts: &'a Facts,
    pub(crate) proofs: &'a Proofs,
    pub(crate) guards: &'a mut Flow,
    pub(crate) nodes: Vec<Node>,
    pub(crate) values: Vec<Vec<Origin>>,
    pub(crate) locals: BTreeMap<LocalId, Bundle>,
    pub(crate) blocks: BTreeMap<BlockId, Scope>,
    pub(crate) current: Vec<usize>,
    pub(crate) work: usize,
    pub(crate) origins: usize,
    pub(crate) missing_calls: Vec<(usize, Span)>,
}

pub(crate) fn check(
    program: &Program,
    facts: &Facts,
    proofs: &Proofs,
    guards: &mut Flow,
) -> std::result::Result<(), Vec<Diagnostic>> {
    let result = (|| {
        Graph::new(program, facts, proofs, guards).check(&program.body, &[])?;
        for function in &program.functions {
            Graph::new(program, facts, proofs, guards).check(&function.body, &function.params)?;
        }
        Ok(())
    })();
    if guards.exceeded() {
        Err(vec![Diagnostic::unsupported(
            "control-flow proof budget exhausted",
            Span { start: 0, end: 0 },
        )])
    } else {
        result.map_err(|error| vec![error])
    }
}

impl<'a> Graph<'a> {
    pub(crate) fn new(
        program: &'a Program,
        facts: &'a Facts,
        proofs: &'a Proofs,
        guards: &'a mut Flow,
    ) -> Self {
        Self {
            program,
            facts,
            proofs,
            guards,
            nodes: vec![Node::default()],
            values: Vec::new(),
            locals: BTreeMap::new(),
            blocks: BTreeMap::new(),
            current: vec![0],
            work: 0,
            origins: 0,
            missing_calls: Vec::new(),
        }
    }

    pub(crate) fn budget() -> Diagnostic {
        Diagnostic::unsupported("loan-analysis budget exhausted", Span { start: 0, end: 0 })
    }

    pub(crate) fn tick(&mut self) -> Result<()> {
        self.charge(1)
    }

    pub(crate) fn charge(&mut self, work: usize) -> Result<()> {
        self.work = self.work.saturating_add(work);
        if self.work > MAX_WORK || self.guards.exceeded() {
            Err(Self::budget())
        } else {
            Ok(())
        }
    }

    pub(crate) fn node(&mut self, node: Node) -> Result<usize> {
        if self.nodes.len() == MAX_NODES {
            return Err(Self::budget());
        }
        let id = self.nodes.len();
        self.nodes.push(node);
        Ok(id)
    }

    pub(crate) fn connect(&mut self, target: usize, guard: Guard, reset: bool) {
        for id in std::mem::take(&mut self.current) {
            self.nodes[id].next.push(Edge {
                target,
                guard,
                reset,
            });
        }
    }

    pub(crate) fn append(&mut self, node: Node) -> Result<usize> {
        let id = self.node(node)?;
        self.connect(id, TRUE, false);
        self.current.push(id);
        Ok(id)
    }

    pub(crate) fn assume(&mut self, proof: Guard) -> Result<()> {
        if proof != TRUE && !self.current.is_empty() {
            let id = self.node(Node::default())?;
            self.connect(id, proof, false);
            self.current.push(id);
        }
        Ok(())
    }

    pub(crate) fn value(&mut self, origins: Vec<Origin>) -> Result<usize> {
        let weight = origins.iter().map(Origin::weight).sum::<usize>();
        if self.values.len() == MAX_VALUES || self.origins + weight > MAX_ORIGINS {
            return Err(Self::budget());
        }
        self.origins += weight;
        let id = self.values.len();
        self.values.push(origins);
        Ok(id)
    }

    pub(crate) fn bundle(&mut self, origins: Vec<Origin>) -> Result<Bundle> {
        let mut groups = BTreeMap::<Path, Vec<Origin>>::new();
        for origin in origins {
            groups
                .entry(origin.component.clone())
                .or_default()
                .push(origin);
        }
        groups
            .into_iter()
            .map(|(path, origins)| Ok((path, self.value(origins)?)))
            .collect()
    }

    pub(crate) fn select(value: Bundle, path: &[Step]) -> Bundle {
        value
            .into_iter()
            .filter_map(|(key, id)| key.strip_prefix(path).map(|path| (path.to_vec(), id)))
            .collect()
    }

    pub(crate) fn copy(&mut self, source: Bundle) -> Result<Bundle> {
        let mut value = Bundle::new();
        for (path, id) in &source {
            let weight = self.values[*id].iter().map(Origin::weight).sum::<usize>();
            self.charge(weight + path.len() + 1)?;
            value.insert(path.clone(), self.value(self.values[*id].clone())?);
        }
        self.append(Node {
            uses: source.into_values().collect(),
            defs: value.values().copied().collect(),
            ..Node::default()
        })?;
        Ok(value)
    }

    pub(crate) fn local(&mut self, id: LocalId) -> Result<Bundle> {
        if let Some(value) = self.locals.get(&id) {
            return Ok(value.clone());
        }
        let origins = self
            .facts
            .locals
            .get(&id)
            .map(|state| state.origins.iter().chain(&state.bounds).cloned().collect())
            .unwrap_or_default();
        let value = self.bundle(origins)?;
        self.locals.insert(id, value.clone());
        Ok(value)
    }

    pub(crate) fn condition(&self, value: &Expr) -> Result<Guard> {
        match value.kind {
            ExprKind::Bool(value) => Ok(if value { TRUE } else { FALSE }),
            _ => self
                .proofs
                .conditions
                .get(&(value.span.start, value.span.end))
                .copied()
                .ok_or_else(|| {
                    Diagnostic::unsupported("missing conditional loan proof", value.span)
                }),
        }
    }

    pub(crate) fn fork(&mut self, guard: Guard) -> Result<(usize, usize)> {
        let branch = self.append(Node::default())?;
        let yes = self.node(Node::default())?;
        let no = self.node(Node::default())?;
        self.nodes[branch].next = vec![
            Edge {
                target: yes,
                guard,
                reset: false,
            },
            Edge {
                target: no,
                guard: self.guards.not(guard),
                reset: false,
            },
        ];
        self.current.clear();
        Ok((yes, no))
    }

    pub(crate) fn block(&mut self, block: &Block) -> Result<Bundle> {
        let origins: Vec<_> = self
            .facts
            .blocks
            .get(&block.id)
            .map(|state| state.origins.iter().chain(&state.bounds).cloned().collect())
            .unwrap_or_default();
        let result = self.bundle(origins.clone())?;
        let value = self.bundle(origins)?;
        let start = self.append(Node {
            defs: result.values().copied().collect(),
            ..Node::default()
        })?;
        let end = self.node(Node {
            uses: result.values().copied().collect(),
            defs: value.values().copied().collect(),
            ..Node::default()
        })?;
        self.blocks.insert(
            block.id,
            Scope {
                start,
                end,
                result,
                ty: block.ty.clone(),
            },
        );
        self.statements(&block.stmts)?;
        self.connect(end, TRUE, false);
        self.blocks.remove(&block.id);
        self.current.push(end);
        if let Some(state) = self.facts.blocks.get(&block.id) {
            self.assume(state.proof)?;
        }
        if block.ty == Type::Never {
            self.current.clear();
        }
        Ok(value)
    }

    pub(crate) fn statements(&mut self, statements: &[Stmt]) -> Result<()> {
        for statement in statements {
            if self.current.is_empty() {
                break;
            }
            match statement {
                Stmt::Bind { id, value } => {
                    let value = self.expression(value)?;
                    let target = if self.program.locals[*id].has_reference() {
                        self.local(*id)?
                    } else {
                        Bundle::new()
                    };
                    self.append(Node {
                        uses: value.into_values().collect(),
                        defs: target.into_values().collect(),
                        ..Node::default()
                    })?;
                    if let Some(state) = self.facts.locals.get(id) {
                        self.assume(state.proof)?;
                    }
                }
                Stmt::Assign { id, value } => {
                    let result = self.expression(value)?;
                    self.append(Node {
                        uses: result.into_values().collect(),
                        write: Some((
                            Place {
                                root: *id,
                                fields: Vec::new(),
                            },
                            value.span,
                        )),
                        ..Node::default()
                    })?;
                }
                Stmt::Emit {
                    target,
                    field,
                    value,
                    ..
                } => {
                    let value = self.expression(value)?;
                    let scope = self.blocks.get(target).expect("emission target");
                    let result = crate::borrow::slot(&scope.ty, field)
                        .map(|(prefix, _)| Self::select(scope.result.clone(), &prefix))
                        .unwrap_or_default();
                    self.append(Node {
                        uses: value.into_values().collect(),
                        defs: result.into_values().collect(),
                        ..Node::default()
                    })?;
                }
                Stmt::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    self.expression(condition)?;
                    let (yes, no) = self.fork(self.condition(condition)?)?;
                    self.current.push(yes);
                    self.statements(then)?;
                    let mut ends = std::mem::take(&mut self.current);
                    self.current.push(no);
                    self.statements(otherwise)?;
                    ends.append(&mut self.current);
                    self.current = ends;
                }
                Stmt::Leave(id) | Stmt::Restart(id) => {
                    let scope = self.blocks.get(id).expect("control target");
                    let restart = matches!(statement, Stmt::Restart(_));
                    self.connect(if restart { scope.start } else { scope.end }, TRUE, restart);
                }
                Stmt::Expr(value) => {
                    let value = self.expression(value)?;
                    self.append(Node {
                        uses: value.into_values().collect(),
                        ..Node::default()
                    })?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn expression(&mut self, expr: &Expr) -> Result<Bundle> {
        self.project(expr, &[])
    }

    pub(crate) fn call(&mut self, site: CallId, args: &[Expr], span: Span) -> Result<Bundle> {
        let mut uses = Vec::new();
        for arg in args {
            uses.extend(self.expression(arg)?.into_values());
            if self.current.is_empty() {
                return Ok(Bundle::new());
            }
        }
        let state = self.facts.calls.get(&site);
        let origins = state
            .map(|state| state.origins.iter().chain(&state.bounds).cloned().collect())
            .unwrap_or_default();
        let proof = state
            .map(|state| self.guards.and(state.present, state.proof))
            .unwrap_or(FALSE);
        let value = self.bundle(origins)?;
        let node = self.append(Node {
            uses,
            defs: value.values().copied().collect(),
            ..Node::default()
        })?;
        if state.is_none() {
            self.missing_calls.push((node, span));
        }
        self.assume(proof)?;
        Ok(value)
    }

    pub(crate) fn inspect(&mut self, expr: &Expr) -> Result<()> {
        self.tick()?;
        match &expr.kind {
            ExprKind::Local(_) => {}
            ExprKind::Field { value, .. }
            | ExprKind::Primary(value)
            | ExprKind::Coerce { value } => self.inspect(value)?,
            ExprKind::Block(block) => {
                self.block(block)?;
            }
            _ => {
                self.expression(expr)?;
            }
        }
        Ok(())
    }

    pub(crate) fn convert(&mut self, value: &Expr, to: &Type, path: &[Step]) -> Result<Bundle> {
        let from = &value.ty;
        self.charge(path.len() + from.members().len() + to.members().len() + 1)?;
        if from == to {
            return self.project(value, path);
        }
        if *from == Type::Never {
            self.inspect(value)?;
            return Ok(Bundle::new());
        }
        let unsupported =
            || Diagnostic::unsupported("unknown borrowed union projection", value.span);
        if let Type::Union(sources) = from {
            if let Type::Union(targets) = to {
                if let Some((Step::Variant(index), tail)) = path.split_first() {
                    let member = targets.get(*index).ok_or_else(unsupported)?;
                    let Some(index) = sources.iter().position(|ty| ty == member) else {
                        self.inspect(value)?;
                        return Ok(Bundle::new());
                    };
                    let path = std::iter::once(Step::Variant(index))
                        .chain(tail.iter().copied())
                        .collect::<Vec<_>>();
                    return self.project(value, &path);
                }
                if !path.is_empty() {
                    return Err(unsupported());
                }
                let source = self.expression(value)?;
                let mut result = Bundle::new();
                for (mut component, id) in source {
                    self.charge(component.len() + targets.len() + 1)?;
                    let Some(Step::Variant(index)) = component.first() else {
                        return Err(unsupported());
                    };
                    let member = sources.get(*index).ok_or_else(unsupported)?;
                    if let Some(index) = targets.iter().position(|ty| ty == member) {
                        component[0] = Step::Variant(index);
                        result.insert(component, id);
                    }
                }
                return Ok(result);
            }
            let index = sources
                .iter()
                .position(|ty| ty == to)
                .ok_or_else(unsupported)?;
            let path = std::iter::once(Step::Variant(index))
                .chain(path.iter().copied())
                .collect::<Vec<_>>();
            return self.project(value, &path);
        }
        if let Type::Union(targets) = to {
            let member = targets
                .iter()
                .position(|ty| ty == from)
                .ok_or_else(unsupported)?;
            if let Some((Step::Variant(index), tail)) = path.split_first() {
                if *index == member {
                    return self.project(value, tail);
                }
                self.inspect(value)?;
                return Ok(Bundle::new());
            }
            if !path.is_empty() {
                return Err(unsupported());
            }
            let source = self.expression(value)?;
            let mut result = Bundle::new();
            for (mut component, id) in source {
                self.charge(component.len() + 1)?;
                component.insert(0, Step::Variant(member));
                result.insert(component, id);
            }
            return Ok(result);
        }
        Err(unsupported())
    }

    pub(crate) fn project(&mut self, expr: &Expr, path: &[Step]) -> Result<Bundle> {
        self.charge(path.len() + 1)?;
        let value = match &expr.kind {
            ExprKind::Borrow(place) => {
                let value = self.value(vec![Origin {
                    component: Vec::new(),
                    source: Source::Local(place.clone()),
                    guard: TRUE,
                }])?;
                self.append(Node {
                    defs: vec![value],
                    ..Node::default()
                })?;
                Bundle::from([(Vec::new(), value)])
            }
            ExprKind::Reborrow { site, value, .. } => {
                let parent = self.expression(value)?;
                if self.current.is_empty() {
                    return Ok(Bundle::new());
                }
                let state = self.facts.reborrows.get(site).ok_or_else(|| {
                    Diagnostic::unsupported("missing shared reborrow proof", expr.span)
                })?;
                self.charge(state.weight() + 1)?;
                let result =
                    self.bundle(state.origins.iter().chain(&state.bounds).cloned().collect())?;
                self.append(Node {
                    uses: parent.into_values().collect(),
                    defs: result.values().copied().collect(),
                    ..Node::default()
                })?;
                Self::select(result, path)
            }
            ExprKind::Local(id) if expr.ty.has_reference() => {
                let local = Self::select(self.local(*id)?, path);
                self.copy(local)?
            }
            ExprKind::Coerce { value } => self.convert(value, &expr.ty, path)?,
            ExprKind::Block(block) => Self::select(self.block(block)?, path),
            ExprKind::Field { value, index } => {
                let prefix: Vec<_> = std::iter::once(Step::Slot(index + 1))
                    .chain(path.iter().copied())
                    .collect();
                self.project(value, &prefix)?
            }
            ExprKind::Primary(value) => {
                let prefix: Vec<_> = std::iter::once(Step::Slot(0))
                    .chain(path.iter().copied())
                    .collect();
                self.project(value, &prefix)?
            }
            ExprKind::Deref(value)
            | ExprKind::Unary { value, .. }
            | ExprKind::StringSize(value)
            | ExprKind::ListSize(value) => {
                let value = self.expression(value)?;
                self.append(Node {
                    uses: value.into_values().collect(),
                    ..Node::default()
                })?;
                Bundle::new()
            }
            ExprKind::List { values, .. } => {
                for value in values {
                    let value = self.expression(value)?;
                    if self.current.is_empty() {
                        return Ok(Bundle::new());
                    }
                    self.append(Node {
                        uses: value.into_values().collect(),
                        ..Node::default()
                    })?;
                }
                Bundle::new()
            }
            ExprKind::ListIndex { value, index } | ExprKind::ListAdd { value, item: index } => {
                let left = self.expression(value)?;
                if self.current.is_empty() {
                    return Ok(Bundle::new());
                }
                let right = self.expression(index)?;
                self.append(Node {
                    uses: left.into_values().chain(right.into_values()).collect(),
                    ..Node::default()
                })?;
                Bundle::new()
            }
            ExprKind::TypeTest { value, .. } => {
                self.inspect(value)?;
                Bundle::new()
            }
            ExprKind::Binary { op, left, right } if ["&&", "||"].contains(&op.as_str()) => {
                self.expression(left)?;
                let guard = self.condition(left)?;
                let guard = if op == "&&" {
                    guard
                } else {
                    self.guards.not(guard)
                };
                let (yes, no) = self.fork(guard)?;
                self.current.push(yes);
                self.expression(right)?;
                self.current.push(no);
                Bundle::new()
            }
            ExprKind::Binary { left, right, .. } => {
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                self.append(Node {
                    uses: left.into_values().chain(right.into_values()).collect(),
                    ..Node::default()
                })?;
                Bundle::new()
            }
            ExprKind::Call { site, args, .. } => {
                Self::select(self.call(*site, args, expr.span)?, path)
            }
            ExprKind::Print { parts, .. } | ExprKind::Panic { parts } => {
                for part in parts {
                    let value = self.expression(part)?;
                    self.append(Node {
                        uses: value.into_values().collect(),
                        ..Node::default()
                    })?;
                }
                if matches!(expr.kind, ExprKind::Panic { .. }) {
                    self.current.clear();
                }
                Bundle::new()
            }
            ExprKind::Null
            | ExprKind::Bool(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Local(_) => Bundle::new(),
        };
        if expr.ty == Type::Never {
            self.current.clear();
        }
        Ok(value)
    }

    pub(crate) fn reach(&mut self) -> Result<Vec<Guard>> {
        let mut reach = vec![FALSE; self.nodes.len()];
        let mut queue = VecDeque::from([0]);
        reach[0] = TRUE;
        while let Some(id) = queue.pop_front() {
            self.tick()?;
            for edge in self.nodes[id].next.clone() {
                let guard = if edge.reset {
                    TRUE
                } else {
                    self.guards.and(reach[id], edge.guard)
                };
                let guard = self.guards.or(reach[edge.target], guard);
                if guard != reach[edge.target] {
                    reach[edge.target] = guard;
                    queue.push_back(edge.target);
                }
            }
        }
        Ok(reach)
    }

    pub(crate) fn outgoing(&mut self, id: usize, live: &[Live]) -> Result<Live> {
        let mut next = Live::new();
        for edge in self.nodes[id].next.clone() {
            if edge.guard == FALSE {
                continue;
            }
            for (value, guard) in &live[edge.target] {
                self.tick()?;
                let guard = if edge.reset {
                    TRUE
                } else {
                    self.guards.and(*guard, edge.guard)
                };
                if guard != FALSE {
                    let prior = next.get(value).copied().unwrap_or(FALSE);
                    next.insert(*value, self.guards.or(prior, guard));
                }
            }
        }
        Ok(next)
    }

    pub(crate) fn liveness(&mut self, reach: &[Guard]) -> Result<Vec<Live>> {
        let mut before = vec![Vec::new(); self.nodes.len()];
        for (id, node) in self.nodes.iter().enumerate() {
            for edge in &node.next {
                before[edge.target].push(id);
            }
        }
        let mut live = vec![Live::new(); self.nodes.len()];
        let mut queue: VecDeque<_> = (0..self.nodes.len()).rev().collect();
        let mut queued = vec![true; self.nodes.len()];
        let mut entries = 0;
        while let Some(id) = queue.pop_front() {
            queued[id] = false;
            self.tick()?;
            if reach[id] == FALSE {
                continue;
            }
            let mut next = self.outgoing(id, &live)?;
            for value in &self.nodes[id].defs {
                next.remove(value);
            }
            for value in &self.nodes[id].uses {
                next.insert(*value, TRUE);
            }
            if next != live[id] {
                entries = entries - live[id].len() + next.len();
                if entries > MAX_LIVE {
                    return Err(Self::budget());
                }
                live[id] = next;
                for id in &before[id] {
                    if !queued[*id] {
                        queued[*id] = true;
                        queue.push_back(*id);
                    }
                }
            }
        }
        Ok(live)
    }

    pub(crate) fn overlap(left: &Place, right: &Place) -> bool {
        left.root == right.root
            && (left.fields.starts_with(&right.fields) || right.fields.starts_with(&left.fields))
    }

    pub(crate) fn check(mut self, block: &Block, params: &[LocalId]) -> Result<()> {
        for id in params {
            let proof = self
                .facts
                .locals
                .get(id)
                .ok_or_else(|| {
                    Diagnostic::unsupported(
                        "missing function input borrow proof",
                        Span { start: 0, end: 0 },
                    )
                })?
                .proof;
            let value = self.local(*id)?;
            self.append(Node {
                defs: value.into_values().collect(),
                ..Node::default()
            })?;
            self.assume(proof)?;
        }
        self.block(block)?;
        let reach = self.reach()?;
        for (node, span) in &self.missing_calls {
            if reach[*node] != FALSE {
                return Err(Diagnostic::unsupported(
                    "missing function result borrow proof",
                    *span,
                ));
            }
        }
        let live = self.liveness(&reach)?;
        for (id, reachable) in reach.iter().enumerate() {
            let Some((place, span)) = self.nodes[id].write.clone() else {
                continue;
            };
            if *reachable == FALSE {
                continue;
            }
            for (value, guard) in self.outgoing(id, &live)? {
                self.charge(self.values[value].len() + 1)?;
                let guard = self.guards.and(guard, *reachable);
                for origin in &self.values[value] {
                    if let Source::Local(source) = &origin.source
                        && Self::overlap(&place, source)
                        && self.guards.overlap(guard, origin.guard)
                    {
                        return Err(Diagnostic::new(
                            "E302",
                            "assignment conflicts with a live shared borrow",
                            span,
                        ));
                    }
                }
            }
        }
        Ok(())
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
    pub(crate) fn assignments_resume_after_the_final_reference_use() {
        accepts("a:=1;r:&a;s:r;v:*s;a=2");
        accepts("a:=1;r:&a;a=*r+1");
        accepts("a:=1;r:&a;same:r==&a;a=2");
        rejects("a:=1;r:&a;a=2;v:*r", "E302");
        rejects("a:=1;r:&a;s:r;a=2;v:*s", "E302");
        rejects("a:=1;r:&a;a=*r+1;v:*r", "E302");
    }

    #[test]
    pub(crate) fn expression_temporaries_keep_borrows_until_their_consumption() {
        accepts("a:=1;r:&a;v:*r+{a=2;->1}");
        accepts("a:=1;r:&a;v:*r=={a=2;->2}");
        rejects("a:=1;b:2;r:&a;same:r=={a=2;->&b}", "E302");
        rejects("a:=1;b:2;same:&a=={a=2;->&b}", "E302");
        rejects("a:=1;r:{->&a;a=2};v:*r", "E302");
    }

    #[test]
    pub(crate) fn guard_proofs_exclude_disjoint_writes_and_uses() {
        accepts("f<null>:(flag<boolean>){a:=1;r:&a;|flag|a=2;|!flag|v:*r}");
        accepts(
            "f<int32>:(flag<boolean>){a:=11;b:=22;r:{|flag|->&a;|!flag|->&b};|flag|b=33;|!flag|a=44;->*r}",
        );
        rejects(
            "f<int32>:(flag<boolean>){a:=11;b:=22;r:{|flag|->&a;|!flag|->&b};|flag|a=33;->*r}",
            "E302",
        );
        accepts("a:=1;r:&a;v:false&&{a=2;->true};x:*r");
        accepts("a:=1;r:&a;v:true||{a=2;->true};x:*r");
    }

    #[test]
    pub(crate) fn restart_solves_liveness_across_iterations() {
        accepts("a:=0;i:=0;'loop {r:&a;v:*r;a=a+1;i=i+1;|i<2|'loop.restart()}");
        rejects(
            "a:=1;r:&a;later:=false;i:=0;'loop {|later|v:*r;|!later|a=2;later=true;i=i+1;|i<2|'loop.restart()}",
            "E302",
        );
        accepts(
            "a:=1;again:=true;r:'out {|again|{'out->&a;a=2;again=false;'out.restart()};->&a};v:*r",
        );
        rejects(
            "a:=1;r:&a;i:=0;'loop {v:*r;a=2;i=i+1;|i<2|'loop.restart()}",
            "E302",
        );
    }

    #[test]
    pub(crate) fn whole_record_writes_overlap_borrowed_fields() {
        accepts("a:={->x:1;->y:2};r:&a.x;v:*r;a={->x:3;->y:4}");
        rejects("a:={->x:1;->y:2};r:&a.x;a={->x:3;->y:4};v:*r", "E302");
        rejects(
            "a:={->nested:{->x:1}};r:&a.nested.x;a={->nested:{->x:2}};v:*r",
            "E302",
        );
    }

    #[test]
    pub(crate) fn scoped_exits_end_unreachable_loan_uses() {
        accepts("a:=1;r:&a;'out {'out.leave();a=2;v:*r}");
        accepts("a:=1;r:&a;v:*'out {|true|{'out->r;'out.leave()};a=2;->r}");
        rejects("a:=1;r:&a;v:*'out {a=2;->r}", "E302");
        accepts("f<int32>:(){a:=1;r:&a;a=*r+1;->a};v:f()");
    }

    #[test]
    pub(crate) fn dense_reference_liveness_rejects_with_a_bounded_diagnostic() {
        let mut source = "a:=1;".to_owned();
        for id in 0..512 {
            source.push_str(&format!("r{id}:&a;"));
        }
        for id in 0..512 {
            source.push_str(&format!("v{id}:*r{id};"));
        }
        let errors = crate::compile(&source).unwrap_err();
        assert_eq!(errors[0].code, "B001", "{errors:?}");
        assert!(errors[0].message.contains("loan-analysis budget"));
    }

    #[test]
    pub(crate) fn aliases_of_many_origins_cannot_expand_storage_without_bound() {
        let params = (0..63)
            .map(|id| format!("p{id}<boolean>"))
            .collect::<Vec<_>>()
            .join(",");
        let mut source = format!("f<null>:({params}){{");
        for id in 0..64 {
            source.push_str(&format!("a{id}:{id};"));
        }
        source.push_str("r:'pick {");
        for id in 0..63 {
            source.push_str(&format!("|p{id}|{{'pick->&a{id};'pick.leave()}};"));
        }
        source.push_str("->&a63};");
        for (count, budget) in [
            (2048, "loan-analysis budget"),
            (4096, "borrow-origin fact budget"),
        ] {
            let mut source = source.clone();
            for id in 0..count {
                source.push_str(&format!("r{id}:r;"));
            }
            source.push('}');
            let errors = crate::compile(&source).unwrap_err();
            assert_eq!(errors[0].code, "B001", "{errors:?}");
            assert!(errors[0].message.contains(budget), "{errors:?}");
        }
    }

    #[test]
    pub(crate) fn projections_only_read_the_selected_record_components() {
        accepts("a:=1;b:=2;r:{->left:&a;->right:&b};b=3;x:*r.left");
        accepts("a:=1;r:{->count:3;->view:&a};a=2;x:r.count");
        accepts("a:=1;b:=2;r:{->left:{->view:&a};->right:{->view:&b}};b=3;x:*r.left.view");
        accepts("a:=1;r:{->5;->view:&a};a=2;d:@\"debug\";d.print(r);x:r+1;number<int32>:r");
        rejects(
            "a:=1;b:=2;r:{->left:&a;->right:&b};b=3;copy:r;x:*copy.left",
            "E302",
        );
        rejects("a:=1;b:=2;r:{->left:&a;->right:&b};a=3;x:*r.left", "E302");
    }

    #[test]
    pub(crate) fn record_operands_and_result_slots_hold_all_copied_components() {
        accepts("a:=1;b:=2;r:{->left:&a;->right:&b};same:r.left=={b=3;->&a}");
        rejects(
            "a:=1;b:=2;r:{->left:&a;->right:&b};same:r=={a=3;->r}",
            "E302",
        );
        rejects(
            "a:=1;b:2;left:{->view:&a};right:{->view:&b};same:left=={a=3;->right}",
            "E302",
        );
        rejects("a:=1;b:=2;r:{->left:&a;a=3;->right:&b};x:*r.left", "E302");
        accepts(
            "a:=1;again:=true;r:'out {|again|{'out->view:&a;a=2;again=false;'out.restart()};->view:&a};x:*r.view",
        );
    }

    #[test]
    pub(crate) fn component_origins_keep_branch_and_iteration_liveness() {
        accepts(
            "f<int32>:(flag<boolean>){a:=1;b:=2;r:{|flag|->view:&a;|!flag|->view:&b};|flag|b=3;|!flag|a=4;->*r.view}",
        );
        rejects(
            "f<int32>:(flag<boolean>){a:=1;b:=2;r:{|flag|->view:&a;|!flag|->view:&b};|flag|a=3;->*r.view}",
            "E302",
        );
        accepts("a:=0;i:=0;'loop {r:{->view:&a};x:*r.view;a=a+1;i=i+1;|i<2|'loop.restart()}");
        rejects(
            "a:=1;r:{->view:&a};i:=0;'loop {x:*r.view;a=2;i=i+1;|i<2|'loop.restart()}",
            "E302",
        );
    }

    #[test]
    pub(crate) fn component_aliases_obey_the_reference_value_budget() {
        let mut source = "a:=1;r:{".to_owned();
        for id in 0..64 {
            source.push_str(&format!("->field{id}:&a;"));
        }
        source.push_str("};");
        for id in 0..1024 {
            source.push_str(&format!("copy{id}:r;"));
        }
        let errors = crate::compile(&source).unwrap_err();
        assert_eq!(errors[0].code, "B001", "{errors:?}");
        assert!(
            errors[0].message.contains("loan-analysis budget"),
            "{errors:?}"
        );
    }

    #[test]
    pub(crate) fn union_tag_inspection_does_not_read_payloads_or_skip_construction() {
        accepts("a:=1;r<&int32><null>:&a;a=2;|r<null>|{}");
        accepts("a:=1;r:{->view<&int32><null>:&a};a=2;|r.view<&int32>|{}");
        accepts("a:=1;r<&int32><null>:&a;a=2;|r<&int32>|{}");
        rejects("a:=1;flag:=true;|({|flag|->&a;a=2})<null>|{}", "E302");
        rejects("a:=1;r<&int32><null>:&a;a=2;copy:r", "E302");
    }

    #[test]
    pub(crate) fn union_extraction_and_retagging_preserve_projection_demand() {
        accepts(
            "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;|r<Row>|{b=3;x:*r.left}",
        );
        accepts(
            "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;wide<Row><null><boolean>:r;|wide<Row>|{b=3;x:*wide.left}",
        );
        rejects(
            "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;|r<Row>|{b=3;copy:r;x:*copy.left}",
            "E302",
        );
        rejects(
            "<Row>:<{left<&int32>;right<&int32>}>;a:=1;b:=2;row<Row>:{->left:&a;->right:&b};r<Row><null>:row;wide<Row><null><boolean>:r;|wide<Row>|{a=3;x:*wide.left}",
            "E302",
        );
    }

    #[test]
    pub(crate) fn union_activity_assumptions_are_reestablished_after_restart() {
        accepts(
            "a:=0;i:=0;'loop {r<&int32><null>:{|i<2|->&a};|r<&int32>|{x:*r<&int32>};a=a+1;i=i+1;|i<3|'loop.restart()}",
        );
        rejects(
            "a:=1;r<&int32><null>:&a;first:=true;i:=0;'loop {|!first&&r<&int32>|{x:*r<&int32>};|first|a=2;first=false;i=i+1;|i<2|'loop.restart()}",
            "E302",
        );
    }

    #[test]
    pub(crate) fn returned_function_views_keep_all_input_bounds_until_final_use() {
        accepts(
            "first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:first(&a,&b);x:*r;b=\"new\";a=2",
        );
        rejects(
            "first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:first(&a,&b);b=\"new\";x:*r",
            "E302",
        );
        rejects(
            "identity<&int32>:(a<&int32>){->a};first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:identity(first(&a,&b));b=\"new\";x:*r",
            "E302",
        );
        rejects(
            "head<&int32>:(r<{left<&int32>;right<&string>}>){->r.left};a:=1;b:=\"old\";r:head({->left:&a;->right:&b});b=\"new\";x:*r",
            "E302",
        );
    }

    #[test]
    pub(crate) fn scalar_function_results_and_projections_end_argument_loans() {
        accepts("read<int32>:(a<&int32>){->*a};a:=1;x:read(&a);a=2");
        accepts(
            "packet<{view<&int32>;count<int32>}>:(a<&int32>,b<&string>){->view:a;->count:*a};a:=1;b:=\"old\";r:packet(&a,&b);a=2;b=\"new\";x:r.count",
        );
        accepts(
            "packet<{view<&int32>;count<int32>}>:(a<&int32>,b<&string>){->view:a;->count:*a};a:=1;b:=\"old\";x:packet(&a,&b).count;a=2;b=\"new\"",
        );
    }

    #[test]
    pub(crate) fn call_arguments_stay_live_until_consumption_and_stop_at_exit() {
        rejects(
            "read<int32>:(a<&int32>,b<int32>){->*a+b};a:=1;x:read(&a,{a=2;->3})",
            "E302",
        );
        accepts("read<int32>:(a<&int32>,b<int32>){->*a+b};a:=1;'out {read(&a,{'out.leave()});a=2}");
        accepts(
            "identity<&int32>:(a<&int32>){->a};a:=1;r<&int32><null>:null;|r<&int32>|{view:identity(&a)};a=2",
        );
    }

    #[test]
    pub(crate) fn symbolic_function_inputs_do_not_hide_local_conflicts() {
        accepts("read<int32>:(a<&int32>){local:=1;r:&local;x:*r;local=2;->*a}");
        rejects(
            "read<int32>:(a<&int32>){local:=1;r:&local;local=2;x:*r;->*a}",
            "E302",
        );
    }
}
