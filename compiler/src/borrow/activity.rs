use super::{
    BTreeMap, BTreeSet, BlockId, Checker, Diagnostic, FALSE, Guard, Guards, LocalId, Path, Result,
    Span, State, Step, TRUE,
};
use crate::borrow_value::Active;

pub(crate) type Member = (Path, usize);
pub(crate) type Choices = BTreeMap<(BlockId, LocalId, Path, usize), Guard>;

pub(crate) struct Activity {
    pub(crate) paths: BTreeMap<Path, Guard>,
    pub(crate) parents: BTreeMap<Path, Guard>,
    pub(crate) members: BTreeMap<Member, Guard>,
}

pub(crate) fn weight(choices: &Choices) -> usize {
    choices.keys().map(|(_, _, path, _)| path.len() + 4).sum()
}

pub(crate) fn path_guard(
    path: &[Step],
    members: &BTreeMap<Member, Guard>,
    present: Guard,
    guards: &mut Guards,
    span: Span,
) -> Result<Guard> {
    let mut guard = present;
    for (index, step) in path.iter().enumerate() {
        if !guards.spend(1) {
            return Err(State::budget(span));
        }
        if let Step::Variant(member) = step {
            let lookup = members.len().checked_ilog2().unwrap_or(0) as usize + 1;
            if !guards.spend((index + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            let key = (path[..index].to_vec(), *member);
            guard = guards.and(guard, members.get(&key).copied().unwrap_or(FALSE));
        }
    }
    Ok(guard)
}

impl Activity {
    pub(crate) fn new(
        shape: &super::header::Shape,
        state: &State,
        canonical: bool,
        guards: &mut Guards,
        span: Span,
    ) -> Result<Self> {
        let weight = state
            .active
            .iter()
            .map(|active| active.component.len() + 1)
            .sum::<usize>()
            + shape.paths.len()
            + shape.unions.len()
            + 1;
        if !guards.spend(weight) {
            return Err(State::budget(span));
        }
        let mut members = BTreeMap::new();
        for active in &state.active {
            let lookup = shape.unions.len().checked_ilog2().unwrap_or(0) as usize
                + members.len().checked_ilog2().unwrap_or(0) as usize
                + 2;
            if !guards.spend((active.component.len() + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            if shape
                .unions
                .get(&active.component)
                .is_none_or(|count| active.member >= *count)
                || (canonical && active.guard == FALSE)
            {
                return Err(Diagnostic::unsupported(
                    "unknown restart header activity",
                    span,
                ));
            }
            let key = (active.component.clone(), active.member);
            let old = members.get(&key).copied().unwrap_or(FALSE);
            members.insert(key, guards.or(old, active.guard));
        }
        let mut parents = BTreeMap::new();
        for (path, count) in &shape.unions {
            let parent = path_guard(path, &members, state.present, guards, span)?;
            let effective = guards.and(parent, state.proof);
            let mut covered = FALSE;
            for member in 0..*count {
                let lookup = members.len().checked_ilog2().unwrap_or(0) as usize + 1;
                if !guards.spend((path.len() + 1).saturating_mul(lookup)) {
                    return Err(State::budget(span));
                }
                let active = members
                    .get(&(path.clone(), member))
                    .copied()
                    .unwrap_or(FALSE);
                let part = guards.and(active, effective);
                if guards.overlap(part, covered) || (canonical && !guards.implies(active, parent)) {
                    return Err(Diagnostic::unsupported(
                        "overlapping restart header activity",
                        span,
                    ));
                }
                covered = guards.or(covered, active);
            }
            if !guards.implies(effective, covered) {
                return Err(Diagnostic::unsupported(
                    "incomplete restart header activity",
                    span,
                ));
            }
            if !guards.spend(
                (path.len() + 1)
                    .saturating_mul(parents.len().checked_ilog2().unwrap_or(0) as usize + 1),
            ) {
                return Err(State::budget(span));
            }
            parents.insert(path.clone(), parent);
        }
        let mut paths = BTreeMap::new();
        for path in &shape.paths {
            let guard = path_guard(path, &members, state.present, guards, span)?;
            if !guards.spend(
                (path.len() + 1)
                    .saturating_mul(paths.len().checked_ilog2().unwrap_or(0) as usize + 1),
            ) {
                return Err(State::budget(span));
            }
            paths.insert(path.clone(), guard);
        }
        Ok(Self {
            paths,
            parents,
            members,
        })
    }
}

impl Checker<'_> {
    pub(crate) fn header_activity(
        &mut self,
        target: BlockId,
        local: LocalId,
        shape: &super::header::Shape,
        seen: BTreeSet<Member>,
        span: Span,
    ) -> Result<Vec<Active>> {
        let mut groups = BTreeMap::<Path, Vec<usize>>::new();
        for (path, member) in seen {
            let lookup = groups.len().checked_ilog2().unwrap_or(0) as usize + 1;
            if !self.guards.spend((path.len() + 1).saturating_mul(lookup)) {
                return Err(State::budget(span));
            }
            groups.entry(path).or_default().push(member);
        }
        let mut members = BTreeMap::new();
        let mut result = Vec::new();
        for path in shape.unions.keys() {
            let parent = path_guard(path, &members, TRUE, self.guards, span)?;
            if !self.guards.spend(
                (path.len() + 1)
                    .saturating_mul(groups.len().checked_ilog2().unwrap_or(0) as usize + 1),
            ) {
                return Err(State::budget(span));
            }
            let available = groups.get(path).map(Vec::as_slice).unwrap_or(&[]);
            if parent != FALSE && available.is_empty() {
                return Err(Diagnostic::unsupported(
                    "missing observed restart member",
                    span,
                ));
            }
            let mut rest = parent;
            for (index, member) in available.iter().enumerate() {
                let guard = if index + 1 == available.len() {
                    rest
                } else {
                    let lookup = self.choices.len().checked_ilog2().unwrap_or(0) as usize + 1;
                    if !self.guards.spend((path.len() + 4).saturating_mul(lookup)) {
                        return Err(State::budget(span));
                    }
                    let key = (target, local, path.clone(), *member);
                    let choice = if let Some(choice) = self.choices.get(&key) {
                        *choice
                    } else {
                        self.reserve_origins(path.len() + 4, span)?;
                        let choice = self.guards.fresh();
                        self.choices.insert(key, choice);
                        choice
                    };
                    let guard = self.guards.and(rest, choice);
                    rest = self.guards.and(rest, self.guards.not(choice));
                    guard
                };
                if guard != FALSE {
                    let lookup = members.len().checked_ilog2().unwrap_or(0) as usize + 2;
                    if !self.guards.spend((path.len() + 1).saturating_mul(lookup))
                        || result.len() >= super::MAX_ORIGINS
                    {
                        return Err(State::budget(span));
                    }
                    members.insert((path.clone(), *member), guard);
                    result.push(Active {
                        component: path.clone(),
                        member: *member,
                        guard,
                    });
                }
            }
        }
        Ok(result)
    }
}
