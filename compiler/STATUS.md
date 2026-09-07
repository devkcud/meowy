# Compiler handoff and work tracker

Updated: 2026-09-06. Guarded mutable-reference branch merging is complete.
Full v0.0.1 remains incomplete; source and validation are complete for this slice.
Implementation: `5c55e44`; native coverage/example: `af3a868`.
Prior mutable-reference versions: `32d093c`, `4b7d655`, `f16c30b`.
Prior reference-bearing temporary support: `c325099`, `fb72c97`, `b10ba7f`.
Organization: native `c83f1f1`, parser `d599149`, borrow `f550947`, loans `717f5af`.
Earlier organization: backend `8c8e90a`, checker `360c8db`, list contexts `e3a0803`.
Prior runtime snapshots: `d92f94c`; generated panic evidence: `eb65cbd`.
This file tracks the compiler; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).

## Current objective

Completed: fixed shared-reference locals support matcher and short-circuit
assignment merges. Each arm starts from the post-condition incoming environment;
only returning states contribute, with origins, bounds, activity and proof masked
by their returning guard. A skipped assignment keeps its old value. Earlier copies,
operand evaluation order and physical cell identity stay unchanged.

Origin/loan branch helpers isolate arm state and transfer merged demand without
reading reference contents. Per-body Facts.merging keeps assignment-free loop
behavior unchanged. Reference assignment with any leave/restart remains B001.
Next is forward leave-state merging, then bounded restart propagation; mutable
reference carriers and exclusive/owned work remain separate.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. Run `python3 -B tools/verify.py --compiler` from the root; it pins the build target
   and actual compiler path for all compiler checks.
4. Direct conformance uses `python3 -B compiler/tests/conformance.py --compiler
   compiler/target/x86_64-unknown-linux-gnu/debug/meowy`; 13 unsupported cases remain.
5. Consult the validation evidence below before claiming any gate passed.
6. After each logical step, update `Next steps` and add a newest-first checkpoint
   to `STATUS_STEP_LOG.md`. Record findings, actual checks, blockers and continuation.

The Rust compiler has no external Rust dependencies. Builds use installed Rust
1.98.1 and LLVM/Clang/LLD 22.1.8. Native context sources are vendored under
`../runtime/vendor/boost-context/` with revision, license and checksum metadata.
There is no bundled sysroot or qualified distribution. This workstation does not
qualify the documented Linux 5.4/glibc 2.31 baseline.

## Implementation map

| Component | Files | Current state |
| --- | --- | --- |
| Workspace and interfaces | `Cargo.toml`, `rust-toolchain.toml`, `src/ast.rs`, `src/hir.rs`, `src/lib.rs` | Offline bootstrap with explicit frontend/backend boundaries |
| Lexer and parser | `src/lexer.rs`, `src/parser.rs`, `src/parser/` | Bootstrap grammar, malformed-input checks and bounded tree depth |
| Names, types, flow | `src/check.rs`, `src/check/`, `src/list.rs`, `src/list_context/`, `src/flow.rs` | Record/list contexts, checked extents and bounded candidate probes; 35 checker, 18 list/context and 5 guard groups |
| Shared storage and loans | `src/borrow_value.rs`, `src/borrow_value/`, `src/borrow_contract.rs`, `src/borrow_contract/`, `src/borrow.rs`, `src/borrow/`, `src/loans.rs`, `src/loans/`, `OWNERSHIP.md` | Scoped origins/bounds, direct call contracts and E302/E303 checks; 40 origin, 46 loan, 15 contract and 2 value-budget groups |
| Native backend | `src/backend.rs`, `src/backend/`, `build.rs`, `native/` | Verified LLVM to ELF pipeline including bounded lists, records, references and tagged unions; 62 focused backend tests |
| CLI and diagnostics | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Native builds, safe output replacement and diagnostic rendering |
| Tests and examples | `tests/native.rs`, `tests/native/`, `tests/conformance.py`, `examples/`, `README.md` | 214 native groups, 4 harness tests and 32 covered examples |

