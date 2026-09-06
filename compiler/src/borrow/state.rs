use super::{
    BTreeMap, BTreeSet, BlockId, CallId, EmitId, Guard, Guards, LocalId, Program, ReborrowId, Span,
    State, Tags, Type,
};

#[derive(Clone)]
pub(crate) struct Alias {
    pub(crate) target: BlockId,
    pub(crate) field: String,
    pub(crate) emission: EmitId,
    pub(crate) root: LocalId,
    pub(crate) span: Span,
}

#[derive(Default)]
pub(crate) struct Proofs {
    pub(crate) completions: BTreeMap<BlockId, Guard>,
    pub(crate) emissions: BTreeMap<EmitId, Guard>,
    pub(crate) conditions: BTreeMap<(usize, usize), Guard>,
    pub(crate) bindings: BTreeMap<LocalId, Guard>,
    pub(crate) tags: Tags,
    pub(crate) mutable: BTreeSet<LocalId>,
    pub(crate) calls: BTreeMap<CallId, Guard>,
    pub(crate) aliases: BTreeMap<LocalId, Alias>,
}

#[derive(Default)]
pub(crate) struct Facts {
    pub(crate) locals: BTreeMap<LocalId, State>,
    pub(crate) blocks: BTreeMap<BlockId, State>,
    pub(crate) calls: BTreeMap<CallId, State>,
    pub(crate) reborrows: BTreeMap<ReborrowId, State>,
}

#[derive(Clone)]
pub(crate) struct Storage {
    pub(crate) block: BlockId,
    pub(crate) state: State,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Exit {
    Leave(BlockId),
    Restart(BlockId),
}

pub(crate) struct Flow {
    pub(crate) next: bool,
    pub(crate) exits: BTreeSet<Exit>,
}

impl Flow {
    pub(crate) fn new() -> Self {
        Self {
            next: true,
            exits: BTreeSet::new(),
        }
    }

    pub(crate) fn append(&mut self, next: Self) {
        self.next = next.next;
        self.exits.extend(next.exits);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.next |= other.next;
        self.exits.extend(other.exits);
    }
}

pub(crate) struct Value {
    pub(crate) state: State,
    pub(crate) flow: Flow,
}

pub(crate) struct Checker<'a> {
    pub(crate) program: &'a Program,
    pub(crate) guards: &'a mut Guards,
    pub(crate) proofs: &'a Proofs,
    pub(crate) locals: BTreeMap<LocalId, Storage>,
    pub(crate) blocks: Vec<BlockId>,
    pub(crate) types: BTreeMap<BlockId, Type>,
    pub(crate) results: BTreeMap<BlockId, State>,
    pub(crate) writes: BTreeMap<(BlockId, Option<String>), Guard>,
    pub(crate) scopes: Vec<Vec<LocalId>>,
    pub(crate) facts: Facts,
    pub(crate) origins: usize,
    pub(crate) assumed: Guard,
    pub(crate) assumed_scopes: Vec<Guard>,
    pub(crate) inputs: BTreeSet<LocalId>,
}
