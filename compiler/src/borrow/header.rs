use super::{
    BTreeSet, Diagnostic, Guards, MAX_ORIGINS, Path, Result, Span, State, Step, TRUE, Type,
};

pub(crate) struct Shape {
    pub(crate) paths: BTreeSet<Path>,
}

impl Shape {
    pub(crate) fn new(ty: &Type, guards: &mut Guards, span: Span) -> Result<Self> {
        if !matches!(ty, Type::Reference(_)) {
            return Err(Diagnostic::unsupported(
                "restart header requires a fixed reference type",
                span,
            ));
        }
        crate::borrow_contract::type_weight(ty, guards, span)?;
        let mut paths = BTreeSet::new();
        let mut pending = vec![(ty, Path::new())];
        while let Some((ty, path)) = pending.pop() {
            let lookup = paths.len().checked_ilog2().unwrap_or(0) as usize + 1;
            if !guards.spend((path.len() + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            match ty {
                Type::Reference(target) => {
                    if paths.len() + pending.len() >= MAX_ORIGINS {
                        return Err(State::budget(span));
                    }
                    paths.insert(path.clone());
                    if target.has_reference() {
                        if paths.len() + pending.len() >= MAX_ORIGINS
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
                        if paths.len() + pending.len() >= MAX_ORIGINS
                            || !guards.spend(path.len() + 1)
                        {
                            return Err(State::budget(span));
                        }
                        let mut nested = path.clone();
                        nested.push(Step::Slot(index));
                        pending.push((field, nested));
                    }
                }
                Type::Union(_) => {
                    return Err(Diagnostic::unsupported(
                        "active-variant restart header summaries",
                        span,
                    ));
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
        Ok(Self { paths })
    }

    pub(crate) fn validate(
        &self,
        state: &State,
        canonical: bool,
        guards: &mut Guards,
        span: Span,
    ) -> Result<()> {
        if state.size() > MAX_ORIGINS || !guards.spend(state.weight() + self.paths.len() + 1) {
            return Err(State::budget(span));
        }
        if !state.active.is_empty() || (canonical && (state.present != TRUE || state.proof != TRUE))
        {
            return Err(Diagnostic::unsupported(
                "noncanonical restart header activity",
                span,
            ));
        }
        let mut covered = BTreeSet::new();
        let lookup = self.paths.len().checked_ilog2().unwrap_or(0) as usize + 1;
        for origin in state.origins.iter().chain(&state.bounds) {
            if !guards.spend((origin.component.len() + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            if !self.paths.contains(&origin.component) || (canonical && origin.guard != TRUE) {
                return Err(Diagnostic::unsupported(
                    "unknown restart header component",
                    span,
                ));
            }
        }
        if canonical {
            for origin in &state.origins {
                if !guards.spend((origin.component.len() + 1).saturating_mul(lookup)) {
                    return Err(State::budget(span));
                }
                covered.insert(origin.component.clone());
            }
            if covered != self.paths {
                return Err(Diagnostic::unsupported(
                    "incomplete restart header reference coverage",
                    span,
                ));
            }
        }
        Ok(())
    }
}