The main checker module retains state and entrypoints, with semantic operations
under `src/check/`. `src/backend/` separates aggregate, list, arithmetic, output
and storage lowering plus focused tests. `src/list_context/` separates orchestration,
effectful blocks and isolated probes. Consult each root module for declarations;
preserve these responsibility boundaries during feature work. `src/parser/`
separates statements, expressions, types, strings and tree bounds. `src/borrow/`
and `src/loans/` separate state, traversal and solving. `tests/native/` groups the
single native target by behavior while sharing one temp-directory counter.
`borrow_value/pointee.rs` transforms summaries, `borrow/pointee.rs` resolves demanded
reads, `borrow_contract/call.rs` handles candidate substitution, and
`loans/transitive.rs` connects summary transfers. Contract tests live beside their
module; `check/temporaries.rs` owns temporary creation and statement metadata.
`borrow/mutable.rs` owns the bounded assignment/exit capability scan.
`borrow/branches.rs` and `loans/branches.rs` own guarded environment restoration and
returning-state joins; `loans/values.rs` owns immutable graph versions. Retain these
module boundaries during further ownership work.

Agents share this checkout. File existence does not prove a component compiles.
Interfaces remain `parser::parse`, `check::check`, `backend::emit_ir`, and
`backend::emit_object`; `src/lib.rs` composes the frontend. Follow explicit
visibility, short names, immutable bindings and the user's no-comment convention.

## Still outside this compiler

Unavailable features must not silently receive different semantics. B001 is an
internal capability diagnostic, not an assigned reference error or conformance pass.
The bootstrap JSON diagnostic stream is not a release artifact schema.

| Area | Work remaining | Required evidence |
| --- | --- | --- |
| Full frontend | Complete grammar, stable item IDs, recovery CST/editor integration, all type forms | Conformance, compact syntax properties, malformed UTF-8 and parser fuzzing |
| Type system | Literal types/unions, subtraction, callable environments, generics/capabilities, full type queries, nominal identity | Type/callable fixtures and negative boundaries |
| Required evaluation | Type-producing helpers, effects, cycle checks, logical budgets, specialization | E211/E219/E220 and deterministic budget tests |
| Ownership | Static/intrinsic sources, exclusive borrows/reborrows, reference assignment exit/backedge joins, moves, partial initialization, captures, cleanup | Caller lifetime substitution, use-after-move/borrow rejection and exact-once cleanup |
| Collections | Remaining contextual constraints, finer alias precision, exclusive access, aliases, slices, arrays, maps, vectors, allocators | Extent/count/bounds cases and allocation failures |
| Runtime | Owned allocations, recoverable panics, unwinding, tasks, channels, timers, cancellation | Generated cleanup, structured joins, one-worker progress and sanitizer coverage |
| Modules/projects | Relative imports, manifests, exports, aliases, root locks, dependency graph | Worked projects, offline locked builds and revision identity |
| Native ABI | Clang ABI adapters, explicit native artifacts, record classification | Separate C fixtures, argument/return layout and safety boundaries |
| Standard library | Meowy sources and APIs beyond foundational bootstrap output | API-to-test coverage, 16 worked projects and pinned data |
| Tooling | Test command, LSP, gatostyle/fmt, inspection and repair | Shared diagnostics, negotiated positions, safe rewrites and isolated tests |
| Artifacts/replay | Canonical readers, sessions, capsules, integrity, runtime events/replay | Schema/budget validation and replay after moving sources |
| Distribution | Bootstrap recipes, source inventory, vendoring, bundled tools/sysroot | Offline rebuild, dependency/license inventory and descriptor digests |
| Target qualification | Baseline host, static/shared closure, LTO, DWARF 5/unwind information | ELF inspection, minimum-host execution, reproducibility and sizes |

## Known limits to preserve

- Fixed shared-reference ordinary locals retain their current full State. Returning
  assignments replace Storage.state while Facts.locals retains the initial snapshot;
  earlier copies keep their own origins, bounds and transitive contents. Graph reads
  and assignments define fresh immutable value IDs. Physical LocalId cells stay stable.
- A live reference-cell view blocks its reassignment with E302; a final-use RHS
  load can end before the store. Expired values may be overwritten without reading
  them. Later reads validate the current value's origins/bounds and report E303.
  Nested assignments and earlier call operands retain evaluation order;
  diverging RHS paths never store. T may be any currently supported referent.
- Matcher arms and short-circuit right operands snapshot the incoming mutable
  reference versions after condition effects. Each arm starts independently under
  its guard; normally returning origin states are masked before merging. The skip
  path retains incoming values, and the join continuation is the union of returning
  guards. Partial panic paths inside nested expressions do not publish versions.
- Facts.merging identifies each body using reference assignments. In that mode,
  returning expressions retain normal-completion proofs after nested scope closure.
  Assignment-free bodies keep the previous proof/loop path. The bounded scan rejects
  reference assignment combined with any Leave/Restart, even unrelated nested
  transfers; separate function bodies are independent. Mutable reference-bearing
  nullable/aggregate bindings and emitted fields remain B001.
