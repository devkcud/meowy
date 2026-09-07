use super::{
    BTreeMap, Diagnostic, Edge, Expr, ExprKind, FALSE, Facts, Flow, Graph, Guard, MAX_NODES,
    MAX_WORK, Node, Program, Proofs, Result, Span, TRUE,
};

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
            statements: Vec::new(),
            current: vec![0],
            work: 0,
            origins: 0,
            merging: false,
            missing_calls: Vec::new(),
            missing_reborrows: Vec::new(),
            missing_headers: Vec::new(),
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
}
