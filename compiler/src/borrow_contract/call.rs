use super::{Leaf, component_type, leaves, projections, reference_leaves, type_weight};
use crate::ast::Span;
use crate::borrow_value::{MAX_PARTS, Origin, Projection, Result, State, Step};
use crate::flow::{FALSE, Flow, Guard};
use crate::hir::Type;

pub(crate) struct Candidate<'a> {
    pub(crate) state: &'a State,
    pub(crate) origin: &'a Origin,
    pub(crate) fields: Vec<Projection>,
    pub(crate) guard: Guard,
}

pub(crate) fn attach(
    result: &mut State,
    leaf: &Leaf<'_>,
    candidates: Vec<Candidate<'_>>,
    flow: &mut Flow,
    span: Span,
) -> Result<()> {
    let Type::Reference(target) = leaf.ty else {
        return Err(State::budget(span));
    };
    let mut rest = leaf.guard;
    let count = candidates.len();
    for (index, candidate) in candidates.into_iter().enumerate() {
        let chosen = if index + 1 == count {
            rest
        } else {
            let fresh = flow.fresh();
            let chosen = flow.and(rest, fresh);
            rest = flow.and(rest, flow.not(fresh));
            chosen
        };
        let mut source = candidate
            .state
            .select(&candidate.origin.component, flow)
            .under(candidate.guard, flow);
        source.origins.retain(|origin| !origin.component.is_empty());
        source.origins.push(Origin {
            component: Vec::new(),
            source: candidate.origin.source.clone(),
            guard: candidate.guard,
        });
        let mut state = source.reborrowed(&candidate.fields, target, flow, span)?;
        let available = flow.and(state.present, candidate.guard);
        let valid = flow.or(flow.not(chosen), available);
        result.proof = flow.and(result.proof, valid);
        state.bounds.clear();
        if !flow.spend(state.weight() + state.size().saturating_mul(leaf.component.len())) {
            return Err(State::budget(span));
        }
        result.merge(
            state.under(chosen, flow).prefix(&leaf.component),
            flow,
            span,
        )?;
    }
    Ok(())
}

pub(crate) fn call(
    ty: &Type,
    args: &[(&Type, State)],
    flow: &mut Flow,
    span: Span,
) -> Result<State> {
    let mut result = State::unknown(ty, flow, span)?;
    if !ty.has_reference() {
        return Ok(result);
    }
    if !flow.spend(result.weight() + result.size()) {
        return Err(State::budget(span));
    }
    result
        .active
        .retain(|active| !active.component.contains(&Step::Deref));
    let mut inputs = Vec::new();
    let mut bounds = Vec::new();
    for (ty, state) in args {
        result.proof = flow.and(result.proof, state.proof);
        for origin in &state.origins {
            let source =
                component_type(ty, &origin.component).ok_or_else(|| State::budget(span))?;
            if !matches!(source, Type::Reference(_)) {
                return Err(State::budget(span));
            }
            let guard = flow.and(state.present, origin.guard);
            if guard != FALSE {
                let weight = type_weight(source, flow, span)?;
                inputs.push((source, state, origin, guard, weight));
            }
            if inputs.len() > MAX_PARTS {
                return Err(State::budget(span));
            }
        }
        for bound in state.origins.iter().chain(&state.bounds) {
            let guard = flow.and(state.present, bound.guard);
            if guard != FALSE {
                bounds.push((bound, guard));
            }
            if bounds.len() > MAX_PARTS {
                return Err(State::budget(span));
            }
        }
    }
    for leaf in reference_leaves(ty, &result, flow, span, false)? {
        let Type::Reference(target) = leaf.ty else {
            return Err(State::budget(span));
        };
        let weight = type_weight(leaf.ty, flow, span)?;
        let mut covered = FALSE;
        let mut candidates = Vec::new();
        for (ty, state, origin, guard, source_weight) in &inputs {
            if !flow.spend(weight + source_weight + origin.weight() + leaf.component.len()) {
                return Err(State::budget(span));
            }
            for fields in projections(ty, leaf.ty, flow, span)? {
                let active = flow.and(leaf.guard, *guard);
                if active == FALSE {
                    continue;
                }
                if !flow.spend(origin.weight() + fields.len() + leaf.component.len()) {
                    return Err(State::budget(span));
                }
                if target.has_reference() {
                    candidates.push(Candidate {
                        state,
                        origin,
                        fields,
                        guard: *guard,
                    });
                } else {
                    result.origins.push(Origin {
                        component: leaf.component.clone(),
                        source: origin.source.project(&fields),
                        guard: active,
                    });
                }
                covered = flow.or(covered, *guard);
                if candidates.len() > MAX_PARTS || result.size() > MAX_PARTS {
                    return Err(State::budget(span));
                }
            }
        }
        let supplied = flow.or(flow.not(leaf.guard), covered);
        result.proof = flow.and(result.proof, supplied);
        if target.has_reference() {
            attach(&mut result, &leaf, candidates, flow, span)?;
        }
    }
    for leaf in leaves(ty, &result, flow, span)? {
        for (bound, guard) in &bounds {
            if !flow.spend(bound.weight() + leaf.component.len() + 1) {
                return Err(State::budget(span));
            }
            let active = flow.and(leaf.guard, *guard);
            if active != FALSE {
                result.bounds.push(Origin {
                    component: leaf.component.clone(),
                    source: bound.source.clone(),
                    guard: active,
                });
            }
            if result.size() > MAX_PARTS {
                return Err(State::budget(span));
            }
        }
    }
    if flow.exceeded() || !flow.spend(result.weight()) {
        return Err(State::budget(span));
    }
    Ok(result)
}
