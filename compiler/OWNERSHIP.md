# Storage and borrow implementation

The language contract is `../docs/reference/memory.md`. This file describes the
implementation boundary; it does not change language rules.

## Shared references and loan liveness

- A physical place names a local ID plus zero or more record field indices.
  Its type is the declared storage type, independent of flow narrowing.
- Borrowing uses that storage's address. It must not copy the referent into a new
  temporary. Distinct live locals retain distinct identities in both profiles.
- Shared references are non-null target-width pointers and copy by value.
  Equality compares addresses; dereference copies the supported copyable referent.
- Eligible roots include ordinary locals, by-value parameters and dispatch receiver
  copies. Reference-bearing referents carry transitive value summaries as described
  below. Parameter/self addresses refer to their
  local storage, not an original caller value. Emitted storage uses the slot model below.
  A narrowed union payload is not an addressable record projection yet.
- Each HIR emission has a unique ID, including generated record components and
  unreachable writes. Private tables associate those IDs and block IDs with the
  existing guard arena; lowering does not reinterpret source spans as identities.
- The borrow-origin pass preserves every possible `(component, place, guard)`
  alternative through immutable references, records and unions carrying references. It checks
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
- Mutable reference-bearing bindings, exclusive loans/reborrows and temporary-owner
  borrows remain B001. Direct signatures and dispatch blocks can carry shared
  references and immutable record/union carriers.
  These are capability boundaries, not new language errors.
- All supported referents are copyable and have no owned cleanup. Nothing in this
  milestone implements moves, owner destruction or panic unwinding.

## Records carrying references

- Immutable record primaries, named fields and nested records can contain shared
  references. Each origin path uses `Slot(0)` for the primary and `Slot(index + 1)`
  for a named field. Component paths are distinct from physical
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
  components. A whole-record copy or equality operand consumes every directly
  contained active reference, even if a later use selects only one component.
- Numeric/scalar contexts and formatting project the primary before loan analysis.
  Formatting a scalar primary does not keep named reference fields live. A primary
  reference still requires explicit dereference for formatting. Record equality
  retains known full-record context through block operands instead of silently
  comparing their scalar primaries; scalar literals retain numeric-primary width.
- Union wrappers containing references and omitted optional reference fields are
  supported. Concrete fields of a carrier can be borrowed without reading its other
  reference components. Whole-carrier and reference-cell borrows preserve separate
  cell and pointee origins. Mutable carriers and indirect/capturing function
  contracts remain separate work.

## Transitive pointee summaries

- `Step::Deref` extends the existing flat component paths. The borrowed cell's
  physical origin is at the reference component; its stored value's origins, bounds
  and active variants are beneath that component's Deref step. Nested references
  repeat this step without recursive State objects or capacity enumeration.
- A whole-carrier or reference-cell borrow snapshots immutable reference-bearing
  contents. Copying a reference transfers its summary metadata, but does not itself
  read the pointee's reference fields. Runtime pointer uses consume the direct
  component; demanded summary uses propagate backwards through explicit CFG
  transfers to the original stored reference values.
- Dereference in a copy context selects the Deref subtree. A copied contained
  reference can survive the containing Local/Slot cell when its own pointee survives.
  The outer cell origin ends at the load. Public function bounds are attached to
  every returned transitive reference leaf, so dereferencing a call result cannot
  remove an ignored input's lifetime constraint.
- Field reads and reborrows project the relevant summary before checking loaded
  origins. Reading `outer.count` or borrowing `&outer.count` does not read an unrelated
  `outer.view` pointee. A whole dereference-copy reads every directly contained
  reference component. Address equality reads only the pointer components.
- A reborrow projects the physical pointer source and keeps its outer bounds.
  Its summary is the selected referent subtree. Crossing an intermediate stored
  reference, as in `&outer.view.field`, loads that pointer once and continues from
  its original pointee source; taking `&outer.view` instead addresses the reference cell.
- Reference-free dereferences still produce fresh unknown activity. They cannot
  replay a mutable referent's borrow-time union tag. Mutation of reference-bearing
  storage remains unavailable, so supported transitive snapshots cannot become stale
  through reference reassignment. Restart continues to clear iteration proofs.
