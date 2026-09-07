use super::{Block, Bundle, Graph, Node, Origin, Place, Result, Scope, Stmt, TRUE, Type};
use crate::hir::WriteStep;

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
        let node = self.copied(&result, &value)?;
        let end = self.node(node)?;
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
                Stmt::Statement { id, stmts } => {
                    self.charge(self.statements.len() + 1)?;
                    self.statements.push(*id);
                    self.statements(stmts)?;
                    self.statements.pop();
                }
                Stmt::Bind { id, value } => {
                    let value = self.expression(value)?;
                    let target = if self.program.locals[*id].has_reference() {
                        self.local(*id)?
                    } else {
                        Bundle::new()
                    };
                    let node = self.copied(&value, &target)?;
                    self.append(node)?;
                    if let Some(state) = self.facts.locals.get(id) {
                        self.assume(state.proof)?;
                    }
                }
                Stmt::SlotAlias {
                    id,
                    target,
                    field,
                    mutable,
                } => {
                    self.charge(1)?;
                    if !self.proofs.aliases.get(id).is_some_and(|alias| {
                        alias.target == *target
                            && alias.field == *field
                            && alias.mutable == *mutable
                    }) {
                        return Err(crate::diagnostic::Diagnostic::unsupported(
                            "missing result-slot alias proof",
                            crate::ast::Span::default(),
                        ));
                    }
                }
                Stmt::Assign { id, value } => {
                    let result = self.expression(value)?;
                    if self.current.is_empty() {
                        continue;
                    }
                    let target = if matches!(self.program.locals[*id], Type::Reference(_)) {
                        if !self.merging {
                            return Err(crate::diagnostic::Diagnostic::unsupported(
                                "missing mutable-reference body proof",
                                value.span,
                            ));
                        }
                        Some(self.version(&result)?)
                    } else {
                        None
                    };
                    let mut node = if let Some(target) = &target {
                        self.copied(&result, target)?
                    } else {
                        Node {
                            uses: result.into_values().collect(),
                            ..Node::default()
                        }
                    };
                    node.write = Some((
                        Place {
                            root: self
                                .proofs
                                .aliases
                                .get(id)
                                .map(|alias| alias.root)
                                .unwrap_or(*id),
                            fields: Vec::new(),
                        },
                        value.span,
                    ));
                    self.append(node)?;
                    if let Some(target) = target {
                        self.locals.insert(*id, target);
                    }
                }
                Stmt::SetPath {
                    id,
                    path,
                    value,
                    span,
                } => {
                    self.charge(path.len() + 1)?;
                    let first = path
                        .iter()
                        .position(|step| matches!(step, WriteStep::Index(_)));
                    let fields = path
                        .iter()
                        .take(first.unwrap_or(path.len()))
                        .map(|step| {
                            let WriteStep::Field(index) = step else {
                                unreachable!()
                            };
                            *index
                        })
                        .collect();
                    let place = Place {
                        root: self
                            .proofs
                            .aliases
                            .get(id)
                            .map(|alias| alias.root)
                            .unwrap_or(*id),
                        fields,
                    };
                    self.charge(place.fields.len() + 1)?;
                    let reservation = if first.is_some() {
                        let value = self.value(vec![Origin {
                            component: Vec::new(),
                            source: self.proofs.source(&Place {
                                root: *id,
                                fields: place.fields.clone(),
                            }),
                            guard: TRUE,
                        }])?;
                        self.append(Node {
                            defs: vec![value],
                            ..Node::default()
                        })?;
                        Some(value)
                    } else {
                        None
                    };
                    for step in path {
                        self.charge(1)?;
                        if let WriteStep::Index(step) = step {
                            let index = self.expression(&step.index)?;
                            if self.current.is_empty() {
                                break;
                            }
                            self.append(Node {
                                uses: std::iter::once(reservation.expect("indexed reservation"))
                                    .chain(index.into_values())
                                    .collect(),
                                ..Node::default()
                            })?;
                        }
                    }
                    if self.current.is_empty() {
                        continue;
                    }
                    let value = self.expression(value)?;
                    if self.current.is_empty() {
                        continue;
                    }
                    self.append(Node {
                        uses: reservation.into_iter().chain(value.into_values()).collect(),
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
                    let scope = self.blocks.get(target).expect("emission target");
                    let slot = crate::borrow::slot(&scope.ty, field)
                        .filter(|(_, ty)| ty.accepts(&value.ty))
                        .map(|(prefix, ty)| (prefix, ty.clone()));
                    let result = slot
                        .as_ref()
                        .map(|(prefix, _)| Self::select(scope.result.clone(), prefix))
                        .unwrap_or_default();
                    let value = if let Some((_, ty)) = slot {
                        self.convert(value, &ty, &[])?
                    } else {
                        self.expression(value)?
                    };
                    let node = self.copied(&value, &result)?;
                    self.append(node)?;
                }
                Stmt::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    self.expression(condition)?;
                    if self.current.is_empty() {
                        continue;
                    }
                    self.conditional(
                        self.condition(condition)?,
                        |graph| graph.statements(then),
                        |graph| graph.statements(otherwise),
                    )?;
                }
                Stmt::Leave(id) | Stmt::Restart(id) => {
                    let scope = self.blocks.get(id).expect("control target");
                    let restart = matches!(statement, Stmt::Restart(_));
                    self.connect(if restart { scope.start } else { scope.end }, TRUE, restart);
                }
                Stmt::Expr(value) => {
                    let value = self.expression(value)?;
                    let uses = self.direct(&value)?;
                    self.append(Node {
                        uses,
                        ..Node::default()
                    })?;
                }
            }
        }
        Ok(())
    }
}
