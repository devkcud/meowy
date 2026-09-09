use super::{BTreeMap, Diagnostic, FALSE, Graph, Guard, Result, Span};
use std::collections::BTreeSet;

pub(crate) struct Header {
    pub(crate) required: BTreeMap<usize, Guard>,
    pub(crate) missing: Guard,
}

impl Graph<'_> {
    pub(crate) fn missing_restart_header() -> Diagnostic {
        Diagnostic::unsupported("missing shared restart header coverage", Span::default())
    }

    pub(crate) fn shared_restart_headers(&mut self, reach: &[Guard]) -> Result<BTreeSet<usize>> {
        if reach.len() != self.nodes.len() {
            return Err(Self::budget());
        }
        self.charge(self.nodes.len() + self.missing_headers.len() + 1)?;
        for (node, active) in &self.missing_headers {
            if *node >= reach.len() || self.guards.overlap(reach[*node], *active) {
                return Err(Self::missing_restart_header());
            }
        }
        let count = self.values.len();
        let mut groups = BTreeMap::<usize, Vec<usize>>::new();
        let mut owners = BTreeMap::new();
        let mut certified = BTreeSet::new();
        for (id, entered) in reach.iter().enumerate() {
            self.charge(self.nodes[id].next.len() + 1)?;
            if *entered == FALSE {
                continue;
            }
            let targets = self.nodes[id]
                .next
                .iter()
                .filter(|edge| edge.reset && edge.guard != FALSE)
                .map(|edge| edge.target)
                .collect::<Vec<_>>();
            if targets.is_empty() && self.nodes[id].header.is_none() {
                continue;
            }
            if targets.len() != 1 || targets[0] >= reach.len() {
                return Err(Self::missing_restart_header());
            }
            let target = targets[0];
            let node = &self.nodes[id];
            let header = node
                .header
                .as_ref()
                .ok_or_else(Self::missing_restart_header)?;
            let size = header.required.len() + node.defs.len() + node.transfers.len() + 1;
            self.reserve_authority(size.saturating_mul(4))?;
            self.charge(size.saturating_mul(count.checked_ilog2().unwrap_or(0) as usize + 3))?;
            let node = &self.nodes[id];
            let header = node
                .header
                .as_ref()
                .ok_or_else(Self::missing_restart_header)?;
            let keys = header.required.keys().copied().collect::<Vec<_>>();
            let defs = node.defs.iter().copied().collect::<BTreeSet<_>>();
            if self.guards.overlap(*entered, header.missing)
                || defs.len() != node.defs.len()
                || !defs.iter().copied().eq(keys.iter().copied())
                || keys.iter().any(|value| *value >= count)
            {
                return Err(Self::missing_restart_header());
            }
            if let Some(prior) = groups.get(&target) {
                if prior != &keys {
                    return Err(Self::missing_restart_header());
                }
            } else {
                groups.insert(target, keys.clone());
            }
            for value in keys {
                if let Some(prior) = owners.insert(value, target)
                    && prior != target
                {
                    return Err(Self::missing_restart_header());
                }
            }
            let mut coverage = BTreeMap::new();
            for (value, source, active) in &node.transfers {
                if *source >= count || !header.required.contains_key(value) {
                    return Err(Self::missing_restart_header());
                }
                let prior = coverage.get(value).copied().unwrap_or(FALSE);
                coverage.insert(*value, self.guards.or(prior, *active));
            }
            for (value, required) in &header.required {
                if !self
                    .guards
                    .implies(*required, coverage.get(value).copied().unwrap_or(FALSE))
                {
                    return Err(Self::missing_restart_header());
                }
            }
            certified.insert(id);
        }
        for (id, entered) in reach.iter().enumerate() {
            let lookup = owners.len().checked_ilog2().unwrap_or(0) as usize + 1;
            self.charge(self.nodes[id].defs.len().saturating_mul(lookup) + 1)?;
            if *entered != FALSE
                && !certified.contains(&id)
                && self.nodes[id]
                    .defs
                    .iter()
                    .any(|value| owners.contains_key(value))
            {
                return Err(Self::missing_restart_header());
            }
        }
        Ok(certified)
    }
}