- Publishing a whole borrowed carrier must prove that its physical cell and every
  transitive reference source outlive the receiving block. An unused field does not
  excuse a shorter-lived source in a published whole-carrier summary. Directly
  borrowing a separate reference-free field retains only that selected storage.
- Extra entries, Deref paths, summary copies, projection scans and graph transfers
  consume the existing work and persistent-state budgets. Long reference chains
  and duplicating carrier shapes fail with B001 before unbounded summary growth.
  Reference construction also caps inferred and aliased chains at 64 reference
  layers before deeper Type copies can reach origin analysis.

## Active union variants

- `src/borrow_value.rs` stores explicit active-member facts alongside reference
  origins, including null and members without references. `Variant(index)` path
  steps differ from record slots. Injection adds the step, extraction removes it,
  and widening/narrowing maps member types to the destination union's normalized
  indices. Reusing an old numeric tag index would change the origin's identity.
- Omitted nullable result slots receive explicit null activity on completing paths
  without writes. Completeness requires origins for every reference leaf of each
  active member; inactive leaves require none. Unknown activity is represented by
  a bounded disjoint guard partition, never by an unexplained empty origin set.
- Each immutable local/field tag domain is linked to its constructor or copied
  activity at the binding. The link is conditional on binding execution and the
  enclosing active variant, so same-named optional fields in different record
  members cannot impose contradictory unconditional tag facts.
- Value snapshots retain the proofs needed by copied or emitted origins after a
  local carrier ends. Origin checking uses cached lexical assumptions; the CFG
  applies snapshots at binding and completed-result edges. There is no global
  assumption reapplied after restart. Reset edges erase those relations along
  with other iteration facts; analyses needing stronger temporal relations remain
  conservative.
- Reads of mutable non-reference storage receive unknown activity. They do not
  reuse initializer tags after assignment or branch joins. Mutable carriers of
  references remain B001, so this does not introduce reference reassignment.
- A type predicate inspects tags without reading payload references. Effectful
  operands still run, and fresh record/block construction still has its normal
  result-transfer uses. Whole copies and equality consume their active payloads;
  projection after extraction reads only demanded components of the selected arm.
- Contextual record constructors infer an actual shape from completing named
  emissions and must select exactly one compatible union member. Candidate field
  and primary types supply literal widths; omitted nullable fields receive member
  defaults. Multiple possible widths or member identities report E207, and
  explicit annotations remain enforced. Already typed record emissions remain
  whole alternatives rather than merging their fields across branches.
- Union equality requires the same normalized union type. Compare a nullable
  union against an explicitly typed null value of that union, or inspect its null
  tag; equality does not introduce a new member conversion.

## Direct function contracts

- Direct, inferred-result, recursive and forward-group function signatures may
  pass and return shared references and immutable record/union carriers. Every
  definition is checked independently, including uncalled and mutually recursive
  definitions. No body choice weakens the public all-input lifetime contract.
- Actual origins distinguish physical `Local` storage, target-owned emitted `Slot`
  cells and symbolic `Input` sources. Each active parameter reference leaf gets its
  own symbolic source;
  the parameter's stack copy is not its referent. A completed return must borrow
  from such input sources. Returning any local actual origin or dependency is
  E303. A parameter's own address is a Local source and can only be used while
  its function storage survives; it never receives the Input-source exemption.
- `State.bounds` is separate from actual pointer origins. At a call, possible
  actual sources are compatible whole input referents, concrete named fields and
  bounded-list element regions, using the result leaf's exact reference type. Each returned reference also inherits every active input
  origin and transitive bound, including ignored inputs of another referent type.
  These are lifetime/loan dependencies, not claims about pointer identity.
- Input component paths can descend through Deref to identify references stored
  inside borrowed cells. A returned reference-bearing pointee receives the matching
  input summary under bounded, mutually exclusive candidate guards. Distinct
  candidate cells cannot assert conflicting active tags simultaneously. Every
  transitive returned reference leaf also receives the public all-input bounds.
