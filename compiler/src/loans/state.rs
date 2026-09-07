use super::branches::{Arm, Versions};
use super::{BTreeMap, BlockId, Bundle, Facts, Flow, Guard, LocalId, Program, Proofs, Span, Type};

#[derive(Clone)]
pub(crate) struct Edge {
    pub(crate) target: usize,
    pub(crate) guard: Guard,
    pub(crate) reset: bool,
}

#[derive(Default)]
pub(crate) struct Node {
    pub(crate) uses: Vec<usize>,
    pub(crate) defs: Vec<usize>,
    pub(crate) transfers: Vec<(usize, usize, Guard)>,
    pub(crate) copies: Vec<(usize, usize, Guard)>,
    pub(crate) opaque: Vec<usize>,
    pub(crate) events: Vec<super::storage::Event>,
    pub(crate) barrier: Option<Span>,
    pub(crate) access: Option<super::access::Access>,
    pub(crate) next: Vec<Edge>,
}

pub(crate) struct Scope {
    pub(crate) life: super::storage::ScopeId,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) result: Bundle,
    pub(crate) ty: Type,
    pub(crate) incoming: Versions,
    pub(crate) leaves: Vec<Arm>,
    pub(crate) restarted: bool,
}

pub(crate) struct Graph<'a> {
    pub(crate) program: &'a Program,
    pub(crate) facts: &'a Facts,
    pub(crate) proofs: &'a Proofs,
    pub(crate) guards: &'a mut Flow,
    pub(crate) nodes: Vec<Node>,
    pub(crate) values: Vec<super::values::Value>,
    pub(crate) loans: Vec<super::authority::Loan>,
    pub(crate) authority: Vec<super::authority::Authority>,
    pub(crate) locals: BTreeMap<LocalId, Bundle>,
    pub(crate) blocks: BTreeMap<BlockId, Scope>,
    pub(crate) scopes: Vec<super::storage::Frame>,
    pub(crate) active: Vec<super::storage::ScopeId>,
    pub(crate) stores: BTreeMap<LocalId, super::storage::Store>,
    pub(crate) availability: Vec<super::init::Values>,
    pub(crate) current: Vec<usize>,
    pub(crate) work: usize,
    pub(crate) origins: usize,
    pub(crate) merging: bool,
    pub(crate) output: Option<BlockId>,
    pub(crate) missing_calls: Vec<(usize, Span)>,
    pub(crate) missing_reborrows: Vec<(usize, Span)>,
    pub(crate) missing_headers: Vec<(usize, Guard)>,
}