- Loan branch ends define merged versions with demand-only transfers for every
  component, including direct references. Merging creates no runtime read. Unchanged
  bundles are reused; changed origins are masked by actual completing CFG reach,
  normalized to their current projected paths and deduplicated. Copies made before
  the branch retain old IDs, while physical cell loans keep their original roots.
- Snapshots, restoration, state/graph growth and completion-reach queries consume
  existing budgets. The initial implementation recomputes reach for changed joins:
  64 repeated self-assignment joins pass, while 1,024 report loan-budget B001.
  Broader scaling remains unqualified; never replace budget rejection with incomplete
  origin or liveness evidence.
- HIR Field contains name, type and mutable flag. Structural Eq/Ord/Hash and
  constructor matching include that flag; physical layout walks only field types.
  E206 rejects declared-slot or completing-branch mutability conflicts; ordinary
  record shape assignment mismatch remains E207, and unequal shapes cannot use
  whole-record equality. Primary record forwarding retains each named field flag.
- SetPath carries a mutable local root, ordered WriteStep::Field/Index path, RHS
  and final target span. It replaces separate field/list write representations.
  `check/mutation.rs` checks reference-free Copy locals or emitted aliases and
  every mutable field gate; immutable roots/fields are E305. No source &! reference is created.
- With indices, static fields before the first index define a precise Place naming
  the whole first collection. That region drives reservations, final write conflicts
  and predicate invalidation. Holder siblings outside it can be disjoint; all views
  inside it conservatively overlap, including different fields of indexed elements.
  All-input call bounds and ancestor/whole-owner loans remain intact.
- Pure-field paths use their complete static Place without an index reservation.
  Fixed offsets survive same-type RHS owner replacement, preserving RHS-updated
  siblings. RHS completes before the final exclusive store; final-use shared reads
  are allowed. Field and indexed writes never write back an old aggregate snapshot.
- Mutable field types/subtrees containing references and mutable primary emissions
  remain B001. No origin-bearing mutable carrier can reach these new writes.
- SlotAlias follows initializer Bind+Emit for supported named emissions and
  maps the local ID to its target, field and declared mutability. `check/aliases.rs`
  requires compatible type and matching field mutability, or proves the emission
  cannot reach target completion. Alias IDs remain outside ordinary places; resolved
  alias metadata explicitly permits borrowed addresses. Actual borrow sites, not
  address hints, request late layout validation. Alias backing must match its type
  or one exact union member; reference-bearing referents require complete bounded
  summaries. Proper subunion storage views remain unsupported.
- `backend/storage.rs` resolves whole-cell reads/stores with final-slot/local type
  conversion, and matching concrete union payloads for SetPath addresses. Actual
  result cells remain backing storage when type and mutability match; only
  proved discarded destinations keep the initialized temporary cell. Backend Bind
  clears old alias metadata, and each generated function starts with an empty map.
- Source::Slot stores target ownership, canonical root, original alias view and typed
  field/element projections. Shared Proofs::source construction feeds origin analysis,
  CFG borrows and indexed-write reservations. Call substitution/reborrows retain the
  same source; overlap checks use canonical roots while type checks use the view.
- Alias backing is resolved as Result or Discarded. Both live under the target's
  partial-result lifetime. Discarded cells use existing entry allocations: once-only
  initialization and E205/inner-restart emission gates prevent reuse while that target
  iteration lives. No new allocation or cleanup capability is implied.
- Slot lifetime checks use the active target block, even after an inner alias scope
  closes. Retaining a borrow in that target's own result or an enclosing result is
  E303. Target restart ends the iteration; inner restart can preserve outer storage
  but cannot bypass future-use E302. Origin weights and lookup work remain bounded.
- Immutable alias IDs never enter mutable proofs. Existing Bind/link_tags preserves
  their activity; declared local constants remain available, and list_fact seeds
  the immutable local length cache before emission. Static E101 and constant-based
  extents use those facts, while dynamic indices retain P001. Direct/field/element
  writes through reference-free immutable aliases report E305. Mutable copies remain
  independent; mutation of reference-bearing aggregates or emitted storage remains
  outside the supported write model.
- Immutable reference-bearing named emissions also register SlotAlias. Bind/Local
  component states and CFG bundles retain original pointee origins, inherited bounds
  and active variants. A copied reference gains no dependency on its containing cell;
  it may outlive that cell while every actual origin and input bound still survives.
