mod call;

pub(crate) use call::call;

use crate::ast::Span;
use crate::borrow_value::{MAX_PARTS, Origin, Path, Projection, Result, Source, State, Step};
use crate::flow::{Flow, Guard};
use crate::hir::{LocalId, Type};

pub(crate) struct Leaf<'a> {
    pub(crate) component: Path,
    pub(crate) ty: &'a Type,
    pub(crate) guard: Guard,
}

pub(crate) fn leaves<'a>(
    ty: &'a Type,
    state: &State,
    flow: &mut Flow,
    span: Span,
) -> Result<Vec<Leaf<'a>>> {
    reference_leaves(ty, state, flow, span, true)
}

pub(crate) fn reference_leaves<'a>(
    ty: &'a Type,
    state: &State,
    flow: &mut Flow,
    span: Span,
    nested: bool,
) -> Result<Vec<Leaf<'a>>> {
    let mut result = Vec::new();
    let mut pending = vec![(ty, Path::new(), state.present)];
    while let Some((ty, path, guard)) = pending.pop() {
        if !flow.spend(path.len() + 1) {
            return Err(State::budget(span));
        }
        match ty {
            Type::Reference(target) => {
                if nested && target.has_reference() {
                    let mut inner = path.clone();
                    inner.push(Step::Deref);
                    pending.push((target.as_ref(), inner, guard));
                }
                result.push(Leaf {
                    component: path,
                    ty,
                    guard,
                });
            }
            Type::Record { primary, fields } => {
                for (index, ty) in std::iter::once(primary.as_ref())
                    .chain(fields.iter().map(|field| &field.ty))
                    .enumerate()
                {
                    if ty.has_reference() {
                        let mut path = path.clone();
                        path.push(Step::Slot(index));
                        pending.push((ty, path, guard));
                        if pending.len() + result.len() > MAX_PARTS {
                            return Err(State::budget(span));
                        }
                    }
                }
            }
            Type::Union(members) => {
                for (index, ty) in members.iter().enumerate() {
                    if ty.has_reference() {
                        let active = state.member(&path, index, flow);
                        let guard = flow.and(guard, active);
                        let mut path = path.clone();
                        path.push(Step::Variant(index));
                        pending.push((ty, path, guard));
                        if pending.len() + result.len() > MAX_PARTS {
                            return Err(State::budget(span));
                        }
                    }
                }
            }
            _ => {}
        }
        if pending.len() + result.len() > MAX_PARTS {
            return Err(State::budget(span));
        }
    }
    Ok(result)
}

pub(crate) fn input(id: LocalId, ty: &Type, flow: &mut Flow, span: Span) -> Result<State> {
    let mut state = State::unknown(ty, flow, span)?;
    for leaf in leaves(ty, &state, flow, span)? {
        state.origins.push(Origin {
            component: leaf.component.clone(),
            source: Source::Input {
                id,
                component: leaf.component,
                fields: Vec::new(),
            },
            guard: leaf.guard,
        });
        if state.size() > MAX_PARTS {
            return Err(State::budget(span));
        }
    }
    Ok(state)
}

pub(crate) fn component_type<'a>(mut ty: &'a Type, path: &[Step]) -> Option<&'a Type> {
    for step in path {
        ty = match (step, ty) {
            (Step::Slot(0), Type::Record { primary, .. }) => primary,
            (Step::Slot(index), Type::Record { fields, .. }) => &fields.get(index - 1)?.ty,
            (Step::Variant(index), Type::Union(members)) => members.get(*index)?,
            (Step::Deref, Type::Reference(target)) => target,
            _ => return None,
        };
    }
    Some(ty)
}

pub(crate) fn projected_type<'a>(mut ty: &'a Type, path: &[Projection]) -> Option<&'a Type> {
    for step in path {
        ty = match (step, ty) {
            (Projection::Field(index), Type::Record { fields, .. }) => &fields.get(*index)?.ty,
            (Projection::Element, Type::List { element, .. }) => element,
            _ => return None,
        };
    }
    Some(ty)
}

pub(crate) fn projections(
    from: &Type,
    to: &Type,
    flow: &mut Flow,
    span: Span,
) -> Result<Vec<Vec<Projection>>> {
    let (Type::Reference(from), Type::Reference(to)) = (from, to) else {
        return Ok(Vec::new());
    };
    let mut result = Vec::new();
    let mut pending = vec![(from.as_ref(), Vec::new())];
    while let Some((ty, path)) = pending.pop() {
        let work = type_weight(ty, flow, span)? + type_weight(to, flow, span)? + path.len() + 1;
        if !flow.spend(work) {
            return Err(State::budget(span));
        }
        if ty == to.as_ref() {
            result.push(path.clone());
        }
        let mut push = |step, field| -> Result<()> {
            if pending.len() + result.len() >= MAX_PARTS {
                return Err(State::budget(span));
            }
            if !flow.spend(path.len() + 1) {
                return Err(State::budget(span));
            }
            let mut nested = path.clone();
            nested.push(step);
            pending.push((field, nested));
            Ok(())
        };
        match ty {
            Type::Record { fields, .. } => {
                for (index, field) in fields.iter().enumerate() {
                    push(Projection::Field(index), &field.ty)?;
                }
            }
            Type::List { element, .. } => push(Projection::Element, element.as_ref())?,
            _ => {}
        }
    }
    Ok(result)
}

pub(crate) fn type_weight(ty: &Type, flow: &mut Flow, span: Span) -> Result<usize> {
    let mut count = 0;
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
        count += 1;
        if !flow.spend(1) {
            return Err(State::budget(span));
        }
        match ty {
            Type::Reference(ty) | Type::List { element: ty, .. } => pending.push(ty),
            Type::Record { primary, fields } => {
                pending.push(primary);
                for field in fields {
                    if pending.len() == MAX_PARTS || !flow.spend(1) {
                        return Err(State::budget(span));
                    }
                    pending.push(&field.ty);
                }
            }
            Type::Union(members) => {
                for ty in members {
                    if pending.len() == MAX_PARTS || !flow.spend(1) {
                        return Err(State::budget(span));
                    }
                    pending.push(ty);
                }
            }
            _ => {}
        }
        if pending.len() > MAX_PARTS {
            return Err(State::budget(span));
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests;
