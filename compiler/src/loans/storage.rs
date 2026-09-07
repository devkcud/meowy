use super::{BlockId, Graph, LocalId, Node, Result, Span};
use crate::hir::StatementId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ScopeId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Key {
    Scope(ScopeId),
    Cell(LocalId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScopeKind {
    Function,
    Block(BlockId),
    Branch,
    Statement(StatementId),
}

pub(crate) struct Frame {
    pub(crate) kind: ScopeKind,
    pub(crate) cells: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Store {
    pub(crate) scope: ScopeId,
    pub(crate) copy: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventKind {
    Enter(ScopeId),
    End(ScopeId),
    Init(LocalId),
    Use { id: LocalId, take: bool },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Event {
    pub(crate) kind: EventKind,
    pub(crate) span: Span,
}

impl Graph<'_> {
    pub(crate) fn new_scope(&mut self, kind: ScopeKind) -> Result<ScopeId> {
        if self.scopes.len() >= super::MAX_NODES {
            return Err(Self::budget());
        }
        self.reserve_authority(3)?;
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Frame { kind, cells: 0 });
        self.active.push(id);
        Ok(id)
    }

    pub(crate) fn event(&mut self, node: &mut Node, kind: EventKind, span: Span) -> Result<()> {
        self.reserve_authority(3)?;
        let valid = match kind {
            EventKind::Enter(scope) | EventKind::End(scope) => scope.0 < self.scopes.len(),
            EventKind::Init(id) | EventKind::Use { id, .. } => self.stores.contains_key(&id),
        };
        if !valid {
            return Err(super::Diagnostic::unsupported(
                "missing storage lifecycle declaration",
                span,
            ));
        }
        node.events.push(Event { kind, span });
        Ok(())
    }

    pub(crate) fn enter_scope(&mut self, kind: ScopeKind) -> Result<ScopeId> {
        let scope = self.new_scope(kind)?;
        let mut node = Node::default();
        self.event(&mut node, EventKind::Enter(scope), Span::default())?;
        self.append(node)?;
        Ok(scope)
    }

    pub(crate) fn close_scope(&mut self, scope: ScopeId) -> Result<()> {
        if self.active.last() != Some(&scope) {
            return Err(Self::budget());
        }
        if !self.current.is_empty() {
            let mut node = Node::default();
            self.event(&mut node, EventKind::End(scope), Span::default())?;
            self.append(node)?;
        }
        self.active.pop();
        Ok(())
    }

    pub(crate) fn end_scopes(&mut self, target: Option<(ScopeId, bool)>) -> Result<()> {
        self.charge(self.active.len() + 1)?;
        if self.current.is_empty() {
            return Ok(());
        }
        let start = if let Some((scope, restart)) = target {
            let index = self
                .active
                .iter()
                .position(|id| *id == scope)
                .ok_or_else(Self::budget)?;
            index + usize::from(!restart)
        } else {
            0
        };
        let scopes = self.active[start..].to_vec();
        let mut node = Node::default();
        for scope in scopes.into_iter().rev() {
            self.event(&mut node, EventKind::End(scope), Span::default())?;
        }
        if !node.events.is_empty() {
            self.append(node)?;
        }
        Ok(())
    }

    pub(crate) fn statement_scope(&self, id: StatementId) -> Option<ScopeId> {
        self.active
            .iter()
            .rev()
            .copied()
            .find(|scope| self.scopes[scope.0].kind == ScopeKind::Statement(id))
    }

    pub(crate) fn cell(&self, id: LocalId) -> LocalId {
        self.proofs.aliases.get(&id).map_or(id, |alias| alias.root)
    }

    pub(crate) fn register_store(&mut self, view: LocalId, span: Span) -> Result<LocalId> {
        self.charge(
            self.active.len() + self.stores.len().checked_ilog2().unwrap_or(0) as usize + 1,
        )?;
        let scope = if let Some(alias) = self.proofs.aliases.get(&view) {
            self.blocks.get(&alias.target).map(|scope| scope.life)
        } else if let Some(statement) = self.proofs.temporaries.get(&view) {
            self.statement_scope(*statement)
        } else {
            self.active
                .iter()
                .rev()
                .copied()
                .find(|scope| !matches!(self.scopes[scope.0].kind, ScopeKind::Statement(_)))
        }
        .ok_or_else(Self::budget)?;
        let ty = self.program.locals.get(view).ok_or_else(Self::budget)?;
        crate::borrow_contract::type_weight(ty, self.guards, span)?;
        let copy = ty.is_copy();
        let id = self.cell(view);
        if let Some(store) = self.stores.get(&id) {
            if store.scope != scope || store.copy != copy {
                return Err(Self::budget());
            }
        } else {
            if self.stores.len() >= super::MAX_VALUES {
                return Err(Self::budget());
            }
            self.reserve_authority(3)?;
            self.stores.insert(id, Store { scope, copy });
            self.scopes[scope.0].cells += 1;
        }
        Ok(id)
    }

    pub(crate) fn owns(&self, scope: ScopeId, key: &Key) -> bool {
        match key {
            Key::Scope(id) => *id == scope,
            Key::Cell(id) => self
                .stores
                .get(id)
                .is_some_and(|store| store.scope == scope),
        }
    }
}
