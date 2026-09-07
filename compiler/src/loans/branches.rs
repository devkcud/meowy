use super::storage::ScopeKind;
use super::{BTreeMap, Bundle, FALSE, Graph, Guard, LocalId, Node, Result, Span, TRUE, Type};
use crate::borrow_value::State;

pub(crate) type Versions = BTreeMap<LocalId, Bundle>;

pub(crate) struct Arm {
    pub(crate) ends: Vec<usize>,
    pub(crate) versions: Versions,
}

impl Graph<'_> {
    pub(crate) fn versions(&mut self) -> Result<Versions> {
        let work = self.locals.iter().fold(1usize, |work, (id, value)| {
            work.saturating_add(
                if self.proofs.mutable.contains(id)
                    && matches!(
                        self.program.locals[*id],
                        Type::Reference(_) | Type::Exclusive(_)
                    )
                {
                    value.keys().map(|path| path.len() + 1).sum::<usize>() + 1
                } else {
                    1
                },
            )
        });
        self.charge(work)?;
        Ok(self
            .locals
            .iter()
            .filter(|(id, _)| {
                self.proofs.mutable.contains(id)
                    && matches!(
                        self.program.locals[**id],
                        Type::Reference(_) | Type::Exclusive(_)
                    )
            })
            .map(|(id, value)| (*id, value.clone()))
            .collect())
    }

    pub(crate) fn restore_versions(&mut self, versions: &Versions) -> Result<()> {
        let paths = versions
            .values()
            .flat_map(|value| value.keys())
            .map(|path| path.len() + 1)
            .sum::<usize>();
        self.charge(
            paths
                + self.locals.len() * (versions.len().checked_ilog2().unwrap_or(0) as usize + 1)
                + 1,
        )?;
        self.locals.retain(|id, _| {
            !self.proofs.mutable.contains(id)
                || !matches!(
                    self.program.locals[*id],
                    Type::Reference(_) | Type::Exclusive(_)
                )
                || versions.contains_key(id)
        });
        self.locals
            .extend(versions.iter().map(|(id, value)| (*id, value.clone())));
        Ok(())
    }

    pub(crate) fn arm(&mut self) -> Result<Arm> {
        let versions = self.versions()?;
        Ok(Arm {
            ends: std::mem::take(&mut self.current),
            versions,
        })
    }

    pub(crate) fn surviving_arm(&mut self, ids: &[LocalId]) -> Result<Arm> {
        let lookup = self.locals.len().checked_ilog2().unwrap_or(0) as usize + 1;
        self.charge(ids.len().saturating_mul(lookup) + 1)?;
        let mut versions = Versions::new();
        if !self.current.is_empty() {
            for id in ids {
                let value = self.locals.get(id).ok_or_else(Self::budget)?;
                let work = value.keys().map(|path| path.len() + 1).sum::<usize>();
                self.charge(work)?;
                versions.insert(*id, self.locals[id].clone());
            }
        }
        Ok(Arm {
            ends: std::mem::take(&mut self.current),
            versions,
        })
    }

    pub(crate) fn conditional(
        &mut self,
        guard: Guard,
        yes: impl FnOnce(&mut Self) -> Result<()>,
        no: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<()> {
        if !self.merging {
            let (left, right) = self.fork(guard)?;
            self.current.push(left);
            let scope = self.enter_scope(ScopeKind::Branch)?;
            yes(self)?;
            self.close_scope(scope)?;
            let mut ends = std::mem::take(&mut self.current);
            self.current.push(right);
            let scope = self.enter_scope(ScopeKind::Branch)?;
            no(self)?;
            self.close_scope(scope)?;
            ends.append(&mut self.current);
            self.current = ends;
            return Ok(());
        }
        let incoming = self.versions()?;
        let (left, right) = self.fork(guard)?;
        self.current.push(left);
        let scope = self.enter_scope(ScopeKind::Branch)?;
        yes(self)?;
        self.close_scope(scope)?;
        let left = self.arm()?;
        self.restore_versions(&incoming)?;
        self.current.push(right);
        let scope = self.enter_scope(ScopeKind::Branch)?;
        no(self)?;
        self.close_scope(scope)?;
        let right = self.arm()?;
        self.merge_versions(incoming, vec![left, right])
    }

    pub(crate) fn merge_versions(&mut self, incoming: Versions, arms: Vec<Arm>) -> Result<()> {
        self.charge(arms.len() + 1)?;
        let mut changed = Vec::new();
        for (id, value) in &incoming {
            self.charge(value.keys().map(|path| path.len() + 1).sum::<usize>() + 1)?;
            for arm in &arms {
                if arm.ends.is_empty() {
                    continue;
                }
                let other = arm.versions.get(id).ok_or_else(Self::budget)?;
                self.charge(other.keys().map(|path| path.len() + 1).sum::<usize>() + 1)?;
                if other != value {
                    changed.push(*id);
                    break;
                }
            }
        }
        self.restore_versions(&incoming)?;
        if changed.is_empty() {
            self.current = arms.into_iter().flat_map(|arm| arm.ends).collect();
            return Ok(());
        }
        let reach = self.reach()?;
        let mut guards = vec![FALSE; arms.len()];
        for (index, arm) in arms.iter().enumerate() {
            self.charge(arm.ends.len())?;
            for end in &arm.ends {
                guards[index] = self.guards.or(guards[index], reach[*end]);
            }
        }
        let mut merged = Versions::new();
        for id in changed {
            let mut state = State::absent();
            for (arm, guard) in arms.iter().zip(&guards) {
                if *guard == FALSE {
                    continue;
                }
                let mut part = State::default();
                for (path, value) in arm.versions.get(&id).ok_or_else(Self::budget)? {
                    let weight = self.values[*value]
                        .iter()
                        .map(|origin| origin.weight() + path.len() + 1)
                        .sum();
                    self.charge(weight)?;
                    let value = self.values[*value].clone();
                    for (input, output) in [
                        (value.origins, &mut part.origins),
                        (value.bounds, &mut part.bounds),
                    ] {
                        for mut origin in input {
                            origin.component = path.clone();
                            origin.guard = self.guards.and(origin.guard, *guard);
                            if origin.guard != FALSE {
                                output.push(origin);
                            }
                        }
                    }
                }
                state.merge(part, self.guards, Span::default())?;
            }
            merged.insert(id, self.bundle(super::values::Value::from(&state))?);
        }
        let mut ends = Vec::new();
        for arm in arms {
            if arm.ends.is_empty() {
                continue;
            }
            let mut node = Node::default();
            for (id, value) in &merged {
                let source = arm.versions.get(id).ok_or_else(Self::budget)?;
                for (path, target) in value {
                    self.charge(path.len() + 1)?;
                    node.defs.push(*target);
                    if let Some(source) = source.get(path) {
                        node.transfers.push((*target, *source, TRUE));
                        self.copy_link(&mut node, *target, *source, TRUE)?;
                    }
                }
            }
            self.current = arm.ends;
            self.append(node)?;
            ends.append(&mut self.current);
        }
        self.current = ends;
        self.locals.extend(merged);
        Ok(())
    }
}
