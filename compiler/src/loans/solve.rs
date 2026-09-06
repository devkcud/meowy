use super::{
    Block, Diagnostic, FALSE, Graph, Guard, Live, LocalId, MAX_LIVE, Node, Place, Projection,
    Result, Source, Span, TRUE, VecDeque,
};

impl<'a> Graph<'a> {
    pub(crate) fn reach(&mut self) -> Result<Vec<Guard>> {
        let mut reach = vec![FALSE; self.nodes.len()];
        let mut queue = VecDeque::from([0]);
        reach[0] = TRUE;
        while let Some(id) = queue.pop_front() {
            self.tick()?;
            for edge in self.nodes[id].next.clone() {
                let guard = if edge.reset {
                    TRUE
                } else {
                    self.guards.and(reach[id], edge.guard)
                };
                let guard = self.guards.or(reach[edge.target], guard);
                if guard != reach[edge.target] {
                    reach[edge.target] = guard;
                    queue.push_back(edge.target);
                }
            }
        }
        Ok(reach)
    }

    pub(crate) fn outgoing(&mut self, id: usize, live: &[Live]) -> Result<Live> {
        let mut next = Live::new();
        for edge in self.nodes[id].next.clone() {
            if edge.guard == FALSE {
                continue;
            }
            for (value, guard) in &live[edge.target] {
                self.tick()?;
                let guard = if edge.reset {
                    TRUE
                } else {
                    self.guards.and(*guard, edge.guard)
                };
                if guard != FALSE {
                    let prior = next.get(value).copied().unwrap_or(FALSE);
                    next.insert(*value, self.guards.or(prior, guard));
                }
            }
        }
        Ok(next)
    }

    pub(crate) fn liveness(&mut self, reach: &[Guard]) -> Result<Vec<Live>> {
        let mut before = vec![Vec::new(); self.nodes.len()];
        for (id, node) in self.nodes.iter().enumerate() {
            for edge in &node.next {
                before[edge.target].push(id);
            }
        }
        let mut live = vec![Live::new(); self.nodes.len()];
        let mut queue: VecDeque<_> = (0..self.nodes.len()).rev().collect();
        let mut queued = vec![true; self.nodes.len()];
        let mut entries = 0;
        while let Some(id) = queue.pop_front() {
            queued[id] = false;
            self.tick()?;
            if reach[id] == FALSE {
                continue;
            }
            let mut next = self.outgoing(id, &live)?;
            for value in &self.nodes[id].defs {
                next.remove(value);
            }
            for value in &self.nodes[id].uses {
                next.insert(*value, TRUE);
            }
            if next != live[id] {
                entries = entries - live[id].len() + next.len();
                if entries > MAX_LIVE {
                    return Err(Self::budget());
                }
                live[id] = next;
                for id in &before[id] {
                    if !queued[*id] {
                        queued[*id] = true;
                        queue.push_back(*id);
                    }
                }
            }
        }
        Ok(live)
    }

    pub(crate) fn overlap(place: &Place, source: &Source) -> bool {
        let Source::Local { id, fields } = source else {
            return false;
        };
        place.root == *id
            && place.fields.iter().zip(fields).all(|(index, field)| {
                matches!(field, Projection::Element) || *field == Projection::Field(*index)
            })
    }

    pub(crate) fn check(mut self, block: &Block, params: &[LocalId]) -> Result<()> {
        for id in params {
            let proof = self
                .facts
                .locals
                .get(id)
                .ok_or_else(|| {
                    Diagnostic::unsupported(
                        "missing function input borrow proof",
                        Span { start: 0, end: 0 },
                    )
                })?
                .proof;
            let value = self.local(*id)?;
            self.append(Node {
                defs: value.into_values().collect(),
                ..Node::default()
            })?;
            self.assume(proof)?;
        }
        self.block(block)?;
        let reach = self.reach()?;
        for (node, span) in &self.missing_calls {
            if reach[*node] != FALSE {
                return Err(Diagnostic::unsupported(
                    "missing function result borrow proof",
                    *span,
                ));
            }
        }
        for (node, span) in &self.missing_reborrows {
            if reach[*node] != FALSE {
                return Err(Diagnostic::unsupported(
                    "missing shared reborrow proof",
                    *span,
                ));
            }
        }
        let live = self.liveness(&reach)?;
        for (id, reachable) in reach.iter().enumerate() {
            let Some((place, span)) = self.nodes[id].write.clone() else {
                continue;
            };
            if *reachable == FALSE {
                continue;
            }
            for (value, guard) in self.outgoing(id, &live)? {
                let work = self.values[value]
                    .iter()
                    .fold(1usize, |work, origin| work.saturating_add(origin.weight()));
                self.charge(work)?;
                let guard = self.guards.and(guard, *reachable);
                for origin in &self.values[value] {
                    if Self::overlap(&place, &origin.source)
                        && self.guards.overlap(guard, origin.guard)
                    {
                        return Err(Diagnostic::new(
                            "E302",
                            "assignment conflicts with a live shared borrow",
                            span,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
