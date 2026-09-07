use super::access::Kind;
use super::storage::{EventKind, ScopeKind};
use super::{Block, Bundle, Graph, Node, Origin, Place, Result, Scope, Stmt, TRUE, Type};
use crate::hir::WriteStep;

impl<'a> Graph<'a> {
    pub(crate) fn block(&mut self, block: &Block) -> Result<Bundle> {
        let life = self.new_scope(ScopeKind::Block(block.id))?;
        let mut incoming = if self.merging {
            self.versions()?
        } else {
            super::branches::Versions::new()
        };
        let header = if self.merging {
            self.restart_header(block.id, &incoming)?
        } else {
            None
        };
        let restarted = header.is_some();
        if let Some(header) = header {
            self.restore_versions(&header)?;
            incoming = header;
        }
        let origins = self
            .facts
            .blocks
            .get(&block.id)
            .map(super::values::Value::from)
            .unwrap_or_default();
        let result = self.bundle(origins.clone())?;
        let value = self.bundle(origins)?;
        let mut node = Node {
            defs: result.values().copied().collect(),
            ..Node::default()
        };
        self.event(&mut node, EventKind::Enter(life), super::Span::default())?;
        let start = self.node(node)?;
        self.connect(start, TRUE, restarted);
        self.current.push(start);
        let node = self.copied(&result, &value)?;
        let end = self.node(node)?;
        self.blocks.insert(
            block.id,
            Scope {
                life,
                start,
                end,
                result,
                ty: block.ty.clone(),
                incoming,
                leaves: Vec::new(),
                restarted,
            },
        );
        self.statements(&block.stmts)?;
        let mut scope = self.blocks.remove(&block.id).expect("completed scope");
        if self.merging {
            self.charge(scope.incoming.len() + 1)?;
            let ids = scope.incoming.keys().copied().collect::<Vec<_>>();
            scope.leaves.push(self.surviving_arm(&ids)?);
            self.merge_versions(scope.incoming, scope.leaves)?;
        }
        self.connect(end, TRUE, false);
        self.current.push(end);
        if let Some(state) = self.facts.blocks.get(&block.id) {
            self.assume(state.proof)?;
        }
        self.close_scope(life)?;
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
                    let scope = self.enter_scope(ScopeKind::Statement(*id))?;
                    self.statements(stmts)?;
                    self.close_scope(scope)?;
                }
                Stmt::Bind { id, value } => {
                    let span = value.span;
                    let cell = self.register_store(*id, span)?;
                    let value = self.expression(value)?;
                    let target = if self.program.locals[*id].has_reference() {
                        self.local(*id)?
                    } else {
                        Bundle::new()
                    };
                    let mut node = self.copied(&value, &target)?;
                    if self.proofs.receivers.contains(id) {
                        node.barrier = Some(span);
                    }
                    self.event(&mut node, EventKind::Init(cell), span)?;
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
                    let target = if matches!(
                        self.program.locals[*id],
                        Type::Reference(_) | Type::Exclusive(_)
                    ) {
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
                    let access =
                        self.access(Kind::Write, self.storage(*id, Vec::new()), &[], value.span)?;
                    node.access = Some(access);
                    self.event(&mut node, EventKind::Init(self.cell(*id)), value.span)?;
                    self.append(node)?;
                    if let Some(target) = target {
                        self.locals.insert(*id, target);
                    }
                }
                Stmt::Store {
                    target,
                    value,
                    span,
                } => {
                    let pointer = self.reference_value(target)?;
                    if self.current.is_empty() {
                        continue;
                    }
                    let value = self.expression(value)?;
                    if self.current.is_empty() {
                        continue;
                    }
                    let access = self.pointee_access(&pointer, &[], Kind::Write, *span)?;
                    let mut uses = self.direct(&pointer)?;
                    uses.extend(value.into_values());
                    self.append(Node {
                        uses,
                        access: Some(access),
                        ..Node::default()
                    })?;
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
                    let mut access = Node::default();
                    self.event(
                        &mut access,
                        EventKind::Use {
                            id: self.cell(*id),
                            take: false,
                        },
                        *span,
                    )?;
                    self.append(access)?;
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
                    let access =
                        self.access(Kind::Write, self.storage(*id, place.fields), &[], *span)?;
                    let mut node = Node {
                        uses: reservation.into_iter().chain(value.into_values()).collect(),
                        access: Some(access),
                        ..Node::default()
                    };
                    self.event(
                        &mut node,
                        EventKind::Use {
                            id: self.cell(*id),
                            take: false,
                        },
                        *span,
                    )?;
                    self.append(node)?;
                }
                Stmt::Emit {
                    target,
                    field,
                    value,
                    ..
                } => {
                    let span = value.span;
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
                    let mut node = self.copied(&value, &result)?;
                    node.barrier = Some(span);
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
                Stmt::Restart { target, site } if self.merging => {
                    let life = self.blocks.get(target).expect("restart scope").life;
                    self.end_scopes(Some((life, true)))?;
                    self.restart(*target, *site)?;
                }
                Stmt::Leave(id) | Stmt::Restart { target: id, .. } => {
                    let restart = matches!(statement, Stmt::Restart { .. });
                    let life = self.blocks.get(id).expect("control scope").life;
                    self.end_scopes(Some((life, restart)))?;
                    if self.merging && !restart {
                        self.charge(
                            self.blocks.get(id).expect("control target").incoming.len() + 1,
                        )?;
                        let ids = self.blocks[id].incoming.keys().copied().collect::<Vec<_>>();
                        let arm = self.surviving_arm(&ids)?;
                        self.blocks
                            .get_mut(id)
                            .expect("control target")
                            .leaves
                            .push(arm);
                        continue;
                    }
                    let scope = self.blocks.get(id).expect("control target");
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
