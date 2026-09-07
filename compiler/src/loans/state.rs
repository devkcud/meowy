use super::{
    BTreeMap, BlockId, Bundle, Facts, Flow, Guard, LocalId, Origin, Place, Program, Proofs, Span,
    Type,
};

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
    pub(crate) transfers: Vec<(usize, usize)>,
    pub(crate) write: Option<(Place, Span)>,
    pub(crate) next: Vec<Edge>,
}

pub(crate) struct Scope {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) result: Bundle,
    pub(crate) ty: Type,
}

pub(crate) struct Graph<'a> {
    pub(crate) program: &'a Program,
    pub(crate) facts: &'a Facts,
    pub(crate) proofs: &'a Proofs,
    pub(crate) guards: &'a mut Flow,
    pub(crate) nodes: Vec<Node>,
    pub(crate) values: Vec<Vec<Origin>>,
    pub(crate) locals: BTreeMap<LocalId, Bundle>,
    pub(crate) blocks: BTreeMap<BlockId, Scope>,
    pub(crate) statements: Vec<crate::hir::StatementId>,
    pub(crate) current: Vec<usize>,
    pub(crate) work: usize,
    pub(crate) origins: usize,
    pub(crate) missing_calls: Vec<(usize, Span)>,
    pub(crate) missing_reborrows: Vec<(usize, Span)>,
}