- `address_storage` resolves concrete inline paths. Whole carriers/reference cells
  now use transitive summaries, while selected fields can keep narrower demands.
  Crossing a stored reference loads it once through Deref/Field HIR and continues
  the pointee reborrow path; it does not confuse reference-cell and referent addresses.
- A reference-free selected-field Borrow creates only the physical Local/Slot origin,
  without reading
  unrelated stored reference components. A discarded carrier's scalar view may
  survive an unrelated pointee's scope while the target cell lives. Retained result
  emission bundles still protect every contained reference until completion.
- Mutable alias IDs enter proofs before origin traversal. Their ref-free activity
  becomes unknown from initialization, preventing stale union/record tags from
  suppressing a real loan while preserving other immutable reference components.
  Mutable alias length caches stay unset. IDs sharing a target field use one canonical
  write/reservation root and share predicate invalidation; value copies stay separate.
- Alias metadata, names, type walks and canonical lookup/invalidation use shared
  bounded work, with a 65,536-entry metadata ceiling. Slot initialization still
  obeys E204/E205/E206; assignment does not become a second emission. Restart resets
  target cells, while an inner restart can update an already-initialized outer slot.
- Pure/list context probes retain field flags and treat earlier emitted-name reads
  as Unknown/B001; they never use a same-named outer binding to choose a false type.
  Explicit annotations still use ordinary lexical checking.
- Statement wrappers are emitted only when that source statement owns temporaries.
  Generated Bind/Emit/SlotAlias sequences stay together; matcher-controlled statements
  share the matcher owner, while nested block statements get fresh IDs. Wrappers do
  not close ordinary lexical locals. Statement/temporary IDs each cap at 65,536.
- TemporaryBorrow materializes one Copy value after its initializer returns,
  preserving reference-bearing contents through the existing borrowed-state helper. Source::Temporary tracks the LocalId, StatementId and selected fields;
  the origin pass requires its statement to remain active. The loan graph validates
  owner proofs and carries the source through calls/reborrows. Backend cells are
  entry allocations reinitialized at the source site, with no loop alloca growth.
- A temporary may pass through a nested call/dispatch while its outer statement
  remains active; a temporary created by a nested binding/emission cannot escape
  that statement into its surrounding expression. Unused expired references are
  not lifetime extensions; actual later reads and retained escapes report E303.
- Reference-bearing temporary initializers retain their actual origins, bounds and
  variant activity beneath Deref. Source::Temporary identifies only the outer cell;
  direct dereference copies drop only that cell access. Original pointee and public
  all-input bounds remain, including any other temporary owner on which they depend.
- Graph::referenced shares bounded summary construction between ordinary borrows
  and temporaries. Materialization passes direct initializer uses; deeper summaries
  get conditional transfer edges. Selecting a scalar field cannot skip whole-owner
  initializer reference reads/effects, while pointer-only uses do not read pointees.
- Existing storage/reborrow paths keep original owners; computed list parents are
  evaluated once before deciding between reference-valued parents and new owners.
  Fields/indices retain the whole temporary root. Static list facts permit E101;
  dynamic P001 checks retain owner/index order. Never leaves no materialized cell;
  unary dereference propagates Never before requiring a reference type.
- Temporary operands keep ordinary types. An unannotated &1 is &int32; explicit
  typed owners work for other widths. Binding mismatch is E207, argument mismatch
  E212, and invalid narrowing remains E208. No reference-width conversion or implicit
  static promotion is introduced. Owned temporary values remain unsupported.
- Lists store initialized length separately from capacity. Only reference-free Copy
  elements are enabled, including nested lists, records and unions. Copies and
  equality inspect initialized elements; append returns the same capacity/type.
  Receiver snapshots precede argument effects. Bounds use one-based positions.
- Capacities use checked scalar constants and expressions. Required checks still
  run on dead runtime paths. Helpers, blocks and effectful required evaluation are
  B001. Bootstrap limits are 65,536 elements and 1 MiB inline layout per list, also
  B001; target-layout arithmetic overflow is E104. Total frame/stack budgets remain
  unqualified. Frontend/backend share `Type::layout()`.
- Unannotated list inference never invents unions, promotions or primary projections.
  Expected list candidates use ordinary assignment, including record-primary copies.
  Capacity, literal representability and typed/fresh aggregate shapes select a unique
  candidate; no smallest-capacity or default-width preference breaks a tie.
