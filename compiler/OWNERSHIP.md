# Storage and borrow implementation

The language contract is `../docs/reference/memory.md`. This file describes the
implementation boundary; it does not change language rules.

## Shared references and loan liveness

- A physical place names an ordinary local plus zero or more record field indices.
  Its type is the declared storage type, independent of flow narrowing.
- Borrowing uses that storage's address. It must not copy the referent into a new
  temporary. Distinct live locals retain distinct identities in both profiles.
- Shared references are non-null target-width pointers and copy by value.
  Equality compares addresses; dereference copies the supported copyable referent.
- Eligible roots are immutable or mutable ordinary local bindings. Parameters, dispatch
  receivers and named emitted bindings require their own place/identity work first.
  A narrowed union payload is not an addressable record projection yet.
- Each HIR emission has a unique ID, including generated record components and
  unreachable writes. Private tables associate those IDs and block IDs with the
  existing guard arena; lowering does not reinterpret source spans as identities.
- The borrow-origin pass preserves every possible `(component, place, guard)`
  alternative through immutable references and records carrying references. It checks
  the intersection of origin, write and target completion guards. A retained root
  must live strictly outside the receiving block; otherwise the escape is E303.
- Completion proofs include named leave and exclude discarded restart, panic and
  enclosing-leave paths. Discarded emissions retain operand evaluation and effects.
  Constant-unreachable results do not manufacture lifetime errors.
- Conditional results may choose different surviving roots in each component.
  Missing per-component origin coverage or more than 4,096 alternatives/components
  in one result reports B001. Guard budget
  exhaustion takes precedence over tentative lifetime diagnostics. Assigning a
  predicate invalidates its old facts; correlated safe transfers after reassignment
  can still be conservatively rejected until stronger dataflow is implemented.
- Mutable reference-bearing bindings, exclusive loans, reference-carrying unions
  and signatures, temporary borrows and reference dispatch are B001.
  These are capability boundaries, not new language errors.
- All supported referents are copyable and have no owned cleanup. Nothing in this
  milestone implements moves, owner destruction or panic unwinding.

## Records carrying references

- Immutable record primaries, named fields and nested records can contain shared
  references. Each origin path uses index 0 for the primary and the HIR field
  index plus one for a named field. Component paths are distinct from physical
  referent field paths and are preserved independently through aliases and copies.
- Emitting a record composes its primary and named components without collapsing
  their origins. Named/nested emissions prefix origin paths; field or primary
  projection removes exactly that prefix. Every retained reference component must
  have origins covering the target slot's complete execution guard.
- A projection can return a surviving owner's reference from a local carrier.
  Returning the entire carrier must prove every component survives, including
  components a later caller might ignore. Escaping any local referent is E303.
- The loan graph gives each reference component its own value ID. Projecting
  `pair.left` reads only that component; reading `pair.count` reads no reference
  components. A whole-record copy or equality operand consumes every contained
  reference, even if a later use selects only one component of the copy.
- Numeric/scalar contexts and formatting project the primary before loan analysis.
  Formatting a scalar primary does not keep named reference fields live. A primary
  reference still requires explicit dereference for formatting. Record equality
  retains known full-record context through block operands instead of silently
  comparing their scalar primaries; scalar literals retain numeric-primary width.
- Union wrappers containing references, including omitted optional reference
  fields, remain B001. Borrowing storage rooted in a reference-carrying record,
  mutable carriers and reference-carrying function signatures remain separate
  work; this milestone copies contained shared references rather than reborrowing
  the carrier or inventing a transitive lifetime contract.

## Control-flow and last use

- `src/loans.rs` builds a separate graph for the entry body and every function.
  Its reference value IDs distinguish immutable local components, expression
  temporaries and block result components from physical referent storage IDs.
- Nodes represent reference definitions and reads, result initialization and
  transfer, consuming operations, ordinary local assignment and branch targets.
  Operand evaluation happens before its consuming node. Assignment writes happen
  after the entire right-hand side, so `owner = *view + 1` is valid when that is
  the view's final use. A reference held by an equality operand remains live
  while the other operand runs.
- Backwards liveness reaches a fixed point over normal, leave and restart edges.
  Value definitions end earlier liveness, allowing each iteration to create and
  finish its own borrow. A reference retained outside a loop remains live when
  any subsequent iteration can use it.
- Normal edges retain the type checker's condition guards. Writes are rejected
  with E302 when their reachability, a live reference's future use and its origin
  can overlap. Proven disjoint write/use branches and conditional origin choices
  remain accepted. Whole-record assignment overlaps every borrowed field.
- Restart edges clear guard correlations propagated across iterations, both for
  reachability and future liveness. This is conservative: a loop whose safety
  requires relations between predicate values in different iterations can report
  E302 even when a stronger temporal proof would establish safety. There is no
  fixed iteration count or assumption that a restart executes at most once.
- A normally completed reference result is evaluated and transferred at the
  block end, so its result slot remains live throughout completing construction.
  Restart, panic and enclosing-leave paths that discard construction have no such
  result use. This preserves the existing result-origin lifetime boundary.
- Bounds are 65,536 graph nodes, 65,536 reference value IDs, 262,144 stored origin
  alternatives, 262,144 retained liveness entries and 1,048,576 analysis work steps
  per body, including final origin-overlap scans. Exhaustion reports
  B001; guard arena exhaustion overrides tentative E302/E303 diagnostics. The
  preceding origin pass also caps its persistent local/block fact cache at 262,144
  alternatives for the whole program. Dense-reference and many-origin alias
  source regressions exercise each stage's rejection before unbounded growth.
  A many-component carrier with repeated copies also verifies the per-leaf value
  budget; sharing one physical owner does not bypass component accounting.
- This graph currently enforces shared-loan/write conflicts only. Field writes,
  exclusive references and reborrows, reference reassignment, owner moves,
  temporary owners, unions carrying references and cleanup edges remain
  unimplemented. Ordinary scalar/record reads may overlap shared references.

## Next analysis stages

1. Extend result-origin support to reference-carrying unions and verified
   function/call contracts, preserving component and branch identity. Keep unknown
   origins rejected until complete caller/callee lifetime evidence exists.
2. Extend the existing graph with owned initialization, moves, scope ends,
   verified call effects and cleanup edges. Keep storage IDs distinct from values.
   Named leave/restart edges must preserve their exact target and owner lifetimes.
3. Improve predicate relationships across loop iterations without replacing the
   conservative restart boundary until sound temporal proofs are available.
4. Check live shared/exclusive loans against overlapping places. Whole-owner access
   overlaps every field; different proven record fields can be disjoint. Reborrows
   suspend conflicting parent access. Dynamic indexing remains conservative.
5. Track copy/move capabilities and partial initialization; reject moved reads and
   moving owners out of borrowed storage. End references before moving/destroying
   their owner. Verify all-input returned-view contracts at functions and callers.
6. Materialize temporary owners to complete-statement boundaries, with cleanup on
   normal, leave, restart and unwind edges. Construction cleans only initialized
   slots. Coordinate task joins before owner cleanup with the runtime prototype.

## Verification

Execute `reference_identity` unchanged and native scalar/record dereference cases
in debug and release. Check complementary owner selections, aliases, discarded
emissions and all possible escaping roots. Check unsupported ownership boundaries,
shadowing and storage provenance. Preserve all scalar/union checks. No reference
fixture depending on `bytes` becomes supported just from pointer lowering.