- Consequently, `first(p, q)` remains bounded by both inputs even if the body
  returns only `p`. An ignored shorter-lived `q` causes E303 on escape, and a write
  to its owner before a returned reference's final use causes E302. Bounds survive
  wrapper calls and copies; scalar-only projections and inactive result variants
  carry no continuing reference loan. Temporary carrier arguments constrain the
  contained referents rather than the carrier's temporary storage.
- Every HIR call has an explicit site ID. `Facts.calls` stores a fresh substituted
  snapshot per site; it is never cached by function ID. The CFG initializes input
  components/proofs, evaluates all arguments in order, consumes their temporary
  references together at the call node and defines the returned components.
  An early argument exit prevents call consumption. Missing snapshots on a
  potentially reachable call are B001, not an invented empty result.
- A completed reference result requires an active compatible input source under
  the currently supported capability set. Shared reborrows preserve those input
  sources or their concrete field/list-element descendants.
  There is no static safe-reference construction, allocation or capture path that
  could supply another source. Calls without such a source have no returning reference path; nullable
  results can still return null. An entered-call guard, captured after argument
  evaluation, conditions the normal-return proof so earlier leaves and skipped
  calls remain reachable. This rule must be extended before enabling static
  reference sources, additional addressable projections or reference-producing intrinsic contracts.
- The CFG applies the call snapshot's presence/proof only on its returning edge;
  restart still erases iteration relations. Signature-based result activity may
  be more conservative than a particular body. Exclusive reborrows/access,
  mutable carriers, captures, indirect calls and owned cleanup remain unsupported.
  Existing string values are literal-backed static views and do not create local
  referent-storage dependencies merely by passing a string value.

## Parameter and dispatch storage

- Parameters and their concrete fields may be borrowed within the function or its
  nested blocks. Returning their own cell addresses,
  directly or through another call/dispatch,
  is E303. Every definition is checked even when callers infer no normal return.
- A by-value dispatch receiver is copied into an immutable `self` local before the
  body runs. Borrowing `self` or its concrete fields observes that copy. Changes
  to an original mutable owner do not change it; its address cannot leave the block.
- A shared-reference receiver instead copies the reference value. Returning `self`
  or a reborrow of its referent retains the original sources and inherited bounds.
  Reference-bearing record/union receivers use the same component/variant facts
  as ordinary immutable bindings, without adding a function-style all-input bound.
- Receiver expressions and function arguments evaluate once in order. Source/native
  tests distinguish original versus copied addresses, nullable carrier dispatch,
  earlier argument copies, effectful receivers and enclosing-scope early leaves.
- Borrowing a whole reference-bearing receiver follows the copied receiver's cell
  lifetime, while its contained reference values keep their original sources.
  Shared dispatch does not enable exclusive `self` mutation or captures.

## Shared reborrows

- `&*view`, `&view.field` and parenthesized concrete field paths address the
  original shared referent. The HIR reborrow node evaluates its parent exactly
  once; lowering applies typed field-address operations without copying records.
  Temporary reference values from calls/blocks are allowed because their referents
  retain the original lifetime. Leading carrier fields may produce that reference,
  as in `&holder.view.field`; the prefix evaluates once. Reaching a reference only
  at the final field instead requests its reference-cell storage (`&holder.view`).
  A holder's own reference-free field uses its physical storage origin instead;
  it does not inherit unrelated contained references. Temporary owners remain B001.
- Pure address hints resolve types without lowering expressions or changing
  application control flow. Regression coverage includes effectful calls in
  equality and argument blocks that leave an enclosing scope before the call.
- Input component paths identify references stored in parameters. Separate
  referent paths use `Field(index)` and `Element` steps for Local, Slot and Input
  sources. Element steps conservatively name any element in that particular list region;
  runtime index expressions remain exclusively in executable HIR.
  Reborrows preserve inherited lifetime bounds unchanged rather than projecting
  ignored-input dependencies as if they were actual pointers.