- Candidate probes never replay live checker state or expression effects. Pure
  scalar unary/binary trees can use literals and same-owner immutable primitive
  constants. A fresh checker retains only those constants, exact types and normalized
  local IDs, reusing ordinary expression checking. Unannotated compounds remain typed.
  Pure terminal aggregate suffixes can use the same isolated checking machinery.
- Preflight suppresses reach-dependent arithmetic errors; source-order probes use
  the actual position's reach and may select a type before later effects. Deferred
  pure elements retain their original reach. A Never prefix can suppress later E107
  but cannot erase prior arithmetic or literal/type errors. Scratch reach is only
  definitely dead or potentially live; live guard IDs never cross into it.
- Raw declared types are borrowed. Actual/expected types, lookups, record shapes,
  constant strings and repeated scratch work are charged to the shared budget.
  Limits are 256 list candidates and 4,096 nodes per scalar scratch tree. Exhaustion
  remains B001 even for pure or dead expressions. No hidden reset grants extra budget.
- When candidate element types differ, `list_effect_block` accepts an unlabeled
  block with prefix bindings/assignments/expression statements followed by terminal
  unconditional unannotated emissions. Field mutability is retained in probes.
  `block_start`/`block_end` share ordinary frame, scope, length and completion
  handling. Prefix checking runs once
  at the element's actual source position; probes never rerun that prefix.
- `list_pure` allows closed scalar/list/fresh-record suffix trees and same-owner
  primitive locals. Local defaults/shadowing resolve before scratch construction.
  Nonconstant/mutable names retain exact types and constant=None, with normalized
  IDs; mutable initializers and narrowed types are never imported. Ordinary scalar
  probes/deferral remain constant-only. Captures, reference/record/union names,
  suffix effects and emitted-name dependencies remain B001 while context is unknown.
- If an unknown suffix local loses a live Boolean relationship, only a diagnostic
  inside a symbolic scratch &&/|| RHS can trigger a fresh dead-reach retry. Group
  wrappers are normalized to the condition's actual lowered span with charged work.
  A disappearing failure preserves Unknown; live checking still decides a sole
  candidate, and multiple uncertain choices remain B001. No scratch Flow IDs or
  borrow/call sites enter live state; unconditional structural errors stay intact.
- A unique suffix fit selects the original list candidate and continues the same
  block frame with that expected type. Multiple fits report E207 only when proved;
  earlier deferred/later element constraints or an unknown suffix fit stay B001.
  Effectful blocks are never deferred. No synthetic union element type is invented.
- Suffix source bytes, AST nodes, names/constants, candidate types and repeated
  scratch work consume existing budgets (256 candidates, 4,096 pure nodes and shared
  proof work). Structural E203/E205/E206 errors remain candidate-local until all
  trials fail with the same code; another valid candidate must survive. Actual
  prefix reach controls dead arithmetic and duplicate-slot behavior.
- Remaining effectful/captured/non-scalar constraints stay B001 when no context is
  proved. Use annotations rather than guessing. Intermediate checked widths, f32
  parsing/operations, short circuits, unsigned negation and grouped signed-minimum
  rules remain those of the ordinary checker; do not evaluate only the final result.
- Known immutable/literal lengths and equal lengths on every completing block path
  give static E101/E103. Mutable/function-result lengths keep runtime checks. Never
  infer a block length from its last emission alone.
- Whole-list and initialized-element shared references address original storage.
  ElementBorrow retains the evaluated parent pointer/length, then evaluates its
  index once and applies E101/P001 before address formation. A parent loan is used
  through returning index effects even if the result is discarded. Index divergence
  preserves prior effects without a derived-reference use or later bounds failure.
- Source paths distinguish Field(index) and Element steps, separate from physical
  HIR Place/Reborrow paths. All indices in a list conservatively overlap for loans;
  runtime address equality still distinguishes actual elements. Contract search
  visits element types once per nested path, independent of capacity. Zero-capacity
  regions retain conservative input bounds but never prove initialization or remove
  runtime checks. Final local element writes conservatively overlap the whole list.
- Derived-reference snapshots are keyed by unique sites. Missing snapshots are
  recorded at their consumption node and validated after reach analysis. Known-dead
  paths do not invent origins; reachable gaps remain B001. All path/snapshot/conflict
  work stays charged. Exclusive borrows, slices, named list aliases, removal,
  reference-bearing/owned elements and list formatting remain unsupported.
