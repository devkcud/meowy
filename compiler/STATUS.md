# Compiler handoff and work tracker

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. Whole union-alias assignment verified.
Full v0.0.1 remains incomplete. No failing checks or unfinished edits remain.
Private owned strings: `e547415`. Streamed runtime snapshots: `ef935da`.
Generated ownership: `4df0e44`; LLVM proof: `6d2d2b0`; contract: `161543e`.
Cleanup bridge: `2288ed5`; native archive/LLVM proof: `2da831f`; contract: `53f8e6b`.
Indexed scalar fields: `1f8295c`; contract/example: `397b3b2`.
Complete indexed-path representation: `16e0230`.
Nested indexed elements: `d4cd292`; contract/example: `345cf64`.
Projected/emitted elements: `bbd08b3`; contract/example: `8010888`.
Owned exclusive scalar elements: `28ca2b0`; contract/example: `cf4eca8`.
Exclusive emitted record fields: `12bee5a`; docs/example: `69353bd`.
Exclusive emitted scalars: `94192da`; contract/example: `ec45d2e`.
Exclusive scalar record fields: `55b3a1d`; contract/example: `59caddd`.
Never-operator fix: `1ff86ca`; anonymous block results: `65eda96`; docs/example: `f3fa667`.
Scalar reference returns/evidence/native matrix: `9dba94a`; contract/example: `e457380`.
Scalar function arguments/native matrix: `c7af472`; contract/example: `d72d5d0`.
Scalar exclusive implementation/native matrix: `3fe8715`; example/docs: `87e7926`.
Lifecycle/taking intent/availability: `d72b413`.
Shared provenance/value roles/regions: `ebc8ebe`.
Access implementation/tests/docs: `698e4b1`.
Expiry implementation: `7906333`; native coverage/example: `324e9ae`.
Restart-site metadata: `b0c9756`; guarded activity: `1df163b`; native/example: `74fac7c`.
Prior transitive headers: `34d2e7b`; native/example: `fa70eab`.
Prior direct restart implementation: `89800dc`, `ca037d3`, `dc7be5b`.
Prior emission fix/Leave support: `b3e875a`, `4ddef85`, `cb4afaf`, `a9d688a`.
Prior guarded merges: `5c55e44`, `af3a868`, `096ce78`.
Prior mutable-reference versions: `32d093c`, `4b7d655`, `f16c30b`.
Prior reference-bearing temporary support: `c325099`, `fb72c97`, `b10ba7f`.
Organization: native `c83f1f1`, parser `d599149`, borrow `f550947`, loans `717f5af`.
Earlier organization: backend `8c8e90a`, checker `360c8db`, list contexts `e3a0803`.
Prior runtime snapshots: `d92f94c`; generated panic evidence: `eb65cbd`.
This file tracks the compiler; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).

## Current milestone

