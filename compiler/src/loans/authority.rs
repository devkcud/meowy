use super::{BTreeMap, FALSE, Graph, Guard, MAX_ORIGINS, MAX_VALUES, Node, Result, TRUE, VecDeque};
use crate::hir::ReferenceMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LoanId(pub(crate) usize);

#[derive(Clone, Debug, Default)]
pub(crate) struct Authority {
    pub(crate) loans: BTreeMap<LoanId, Guard>,
    pub(crate) opaque: Guard,
}

impl Authority {
    pub(crate) fn weight(&self) -> usize {
        self.loans.len().saturating_mul(2) + 1
    }
}

pub(crate) struct Loan {
    pub(crate) mode: ReferenceMode,
    pub(crate) node: usize,
    pub(crate) value: usize,
    pub(crate) parent: Option<usize>,
    pub(crate) parents: Authority,
}

impl Graph<'_> {
    pub(crate) fn reserve_authority(&mut self, weight: usize) -> Result<()> {
        self.charge(weight)?;
        if self.origins.saturating_add(weight) > MAX_ORIGINS {
            return Err(Self::budget());
        }
        self.origins += weight;
        Ok(())
    }

    pub(crate) fn copy_link(
        &mut self,
        node: &mut Node,
        target: usize,
        source: usize,
        guard: Guard,
    ) -> Result<()> {
        if target >= self.values.len() || source >= self.values.len() {
            return Err(Self::budget());
        }
        self.reserve_authority(3)?;
        node.copies.push((target, source, guard));
        Ok(())
    }

    pub(crate) fn opaque_value(&mut self, node: &mut Node, value: usize) -> Result<()> {
        if value >= self.values.len() {
            return Err(Self::budget());
        }
        self.reserve_authority(1)?;
        node.opaque.push(value);
        Ok(())
    }

    pub(crate) fn grant_mode(
        &mut self,
        node: usize,
        value: usize,
        parent: Option<usize>,
        mode: ReferenceMode,
    ) -> Result<()> {
        if self.loans.len() >= MAX_VALUES
            || node >= self.nodes.len()
            || value >= self.values.len()
            || parent.is_some_and(|id| id >= self.values.len())
        {
            return Err(Self::budget());
        }
        self.reserve_authority(6)?;
        self.loans.push(Loan {
            mode,
            node,
            value,
            parent,
            parents: Authority::default(),
        });
        Ok(())
    }

    pub(crate) fn merge_authority(
        &mut self,
        target: &mut Authority,
        source: &Authority,
        guard: Guard,
    ) -> Result<bool> {
        let lookup = target.loans.len().checked_ilog2().unwrap_or(0) as usize + 1;
        self.charge(source.weight().saturating_mul(lookup))?;
        let mut changed = false;
        let opaque = self.guards.and(source.opaque, guard);
        let opaque = self.guards.or(target.opaque, opaque);
        if opaque != target.opaque {
            target.opaque = opaque;
            changed = true;
        }
        for (id, active) in &source.loans {
            let active = self.guards.and(*active, guard);
            if active == FALSE {
                continue;
            }
            let prior = target.loans.get(id).copied().unwrap_or(FALSE);
            let active = self.guards.or(prior, active);
            if prior != active {
                if prior == FALSE {
                    self.reserve_authority(2)?;
                }
                target.loans.insert(*id, active);
                changed = true;
            }
        }
        Ok(changed)
    }

    pub(crate) fn solve_authority(&mut self, reach: &[Guard]) -> Result<()> {
        if !self.authority.is_empty() || reach.len() != self.nodes.len() {
            return Err(Self::budget());
        }
        let count = self.values.len();
        self.reserve_authority(count.saturating_mul(2))?;
        self.authority.resize_with(count, Authority::default);
        let mut edges = vec![Vec::new(); count];
        let mut restart = false;
        for (node, active) in reach.iter().enumerate() {
            self.charge(self.nodes[node].next.len() + 1)?;
            if *active == FALSE {
                continue;
            }
            restart |= self.nodes[node].next.iter().any(|edge| edge.reset);
            self.charge(self.nodes[node].opaque.len())?;
            for value in &self.nodes[node].opaque {
                self.authority[*value].opaque = TRUE;
            }
            let size = self.nodes[node].copies.len();
            self.reserve_authority(size.saturating_mul(4))?;
            for (target, source, guard) in &self.nodes[node].copies {
                let guard = self.guards.and(*guard, *active);
                if guard != FALSE {
                    edges[*source].push((*target, guard, false));
                }
            }
        }
        if restart {
            self.charge(count)?;
            for value in &mut self.authority {
                value.opaque = TRUE;
            }
        }
        for id in 0..self.loans.len() {
            let value = self.loans[id].value;
            let node = self.loans[id].node;
            if let Some(parent) = self.loans[id].parent
                && reach[node] != FALSE
            {
                self.reserve_authority(4)?;
                edges[parent].push((value, reach[node], true));
            }
            self.charge(self.values[value].len() + 1)?;
            let mut guard = FALSE;
            for origin in &self.values[value].origins {
                guard = self.guards.or(guard, origin.guard);
            }
            let guard = self.guards.and(guard, reach[node]);
            if guard != FALSE {
                self.reserve_authority(2)?;
                self.authority[value].loans.insert(LoanId(id), guard);
            }
        }
        let mut queue: VecDeque<_> = (0..count).collect();
        let mut queued = vec![true; count];
        while let Some(source) = queue.pop_front() {
            queued[source] = false;
            let weight = self.authority[source].weight();
            self.charge(weight + edges[source].len() + 1)?;
            if self.origins.saturating_add(weight) > MAX_ORIGINS {
                return Err(Self::budget());
            }
            let part = self.authority[source].clone();
            let opaque = Authority {
                opaque: part.opaque,
                ..Authority::default()
            };
            for (target, guard, derived) in &edges[source] {
                let mut next = std::mem::take(&mut self.authority[*target]);
                let changed = self.merge_authority(
                    &mut next,
                    if *derived { &opaque } else { &part },
                    *guard,
                )?;
                self.authority[*target] = next;
                if changed && !queued[*target] {
                    queued[*target] = true;
                    queue.push_back(*target);
                }
            }
        }
        for id in 0..self.loans.len() {
            let Some(parent) = self.loans[id].parent else {
                continue;
            };
            let guard = reach[self.loans[id].node];
            let weight = self.authority[parent].weight();
            self.charge(weight)?;
            if self.origins.saturating_add(weight) > MAX_ORIGINS {
                return Err(Self::budget());
            }
            let source = self.authority[parent].clone();
            let mut parents = Authority::default();
            self.merge_authority(&mut parents, &source, guard)?;
            let mut covered = parents.opaque;
            for active in parents.loans.values() {
                covered = self.guards.or(covered, *active);
            }
            let missing = self.guards.and(guard, self.guards.not(covered));
            parents.opaque = self.guards.or(parents.opaque, missing);
            self.loans[id].parents = parents;
        }
        self.check_parents()?;
        self.normalize_accesses(reach, restart)
    }

    pub(crate) fn check_parents(&mut self) -> Result<()> {
        self.reserve_authority(self.loans.len())?;
        let mut state = vec![0u8; self.loans.len()];
        for root in 0..self.loans.len() {
            let mut stack = vec![(LoanId(root), false)];
            while let Some((id, done)) = stack.pop() {
                self.charge(1)?;
                if id.0 >= self.loans.len() {
                    return Err(Self::budget());
                }
                if done {
                    state[id.0] = 2;
                    continue;
                }
                if state[id.0] == 2 {
                    continue;
                }
                if state[id.0] == 1 {
                    return Err(Self::budget());
                }
                state[id.0] = 1;
                self.charge(self.loans[id.0].parents.weight())?;
                if stack
                    .len()
                    .saturating_add(self.loans[id.0].parents.loans.len() + 1)
                    > MAX_VALUES
                {
                    return Err(Self::budget());
                }
                stack.push((id, true));
                for parent in self.loans[id.0].parents.loans.keys().rev() {
                    stack.push((*parent, false));
                }
            }
        }
        Ok(())
    }
}
