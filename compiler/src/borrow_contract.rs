use crate::ast::Span;
use crate::borrow_value::{MAX_PARTS, Origin, Path, Result, Source, State, Step};
use crate::flow::{FALSE, Flow, Guard};
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
    let mut result = Vec::new();
    let mut pending = vec![(ty, Path::new(), state.present)];
    while let Some((ty, path, guard)) = pending.pop() {
        if !flow.spend(path.len() + 1) {
            return Err(State::budget(span));
        }
        match ty {
            Type::Reference(_) => result.push(Leaf {
                component: path,
                ty,
                guard,
            }),
            Type::Record { primary, fields } => {
                for (index, ty) in std::iter::once(primary.as_ref())
                    .chain(fields.iter().map(|(_, ty)| ty))
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
            (Step::Slot(index), Type::Record { fields, .. }) => &fields.get(index - 1)?.1,
            (Step::Variant(index), Type::Union(members)) => members.get(*index)?,
            _ => return None,
        };
    }
    Some(ty)
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
            Type::Reference(ty) => pending.push(ty),
            Type::Record { primary, fields } => {
                pending.push(primary);
                for (_, ty) in fields {
                    if pending.len() == MAX_PARTS || !flow.spend(1) {
                        return Err(State::budget(span));
                    }
                    pending.push(ty);
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
    let mut inputs = Vec::new();
    let mut bounds = Vec::new();
    for (ty, state) in args {
        result.proof = flow.and(result.proof, state.proof);
        for origin in &state.origins {
            let source =
                component_type(ty, &origin.component).ok_or_else(|| State::budget(span))?;
            let guard = flow.and(state.present, origin.guard);
            if guard != FALSE {
                let weight = type_weight(source, flow, span)?;
                inputs.push((source, origin, guard, weight));
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
    for leaf in leaves(ty, &result, flow, span)? {
        let weight = type_weight(leaf.ty, flow, span)?;
        let mut covered = FALSE;
        for (ty, origin, guard, source_weight) in &inputs {
            if !flow.spend(weight + source_weight + origin.weight() + leaf.component.len()) {
                return Err(State::budget(span));
            }
            if *ty == leaf.ty {
                let active = flow.and(leaf.guard, *guard);
                if active != FALSE {
                    result.origins.push(Origin {
                        component: leaf.component.clone(),
                        source: origin.source.clone(),
                        guard: active,
                    });
                    covered = flow.or(covered, *guard);
                }
                if result.size() > MAX_PARTS {
                    return Err(State::budget(span));
                }
            }
        }
        let supplied = flow.or(flow.not(leaf.guard), covered);
        result.proof = flow.and(result.proof, supplied);
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
    if !flow.spend(result.weight()) {
        return Err(State::budget(span));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    pub(crate) fn accepts(source: &str) {
        let result = crate::compile(source);
        assert!(result.is_ok(), "{source}: {result:?}");
    }

    pub(crate) fn rejects(source: &str, code: &str) {
        let errors = crate::compile(source).unwrap_err();
        assert_eq!(errors[0].code, code, "{source}: {errors:?}");
    }

    #[test]
    pub(crate) fn direct_functions_return_symbolic_inputs_and_carrier_components() {
        accepts("id<&int32>:(p<&int32>){->p};a:1;r:id(&a);v:*r");
        accepts("id:(p<&int32>){->p};a:1;r:id(&a);v:*r");
        accepts(
            "pick<&int32>:(p<{left<&int32>;right<&string>}>){->p.left};a:1;b:\"s\";r:pick({->left:&a;->right:&b});v:*r",
        );
        accepts(
            "copy<{view<&int32>;count<int32>}>:(p<&int32>){->view:p;->count:7};a:1;r:copy(&a);v:*r.view",
        );
        accepts("id<&int32><null>:(p<&int32><null>){->p};u:id(null);|u<&int32>|x:*u<&int32>");
    }

    #[test]
    pub(crate) fn ignored_and_transitive_inputs_still_bound_returned_views() {
        rejects(
            "first<&int32>:(p<&int32>,q<&string>){->p};wrap<&int32>:(p<&int32>){local:\"s\";->first(p,&local)}",
            "E303",
        );
        rejects(
            "first<&int32>:(p<&int32>,q<&string>){->p};id<&int32>:(p<&int32>){->p};wrap<&int32>:(p<&int32>){local:\"s\";r:first(p,&local);->id(r)}",
            "E303",
        );
        rejects(
            "pick<&int32>:(p<{left<&int32>;right<&string>}>){->p.left};wrap<&int32>:(p<&int32>){local:\"s\";pair:{->left:p;->right:&local};->pick(pair)}",
            "E303",
        );
        accepts(
            "first<&int32>:(p<&int32>,q<&string>){->p};wrap<int32>:(p<&int32>){local:\"s\";->*first(p,&local)}",
        );
        accepts("first<&int32>:(p<&int32>,q<&string><null>){->p};a:1;r:first(&a,null);x:*r");
    }

    #[test]
    pub(crate) fn every_body_rejects_local_return_origins_even_when_uncalled() {
        rejects("bad<&int32>:(){local:1;->&local}", "E303");
        rejects("bad<&int32>:(p<&int32>){local:1;->&local}", "E303");
        rejects(
            "bad<{view<&int32>}>:(p<&int32>){local:1;->view:&local}",
            "E303",
        );
        rejects(
            "bad<&int32><null>:(flag<boolean>){local:1;|flag|->&local}",
            "E303",
        );
    }

    #[test]
    pub(crate) fn normal_return_proofs_preserve_earlier_leaves_and_separate_definitions() {
        accepts(
            "d:@\"debug\";stop<&int32>:(n<int32>){d.panic(\"stop\")};wrap<int32>:(flag<boolean>) 'out {->*stop({|flag|{'out->7;'out.leave()};->0})}",
        );
        accepts("d:@\"debug\";stop<&int32>:(){d.panic(\"stop\")};wrap<&int32>:(){->stop()}");
        accepts("dead<&int32>:(){->dead()}");
        rejects(
            "dead<&int32>:(s<&string>){->dead(s)};bad<&int32>:(flag<boolean>,s<&string>){|flag|dead(s);local:1;->&local}",
            "E303",
        );
        accepts(
            "pick<&int32>:(p<&int32>,n<int32>) 'out {|n==0|{'out->p;'out.leave()};->pick(p,n-1)};a:1;r:pick(&a,2);x:*r",
        );
    }

    #[test]
    pub(crate) fn returned_component_by_input_expansion_has_a_public_source_budget() {
        let fields = (0..64)
            .map(|id| format!("field{id}<&int32>;"))
            .collect::<String>();
        let params = (0..64)
            .map(|id| format!("p{id}<&int32>"))
            .collect::<Vec<_>>()
            .join(",");
        let mut source = format!("make<{{{fields}}}>:({params}){{");
        for id in 0..64 {
            source.push_str(&format!("->field{id}:p0;"));
        }
        source.push_str("};");
        for id in 0..64 {
            source.push_str(&format!("a{id}:{id};"));
        }
        let args = (0..64)
            .map(|id| format!("&a{id}"))
            .collect::<Vec<_>>()
            .join(",");
        source.push_str(&format!("value:make({args})"));
        let errors = crate::compile(&source).unwrap_err();
        assert_eq!(errors[0].code, "B001", "{errors:?}");
        assert!(errors[0].message.contains("budget"));
    }

    #[test]
    pub(crate) fn wide_reference_targets_do_not_expand_an_unbounded_frontier() {
        let fields = (0..4097)
            .map(|id| format!("field{id}<int32>;"))
            .collect::<String>();
        let source = format!(
            "<Wide>:<{{{fields}}}>;id<&Wide>:(p<&Wide>){{->p}};wrap<&Wide>:(p<&Wide>){{->id(p)}}"
        );
        let errors = crate::compile(&source).unwrap_err();
        assert_eq!(errors[0].code, "B001", "{errors:?}");
        assert!(errors[0].message.contains("budget"));
    }
}