- Each HIR reborrow has a unique site and bounded snapshot. The CFG consumes the
  parent dependencies at reborrow creation, then defines the projected actual
  sources plus inherited bounds. Shared parents remain readable; this does not
  implement exclusive-parent suspension or mutable reference reassignment.
  Missing snapshots are checked against final node reachability: known-dead
  branches need no invented origins, while any reachable proof gap remains B001.
- Function contracts enumerate compatible whole referents, concrete named fields
  and list-element types once per nested type path, independently of capacity.
  An input `&Record` or `&List` can supply a descendant reference with a different
  target type. The abstract list region preserves potential input bounds even at
  capacity zero; it does not prove initialized storage or bypass bounds checks.
  Fields inside union payloads, primary-ascription addresses,
  static sources and reference-producing intrinsic contracts remain separate work.
- Type walks, candidate frontiers, projected paths and snapshot expansion consume
  existing work/storage budgets. Many matching fields multiplied by returned
  reference components reject with B001 before unbounded contract expansion.

## Mutable record fields

- HIR `Field { name, ty, mutable }` keeps named-field mutability in normalized
  shape equality, ordering and union membership. LLVM payload order/layout is
  unchanged. Inferred, annotated and forwarded records retain each field's own
  flag; forwarding never stamps the outer emission flag onto nested fields.
  Branch or expected-slot mutability conflicts report E206. Whole incompatible
  record assignment keeps E207, and nullable omissions inherit declared flags.
- `owner.child.field = rhs` addresses mutable reference-free Copy local
  storage. Every crossed named field must be mutable; an immutable root/path is
  E305. Shared-reference, temporary and union-root payload targets remain
  B001. A field may hold any currently supported Copy value, including a whole
  list or record. Mixed paths check every field boundary and initialized index.
- An all-field `SetPath` retains static physical offsets and the target span. RHS evaluates
  once before the selected store. These fixed typed offsets remain valid across
  same-shape Copy owner replacement inside RHS, so no list-style reservation is
  needed. Nonreturning RHS forms no later access. Final-use RHS shared reads can
  finish; surviving overlapping field/ancestor loans report E302, while proven
  siblings remain disjoint. Function all-input bounds keep their existing scope.
- Only target and overlapping ancestor/descendant refinement domains are forgotten
  after RHS checking. Unrelated sibling predicates and immutable copied values
  remain valid. Path lookup, bounded depth and refinement scans consume shared
  work budgets; path collection checks its 256-field limit before each push.
- Mutable primary emissions and mutable fields
  whose subtree carries references also remain B001; no exclusive reference or
  mutable reference carrier capability is implied by this metadata.
- List candidate probes compare mutable flags and keep later emitted-name
  dependencies unresolved instead of reading a same-named outer binding. A known
  context permits ordinary once-only checking; unresolved scope-dependent choices
  remain B001.

## Emitted slot aliases

- A named emission evaluates its initializer once through the existing
  Bind/Emit sequence, then `SlotAlias` binds that lexical name to the actual target
  block field. The marker and private proof retain declared field mutability;
  real backing requires an exact mutability match. Reads and permitted mutable
  assignments/SetPath operations resolve that result storage; assignment is not
  a second initialization. Direct immutable alias assignment is E305; field/element
  writes require the supported reference-free owner model and mutable boundaries.
  Ordinary bindings still receive independent copies. E204/E205 initialization
  rules are unchanged.
- The lexical alias type and final slot type remain distinct. Compatible concrete
  record destinations load/coerce from the final slot and inject assignments back
  into it. A concrete Record/List alias inside a wider union field addresses that
  active member's payload for SetPath; a union-typed alias itself has no field/index
  write path. Every crossed mutable-field and bounds rule still applies.
- Missing, incompatible or non-record final destinations are permitted only when
  the original emission is proved disjoint from target completion. The initialized
  local cell then represents that discarded partial slot until leave/restart/exit.
  A retained unsupported destination is B001. Named outer emissions resolve the
  target frame rather than the alias's local declaration scope. Restarting the target
  block reinitializes its cells; inner restarts preserve initialized outer slots.
  Alias names remain lexically scoped; their shared views follow target ownership below.