[Whole union-alias assignment](OWNERSHIP.md#whole-union-alias-assignment) now supports
fixed borrowed lexical unions contained in larger result unions. Shared result_slot
returns both the component path and backing type. State::convert remaps result tags,
origins and bounds; Graph::retag maps loan keys using the same normalized members and
is reused by ordinary union conversion. Local alias state stays in its lexical domain.

Whole assignments evaluate RHS once, preserve earlier copies and synchronize result
definitions. Null/reference transitions, nested record variants, branches, Leave,
reset-scope Restart and discarded effects retain their rules. Addresses/field writes
through subset views, surviving published headers and allocator-only bounds stay gated.

Six new library and five native groups pass, including shifted tag indexes, nested
members, old copies, Leave, retained lifetimes, reset scopes and field/address gates.
All fourteen combined checks pass: 468 library and 486 native tests, 35 Python tests,
61 examples, editors, contracts and runtime sanitizers. The union-aliases example
prints 7, clear, 7.

No runtime/backend representation, syntax, reference fixture or dependency changes.
Lists, dynamic origins, owning cleanup, source error APIs, tasks, DWARF and full release remain open.

## Prior implemented milestone

Completed: restart headers retain terminal Source::Expired identities for ended or
iteration-owned Local/Slot/Temporary sources and public lifetime bounds. Each marker
keeps the original local, slot-view or temporary site ID; physical projections may
collapse only after expiry. Reinitializing that static site cannot revive an old
reference. Overwrite-before-read is accepted; actual expired use reports E303.

Live ancestor owners, including an enclosing statement's temporary, survive inner
restarts. Component paths, source/bound roles and parent-conditioned activity remain
intact. Exact initial/backedge predecessor snapshots preserve incoming sources before
boundary expiry and retain entered guards; demand-only transfers preserve physical
loans. Expired markers are nonphysical in overlap checks. Raw malformed-source and
budget errors remain B001. Generated storage, runtime ABI and dependencies did not
change. Old copies, lazy pointee access and full call-entry validation retain their
existing rules. Independent field/owner and temporal correlations may still widen.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. After new changes or an unresolved concern, run
   `python3 -B tools/verify.py --compiler` from the root; it pins the build target
   and actual compiler path. Otherwise preserve the green evidence below.
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
| Names, types, flow | `src/check.rs`, `src/check/`, `src/list.rs`, `src/list_context/`, `src/flow.rs` | Record/list contexts, checked extents and bounded candidate probes; 40 checker, 18 list/context and 5 guard groups |
| Storage, origins and permissions | `src/borrow_value.rs`, `src/borrow_value/`, `src/borrow_contract.rs`, `src/borrow_contract/`, `src/borrow.rs`, `src/borrow/`, `src/loans.rs`, `src/loans/`, `OWNERSHIP.md` | Scoped origins/bounds, availability, direct call contracts and scalar exclusive local/input permissions; 60 origin, 130 loan, 15 contract and 2 value-budget groups |
| Native backend | `src/backend.rs`, `src/backend/`, `build.rs`, `native/` | Verified LLVM to ELF pipeline including bounded lists, records, references and tagged unions; 62 focused backend tests |
| CLI and diagnostics | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Native builds, safe output replacement and diagnostic rendering |
| Tests and examples | `tests/native.rs`, `tests/native/`, `tests/conformance.py`, `examples/`, `README.md` | 486 native groups, 4 harness tests and 61 covered examples |

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
`loans/transitive.rs` connects summary transfers; `loans/access.rs` owns access
records, typed inspection paths, metadata charging and region resolution.
`loans/elements.rs` owns exclusive element reservations/acquisition;
`Proofs::exclusive_path_type` validates mutable owned Place/WriteStep paths,
intermediate types and alias backing.
`loans/authority.rs` owns acquisition IDs, guarded provenance, opacity and parent checks.
`loans/storage.rs` owns lifecycle events/scopes; `loans/init.rs` owns storage demand
and forward guarded availability. `loans/permissions.rs` owns mode-aware access,
ancestor checks and semantic boundary gates. `loans/returns.rs` consumes guarded
call-result parent evidence from `borrow_contract/returns.rs`. Contract tests live beside their
module; `check/temporaries.rs` owns temporary creation and statement metadata.
`borrow/mutable.rs` owns the bounded assignment/restart/capability scan.
`borrow/branches.rs` and `loans/branches.rs` own guarded environment restoration and
returning-state joins. `borrow/exits.rs` owns target-entry and queued Leave snapshots;
loan Scope captures target versions and exit predecessors. `loans/values.rs` owns
immutable graph versions. `borrow/replay.rs` isolates body passes and fact publication;
`borrow/restart.rs` owns canonical headers and source expiry; `borrow/header.rs` supplies
the shared charged Shape validator to origin and loan analysis. `borrow/activity.rs`
owns member partitions and path activation. `loans/restarts.rs` owns stable header
IDs and source-guarded predecessor transfers. `check/statements.rs` assigns bounded
RestartId values after resolved control lookup. Retain these module boundaries.

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
| Ownership | Static/intrinsic sources, wider exclusive shapes/contracts, payload moves, partial initialization, captures, cleanup | Caller lifetime substitution, use-after-move/borrow rejection and exact-once cleanup |
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
- Facts.merging identifies bodies using reference assignments or exclusive values; the bounded HIR scan
  also identifies their restart targets. Returning expressions retain normal proofs
  after nested scope closure. Assignment-free bodies keep the previous loop path;
  fixed nullable/aggregate bindings use the same version model; written published outer result slots enclosing an inner Restart stay B001.
- Each restarted target in that mode includes all mutable-reference IDs present
  at entry, even when an iteration leaves one unchanged. Initial and feasible
  backedge values accumulate separate actual-origin and lifetime-bound `(Path,
  Source)` sets plus observed `(union Path, member)` keys. Component paths preserve
  repeated owners in distinct fields/layers. Stable choice guards are keyed by
  target, local, union path and member and retained across body replay passes.
- Shared Shape validation follows record Slot paths and reference leaves, entering
  Deref only when the referent contains references. Stored unions have disjoint,
  complete activity conditioned on their parent. Every active typed reference path
  requires actual-origin coverage; bounds cannot supply missing pointers. Unknown
  paths/members, missing active components or noncanonical guards/proof/presence
  fail B001. Inactive null/empty payload paths may contain no reference origins.
- Canonical headers partition only observed members. Child activation includes
  its parent, and origins/bounds use their structural path guard. Reference-free
  referents remain traversal cut points, including scalar unions behind nested
  references. Reference-bearing lists and written published result slots enclosing an inner Restart remain B001; unsupported activity is never erased to admit a type.
- Every feasible carried source/bound keeps its structural component and role.
  Inputs and live ancestor-owned Local/Slot/Temporary sources survive. Ended or
  target/descendant-owned sources become terminal Source::Expired site identities;
  further projections and repeated restarts cannot revive them. Physical projections
  may collapse only after expiry. Overwrite-before-read adds no lifetime use; actual
  expired reads report E303 before current storage lookup. Live ancestor statement
  temporaries survive inner restarts. Genuine unsupported/budget errors stay B001.
- Expired sources are nonphysical in loan overlap. Initial/backedge predecessor
  snapshots retain their original sources and activity, so current-iteration writes,
  old copies, physical cells and public bounds still enforce E302. Transitive payload
  reads stay lazy; whole copies and full call entry validate demanded components.
- Origin replay creates a fresh Checker for each body pass, reinitializing inputs
  and discarding tentative locals/calls/reborrows/results/exits/predecessors.
  Headers grow monotonically and compare canonical source roles and member activity
  including stable guards. Only stable final Facts publish; effects are never
  lowered or executed during replay. Nested headers solve together.
- Every resolved restart carries a unique RestartId, with a charged 65,536-site
  checker-wide limit including functions and resolved aliases. Backend lowering
  ignores that metadata. Facts.header_inputs stores initial snapshots by target;
  Facts.restart_inputs stores restart snapshots by site. Each snapshot captures
  incoming State under its entered guard before widening/reset. Shape inspection
  uses this exact predecessor to prove path activity.
- Loan headers allocate stable value IDs once. Initial/backedge transfers define
  those IDs before reset edges; header start itself does not redefine them. Backward
  propagation resets future-iteration demand before applying predecessor source
  activation. Destination/future activity must not filter that current transfer.
  Ordinary transfers remain unconditional; all transfers remain demand-only.
  Missing paths require proved inactivity. Missing snapshots or active path evidence
  report B001 when their guard overlaps final graph reach.
- Before-entry copies keep initial precision; current-iteration branches, calls,
  leaves and result guards still apply. Header entry/reset erases incoming and
  iteration correlations; independent union-field/owner correlations may widen.
  Some safe programs needing stronger relationships remain conservatively rejected.
- Replay retains the 64-pass, shared-work, 4,096-part and 262,144 weighted fact/cache
  limits. Both retained/cloned header and choice seed maps are checked before cloning;
  choice keys and predecessor snapshots are counted, and prior-body counters carry
  forward. These are logical budgets, not allocator byte peaks. Transient states
  retain existing per-value/work limits; exhaustion reports B001.
- Each active target in merging mode records the mutable-reference IDs already
  present at entry. Leave captures only those surviving values under the actual
  exit guard before branch/scope restoration. The exact target merges its queued
  exits plus normal fallthrough and installs their union continuation before
  result-slot proof/completeness, even if no mutable-reference IDs survive.
- Outer-target exits remain queued across inner closes. They create no inner
  continuation, and an unfinished assignment/call/index is not resumed. Target-local
  bindings still end and temporary/slot ownership is never revived. An expired
  captured version may be overwritten without reading; a later actual use is E303.
- Loan targets retain entry versions and queued predecessor arms. Leave defers its
  edge until the target joins those arms through the existing demand-only merge,
  then reaches the original result-copy node. Real emitted result components retain
  their own uses/loans; the exit snapshot itself adds no read. Target/exit snapshots
  and lookup/merge work count toward existing persistent origin and work limits.
- Emission assumptions are conjoined before masking the value by its retained
  emission guard. Disjoint paths therefore contribute conditional proofs. Their
  conjunction cannot make the entire result falsely unreachable and suppress
  downstream lifetime/loan checks. The no-Leave E302 regression covers the prior bug.
- Loan branch ends define merged versions with demand-only transfers for every
  component, including direct references. Merging creates no runtime read. Unchanged
  bundles are reused; changed origins are masked by actual completing CFG reach,
  normalized to their current projected paths and deduplicated. Copies made before
  the branch retain old IDs, while physical cell loans keep their original roots.
- Snapshots, restoration, state/graph growth and completion-reach queries consume
  existing budgets. The initial implementation recomputes reach for changed joins:
  64 repeated self-assignment joins and 64 forward exits pass; 1,024 repeated
  joins report loan-budget B001.
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
  independent; bounded allocator-only emitted storage remains outside the supported write
  model. Fixed borrowed carriers use stored versions.
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
  exclusive dispatch/carrier borrows and broader emitted-alias writes remain unsupported.
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
  conditioned on the enclosing variant. Fixed borrowed storage retains current
  activity and links predicate-site tags at inspection; other mutable reference-free
  storage reads receive unknown activity. Record fields
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
  or terminator. Calls propagate owning panic outcomes before reading result storage;
  Never has an unreachable success continuation. Root failures exit status 1;
  automatic resource cleanup, recovery, rich source identities and replay remain pending.
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

- `python3 -B tools/verify.py --all`: all fourteen selected checks passed. Rust:
  468 library + 486 native (954 total). Python: 16 tooling + 15 runtime + 4 compiler
  (35). All 61 examples execute in debug/release. Both editors, formatting, Clippy,
  build, schemas/identities and catalog pass. No selected check was skipped.
- Runtime debug/release/ASan/UBSan/LSan passes 100 groups/profile plus required
  fatal/admission/guard/fiber probes with approved process access. Runtime/backend
  representations, syntax, reference fixtures and dependencies are unchanged.
- Six new library and five native groups cover differing lexical/backing tag indexes,
  null/reference transitions, nested record members, old-copy/source loans, branches,
  Leave, retained lifetimes, declared/inferred backing and reset iterations. Whole
  alias stores remap both activity/origins and loan keys; address/field views stay gated.
- Two historical subset-write B001 expectations now accept supported writes. One
  initial Restart fixture hit earlier E205 after losing emission exclusivity; its
  declared backing replacement tests the intended ownership gate. All final checks
  pass, including existing ordinary union conversion after retag-helper reuse.
- Final handoff passes 1003 local links in 99 Markdown files and Git whitespace checks.
  The union-aliases example prints 7, clear, 7. Conformance remains 10 passed,
  13 unsupported, 0 failed. Surviving published headers, union-view addresses/fields,
  allocator-only alias bounds, lists/exclusive carriers, dynamic origins, owning
  drops, source error APIs, tasks, DWARF and complete release remain unqualified.

## Next steps

1. Add surviving published result snapshots to restart header merging before lifting their ownership gate.
   Union-view addresses/field paths and allocator-only bounds need separate proofs.
   Preserve lexical/backing tag domains, shared retagging, old copies, transient
   lifetime and RHS effects. Lists need bounded summaries; dynamic origins and
   [OWNING_HIR.md](OWNING_HIR.md) cleanup schedules follow those proofs.
2. Extend aggregate/emitted-name/cross-element constraints in `list_context/` with
   explicit scope/dependency models and unchanged effect order. Add static/intrinsic
   sources only with lifetime contracts and no-return assumptions.
3. Build the manifest/module graph and initial Meowy library layer; implement
   required evaluation and specialization before enabling their reference fixtures.
   Continue root runtime/editor/library tracking alongside compiler work.
4. Extend diagnostic source identities and evidence without treating bootstrap
   byte-span text as a complete replay artifact. Run the combined repository gate
   after wider integrations; preserve current compiler evidence until code changes.
   Strict conformance still requires zero unsupported cases; this host does not
   qualify the minimum host, bundled sysroot or complete v0.0.1 release.
