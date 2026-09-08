use super::{
    BTreeMap, BTreeSet, Diagnostic, FALSE, Guards, MAX_ORIGINS, Path, Result, Span, State, Step,
    TRUE, Type,
};

pub(crate) struct Shape {
    pub(crate) paths: BTreeSet<Path>,
    pub(crate) unions: BTreeMap<Path, usize>,
}

impl Shape {
    pub(crate) fn new(ty: &Type, guards: &mut Guards, span: Span) -> Result<Self> {
        if !matches!(ty, Type::Reference(_) | Type::Exclusive(_)) {
            return Err(Diagnostic::unsupported(
                "restart header requires a fixed reference type",
                span,
            ));
        }
        crate::borrow_contract::type_weight(ty, guards, span)?;
        let mut paths = BTreeSet::new();
        let mut unions = BTreeMap::new();
        let mut pending = vec![(ty, Path::new())];
        while let Some((ty, path)) = pending.pop() {
            let lookup = paths.len().checked_ilog2().unwrap_or(0) as usize + 1;
            if !guards.spend((path.len() + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            match ty {
                Type::Reference(target) | Type::Exclusive(target) => {
                    if paths.len() + unions.len() + pending.len() >= MAX_ORIGINS {
                        return Err(State::budget(span));
                    }
                    paths.insert(path.clone());
                    if target.has_borrowed() {
                        if paths.len() + unions.len() + pending.len() >= MAX_ORIGINS
                            || !guards.spend(path.len() + 1)
                        {
                            return Err(State::budget(span));
                        }
                        let mut path = path;
                        path.push(Step::Deref);
                        pending.push((target.as_ref(), path));
                    }
                }
                Type::Record { primary, fields } => {
                    for (index, field) in std::iter::once(primary.as_ref())
                        .chain(fields.iter().map(|field| &field.ty))
                        .enumerate()
                    {
                        if paths.len() + unions.len() + pending.len() >= MAX_ORIGINS
                            || !guards.spend(path.len() + 1)
                        {
                            return Err(State::budget(span));
                        }
                        let mut nested = path.clone();
                        nested.push(Step::Slot(index));
                        pending.push((field, nested));
                    }
                }
                Type::Union(members) => {
                    if paths.len() + unions.len() + pending.len() >= MAX_ORIGINS
                        || !guards.spend(path.len() + 1)
                    {
                        return Err(State::budget(span));
                    }
                    unions.insert(path.clone(), members.len());
                    for (index, member) in members.iter().enumerate() {
                        if paths.len() + unions.len() + pending.len() >= MAX_ORIGINS
                            || !guards.spend(path.len() + 1)
                        {
                            return Err(State::budget(span));
                        }
                        let mut nested = path.clone();
                        nested.push(Step::Variant(index));
                        pending.push((member, nested));
                    }
                }
                Type::List { element, .. } if element.has_reference() => {
                    return Err(Diagnostic::unsupported(
                        "reference-bearing list restart headers",
                        span,
                    ));
                }
                _ => {}
            }
        }
        Ok(Self { paths, unions })
    }

    pub(crate) fn validate(
        &self,
        state: &State,
        canonical: bool,
        guards: &mut Guards,
        span: Span,
    ) -> Result<()> {
        self.inspect(state, canonical, guards, span).map(|_| ())
    }

    pub(crate) fn inspect(
        &self,
        state: &State,
        canonical: bool,
        guards: &mut Guards,
        span: Span,
    ) -> Result<super::activity::Activity> {
        if state.size() > MAX_ORIGINS
            || !guards.spend(state.weight() + self.paths.len() + self.unions.len() + 1)
        {
            return Err(State::budget(span));
        }
        if canonical && (state.present != TRUE || state.proof != TRUE) {
            return Err(Diagnostic::unsupported(
                "noncanonical restart header activity",
                span,
            ));
        }
        let activity = super::activity::Activity::new(self, state, canonical, guards, span)?;
        let mut covered = BTreeMap::new();
        let lookup = self.paths.len().checked_ilog2().unwrap_or(0) as usize + 1;
        for origin in state.origins.iter().chain(&state.bounds) {
            if !guards.spend((origin.component.len() + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            let Some(active) = activity.paths.get(&origin.component) else {
                return Err(Diagnostic::unsupported(
                    "unknown restart header component",
                    span,
                ));
            };
            if canonical && (origin.guard != *active || *active == FALSE) {
                return Err(Diagnostic::unsupported(
                    "noncanonical restart origin activity",
                    span,
                ));
            }
        }
        for origin in &state.origins {
            if !guards.spend((origin.component.len() + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            let old = covered.get(&origin.component).copied().unwrap_or(FALSE);
            covered.insert(origin.component.clone(), guards.or(old, origin.guard));
        }
        for (path, active) in &activity.paths {
            if !guards.spend(path.len() + lookup) {
                return Err(State::budget(span));
            }
            let active = guards.and(*active, state.proof);
            if !guards.implies(active, covered.get(path).copied().unwrap_or(FALSE)) {
                return Err(Diagnostic::unsupported(
                    "incomplete restart header reference coverage",
                    span,
                ));
            }
        }
        Ok(activity)
    }
}