- Slot aliases stay separate from ordinary addressable places. The address
  resolver supports `&name` and concrete field/list borrows only when the
  alias cell has the lexical type or contains
  it as one exact union member. A lexical union that is only a proper subset of the
  stored union has incompatible tags/layout and remains B001; no copied borrow view or silent widening is made.
  Type hints do not register a borrow; actual uses receive final backing validation.
- `Source::Slot` carries the owning target block, canonical slot root, original
  alias view ID and typed field/element projections. Lifetime lookup uses the
  target's active region rather than the alias declaration's local Storage entry;
  type lookup retains the original view even after that declaration scope closes.
  An inner block can return a slot view to code still inside its target. Publishing
  a view in that target's own result, or beyond it, is E303: completed records
  cannot contain references into their own movable result storage.
- Resolved alias metadata distinguishes compatible Result backing from Discarded
  transient backing. The latter's initialized entry cell is explicitly retained
  under the target's partial-result lifetime, including after an inner alias scope
  ends. Current Copy-only storage needs no cleanup relocation; same-slot E205 and
  the inner-restart-after-outer-emission boundary prevent reuse within a target
  iteration. Target restart ends the iteration's references; inner-loop future
  uses still keep their parent slot loans live through writes.
- Both origin analysis and the CFG use one slot-source constructor. Canonical
  roots match alias assignment/reservation identities, preserving E302 and disjoint
  fields even when aliases arise in different guarded scopes. Slot/view metadata,
  projections and lifetime/type lookups consume existing weighted work/storage
  limits. Immutable reference-bearing emitted names also use actual slot cells.
  Their copied value components preserve the initializer's pointee origins and
  all-input bounds; copying a reference adds no dependency on its containing slot.
  Borrowing a selected reference-free field instead creates only its physical Slot
  origin. A contained reference crossed by `&carrier.view.field` keeps the existing
  pointee reborrow path. Whole-carrier/reference-cell borrows add transitive summaries
  without replacing the stored value's sources. Exclusive references and mutable
  reference-bearing fields remain unavailable.
- Only mutable alias IDs enter mutable proofs and omit initializer constant/length caches.
  Existing mutable Bind/Local analysis seeds unknown activity of the lexical type
  before emission, preserving narrower type bounds and nullable omission guards.
  A later immutable result snapshot cannot reuse a pre-mutation union tag; unrelated
  immutable reference-field origins remain intact. This can conservatively lose
  knowledge about an unmodified mutable field's initial tag.
- Immutable aliases preserve their initializer constants, immutable variant/origin
  facts and known initialized list lengths. False Boolean conditions can still
  suppress unreachable arithmetic, constant arithmetic overflow retains E107, and a
  known immutable alias length supports E101 element checks. No fabricated facts
  or mutable initializer values enter this immutable path.
- Aliases of the same target block/field share a canonical conflict identity;
  mutation invalidates related alias predicate domains. Distinct initialized result
  fields stay separate. Slot identity does not convert ordinary copies into aliases.
  Alias metadata caps at 65,536 entries and charges name copies, lookup, validation
  and predicate work to the shared budget. Final validation uses completion guards
  from the same arena and never invents an absent-field address.

## Control-flow and last use

- The `src/loans.rs` entrypoint and `src/loans/` modules build a separate graph
  for the entry body and every function.
  Its reference value IDs distinguish immutable local components, expression
  temporaries and block result components from physical referent storage IDs.
- Nodes represent reference definitions and reads, result initialization and
  transfer, direct calls, consuming operations, local assignment and branch targets.
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
- Bounds are 65,536 graph nodes, 65,536 reference value IDs, 262,144 weighted origin
  entries, 262,144 retained liveness entries and 1,048,576 analysis work steps
  per body, including final origin-overlap scans. Exhaustion reports
  B001; guard arena exhaustion overrides tentative E302/E303 diagnostics. The
  preceding origin pass also caps its persistent local/block fact cache at 262,144
  weighted entries for the whole program, counting active-member facts and path
  lengths as well as origins. Each value has at most 4,096 origin/activity parts.
  The shared proof ledger limits snapshot/variant work to 4,194,304 charged steps,
  including path copies and growing merge scans. Fanout stops inside each push,
  before a whole oversized type is expanded. Dense-reference and many-origin alias
  source regressions exercise each stage's rejection before unbounded growth.
  A many-component carrier with repeated copies also verifies the per-leaf value
  budget; sharing one physical owner does not bypass component accounting.
  Wide unknown tag domains and repeated inactive-payload copies also exercise the
  activity/proof limits, even when no reference origin is currently active.
