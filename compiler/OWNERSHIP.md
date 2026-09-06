# Storage and borrow implementation

The language contract is `../docs/reference/memory.md`. This file describes the
implementation boundary; it does not change language rules.

## First shared-reference milestone

- A physical place names an ordinary local plus zero or more record field indices.
  Its type is the declared storage type, independent of flow narrowing.
- Borrowing uses that storage's address. It must not copy the referent into a new
  temporary. Distinct live locals retain distinct identities in both profiles.
- Shared references are non-null target-width pointers and copy by value.
  Equality compares addresses; dereference copies the supported copyable referent.
- Eligible roots are immutable ordinary local bindings. Parameters, dispatch
  receivers and named emitted bindings require their own place/identity work first.
  A narrowed union payload is not an addressable record projection yet.
- The borrow-origin pass follows immutable reference aliases and physical roots.
  A final direct reference emission into its own completing block, outside a
  conditional branch, is E303 when its storage ends with that block. Other reference emissions are B001 pending
  guarded result-origin contracts, including discarded and uncertain transfers.
  Emission alone does not prove that a result survives to scope completion.
- Mutable reference bindings, mutable roots, exclusive loans, reference-carrying
  aggregates/signatures, temporary borrows and reference dispatch are B001.
  These are capability boundaries, not new language errors.
- All supported referents are copyable and have no owned cleanup. Nothing in this
  milestone implements moves, owner destruction or panic unwinding.

## Next analysis stages

1. Introduce a control-flow graph with explicit reads, writes, initialization,
   scope ends, calls and cleanup edges. Keep storage IDs distinct from SSA values.
   Named leave/restart edges must preserve their exact target and owner lifetimes.
2. Propagate borrow origins through values and result slots. Compute backwards
   last-use liveness to a fixed point over loops; retain current guard proofs when
   proving disjoint paths. Budget exhaustion must reject with B001.
3. Check live shared/exclusive loans against overlapping places. Whole-owner access
   overlaps every field; different proven record fields can be disjoint. Reborrows
   suspend conflicting parent access. Dynamic indexing remains conservative.
4. Track copy/move capabilities and partial initialization; reject moved reads and
   moving owners out of borrowed storage. End references before moving/destroying
   their owner. Verify all-input returned-view contracts at functions and callers.
5. Materialize temporary owners to complete-statement boundaries, with cleanup on
   normal, leave, restart and unwind edges. Construction cleans only initialized
   slots. Coordinate task joins before owner cleanup with the runtime prototype.

## Verification

Execute `reference_identity` unchanged and native scalar/record dereference cases
in debug and release. Check local escapes, unsupported ownership boundaries,
shadowing and storage provenance. Preserve all scalar/union checks. No reference
fixture depending on `bytes` becomes supported just from pointer lowering.