- Each WriteStep::Index retains its actual indexed-prefix span. Backend traversal
  loads the selected list's own initialized length before evaluating that index,
  applies the P001 guard, then derives the pointer for the next step. Frontend
  checks retain E101 for statically invalid positions.
  Captured indices cannot be retargeted by later changes to index variables.
- The CFG consumes the first-collection reservation after every returning index
  and at the final store. Returning writes within that region fail E302 even if a
  later phase diverges. A Never index/RHS skips later uses and stores, so owner
  replacement before an immediate exit is allowed when no other loan survives.
- Target invalidation uses the first-list field prefix or full pure-field path,
  preserving disjoint sibling facts. Flattening is iterative and capped at 256
  mixed steps before allocation, with parser depth limits also applying. Typed
  walks, field lookup, region construction and conflict work use existing budgets;
  no list capacity is expanded into per-element paths.
- Locals, parameters and copied dispatch self bindings have local storage. Their
  cell addresses cannot escape that region, while direct copies of contained
  references retain their separate pointee origins. Shared self retains its original
  Local/Slot/Temporary/Input origins and inherited call bounds.
- Reference-bearing record/union dispatch preserves component/variant facts and
  evaluates receivers/arguments once. `&holder.view` borrows a reference cell;
  `&holder.view.field` can load that reference then reborrow its pointee field.
  Owned temporary values, arbitrary union-payload/primary-ascription addresses,
  exclusive borrows and mutable reference carriers remain unsupported.
- Dedicated reborrow sites retain bounded snapshots. Actual sources append referent
  field indices while inherited bounds stay unchanged. Lowering evaluates the parent
  once and derives addresses without record copies. Address hints perform no lowering;
  effectful equality operands and argument early leaves have native regressions.
- Function input components can descend through Deref, distinct from physical
  Field/Element projections. Recursive call substitution chooses guarded candidate
  pointee snapshots, preserving active union/null facts. Unrelated arguments add
  bounds without inventing contents. Every returned reference layer retains all
  active input/transitive bounds, including the outer borrowed cell where applicable.
  Scalar results and scalar-only projections release loans after consumption.
- Contract facts use unique call-site IDs and entered guards captured after argument
  evaluation. After all arguments return, full active origins/bounds are validated
  under the entered-call guard before any scalar/reference result is constructed.
  This catches expired nested temporary summaries. An early argument exit skips
  full call-entry validation; result proofs apply only on the returning edge. Every body,
  including uncalled and recursive definitions, must prove its own return sources.
- A reference result needs an active compatible input reference, a typed field/element
  projection, or a contained reference reachable from one. Without a valid source
  that path cannot return; nullable results may return null. Extend this rule before
  static sources or reference-producing intrinsics. String values remain literal-backed
  static views rather than local storage dependencies merely by value.
- Every active retained reference component must outlive its receiving block,
  even when a later consumer ignores it. A safe-field projection may leave a local
  carrier; returning the whole carrier validates all active references. Discarded
  emissions retain operand effects without escaping on discarded paths.
- Value copies read every directly contained active reference. Field/scalar-primary
  projections read only selected leaves. Tag inspection skips reference payloads in
  both origin and loan passes, but dereferences needed to access a tag must remain
  live; computed block/call operands still run their complete effects/checks.
  Fresh block/record construction still retains result-slot loans to completion.
  Reference formatting needs explicit dereference. Record equality keeps full
  shape; compatible scalar comparisons project the primary. Union equality needs
  identical normalized union types, so a raw null comparison can report E222.
- Immutable binding/result snapshots link type-test tags to actual variant activity,
  conditioned on the enclosing variant. Mutable reference-free storage reads receive
  unknown activity, avoiding stale initializer tags after writes. Record fields
  preserve mutability in their types. SetPath invalidates the selected region and
  overlapping ancestors/descendants after RHS; disjoint sibling facts survive.
- Restart edges erase iteration-specific correlations and scoped assumptions.
  Predicate assignments invalidate prior facts. Safe programs needing stronger
  temporal relationships may still receive conservative E302/E303; reads may
  overlap shared loans. No global assumption is reapplied after a reset.
- Step::Deref flattens pointee origins, bounds and activity into existing component
  paths. Borrow prefixes the current stored summary; direct dereference selects
  one subtree and drops only the outer access. Reference-free pointee reads remain
  fresh unknown states so mutable initial tags cannot become stale proofs.
- Graph Node transfers propagate destination demand back to source summary IDs
  before definitions are killed. Borrow/copy/bind/emission/block-return paths use
  these links; eager direct uses model actual pointer/value reads. Field and tag
  inspection avoid unrelated contents, while whole copies consume them. Missing
  reborrow summary links fail B001 instead of inventing untracked references.