- Call snapshots count actual origins and all-input bounds separately. The
  returned-component by input expansion stops at the same per-value limit while
  it grows, and type comparison walks check each frontier push. Public-source
  regressions cover a many-input/many-result contract and an oversized referent
  type without requiring a large physical allocation.
- This graph currently enforces shared-loan/write conflicts only. Field writes,
  exclusive references/reborrows, reference reassignment, owner moves,
  temporary owners, indirect/capturing contracts and cleanup edges remain
  unimplemented. Ordinary scalar/record reads may overlap shared references.

## Inline bounded lists

- `T[N]` stores a runtime initialized length and inline capacity, with no allocator
  or automatic growth. This milestone accepts reference-free, currently copyable
  elements, including records, unions and nested bounded lists. Reference-bearing
  or uninhabited elements remain B001 until their value/cleanup semantics are ready.
- List literals, `.size()`, one-based copy indexing, same-type/capacity equality,
  whole-value replacement and value-returning `.add()` are implemented. `.add()`
  copies these Copy lists, so an immutable source stays unchanged. Only assigning
  its result back to storage requires a mutable binding.
- Inferred literals require identical normalized types among already typed
  elements. Only pure scalar literals are deferred for contextual checking; other
  expressions are checked once in source order and the emitted operands retain
  that order. No union, numeric promotion or record-primary projection reconciles
  inferred elements. Empty literals need an expected element type.
- Expected list alternatives are filtered by capacity, scalar representability,
  concrete element types and fresh list/record literal shapes. Probes never check
  expression effects or change flow proofs. Raw declared source types keep probes
  conservative when later effects can invalidate a narrowing fact.
- A unique candidate supplies element type and capacity. Pure scalar compounds
  also constrain expected candidates: grouped/chained `-`, `!`, `~`, arithmetic,
  bitwise, Boolean and comparison expressions may use literals and resolved
  immutable scalar constants. A fresh checker retains only referenced constants,
  their exact primitive types and normalized local IDs; it invokes the ordinary
  expression checker. Mutable or captured values, calls and effectful blocks are
  never replayed. Unannotated list compounds retain their already-typed semantics.
- Preflight compound probes suppress reach-dependent arithmetic errors. During
  the source-order pass, pure elements are probed at their actual reach and may
  establish a unique context before later effects. Still-ambiguous pure elements
  wait while typed/effectful expressions are checked once; their saved reach is
  restored for final checking. A nonreturning prefix can suppress later E107, but
  cannot suppress earlier arithmetic or dead-path literal/type errors. Scratch
  checks receive only definitely-dead or potentially-live reach, never live guard
  identities. Retained HIR keeps source order and ordinary assignment coercions.
- An unlabeled effectful result block may check a context-independent prefix of
  bindings, assignments and expression statements once in its ordinary live frame.
  A terminal suffix of unconditional, unannotated emissions is then
  checked against candidate element types using the prefix's actual reach. A
  unique candidate supplies that same frame's expected type before suffix checking
  and normal block finalization. The prefix is never replayed or deferred past a
  later list element, and no synthetic union substitutes for the candidates.
- Suffix probes admit pure scalar/list/fresh-record construction and same-owner
  primitive locals from the prefix or surrounding scope, preserving exact types.
  Nonconstant/mutable locals use typed unknown values with normalized local IDs;
  mutable initializer values, narrowings and live guard/borrow identities are not
  copied. Ordinary scalar probes and deferral stay constant-only, so these reads
  cannot move across later effects. Captures, reference/record/union names,
  references to names emitted by the suffix, direct matcher prefixes and effects
  after the first emission remain B001 when context is unresolved. These limits
  do not restrict ordinary checking after earlier constraints select one type.
