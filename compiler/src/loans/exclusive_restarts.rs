use super::{Diagnostic, FALSE, Graph, Guard, Origin, Result, Source, Span, Type, VecDeque};
use crate::hir::ReferenceMode;

pub(crate) const ROOTED: u8 = 1;
pub(crate) const EXCLUSIVE: u8 = 2;
pub(crate) const OPAQUE: u8 = 4;

impl Graph<'_> {
    pub(crate) fn exclusive_restart_source(&mut self, value: usize, span: Span) -> Result<()> {
        let origins = &self.values[value].origins;
        if origins.is_empty() {
            return Err(Self::budget());
        }
        self.charge(origins.iter().map(Origin::weight).sum::<usize>() + 1)?;
        let origins = self.values[value].origins.clone();
        for origin in origins {
            let Source::Slot {
                target,
                root,
                view,
                fields,
            } = origin.source
            else {
                return Err(Diagnostic::unsupported(
                    "exclusive restart loans outside carried scalar storage",
                    span,
                ));
            };
            self.charge(self.proofs.aliases.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
            let alias = self.proofs.aliases.get(&view).ok_or_else(Self::budget)?;
            self.charge((alias.field.len() + 1).saturating_mul(
                self.proofs.carried.len().checked_ilog2().unwrap_or(0) as usize + 1,
            ))?;
            let slot = self
                .proofs
                .carried
                .get(&(target, Some(alias.field.clone())));
            if !origin.component.is_empty()
                || !fields.is_empty()
                || alias.target != target
                || alias.root != root
                || !slot.is_some_and(|slot| {
                    matches!(&slot.ty, Type::Bool | Type::Int { .. } | Type::Float { .. })
                })
            {
                return Err(Diagnostic::unsupported(
                    "exclusive restart loans outside carried scalar storage",
                    span,
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn restart_marks(&mut self, edges: &[Vec<usize>], marks: &mut [u8]) -> Result<()> {
        self.charge(marks.len())?;
        let mut pending: VecDeque<_> = marks
            .iter()
            .enumerate()
            .filter_map(|(id, mark)| (*mark != 0).then_some(id))
            .collect();
        let mut queued = marks.iter().map(|mark| *mark != 0).collect::<Vec<_>>();
        while let Some(source) = pending.pop_front() {
            queued[source] = false;
            self.charge(edges[source].len() + 1)?;
            for target in &edges[source] {
                let next = marks[*target] | marks[source];
                if marks[*target] != next {
                    marks[*target] = next;
                    if !queued[*target] {
                        queued[*target] = true;
                        pending.push_back(*target);
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn exclusive_restart_frontiers(&mut self, reach: &[Guard]) -> Result<()> {
        if reach.len() != self.nodes.len() {
            return Err(Self::budget());
        }
        let count = self.values.len();
        self.reserve_authority(count.saturating_mul(5) + self.loans.len().saturating_mul(3))?;
        let mut edges = vec![Vec::new(); count];
        let mut marks = vec![0u8; count];
        let mut span = Span::default();
        for (id, entered) in reach.iter().enumerate() {
            let node = &self.nodes[id];
            let work = node.copies.len().saturating_add(node.transfers.len());
            self.reserve_authority(work.saturating_mul(3) + node.opaque.len() + 1)?;
            let node = &self.nodes[id];
            if node.next.iter().any(|edge| edge.target >= reach.len())
                || node
                    .uses
                    .iter()
                    .chain(&node.defs)
                    .any(|value| *value >= count)
                || node.opaque.iter().any(|value| *value >= count)
            {
                return Err(Self::budget());
            }
            for (target, source, active) in node.copies.iter().chain(&node.transfers) {
                if *target >= count || *source >= count {
                    return Err(Self::budget());
                }
                if *entered != FALSE && *active != FALSE {
                    edges[*source].push(*target);
                }
            }
            if *entered != FALSE {
                for value in &node.opaque {
                    marks[*value] |= OPAQUE;
                }
            }
        }
        for id in 0..self.loans.len() {
            let loan = &self.loans[id];
            let (value, node, parent, mode) = (loan.value, loan.node, loan.parent, loan.mode);
            if value >= count || node >= reach.len() || parent.is_some_and(|id| id >= count) {
                return Err(Self::budget());
            }
            if reach[node] == FALSE {
                continue;
            }
            if let Some(parent) = parent {
                edges[parent].push(value);
            } else {
                marks[value] |= ROOTED;
            }
            if mode == ReferenceMode::Exclusive {
                span = self.nodes[node]
                    .access
                    .as_ref()
                    .map_or(span, |access| access.span);
                self.exclusive_restart_source(value, span)?;
                marks[value] |= EXCLUSIVE;
            }
        }
        for (id, mark) in marks.iter_mut().enumerate() {
            self.charge(self.values[id].origins.len() + self.values[id].bounds.len() + 1)?;
            if self.values[id]
                .origins
                .iter()
                .chain(&self.values[id].bounds)
                .any(|origin| {
                    matches!(origin.source, Source::Input { .. } | Source::Expired { .. })
                })
            {
                *mark |= OPAQUE;
            }
        }
        self.restart_marks(&edges, &mut marks)?;
        self.charge(count)?;
        for mark in &mut marks {
            if *mark & ROOTED == 0 {
                *mark |= OPAQUE;
            }
        }
        self.restart_marks(&edges, &mut marks)?;
        let live = self.liveness(reach)?;
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
            for target in targets {
                self.charge(live[target].len() + 1)?;
                if live[target].iter().any(|(value, active)| {
                    *active != FALSE && marks[*value] & (EXCLUSIVE | OPAQUE) != 0
                }) {
                    return Err(Diagnostic::unsupported(
                        "exclusive or opaque loan ancestry crossing a restart edge",
                        span,
                    ));
                }
            }
        }
        Ok(())
    }
}
