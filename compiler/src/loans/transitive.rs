use super::access::Kind;
use super::storage::EventKind;
use super::{Bundle, Expr, ExprKind, Graph, Node, Origin, Place, Result, Source, Span, Step, TRUE};
use crate::hir::ReferenceMode;

impl Graph<'_> {
    pub(crate) fn copied(&mut self, source: &Bundle, target: &Bundle) -> Result<Node> {
        self.charge(source.len() + target.len() + 1)?;
        let mut node = Node {
            defs: target.values().copied().collect(),
            ..Node::default()
        };
        for (path, id) in source {
            self.charge(path.len() + 1)?;
            if let Some(target) = target.get(path) {
                self.copy_link(&mut node, *target, *id, TRUE)?;
            }
            if path.contains(&Step::Deref) {
                if let Some(target) = target.get(path) {
                    node.transfers.push((*target, *id, TRUE));
                }
            } else {
                node.uses.push(*id);
            }
        }
        Ok(node)
    }

    pub(crate) fn direct(&mut self, value: &Bundle) -> Result<Vec<usize>> {
        let mut result = Vec::new();
        for (path, id) in value {
            self.charge(path.len() + 1)?;
            if !path.contains(&Step::Deref) {
                result.push(*id);
            }
        }
        Ok(result)
    }

    pub(crate) fn borrowed(
        &mut self,
        place: &Place,
        span: Span,
        mode: ReferenceMode,
    ) -> Result<Bundle> {
        self.charge(place.fields.len() + 1)?;
        let path = place
            .fields
            .iter()
            .map(|index| Step::Slot(index + 1))
            .collect::<Vec<_>>();
        let stored = Self::select(self.local(place.root)?, &path);
        self.referenced(self.proofs.source(place), stored, Vec::new(), span, mode)
    }

    pub(crate) fn referenced(
        &mut self,
        source: Source,
        stored: Bundle,
        uses: Vec<usize>,
        span: Span,
        mode: ReferenceMode,
    ) -> Result<Bundle> {
        self.charge(stored.len() + uses.len() + 1)?;
        let owner = match &source {
            Source::Local { id, .. } | Source::Temporary { id, .. } => Some(*id),
            Source::Slot { root, .. } => Some(*root),
            _ => None,
        };
        let temporary = matches!(source, Source::Temporary { .. });
        if mode == ReferenceMode::Exclusive
            && let Source::Slot { view, .. } = &source
        {
            crate::borrow::carried::storage(self.proofs, *view, self.guards, span)?;
        }
        let emission = self.emission_acquire(&source, span)?;
        let pointer = self.value(vec![Origin {
            component: Vec::new(),
            source,
            guard: TRUE,
        }])?;
        let mut result = Bundle::from([(Vec::new(), pointer)]);
        let mut node = Node {
            uses,
            defs: vec![pointer],
            ..Node::default()
        };
        if let Some(event) = emission {
            node.emissions.push(event);
        }
        if let Some(id) = owner {
            if temporary {
                self.event(&mut node, EventKind::Init(id), span)?;
            }
            self.event(&mut node, EventKind::Use { id, take: false }, span)?;
        }
        if !stored.is_empty() {
            node.barrier = Some(span);
        }
        for (path, source) in stored {
            self.charge(
                path.len()
                    + self.values[source]
                        .iter()
                        .map(Origin::weight)
                        .sum::<usize>()
                    + 1,
            )?;
            let target = self.value(self.values[source].clone())?;
            let path = std::iter::once(Step::Deref).chain(path).collect();
            result.insert(path, target);
            node.defs.push(target);
            node.transfers.push((target, source, TRUE));
            self.copy_link(&mut node, target, source, TRUE)?;
        }
        let access = self.access(
            Kind::Borrow,
            super::access::Target::Pointee(pointer),
            &[],
            span,
        )?;
        node.access = Some(access);
        let node = self.append(node)?;
        self.grant_mode(node, pointer, None, mode)?;
        Ok(result)
    }

    pub(crate) fn dereferenced(&mut self, value: &Expr, path: &[Step]) -> Result<Bundle> {
        let parent = self.reference_value(value)?;
        if self.current.is_empty() {
            return Ok(Bundle::new());
        }
        let uses = self.direct(&parent)?;
        let access = self.pointee_access(&parent, path, Kind::Read, value.span)?;
        let prefix = std::iter::once(Step::Deref)
            .chain(path.iter().copied())
            .collect::<Vec<_>>();
        self.charge(prefix.len() + parent.len())?;
        let source = Self::select(parent, &prefix);
        let mut result = Bundle::new();
        for (path, id) in &source {
            self.charge(
                path.len() + self.values[*id].iter().map(Origin::weight).sum::<usize>() + 1,
            )?;
            result.insert(path.clone(), self.value(self.values[*id].clone())?);
        }
        let mut node = self.copied(&source, &result)?;
        node.uses.extend(uses);
        node.access = Some(access);
        self.append(node)?;
        Ok(result)
    }

    pub(crate) fn derived(&mut self, expr: &Expr, path: &[Step]) -> Result<Bundle> {
        let (site, value) = match &expr.kind {
            ExprKind::Reborrow { site, value, .. }
            | ExprKind::ElementBorrow { site, value, .. } => (*site, value),
            _ => unreachable!(),
        };
        let parent = self.reference_value(value)?;
        if self.current.is_empty() {
            return Ok(Bundle::new());
        }
        let mut uses = self.direct(&parent)?;
        if let ExprKind::ElementBorrow { index, .. } = &expr.kind {
            uses.extend(self.expression(index)?.into_values());
        }
        if self.current.is_empty() {
            return Ok(Bundle::new());
        }
        let Some(state) = self.facts.reborrows.get(&site) else {
            let node = self.append(Node {
                uses,
                ..Node::default()
            })?;
            self.missing_reborrows.push((node, expr.span));
            self.assume(super::FALSE)?;
            return Ok(Bundle::new());
        };
        self.charge(state.weight() + 1)?;
        let result = self.bundle(super::values::Value::from(state), &expr.ty)?;
        let mut node = Node {
            uses,
            defs: result.values().copied().collect(),
            ..Node::default()
        };
        if let ExprKind::Reborrow { fields, .. } = &expr.kind {
            let prefix = std::iter::once(Step::Deref)
                .chain(fields.iter().map(|index| Step::Slot(index + 1)))
                .collect::<Vec<_>>();
            for (component, target) in &result {
                self.charge(component.len() + prefix.len() + 1)?;
                if let Some(tail) = component.strip_prefix(&[Step::Deref]) {
                    let source = prefix.iter().chain(tail).copied().collect::<Vec<_>>();
                    let source = parent.get(&source).ok_or_else(|| {
                        crate::diagnostic::Diagnostic::unsupported(
                            "missing shared pointee loan transfer",
                            expr.span,
                        )
                    })?;
                    node.transfers.push((*target, *source, TRUE));
                    self.copy_link(&mut node, *target, *source, TRUE)?;
                }
            }
        }
        node.access = Some(self.pointee_access(&result, &[], Kind::Borrow, expr.span)?);
        let root = result.get(&Vec::new()).copied();
        let parent = parent.get(&Vec::new()).copied();
        if parent.is_none()
            && let Some(root) = root
        {
            self.opaque_value(&mut node, root)?;
        }
        let node = self.append(node)?;
        if let (Some(root), Some(parent)) = (root, parent) {
            self.grant_mode(
                node,
                root,
                Some(parent),
                expr.ty.reference_mode().ok_or_else(Self::budget)?,
            )?;
        }
        Ok(Self::select(result, path))
    }
}
