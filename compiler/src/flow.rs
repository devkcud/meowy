use std::collections::HashMap;

pub(crate) type Guard = usize;

pub(crate) const FALSE: Guard = 0;
pub(crate) const TRUE: Guard = 1;
pub(crate) const MAX_NODES: usize = 65_536;
pub(crate) const MAX_CACHE: usize = 131_072;
pub(crate) const MAX_TASKS: usize = 131_072;
pub(crate) const MAX_STEPS: usize = 1_048_576;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) var: usize,
    pub(crate) low: Guard,
    pub(crate) high: Guard,
}

pub(crate) enum Task {
    Apply(Guard, Guard),
    Join(Guard, Guard, usize),
}

pub(crate) struct Flow {
    pub(crate) nodes: Vec<Node>,
    pub(crate) unique: HashMap<Node, Guard>,
    pub(crate) cache: HashMap<(Guard, Guard), Guard>,
    pub(crate) vars: usize,
    pub(crate) full: bool,
}

impl Default for Flow {
    fn default() -> Self {
        Self::new()
    }
}

impl Flow {
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![Node {
                var: 0,
                low: FALSE,
                high: TRUE,
            }],
            unique: HashMap::new(),
            cache: HashMap::new(),
            vars: 0,
            full: false,
        }
    }

    pub(crate) fn fresh(&mut self) -> Guard {
        if self.full {
            return TRUE;
        }
        self.vars += 1;
        self.node(self.vars, FALSE, TRUE)
    }

    pub(crate) fn and(&mut self, a: Guard, b: Guard) -> Guard {
        if self.full {
            return TRUE;
        }
        let mut tasks = vec![Task::Apply(a, b)];
        let mut values = Vec::new();
        let mut steps = 0;
        while let Some(task) = tasks.pop() {
            steps += 1;
            if steps > MAX_STEPS {
                self.full = true;
                return TRUE;
            }
            match task {
                Task::Apply(a, b) => {
                    let (a, b) = if a <= b { (a, b) } else { (b, a) };
                    if let Some(value) = self.known(a, b) {
                        values.push(value);
                        continue;
                    }
                    if tasks.len() + 3 > MAX_TASKS {
                        self.full = true;
                        return TRUE;
                    }
                    let var = self.nodes[a >> 1].var.max(self.nodes[b >> 1].var);
                    let (a0, a1) = self.split(a, var);
                    let (b0, b1) = self.split(b, var);
                    tasks.push(Task::Join(a, b, var));
                    tasks.push(Task::Apply(a1, b1));
                    tasks.push(Task::Apply(a0, b0));
                }
                Task::Join(a, b, var) => {
                    let high = values.pop().expect("high guard");
                    let low = values.pop().expect("low guard");
                    let value = self.node(var, low, high);
                    if self.full || self.cache.len() >= MAX_CACHE {
                        self.full = true;
                        return TRUE;
                    }
                    self.cache.insert((a, b), value);
                    values.push(value);
                }
            }
        }
        values.pop().expect("result guard")
    }

    pub(crate) fn or(&mut self, a: Guard, b: Guard) -> Guard {
        let value = self.and(self.not(a), self.not(b));
        if self.full { TRUE } else { self.not(value) }
    }

    pub(crate) fn not(&self, value: Guard) -> Guard {
        value ^ 1
    }

    pub(crate) fn overlap(&mut self, a: Guard, b: Guard) -> bool {
        self.and(a, b) != FALSE
    }

    pub(crate) fn implies(&mut self, a: Guard, b: Guard) -> bool {
        self.and(a, self.not(b)) == FALSE && !self.full
    }

    pub(crate) fn exceeded(&self) -> bool {
        self.full
    }

    pub(crate) fn known(&self, a: Guard, b: Guard) -> Option<Guard> {
        if a == FALSE || b == FALSE || a == self.not(b) {
            Some(FALSE)
        } else if a == TRUE || a == b {
            Some(b)
        } else if b == TRUE {
            Some(a)
        } else {
            self.cache.get(&(a, b)).copied()
        }
    }

    pub(crate) fn split(&self, value: Guard, var: usize) -> (Guard, Guard) {
        let node = self.nodes[value >> 1];
        if node.var == var {
            (node.low ^ (value & 1), node.high ^ (value & 1))
        } else {
            (value, value)
        }
    }

    pub(crate) fn node(&mut self, var: usize, low: Guard, high: Guard) -> Guard {
        if low == high {
            return low;
        }
        let bit = low & 1;
        let node = Node {
            var,
            low: low ^ bit,
            high: high ^ bit,
        };
        if let Some(value) = self.unique.get(&node) {
            return value ^ bit;
        }
        if self.nodes.len() >= MAX_NODES {
            self.full = true;
            return TRUE;
        }
        let value = self.nodes.len() << 1;
        self.nodes.push(node);
        self.unique.insert(node, value);
        value ^ bit
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn eval(flow: &Flow, mut guard: Guard, values: &[bool]) -> bool {
        while guard > TRUE {
            let node = flow.nodes[guard >> 1];
            let next = if values[node.var - 1] {
                node.high
            } else {
                node.low
            };
            guard = next ^ (guard & 1);
        }
        guard == TRUE
    }

    #[test]
    pub(crate) fn boolean_operations_match_truth_tables() {
        let mut flow = Flow::default();
        let a = flow.fresh();
        let b = flow.fresh();
        let c = flow.fresh();
        let both = flow.and(a, b);
        let either = flow.or(a, b);
        let left = flow.and(a, b);
        let right = flow.and(flow.not(a), c);
        let select = flow.or(left, right);
        for mask in 0..8 {
            let values = [mask & 1 != 0, mask & 2 != 0, mask & 4 != 0];
            assert_eq!(eval(&flow, both, &values), values[0] && values[1]);
            assert_eq!(eval(&flow, either, &values), values[0] || values[1]);
            assert_eq!(
                eval(&flow, flow.not(either), &values),
                !(values[0] || values[1])
            );
            assert_eq!(
                eval(&flow, select, &values),
                if values[0] { values[1] } else { values[2] }
            );
        }
        assert!(!flow.exceeded());
    }

    #[test]
    pub(crate) fn canonical_guards_preserve_boolean_identities() {
        let mut flow = Flow::new();
        let a = flow.fresh();
        let b = flow.fresh();
        assert_eq!(flow.and(a, TRUE), a);
        assert_eq!(flow.or(a, FALSE), a);
        assert_eq!(flow.and(a, flow.not(a)), FALSE);
        assert_eq!(flow.or(a, flow.not(a)), TRUE);
        assert_eq!(flow.not(flow.not(a)), a);
        assert_eq!(flow.and(a, b), flow.and(b, a));
        let either = flow.or(a, b);
        assert_eq!(flow.and(a, either), a);
        let neither = flow.and(flow.not(a), flow.not(b));
        assert_eq!(flow.not(either), neither);
    }

    #[test]
    pub(crate) fn correlated_branches_partition_the_parent_guard() {
        let mut flow = Flow::new();
        let parent = flow.fresh();
        let present = flow.fresh();
        let yes = flow.and(parent, present);
        let no = flow.and(parent, flow.not(present));
        assert!(!flow.overlap(yes, no));
        assert!(flow.overlap(parent, yes));
        assert!(flow.implies(yes, present));
        assert!(flow.implies(no, flow.not(present)));
        assert!(!flow.implies(parent, present));
        assert_eq!(flow.or(yes, no), parent);
        assert!(flow.implies(FALSE, present));
    }

    #[test]
    pub(crate) fn long_guards_use_bounded_storage_and_no_recursive_walk() {
        let mut flow = Flow::new();
        let first = flow.fresh();
        let mut guard = first;
        for _ in 1..16_384 {
            let next = flow.fresh();
            guard = flow.and(guard, next);
        }
        assert!(eval(&flow, guard, &vec![true; 16_384]));
        assert!(flow.implies(guard, first));
        assert!(!flow.overlap(guard, flow.not(first)));
        assert_eq!(flow.or(guard, flow.not(guard)), TRUE);
        assert!(!flow.exceeded());
        assert!(flow.nodes.len() < 2 * 16_384 + 1);
        assert!(flow.cache.len() < 3 * 16_384);
    }

    #[test]
    pub(crate) fn resource_exhaustion_is_sticky_and_cannot_prove_exclusion() {
        let mut flow = Flow::new();
        let guard = flow.fresh();
        for _ in 1..MAX_NODES {
            flow.fresh();
        }
        assert!(flow.exceeded());
        assert_eq!(flow.nodes.len(), MAX_NODES);
        assert!(flow.overlap(guard, flow.not(guard)));
        assert!(!flow.implies(FALSE, TRUE));
        assert_eq!(flow.fresh(), TRUE);
        assert_eq!(flow.or(FALSE, FALSE), TRUE);
    }
}
