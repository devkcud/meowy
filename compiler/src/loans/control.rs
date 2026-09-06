use super::{Block, Bundle, Graph, Node, Origin, Place, Result, Scope, Source, Stmt, TRUE, Type};

impl<'a> Graph<'a> {
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
                Stmt::SetElement {
                    id,
                    path,
                    value,
                    span,
                } => {
                    let place = Place {
                        root: *id,
                        fields: Vec::new(),
                    };
                    let reservation = self.value(vec![Origin {
                        component: Vec::new(),
                        source: Source::local(&place),
                        guard: TRUE,
                    }])?;
                    self.append(Node {
                        defs: vec![reservation],
                        ..Node::default()
                    })?;
                    for step in path {
                        self.charge(1)?;
                        let index = self.expression(&step.index)?;
                        if self.current.is_empty() {
                            break;
                        }
                        self.append(Node {
                            uses: std::iter::once(reservation)
                                .chain(index.into_values())
                                .collect(),
                            ..Node::default()
                        })?;
                    }
                    if self.current.is_empty() {
                        continue;
                    }
                    let value = self.expression(value)?;
                    if self.current.is_empty() {
                        continue;
                    }
                    self.append(Node {
                        uses: std::iter::once(reservation)
                            .chain(value.into_values())
                            .collect(),
                        write: Some((place, *span)),
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
}
