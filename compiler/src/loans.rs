use std::collections::{BTreeMap, VecDeque};

use crate::ast::Span;
use crate::borrow::{Facts, Origin, Proofs};
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow, Guard, TRUE};
use crate::hir::{Block, BlockId, Expr, ExprKind, LocalId, Place, Program, Stmt, Type};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Live = BTreeMap<usize, Guard>;
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
    pub(crate) result: Option<usize>,
}

pub(crate) struct Graph<'a> {
    pub(crate) program: &'a Program,
    pub(crate) facts: &'a Facts,
    pub(crate) proofs: &'a Proofs,
    pub(crate) guards: &'a mut Flow,
    pub(crate) nodes: Vec<Node>,
    pub(crate) values: Vec<Vec<Origin>>,
    pub(crate) locals: BTreeMap<LocalId, usize>,
    pub(crate) blocks: BTreeMap<BlockId, Scope>,
    pub(crate) current: Vec<usize>,
    pub(crate) work: usize,
    pub(crate) origins: usize,
}

pub(crate) fn check(
    program: &Program,
    facts: &Facts,
    proofs: &Proofs,
    guards: &mut Flow,
) -> std::result::Result<(), Vec<Diagnostic>> {
    let result = (|| {
        Graph::new(program, facts, proofs, guards).check(&program.body)?;
        for function in &program.functions {
            Graph::new(program, facts, proofs, guards).check(&function.body)?;
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
        }
    }

    pub(crate) fn budget() -> Diagnostic {
        Diagnostic::unsupported("loan-analysis budget exhausted", Span { start: 0, end: 0 })
    }

    pub(crate) fn tick(&mut self) -> Result<()> {
        self.charge(1)
    }

    pub(crate) fn charge(&mut self, work: usize) -> Result<()> {
        self.work += work;
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

    pub(crate) fn value(&mut self, origins: Vec<Origin>) -> Result<usize> {
        if self.values.len() == MAX_VALUES || self.origins + origins.len() > MAX_ORIGINS {
            return Err(Self::budget());
        }
        self.origins += origins.len();
        let id = self.values.len();
        self.values.push(origins);
        Ok(id)
    }

    pub(crate) fn local(&mut self, id: LocalId) -> Result<usize> {
        if let Some(value) = self.locals.get(&id) {
            return Ok(*value);
        }
        let origins = self.facts.locals.get(&id).cloned().unwrap_or_default();
        let value = self.value(origins)?;
        self.locals.insert(id, value);
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

    pub(crate) fn block(&mut self, block: &Block) -> Result<Option<usize>> {
        let origins = self
            .facts
            .blocks
            .get(&block.id)
            .cloned()
            .unwrap_or_default();
        let reference = matches!(block.ty, Type::Reference(_));
        let result = reference.then(|| self.value(origins.clone())).transpose()?;
        let value = reference.then(|| self.value(origins)).transpose()?;
        let start = self.append(Node {
            defs: result.into_iter().collect(),
            ..Node::default()
        })?;
        let end = self.node(Node {
            uses: result.into_iter().collect(),
            defs: value.into_iter().collect(),
            ..Node::default()
        })?;
        self.blocks.insert(block.id, Scope { start, end, result });
        self.statements(&block.stmts)?;
        self.connect(end, TRUE, false);
        self.blocks.remove(&block.id);
        self.current.push(end);
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
                    let target = self.program.locals[*id]
                        .has_reference()
                        .then(|| self.local(*id))
                        .transpose()?;
                    self.append(Node {
                        uses: value.into_iter().collect(),
                        defs: target.into_iter().collect(),
                        ..Node::default()
                    })?;
                }
                Stmt::Assign { id, value } => {
                    let result = self.expression(value)?;
                    self.append(Node {
                        uses: result.into_iter().collect(),
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
                    let result = if field.is_none() {
                        self.blocks.get(target).and_then(|scope| scope.result)
                    } else {
                        None
                    };
                    self.append(Node {
                        uses: value.into_iter().collect(),
                        defs: result.into_iter().collect(),
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
                        uses: value.into_iter().collect(),
                        ..Node::default()
                    })?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn expression(&mut self, expr: &Expr) -> Result<Option<usize>> {
        let value = match &expr.kind {
            ExprKind::Borrow(place) => {
                let value = self.value(vec![Origin {
                    place: place.clone(),
                    guard: TRUE,
                }])?;
                self.append(Node {
                    defs: vec![value],
                    ..Node::default()
                })?;
                Some(value)
            }
            ExprKind::Local(id) if matches!(expr.ty, Type::Reference(_)) => {
                let local = self.local(*id)?;
                let value = self.value(self.values[local].clone())?;
                self.append(Node {
                    uses: vec![local],
                    defs: vec![value],
                    ..Node::default()
                })?;
                Some(value)
            }
            ExprKind::Coerce { value } => self.expression(value)?,
            ExprKind::Block(block) => self.block(block)?,
            ExprKind::Deref(value)
            | ExprKind::Unary { value, .. }
            | ExprKind::Field { value, .. }
            | ExprKind::Primary(value)
            | ExprKind::StringSize(value)
            | ExprKind::TypeTest { value, .. } => {
                let value = self.expression(value)?;
                self.append(Node {
                    uses: value.into_iter().collect(),
                    ..Node::default()
                })?;
                None
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
                None
            }
            ExprKind::Binary { left, right, .. } => {
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                self.append(Node {
                    uses: left.into_iter().chain(right).collect(),
                    ..Node::default()
                })?;
                None
            }
            ExprKind::Call { args, .. } => {
                let mut uses = Vec::new();
                for arg in args {
                    uses.extend(self.expression(arg)?);
                }
                self.append(Node {
                    uses,
                    ..Node::default()
                })?;
                None
            }
            ExprKind::Print { parts, .. } | ExprKind::Panic { parts } => {
                for part in parts {
                    let value = self.expression(part)?;
                    self.append(Node {
                        uses: value.into_iter().collect(),
                        ..Node::default()
                    })?;
                }
                if matches!(expr.kind, ExprKind::Panic { .. }) {
                    self.current.clear();
                }
                None
            }
            ExprKind::Null
            | ExprKind::Bool(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Local(_) => None,
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

    pub(crate) fn check(mut self, block: &Block) -> Result<()> {
        self.block(block)?;
        let reach = self.reach()?;
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
                    if Self::overlap(&place, &origin.place)
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
}
