use crate::ast::Span;
use crate::diagnostic::Diagnostic;
use crate::flow::{FALSE, Flow, Guard, TRUE};
use crate::hir::{LocalId, Place, Type};

pub(crate) const MAX_PARTS: usize = 4_096;
pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
pub(crate) type Path = Vec<Step>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Step {
    Slot(usize),
    Variant(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Projection {
    Field(usize),
    Element,
}

#[derive(Clone)]
pub(crate) struct Origin {
    pub(crate) component: Path,
    pub(crate) source: Source,
    pub(crate) guard: Guard,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Source {
    Local {
        id: LocalId,
        fields: Vec<Projection>,
    },
    Input {
        id: LocalId,
        component: Path,
        fields: Vec<Projection>,
    },
}

impl Source {
    pub(crate) fn local(place: &Place) -> Self {
        Self::Local {
            id: place.root,
            fields: place
                .fields
                .iter()
                .copied()
                .map(Projection::Field)
                .collect(),
        }
    }

    pub(crate) fn project(&self, path: &[Projection]) -> Self {
        let mut source = self.clone();
        let fields = match &mut source {
            Self::Local { fields, .. } | Self::Input { fields, .. } => fields,
        };
        fields.extend_from_slice(path);
        source
    }
}

impl Origin {
    pub(crate) fn weight(&self) -> usize {
        1 + self.component.len()
            + match &self.source {
                Source::Local { fields, .. } => fields.len(),
                Source::Input {
                    component, fields, ..
                } => component.len() + fields.len(),
            }
    }
}

#[derive(Clone)]
pub(crate) struct Active {
    pub(crate) component: Path,
    pub(crate) member: usize,
    pub(crate) guard: Guard,
}

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) origins: Vec<Origin>,
    pub(crate) bounds: Vec<Origin>,
    pub(crate) active: Vec<Active>,
    pub(crate) present: Guard,
    pub(crate) proof: Guard,
}

impl Default for State {
    fn default() -> Self {
        Self {
            origins: Vec::new(),
            bounds: Vec::new(),
            active: Vec::new(),
            present: TRUE,
            proof: TRUE,
        }
    }
}

impl State {
    pub(crate) fn absent() -> Self {
        Self {
            present: FALSE,
            ..Self::default()
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.origins.len() + self.bounds.len() + self.active.len()
    }

    pub(crate) fn weight(&self) -> usize {
        self.origins
            .iter()
            .chain(&self.bounds)
            .map(Origin::weight)
            .sum::<usize>()
            + self
                .active
                .iter()
                .map(|active| 1 + active.component.len())
                .sum::<usize>()
    }

    pub(crate) fn budget(span: Span) -> Diagnostic {
        Diagnostic::unsupported("borrow-origin budget exhausted", span)
    }

    pub(crate) fn member(&self, path: &[Step], member: usize, flow: &mut Flow) -> Guard {
        flow.spend(self.active.len() + path.len());
        let mut guard = FALSE;
        for active in &self.active {
            if active.component == path && active.member == member {
                guard = flow.or(guard, active.guard);
            }
        }
        guard
    }

    pub(crate) fn under(mut self, guard: Guard, flow: &mut Flow) -> Self {
        flow.spend(self.size());
        self.present = flow.and(self.present, guard);
        let absent = flow.not(guard);
        self.proof = flow.or(absent, self.proof);
        for origin in self.origins.iter_mut().chain(&mut self.bounds) {
            origin.guard = flow.and(origin.guard, guard);
        }
        for active in &mut self.active {
            active.guard = flow.and(active.guard, guard);
        }
        self.origins.retain(|origin| origin.guard != FALSE);
        self.bounds.retain(|origin| origin.guard != FALSE);
        self.active.retain(|active| active.guard != FALSE);
        self
    }

    pub(crate) fn prefix(mut self, prefix: &[Step]) -> Self {
        for origin in self.origins.iter_mut().chain(&mut self.bounds) {
            origin.component.splice(0..0, prefix.iter().copied());
        }
        for active in &mut self.active {
            active.component.splice(0..0, prefix.iter().copied());
        }
        self
    }

    pub(crate) fn select(&self, path: &[Step], flow: &mut Flow) -> Self {
        flow.spend(self.weight() + path.len());
        let mut present = self.present;
        for (index, step) in path.iter().enumerate() {
            if let Step::Variant(member) = step {
                let guard = self.member(&path[..index], *member, flow);
                present = flow.and(present, guard);
            }
        }
        Self {
            origins: self
                .origins
                .iter()
                .filter_map(|origin| {
                    origin.component.strip_prefix(path).map(|component| Origin {
                        component: component.to_vec(),
                        source: origin.source.clone(),
                        guard: origin.guard,
                    })
                })
                .collect(),
            bounds: self
                .bounds
                .iter()
                .filter_map(|origin| {
                    origin.component.strip_prefix(path).map(|component| Origin {
                        component: component.to_vec(),
                        source: origin.source.clone(),
                        guard: origin.guard,
                    })
                })
                .collect(),
            active: self
                .active
                .iter()
                .filter_map(|active| {
                    active.component.strip_prefix(path).map(|component| Active {
                        component: component.to_vec(),
                        member: active.member,
                        guard: active.guard,
                    })
                })
                .collect(),
            present,
            proof: self.proof,
        }
        .under(present, flow)
    }

    pub(crate) fn merge(&mut self, other: Self, flow: &mut Flow, span: Span) -> Result<()> {
        if !flow.spend((self.weight() + other.weight()).saturating_mul(other.size().max(1))) {
            return Err(Self::budget(span));
        }
        self.present = flow.or(self.present, other.present);
        self.proof = flow.and(self.proof, other.proof);
        for origin in other.origins {
            if let Some(prior) = self
                .origins
                .iter_mut()
                .find(|prior| prior.component == origin.component && prior.source == origin.source)
            {
                prior.guard = flow.or(prior.guard, origin.guard);
            } else {
                self.origins.push(origin);
            }
            if self.size() > MAX_PARTS {
                return Err(Self::budget(span));
            }
        }
        for bound in other.bounds {
            if let Some(prior) = self
                .bounds
                .iter_mut()
                .find(|prior| prior.component == bound.component && prior.source == bound.source)
            {
                prior.guard = flow.or(prior.guard, bound.guard);
            } else {
                self.bounds.push(bound);
            }
            if self.size() > MAX_PARTS {
                return Err(Self::budget(span));
            }
        }
        for active in other.active {
            if let Some(prior) = self
                .active
                .iter_mut()
                .find(|prior| prior.component == active.component && prior.member == active.member)
            {
                prior.guard = flow.or(prior.guard, active.guard);
            } else {
                self.active.push(active);
            }
            if self.size() > MAX_PARTS {
                return Err(Self::budget(span));
            }
        }
        Ok(())
    }

    pub(crate) fn inject(self, member: usize, flow: &mut Flow, span: Span) -> Result<Self> {
        if !flow.spend(self.weight() + self.size()) {
            return Err(Self::budget(span));
        }
        let present = self.present;
        let mut state = self.prefix(&[Step::Variant(member)]);
        state.active.push(Active {
            component: Vec::new(),
            member,
            guard: present,
        });
        if state.size() > MAX_PARTS || flow.exceeded() {
            return Err(Self::budget(span));
        }
        Ok(state)
    }

    pub(crate) fn convert(
        self,
        from: &Type,
        to: &Type,
        flow: &mut Flow,
        span: Span,
    ) -> Result<Self> {
        if from == to {
            return Ok(self);
        }
        if *from == Type::Never {
            return Ok(Self::absent());
        }
        if let Type::Union(members) = from {
            let mut result = Self::absent();
            for (index, member) in members.iter().enumerate() {
                if !to.accepts(member) {
                    continue;
                }
                let state = self.select(&[Step::Variant(index)], flow);
                let state = if let Type::Union(targets) = to {
                    let target = targets
                        .iter()
                        .position(|ty| ty == member)
                        .expect("union member");
                    state.inject(target, flow, span)?
                } else {
                    state
                };
                result.merge(state, flow, span)?;
            }
            return Ok(result);
        }
        if let Type::Union(members) = to {
            let member = members.iter().position(|ty| ty == from).ok_or_else(|| {
                Diagnostic::unsupported("unknown borrowed union conversion", span)
            })?;
            return self.inject(member, flow, span);
        }
        Err(Diagnostic::unsupported(
            "unknown borrowed value conversion",
            span,
        ))
    }

    pub(crate) fn unknown(ty: &Type, flow: &mut Flow, span: Span) -> Result<Self> {
        let mut result = Self::default();
        let mut pending = vec![(ty, Vec::new(), TRUE)];
        while let Some((ty, path, present)) = pending.pop() {
            match ty {
                Type::Record { primary, fields } => {
                    for (index, ty) in std::iter::once(primary.as_ref())
                        .chain(fields.iter().map(|(_, ty)| ty))
                        .enumerate()
                    {
                        let mut path = path.clone();
                        path.push(Step::Slot(index));
                        let work = path.len() + 1;
                        pending.push((ty, path, present));
                        if result.size() + pending.len() > MAX_PARTS || !flow.spend(work) {
                            return Err(Self::budget(span));
                        }
                    }
                }
                Type::Union(members) => {
                    let mut rest = present;
                    for (index, ty) in members.iter().enumerate() {
                        let guard = if index + 1 == members.len() {
                            rest
                        } else {
                            let fresh = flow.fresh();
                            let guard = flow.and(rest, fresh);
                            let absent = flow.not(fresh);
                            rest = flow.and(rest, absent);
                            guard
                        };
                        result.active.push(Active {
                            component: path.clone(),
                            member: index,
                            guard,
                        });
                        let mut nested = path.clone();
                        nested.push(Step::Variant(index));
                        pending.push((ty, nested, guard));
                        if result.size() + pending.len() > MAX_PARTS
                            || !flow.spend((path.len() + 1) * 2)
                        {
                            return Err(Self::budget(span));
                        }
                    }
                }
                _ => {}
            }
            if result.size() + pending.len() > MAX_PARTS || flow.exceeded() {
                return Err(Self::budget(span));
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub(crate) fn wide_unknown_tag_domains_stop_before_expanding_every_member() {
        let members: String = (0..4097)
            .map(|id| format!("<{{field{id}<int32>}}>"))
            .collect();
        let source = format!("f<null>:(value{members}){{}}");
        let errors = crate::compile(&source).unwrap_err();
        assert_eq!(errors[0].code, "B001", "{errors:?}");
        assert!(errors[0].message.contains("budget"));
    }

    #[test]
    pub(crate) fn inactive_payloads_still_count_toward_activity_proof_budgets() {
        let fields: String = (0..64)
            .map(|id| format!("field{id}<&int32><null>;"))
            .collect();
        let mut source = format!("value<{{{fields}}}>:{{}};");
        for id in 0..512 {
            source.push_str(&format!("copy{id}:value;"));
        }
        let errors = crate::compile(&source).unwrap_err();
        assert_eq!(errors[0].code, "B001", "{errors:?}");
        assert!(errors[0].message.contains("budget"));
    }
}
