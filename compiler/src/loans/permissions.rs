use super::access::{Access, Kind, Target};
use super::authority::{Authority, LoanId};
use super::{
    BTreeMap, Diagnostic, FALSE, Graph, Guard, Live, MAX_ORIGINS, Result, Source, VecDeque,
};
use crate::hir::ReferenceMode;

impl Graph<'_> {
    pub(crate) fn lineage(&mut self, authority: &Authority) -> Result<BTreeMap<LoanId, Guard>> {
        self.charge(authority.weight())?;
        let mut result = authority.loans.clone();
        let mut queue: VecDeque<_> = result.keys().copied().collect();
        while let Some(id) = queue.pop_front() {
            self.charge(self.loans[id.0].parents.weight() + 1)?;
            let parents = self.loans[id.0].parents.loans.clone();
            for (parent, guard) in parents {
                let guard = self.guards.and(result[&id], guard);
                let prior = result.get(&parent).copied().unwrap_or(FALSE);
                let guard = self.guards.or(prior, guard);
                if prior != guard {
                    if self
                        .origins
                        .saturating_add(result.len().saturating_mul(3) + queue.len() + 4)
                        > MAX_ORIGINS
                    {
                        return Err(Self::budget());
                    }
                    result.insert(parent, guard);
                    queue.push_back(parent);
                }
            }
        }
        Ok(result)
    }

    pub(crate) fn physical_region(source: &Source) -> Option<(usize, &[super::Projection])> {
        match source {
            Source::Local { id, fields } | Source::Temporary { id, fields, .. } => {
                Some((*id, fields))
            }
            Source::Slot { root, fields, .. } => Some((*root, fields)),
            _ => None,
        }
    }

    pub(crate) fn physical_overlap(left: &Source, right: &Source) -> bool {
        if let (
            Source::Input {
                id: left,
                component: a,
                fields: x,
            },
            Source::Input {
                id: right,
                component: b,
                fields: y,
            },
        ) = (left, right)
        {
            return left == right && a == b && Self::projection_overlap(x, y);
        }
        let (Some((left, a)), Some((right, b))) =
            (Self::physical_region(left), Self::physical_region(right))
        else {
            return false;
        };
        left == right && Self::projection_overlap(a, b)
    }

    pub(crate) fn access_overlap(region: &super::Origin, source: &Source) -> bool {
        if !Self::physical_overlap(&region.source, source) {
            return false;
        }
        if region.component.first() != Some(&super::Step::Slot(0)) {
            return true;
        }
        let (base, fields) = match (&region.source, source) {
            (Source::Input { fields: base, .. }, Source::Input { fields, .. }) => {
                (base.as_slice(), fields.as_slice())
            }
            _ => {
                let (Some((_, base)), Some((_, fields))) = (
                    Self::physical_region(&region.source),
                    Self::physical_region(source),
                ) else {
                    return true;
                };
                (base, fields)
            }
        };
        !(fields.starts_with(base)
            && matches!(fields.get(base.len()), Some(super::Projection::Field(_))))
    }

    pub(crate) fn projection_overlap(a: &[super::Projection], b: &[super::Projection]) -> bool {
        a.iter().zip(b).all(|(a, b)| {
            a == b
                || matches!(a, super::Projection::Element)
                || matches!(b, super::Projection::Element)
        })
    }

    pub(crate) fn exclusive_boundary(&mut self, id: usize, entered: Guard) -> Result<()> {
        let Some(span) = self.nodes[id].barrier else {
            return Ok(());
        };
        self.charge(self.nodes[id].uses.len() + self.nodes[id].copies.len() + 1)?;
        let values: Vec<_> = self.nodes[id]
            .uses
            .iter()
            .copied()
            .chain(self.nodes[id].copies.iter().map(|(_, source, _)| *source))
            .collect();
        for value in values {
            self.charge(self.authority[value].weight())?;
            let held = self.authority[value].clone();
            for (loan, guard) in self.lineage(&held)? {
                if self.loans[loan.0].mode == ReferenceMode::Exclusive
                    && self.guards.overlap(entered, guard)
                {
                    return Err(Diagnostic::unsupported(
                        "exclusive ancestry crossing a call, result or reference cell",
                        span,
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn permission(
        &mut self,
        access: &Access,
        entered: Guard,
        outgoing: Live,
    ) -> Result<()> {
        let actor = if let Target::Pointee(value) = access.target {
            self.charge(self.authority[value].weight())?;
            self.authority[value].clone()
        } else {
            Authority::default()
        };
        let allowed = self.lineage(&actor)?;
        let mut exclusive = access.kind == Kind::Write;
        if access.kind == Kind::Borrow {
            exclusive = actor
                .loans
                .keys()
                .any(|loan| self.loans[loan.0].mode == ReferenceMode::Exclusive);
        }
        if exclusive && matches!(access.target, Target::Pointee(_)) {
            let mut known = FALSE;
            for (loan, guard) in &actor.loans {
                if self.loans[loan.0].mode == ReferenceMode::Exclusive {
                    known = self.guards.or(known, *guard);
                }
            }
            if !self.guards.implies(entered, known) || self.guards.overlap(entered, actor.opaque) {
                return Err(Diagnostic::unsupported(
                    "missing exclusive access authority",
                    access.span,
                ));
            }
        }
        for (value, live) in outgoing {
            self.charge(self.authority[value].weight() + self.values[value].len() + 1)?;
            let held = self.authority[value].clone();
            let lineage = self.lineage(&held)?;
            let live = self.guards.and(entered, live);
            let mut conflict = FALSE;
            if exclusive {
                let mut authorized = FALSE;
                for (loan, guard) in &held.loans {
                    let permit = allowed.get(loan).copied().unwrap_or(FALSE);
                    let permit = self.guards.and(permit, *guard);
                    authorized = self.guards.or(authorized, permit);
                }
                conflict = self.guards.and(live, self.guards.not(authorized));
            }
            for (loan, guard) in lineage {
                if self.loans[loan.0].mode == ReferenceMode::Exclusive || exclusive {
                    let permit = allowed.get(&loan).copied().unwrap_or(FALSE);
                    let denied = self.guards.and(guard, self.guards.not(permit));
                    let denied = self.guards.and(live, denied);
                    conflict = self.guards.or(conflict, denied);
                }
            }
            if conflict == FALSE {
                continue;
            }
            let weight = self.values[value]
                .iter()
                .map(super::Origin::weight)
                .sum::<usize>();
            self.charge(weight.saturating_mul(access.regions.len() + 1))?;
            for region in &access.regions {
                for origin in self.values[value].origins.iter().chain(
                    self.values[value]
                        .bounds
                        .iter()
                        .filter(|_| exclusive && !self.values[value].allocator),
                ) {
                    let guard = self.guards.and(conflict, region.guard);
                    if Self::access_overlap(region, &origin.source)
                        && self.guards.overlap(guard, origin.guard)
                    {
                        return Err(Diagnostic::new(
                            "E302",
                            "access conflicts with a live borrow",
                            access.span,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn permissions(&mut self, reach: &[Guard], live: &[Live]) -> Result<()> {
        if !self
            .loans
            .iter()
            .any(|loan| loan.mode == ReferenceMode::Exclusive)
        {
            return Ok(());
        }
        for (id, entered) in reach.iter().enumerate() {
            self.tick()?;
            if *entered == FALSE {
                continue;
            }
            self.exclusive_boundary(id, *entered)?;
            if let Some(access) = &self.nodes[id].access {
                let weight = access.weight()
                    + access
                        .regions
                        .iter()
                        .map(super::Origin::weight)
                        .sum::<usize>();
                self.charge(weight)?;
                let access = self.nodes[id].access.as_ref().expect("access").clone();
                let outgoing = self.outgoing(id, live)?;
                self.permission(&access, *entered, outgoing)?;
            }
        }
        Ok(())
    }
}