- Shared reference construction uses a charged type walk and a 64-layer limit,
  including inferred and type-alias chains. Nested type prefixes remain separate
  tokens (`<& &int32>` or an alias); no parser/token grammar was changed.
- Values are capped at 4,096 origin/activity parts. Persistent facts and CFG origin
  storage each cap 262,144 weighted entries, counting active tags and path lengths.
  Shared snapshot/variant work is capped at 4,194,304 charged steps; CFG nodes,
  values, live entries and work retain their separate bounds. Fanout is checked
  during expansion. Exhaustion is B001 even with entirely inactive payloads.
- Union record constructors select exactly one compatible shape from completing
  emissions. Unique field/primary widths supply literal context; nullable fields
  get defaults. Ambiguous widths/member choices report E207. Discarded named writes
  do not determine the completed shape. Typed record alternatives remain whole.
- Only direct noncapturing functions are lowered. Declaration aliases work;
  first-class function-pointer storage, captures and indirect calls remain unavailable.
- Ambiguous numeric widths in an expected union report E207; bind a typed member.
  No silent width conversion is permitted. Float32 literals round directly to float32.
- Interpolation streams at print/panic boundaries without allocating a string.
  P002 retains operator, original operands, signed width/range, overflow versus
  zero-divisor cause and the full expression byte span. Signed-minimum remainder
  by -1 is zero; division panics. Integer diagnostics never re-evaluate operands.
- P006 appends its call-site bytes only after all message parts finish. Nested
  failure/leave paths keep already-streamed effects without appending an outer site
  or terminator. Calls with result Never lower arguments and the call, then emit
  unreachable and end the builder continuation. These text failures exit status 1;
  recovery, source identities, structured evidence and replay remain pending.
- Do not infer a block constant from its last emission: earlier paths may leave.
  Named blocks conservatively forget mutable proofs. Restart after a new emission
  into a surviving enclosing result remains B001; updates to an already initialized
  outer alias are supported. Loan loop fixed points exist; broader ownership and
  emission-flow fixed points remain pending.
- Parser descent and AST depth are capped at 256 before recursive passes. A prior
  65,536-parameter source timed out after 60 seconds. Broad frontend scaling, full
  input resource bounds, rich related spans and comprehensive fuzzing are unqualified.
- LLVM IR is verified before/after optimization; the C++ bridge copies bounded
  source bytes into owned LLVM storage. Compiler binaries embed the scalar runtime
  archive but require installed LLVM and exact native-tool paths. Generated output
  is separate from the Rust compiler and LLVM libraries.
- `../runtime/` has bounded cleanup, guarded contexts, parent-owned child waits and
  explicit owned capture/result transfers. `Owned` reserves caller buffers, commits
  initialized objects, relocates via static descriptors and releases exactly once.
  Accepted admission failures drop captures; failed joins retain results untouched.
  Move/drop callbacks cannot suspend; results need independently surviving storage.
  This is a trusted private C++ API, not compiler-checked payload layout/Send/Sync.
- Runtime body/cleanup callbacks must finish child joins and close every task mark
  before borrowed locals expire; otherwise a private fatal protocol error stops
  execution. Task::mark/close uses 16 fixed scope records per slot, selects children
  admitted since the innermost mark, waits on their worker and discards owned results.
- Close counts only successfully reclaimed children and preserves progress/first
  failure across retries. It reports an unreclaimed child separately; callers must
  handle child failure counts even when close itself returns ok. Detailed close
  writes every consumed child failure into a caller-provided batch,
  with per-call `reported` separate from cumulative counts. `report_full` retains
  the pending failed child and mark; release failure writes no detail for that
  child. Process every returned prefix before reusing the buffer. Summary-only
  close retains the first diagnostic. Panic values own 256 inline message bytes,
  captured while source text is live; they preserve code, original length and a
  truncation flag. Valid UTF-8 truncation keeps complete codepoints and fatal P008
  prints an explicit original-length marker. Copies survive source destruction.
- A message() view borrows that particular Panic/outcome value, so retaining a view
  from a temporary join/close result is invalid. Operation names remain separately
  borrowed/static. On this host Panic is 280 bytes, TaskSlot 6,264 and ScopeClose
  720; the 17 task-slot snapshots add 4,352 bytes. These private layouts are not a
  portable ABI or total stack budget. Generated scope exits, cancellation,
  automatic propagation and native unwinding remain unimplemented.