- Prefix errors retain normal diagnostics, and Never prefixes suppress only the
  checks the ordinary checker suppresses. Common duplicate-slot/declaration errors
  are retained after every candidate fails; a candidate-specific record-composition
  error cannot discard another valid context. Multiple fits are E207 only when no
  earlier deferred or later element constraints remain; otherwise this bounded
  path returns B001 without moving later effects ahead. Pure suffix AST size/bytes
  are charged before copying, and all candidate/context work shares existing caps.
- Unknown suffix locals can lose enclosing Boolean proof relationships. If a
  failing diagnostic lies within an `&&`/`||` RHS whose scratch condition remains
  symbolic, one fresh dead-reach probe checks whether that failure is conditional.
  Group wrappers use the condition's actual lowered span. A disappearing failure
  preserves an Unknown candidate; ordinary live checking still decides a sole
  candidate, while unresolved alternatives remain B001. Unconditional structural
  failures are unchanged. Both probes and the bounded control scan share the same
  work budget; no placeholder is assigned a fabricated zero/false/string value.
- Multiple proved candidates report E207, as does no element-compatible candidate;
  when every capacity is too small, E103 applies. There is no smallest-capacity or
  default-width preference. Single-candidate literal failures retain their existing
  codes. Context-dependent effects or nested constraints that remain unresolved
  report B001 and need an explicit annotation. Selection caps alternatives at 256
  and charges source/type probe work against the shared analysis budget.
  Probes borrow declared type descriptions instead of cloning them per candidate;
  actual-type walks, failed field searches and record-shape scans also consume work.
  Scalar scratch trees are limited to 4,096 nodes. Referenced string bytes are
  charged before copying; repeated node, constant, type and lookup work consumes
  the same shared budget. Floating operations retain ordinary per-operation
  rounding/infinity behavior, unsigned negation remains invalid, and grouped
  positive signed-minimum magnitudes are not folded into compact negative literals.
- Unary operators use the operand's type or a unique literal context before
  assignment injects their result into a union. Expected unions do not turn a
  scalar operand into a union before applying `!`, `-` or `~`.
- Extents use existing checked scalar expression/constant rules. Typed width
  overflow remains E107, mixed widths remain E213, and negative or nonconstant
  extents are E104. Required extent checks run even on dead runtime paths. Effectful
  or helper-driven required evaluation remains B001. Pure constant lookup for a
  type extent does not introduce a runtime capture.
- `Type::layout()` is the checked layout source used by frontend and backend.
  Bootstrap limits are 65,536 elements and 1 MiB of inline list storage; exceeding
  those implementation budgets is B001. Target-layout arithmetic overflow is E104.
  These limits do not qualify a total native frame or stack-size budget.
- Known immutable/literal lengths and lengths shared by every completing block
  emission support E101/E103. Discarded restart/leave paths are excluded; different
  completing lengths remain unknown. Mutable bindings and function results keep
  runtime length checks. Static facts never come from the last emission alone.
- Receiver values are captured before index or append arguments run. Index checks
  use initialized length, not capacity, and preserve signedness before range checks.
  Runtime failures report P001 for bounds and P003 for capacity with the relevant
  lengths/capacity, position and source span. Equality reads only initialized
  elements, preserving element equality semantics such as NaN and signed zero.
- Whole lists and concrete record fields containing lists use existing shared
  places, reborrows and E302/E303 checks. A copied list has no continuing reference
  origin; dereference copies finish their loan before later operand effects unless
  another reference use keeps it live.
- `&values[index]` forms a checked shared reference into original list storage.
  Nested lists and concrete record fields compose element and field reborrows.
  A temporary reference-valued parent is allowed; temporary owner lists and
  copied/ascribed owner expressions remain B001. Borrowed copied parameters and
  dispatch `self` belong to their local storage and cannot escape (E303).
