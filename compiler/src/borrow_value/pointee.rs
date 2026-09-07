use super::{Flow, MAX_PARTS, Origin, Projection, Result, Source, Span, State, Step, Type};

impl State {
    pub(crate) fn borrowed(
        self,
        source: Source,
        ty: &Type,
        flow: &mut Flow,
        span: Span,
    ) -> Result<Self> {
        if !flow.spend(self.weight() + self.size() + 1) {
            return Err(Self::budget(span));
        }
        let mut state = if ty.has_reference() {
            self.prefix(&[Step::Deref])
        } else {
            Self::default()
        };
        if state.size() >= MAX_PARTS {
            return Err(Self::budget(span));
        }
        state.origins.push(Origin {
            component: Vec::new(),
            source,
            guard: state.present,
        });
        Ok(state)
    }

    pub(crate) fn dereferenced(&self, ty: &Type, flow: &mut Flow, span: Span) -> Result<Self> {
        if ty.has_reference() {
            let state = self.select(&[Step::Deref], flow);
            if flow.exceeded() {
                return Err(Self::budget(span));
            }
            Ok(state)
        } else {
            Self::unknown(ty, flow, span)
        }
    }

    pub(crate) fn reborrowed(
        &self,
        fields: &[Projection],
        ty: &Type,
        flow: &mut Flow,
        span: Span,
    ) -> Result<Self> {
        if !flow.spend(self.weight() + fields.len().saturating_mul(self.size() + 1)) {
            return Err(Self::budget(span));
        }
        let mut state = if ty.has_reference() {
            let mut path = vec![Step::Deref];
            for field in fields {
                let Projection::Field(index) = field else {
                    return Err(Self::budget(span));
                };
                path.push(Step::Slot(index + 1));
            }
            self.select(&path, flow).prefix(&[Step::Deref])
        } else {
            Self {
                present: self.present,
                proof: self.proof,
                ..Self::default()
            }
        };
        for origin in self
            .origins
            .iter()
            .filter(|origin| origin.component.is_empty())
        {
            state.origins.push(Origin {
                component: Vec::new(),
                source: origin.source.project(fields),
                guard: origin.guard,
            });
            if state.size() > MAX_PARTS {
                return Err(Self::budget(span));
            }
        }
        for bound in self
            .bounds
            .iter()
            .filter(|bound| bound.component.is_empty())
        {
            state.bounds.push(bound.clone());
            if state.size() > MAX_PARTS {
                return Err(Self::budget(span));
            }
        }
        if flow.exceeded() {
            return Err(Self::budget(span));
        }
        Ok(state)
    }
}