- Timeout supervision kills/reaps the entire compiler/application process group.
  B001 cannot hide a later fault or count as an expected language rejection.

## Validation evidence

- Final `python3 -B tools/verify.py --compiler`: all 10 selected checks pass on
  frozen production source. This slice did not rerun editor/native-runtime checks;
  their files, HIR, backend storage, runtime ABI and dependencies are unchanged.
- Rust: 239 library and 214 native groups pass (453 total). New groups: 4 origin,
  6 loan and 8 native. All 40 origin, 46 loan, 62 backend and 35 checker groups pass.
  Formatting and Clippy `-D warnings` pass; 32 examples run in both native profiles.
- Native coverage includes partial/complementary assignments, old copies, guarded
  owner/cell conflicts, scoped/temporary and public-bound expiry, nullable pointees,
  condition/argument effects, panic and nested short circuits. Independent review
  passed 12 checks and 3 effect programs in both profiles (6 executions).
- Five initial native groups failed at the previous conditional-assignment B001
  gate as the expected baseline. Rustfmt caught one misplaced raw-string test block;
  it was corrected before integration. All eight new groups passed first source
  integration and the final gate. Obsolete conditional B001 rows now preserve the
  remaining exit/backedge boundary. No reference fixture or REQUIRED changed.
- Optimized compiler build passed. That binary built and ran
  `examples/guarded-references.mwy` in the release profile: exit 0, exact stdout
  `7\n7\n7\n9\n`, empty stderr. No new ELF/linkage qualification is claimed.
- Fresh Python evidence: 16 tooling and 4 compiler-harness groups (20 total).
  Documentation checks 868 links in 86 Markdown files; schemas/catalog pass.
  Static metadata validation remains separate from compiler execution.
- Historical unchanged-runtime/editor evidence at `f16c30b`: Vim/Neovim, 15 runtime
  Python groups and native debug/release/sanitized checks passed. Runtime groups
  were 6 diagnostic, 14 cleanup, 10 stack, 10 context, 25 scheduler and 14 owned,
  with fatal, truncation, lifetime, guard and admission probes. ASan/UBSan/LSan and
  expired fiber-local detection passed outside the sandbox. These are prior checks,
  not reruns or new generated-cleanup/unwind qualification for this slice.
- Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles. Full
  language/release qualification remains open; generic temporary-borrow fixtures
  remain unsupported despite independently executed temporary-owner support.
- Prior ELF evidence found x86-64 PIE, only libc.so.6 in DT_NEEDED and GLIBC_2.34.
  It was not repeated. Baseline-host execution, bundled distribution, full panic
  artifacts/replay and v0.0.1 remain unqualified. Git whitespace passes.
- Preserve the unchanged vendored fcontext.hpp EOF exception and checksum.

## Next steps

1. Implement forward leave-state capture and target joins in `borrow/control.rs`,
   `borrow/branches.rs`, `loans/control.rs` and `loans/branches.rs`. Capture surviving
   mutable-reference versions under the exit guard, merge them with normal target
   completion, and preserve source scope/temporary/result-slot lifetimes. Verify
   conditional leaves, nested targets, skipped assignment RHS stores, old copies,
   cell views and public call bounds before relaxing the per-body Leave gate.
   Keep Restart excluded until an explicit bounded fixed point handles initial and
   loop-carried versions with guard resets. Reference carriers/exclusive/owned work
   remain separate. Preserve charged work or improve reach-query reuse as needed.
2. Preserve first-collection and canonical slot conflicts while expanding capabilities.
   Shared-reference/temporary write roots, mutable reference-bearing fields and source
   exclusive references require explicit origin, move/initialization and cleanup
   models before enabling new writes.
3. Define generated payload/diagnostic layouts and scope cleanup using runtime
   mark/close while parents live. Retain owning outcomes, drain reports and preserve
   interleaved cleanup before cancellation and pinned unwinding.
4. Extend aggregate/emitted-name/cross-element constraints in `list_context/` only
   with explicit scope/dependency models and unchanged effect order. Add
   static/intrinsic sources and new projections only with updated lifetime
   contracts/no-return assumptions. Extend diagnostic source identities and evidence
   without treating bootstrap byte-span text as a complete replay artifact.
5. Implement required evaluation, specialization and the project/module graph;
   enable fixtures only after actual support, then progress libraries and tools.
6. Keep the combined gate and both handoffs/logs current. Strict conformance needs
   zero unsupported cases and still covers only part of v0.0.1 qualification.