- `ElementBorrow` evaluates its parent pointer and snapshots initialized length
  before checking its once-evaluated index. It shares integer/static E101 rules
  with copy indexing; runtime checks report P001 before address formation. The CFG
  holds parent actual origins and inherited bounds through any returning index
  evaluation, even when the resulting element reference is discarded. Index
  panic/leave paths consume no derived reference. Owner writes conflict with a
  live parent/element loan (E302), and resume after its final use.
- Physical element pointers retain actual index identity. Abstract element paths
  conservatively overlap all indices within a list, preserving enclosing record
  fields and nested list prefixes without enumerating capacity. They retain every
  direct-function all-input dependency.
- `holder.rows[first].items[next] = rhs` replaces initialized storage through
  concrete list and mutable field layers of a mutable reference-free Copy local
  or emitted slot alias. Parentheses around path prefixes are allowed. Writes through
  shared-reference targets, narrowed union-typed roots and temporary owners remain
  B001; a concrete alias may still address its payload in a wider final slot.
  Immutable list bindings report E305. This adds no source exclusive-reference
  value or `&!` semantics; the final store requires exclusive collection access.
- `SetPath` replaces separate field/element assignment nodes. It retains the local
  ID, final target span and ordered `WriteStep::Field`/`WriteStep::Index` entries.
  Each index step contains its once-evaluated operand and original prefix span.
  Lowering captures each current list's initialized length, evaluates/checks that
  index and forms its actual element address before inspecting the next layer.
  After every index succeeds, it evaluates the contextually typed RHS once and
  stores only the selected leaf. Lengths, capacities and other elements stay intact.
  Static positions use E101; P001 identifies the failing indexed prefix, excluding
  later fields, indices and RHS. Later effects cannot precede an earlier bounds check.
- Static fields before the first index identify the whole collection region used
  for reservations and final write conflicts. All indices and contained fields
  in that first list remain conservative aliases; holder fields outside it can
  stay disjoint. Finer field/index exclusion inside the list requires later proof.
  Pure field paths keep the full precise static place and need no reservation.
- The CFG defines an internal collection reservation before the first index, reads
  it at every returning bounds/address phase, then consumes it at the final store. Completing index
  or RHS owner replacements/nested writes conflict (E302), while shared reads and
  final-use RHS borrows can finish before the store. The whole collection region
  is used for exclusive-access checks; an external shared loan live after the store is
  rejected. The reservation is not a user reference and never escapes.
- A nonreturning index skips its bounds check, later indices, RHS and store;
  a nonreturning RHS skips the store. Earlier returning phases retain their uses.
  The reservation has no use after the corresponding last executed phase, so
  owner replacement before an immediate panic/leave is allowed when no other loan
  survives. Returning index effects still require valid parent storage even when
  RHS later diverges. After RHS checking, refinement invalidation uses that same
  first-list prefix, or the complete field path for static writes; holder siblings
  and immutable copied values keep their own facts. Nodes,
  reservation values and conflict work use the existing bounded CFG budgets.
  Target flattening is iterative and limited to 256 mixed steps before allocation;
  parser depth bounds also apply. Root types are charged once and consumed layer
  by layer, avoiding repeated suffix copies. Every target/typed/CFG step is charged,
  independently of list capacities.
- Exclusive references, slices, named list aliases, removal, reference-bearing/owned elements
  and list formatting remain B001.

## Next analysis stages

1. Extend origin and all-input bounds to exclusive reborrows, static reference
   sources and documented intrinsic contracts before enabling those capabilities.
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
7. Extend storage and transitive summaries only alongside proved mutation, source
   and lifetime rules. Preserve separate cell/pointee origins across future temporary
   owners and exclusive reborrows. Add finer indexed disjointness,
   exclusive references, slice/alias metadata and non-Copy state separately.

## Verification

Execute `reference_identity` unchanged and native scalar/record dereference cases
in debug and release. Check complementary owner selections, aliases, discarded
emissions and all possible escaping roots. Check unsupported ownership boundaries,
shadowing and storage provenance. Preserve all scalar/union checks. No reference
fixture depending on `bytes` becomes supported just from pointer lowering.
