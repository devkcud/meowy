use super::{
    BTreeMap, Bundle, CallId, Diagnostic, Expr, ExprKind, FALSE, Graph, LocalId, MAX_ORIGINS,
    MAX_VALUES, Node, Origin, Path, Result, Span, Step, Type,
};

impl<'a> Graph<'a> {
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
        let node = self.copied(&source, &value)?;
        self.append(node)?;
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
            ExprKind::Deref(value) => {
                let value = self.expression(value)?;
                let uses = self.direct(&value)?;
                self.append(Node {
                    uses,
                    ..Node::default()
                })?;
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
            ExprKind::Borrow(place) => Self::select(self.borrowed(place)?, path),
            ExprKind::Reborrow { .. } | ExprKind::ElementBorrow { .. } => {
                self.derived(expr, path)?
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
            ExprKind::Deref(value) => self.dereferenced(value, path)?,
            ExprKind::Unary { value, .. }
            | ExprKind::StringSize(value)
            | ExprKind::ListSize(value) => {
                let value = self.expression(value)?;
                let uses = self.direct(&value)?;
                self.append(Node {
                    uses,
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
                    let uses = self.direct(&value)?;
                    self.append(Node {
                        uses,
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
                let mut uses = self.direct(&left)?;
                uses.extend(self.direct(&right)?);
                self.append(Node {
                    uses,
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
                let mut uses = self.direct(&left)?;
                uses.extend(self.direct(&right)?);
                self.append(Node {
                    uses,
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
                    let uses = self.direct(&value)?;
                    self.append(Node {
                        uses,
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
}
