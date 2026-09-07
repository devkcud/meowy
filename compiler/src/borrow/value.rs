use super::{Checker, Expr, ExprKind, FALSE, Flow, Projection, Result, State, Step, Type, Value};

impl Checker<'_> {
    pub(crate) fn expression(&mut self, expr: &Expr) -> Result<Value> {
        let mut flow = Flow::new();
        let state = match &expr.kind {
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
                        .ty;
                }
                if expr.ty != Type::Reference(Box::new(ty.clone())) {
                    return Err(Self::unsupported(expr.span));
                }
                let path = place
                    .fields
                    .iter()
                    .map(|index| Step::Slot(index + 1))
                    .collect::<Vec<_>>();
                let state = self.locals[&place.root].state.select(&path, self.guards);
                state.borrowed(self.proofs.source(place), ty, self.guards, expr.span)?
            }
            ExprKind::Reborrow { site, value, .. }
            | ExprKind::ElementBorrow { site, value, .. } => {
                let result = self.expression(value)?;
                flow = result.flow;
                let fields = match &expr.kind {
                    ExprKind::Reborrow { fields, .. } => {
                        if !self.guards.spend(fields.len() + 1) {
                            return Err(State::budget(expr.span));
                        }
                        fields
                            .iter()
                            .copied()
                            .map(Projection::Field)
                            .collect::<Vec<_>>()
                    }
                    ExprKind::ElementBorrow { index, .. } => {
                        if flow.next {
                            flow.append(self.expression(index)?.flow);
                        }
                        vec![Projection::Element]
                    }
                    _ => unreachable!(),
                };
                let Type::Reference(ty) = &value.ty else {
                    return Err(Self::unsupported(expr.span));
                };
                let target = crate::borrow_contract::projected_type(ty, &fields)
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                if expr.ty != Type::Never && expr.ty != Type::Reference(Box::new(target.clone())) {
                    return Err(Self::unsupported(expr.span));
                }
                let state = result
                    .state
                    .reborrowed(&fields, target, self.guards, expr.span)?;
                self.reserve_origins(state.weight() + 1, expr.span)?;
                self.facts.reborrows.insert(*site, state.clone());
                state
            }
            ExprKind::Local(id) => {
                if self.proofs.mutable.contains(id) {
                    State::unknown(&expr.ty, self.guards, expr.span)?
                } else {
                    let state = self
                        .locals
                        .get(id)
                        .map(|storage| storage.state.clone())
                        .ok_or_else(|| Self::unsupported(expr.span))?;
                    self.live_value(&state, expr.span)?;
                    state
                }
            }
            ExprKind::Coerce { value } => {
                let result = self.expression(value)?;
                flow = result.flow;
                result
                    .state
                    .convert(&value.ty, &expr.ty, self.guards, expr.span)?
            }
            ExprKind::Deref(_) | ExprKind::Field { .. } | ExprKind::Primary(_) => {
                let result = self.read(expr, &[])?;
                flow = result.flow;
                result.state
            }
            ExprKind::Unary { value, .. }
            | ExprKind::StringSize(value)
            | ExprKind::ListSize(value)
            | ExprKind::TypeTest { value, .. } => {
                flow = self.expression(value)?.flow;
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::List { values, .. } => {
                for value in values {
                    if !flow.next {
                        break;
                    }
                    flow.append(self.expression(value)?.flow);
                }
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::ListIndex { value, index } | ExprKind::ListAdd { value, item: index } => {
                flow = self.expression(value)?.flow;
                if flow.next {
                    flow.append(self.expression(index)?.flow);
                }
                State::unknown(&expr.ty, self.guards, expr.span)?
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
                State::unknown(&expr.ty, self.guards, expr.span)?
            }
            ExprKind::Call { site, args, .. } => {
                let mut inputs = Vec::new();
                for arg in args {
                    if !flow.next {
                        break;
                    }
                    let value = self.expression(arg)?;
                    inputs.push((&arg.ty, value.state));
                    flow.append(value.flow);
                }
                let state = if flow.next {
                    crate::borrow_contract::call(&expr.ty, &inputs, self.guards, expr.span)?
                } else {
                    State::absent()
                };
                let entered = self
                    .proofs
                    .calls
                    .get(site)
                    .copied()
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                let normal = self.guards.and(state.present, state.proof);
                let post = self.guards.or(self.guards.not(entered), normal);
                self.assumed = self.guards.and(self.assumed, post);
                self.reserve_origins(state.weight() + 1, expr.span)?;
                self.facts.calls.insert(*site, state.clone());
                state
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
                State::default()
            }
            ExprKind::Block(block) => {
                let value = self.block(block)?;
                flow = value.flow;
                value.state
            }
            ExprKind::Null
            | ExprKind::Bool(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_) => State::default(),
        };
        let assumptions = self.assumptions();
        let proof = self.guards.and(state.proof, assumptions);
        if self.guards.and(state.present, proof) == FALSE {
            flow.next = false;
        }
        if flow.next {
            self.complete(&expr.ty, &state, expr.span)?;
        }
        if expr.ty == Type::Never {
            flow.next = false;
        }
        Ok(Value { state, flow })
    }
}
