# Compiler handoff and work tracker

Updated: 2026-09-06. Pure-compound expected-list inference passes the final gate.
Full v0.0.1 remains incomplete; no unfinished source work or active workers remain.
Implementation: `e8a6157`; native coverage/example: `15a5ff6`.
Prior runtime snapshots: `d92f94c`; generated panic evidence: `eb65cbd`.
This file tracks the compiler; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).

## Current objective

Completed: expected-list inference now admits pure scalar unary/binary compounds
with literal or resolved immutable scalar-constant leaves. It preserves ordinary
operator/literal rules, typed widths, intermediate overflow, floating behavior,
short circuits and each expression's original reach. Minimal isolated checkers do
not copy live state or replay effects; all extra work and constant bytes are charged.

Complex effectful blocks, captured/mutable values and non-scalar constraints are
not speculatively checked; existing once-only paths or explicit B001 remain.
Unannotated list common-type rules are unchanged. Element places/ownership,
generated cleanup/task integration, cancellation and DWARF remain pending.
Runtime message ownership and generated failure evidence are unchanged this turn.

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
| Lexer and parser | `src/lexer.rs`, `src/parser.rs` | Bootstrap grammar, malformed-input checks and bounded tree depth |
| Names, types, flow | `src/check.rs`, `src/list.rs`, `src/list_context.rs`, `src/flow.rs` | Record/list contexts, checked extents and bounded candidate probes; 27 checker, 10 list/context and 5 guard groups |
| Shared storage and loans | `src/borrow_value.rs`, `src/borrow_contract.rs`, `src/borrow.rs`, `src/loans.rs`, `OWNERSHIP.md` | Scoped origins/bounds, direct call contracts and E302/E303 checks; 14 origin, 19 loan, 8 contract and 2 value-budget groups |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM to ELF pipeline including bounded lists, records, references and tagged unions; 17 focused backend tests |
| CLI and diagnostics | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Native builds, safe output replacement and diagnostic rendering |
| Tests and examples | `tests/`, `examples/`, `README.md` | 94 native groups, 4 harness tests and 16 covered examples |

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
| Ownership | Static/intrinsic sources, exclusive borrows/reborrows, reassignment, moves, partial initialization, captures, cleanup | Caller lifetime substitution, use-after-move/borrow rejection and exact-once cleanup |
| Collections | Remaining contextual constraints, element places/mutation, aliases, slices, arrays, maps, vectors, allocators | Extent/count/bounds cases and allocation failures |
| Runtime | Owned allocations, recoverable panics, unwinding, tasks, channels, timers, cancellation | Generated cleanup, structured joins, one-worker progress and sanitizer coverage |
| Modules/projects | Relative imports, manifests, exports, aliases, root locks, dependency graph | Worked projects, offline locked builds and revision identity |
| Native ABI | Clang ABI adapters, explicit native artifacts, record classification | Separate C fixtures, argument/return layout and safety boundaries |
| Standard library | Meowy sources and APIs beyond foundational bootstrap output | API-to-test coverage, 16 worked projects and pinned data |
| Tooling | Test command, LSP, gatostyle/fmt, inspection and repair | Shared diagnostics, negotiated positions, safe rewrites and isolated tests |
| Artifacts/replay | Canonical readers, sessions, capsules, integrity, runtime events/replay | Schema/budget validation and replay after moving sources |
| Distribution | Bootstrap recipes, source inventory, vendoring, bundled tools/sysroot | Offline rebuild, dependency/license inventory and descriptor digests |
| Target qualification | Baseline host, static/shared closure, LTO, DWARF 5/unwind information | ELF inspection, minimum-host execution, reproducibility and sizes |

## Known limits to preserve

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
- Preflight suppresses reach-dependent arithmetic errors; source-order probes use
  the actual position's reach and may select a type before later effects. Deferred
  pure elements retain their original reach. A Never prefix can suppress later E107
  but cannot erase prior arithmetic or literal/type errors. Scratch reach is only
  definitely dead or potentially live; live guard IDs never cross into it.
- Raw declared types are borrowed. Actual/expected types, lookups, record shapes,
  constant strings and repeated scratch work are charged to the shared budget.
  Limits are 256 list candidates and 4,096 nodes per scalar scratch tree. Exhaustion
  remains B001 even for pure or dead expressions. No hidden reset grants extra budget.
- Complex effectful/captured/non-scalar constraints remain B001 when no context is
  proved. Use annotations rather than guessing. Intermediate checked widths, f32
  parsing/operations, short circuits, unsigned negation and grouped signed-minimum
  rules remain those of the ordinary checker; do not evaluate only the final result.
- Known immutable/literal lengths and equal lengths on every completing block path
  give static E101/E103. Mutable/function-result lengths keep runtime checks. Never
  infer a block length from its last emission alone.
- Whole-list references and concrete record list fields use existing places and
  lifetime contracts. Element borrows/writes, reference/owned elements, aliases,
  slices, removal and formatting remain unavailable. Do not add indexed borrowed
  descendants until their storage/loan representation exists.
- Ordinary locals, reference-free parameters and copied dispatch self bindings have
  local addressable storage. Parameter/self addresses may be used in nested scopes
  but cannot escape their storage region. Shared self instead contains a reference
  value, so returning/reborrowing it preserves the original Local/Input origins.
- Reference-bearing record/union dispatch uses ordinary component/variant facts;
  it does not invent a function-style lifetime contract. Inherited call bounds still
  survive. Receiver/argument expressions evaluate once and in order.
- A leading carrier field can produce the reference used by `&holder.view.field`.
  The reference prefix is copied once, then reborrowed. `&holder.view` and addresses
  of the carrier's own scalar fields remain B001. Temporary owners, named emitted
  storage, reference-bearing pointees, union-payload/primary-ascription addresses,
  exclusive borrows and mutable reference carriers remain unsupported.
- Dedicated reborrow sites retain bounded snapshots. Actual sources append referent
  field indices while inherited bounds stay unchanged. Lowering evaluates the parent
  once and derives addresses without record copies. Address hints perform no lowering;
  effectful equality operands and argument early leaves have native regressions.
- Function input leaves use symbolic Input sources, distinct from the parameter's
  stack copy. Separate referent paths identify storage reached through each input.
  Calls substitute compatible whole referents/concrete named descendants and attach
  every active input origin/transitive bound to each returned reference leaf.
  Ignored heterogeneous inputs still constrain lifetime and caller writes. Scalar
  results and scalar-only projections release these loans after consumption.
- Contract facts use unique call-site IDs and entered guards captured after argument
  evaluation. Call arguments remain live together until consumed; an early argument
  exit skips the call. Result proofs apply only on the returning edge. Every body,
  including uncalled and recursive definitions, must prove its own return sources.
- Under the current capabilities, a reference result needs an active compatible
  borrowed input referent or concrete named descendant; otherwise that path cannot return. Nullable results may
  still return null. Extend this rule before adding static safe-reference sources,
  additional addressable projections or reference-producing intrinsic contracts. Existing string values are
  literal-backed static views, not local storage dependencies merely by value.
- Every active retained reference component must outlive its receiving block,
  even when a later consumer ignores it. A safe-field projection may leave a local
  carrier; returning the whole carrier validates all active references. Discarded
  emissions retain operand effects without escaping on discarded paths.
- Copies read every active reference. Field/scalar-primary projections read only
  selected leaves; tag predicates inspect discriminants without copying payloads.
  Fresh block/record construction still retains result-slot loans to completion.
  Reference formatting needs explicit dereference. Record equality keeps full
  shape; compatible scalar comparisons project the primary. Union equality needs
  identical normalized union types, so a raw null comparison can report E222.
- Immutable binding/result snapshots link type-test tags to actual variant activity,
  conditioned on the enclosing variant. Mutable ordinary storage reads receive
  unknown activity, avoiding stale initializer tags after writes. Record fields
  remain immutable; replacing non-reference records invalidates field proofs.
- Restart edges erase iteration-specific correlations and scoped assumptions.
  Predicate assignments invalidate prior facts. Safe programs needing stronger
  temporal relationships may still receive conservative E302/E303; reads may
  overlap shared loans. No global assumption is reapplied after a reset.
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
  Named blocks conservatively forget mutable proofs; restart writes into surviving
  outer result slots are B001. Loan loop fixed points exist; broader ownership and
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

- Final `python3 -B tools/verify.py --all`: all 14 checks pass. Cargo target/output
  and compiler path are explicit. Runtime sanitizers execute outside ptrace/sandbox
  supervision; metadata validation remains separate from source execution.
- Rust: 118 library and 94 native groups pass. New coverage is 4 context groups and
  6 native groups, including intermediate widths, typed constants, f32 behavior,
  Boolean/string comparisons, short circuit, nested list/record fields, effects,
  Never ordering, mutable runtime inputs and source/capability diagnostics.
  Formatting and all-target Clippy with `-D warnings` pass.
- The optimized compiler builds/runs `examples/compound-lists.mwy` in release with
  empty stderr and exact stdout `128\n-128\nfalse\n260\n`.
- Independent differential checking passed 408 comparisons: 129 unique accepts,
  151 ambiguous and 128 no-fit cases, with 994 compiler checks and zero mismatches.
  The reviewer used a fresh pinned compiler snapshot; evidence/script remain in
  /tmp/meowy_pure_list_audit.json and /tmp/meowy_pure_list_audit.py. These supplement
  committed tests and are not a release replay artifact.
- The old grouped-negation B001 expectation was updated after both candidates became
  provably invalid (E207). Other unsupported constructs retain explicit diagnostics.
  Shared resource tests include repeated long constant strings and many candidates.
- Python: 16 tooling, 15 runtime and 4 compiler-harness groups pass. Documentation
  checks 852 local links; both editors, schemas/catalog and build checks pass.
- Runtime is unchanged. Each debug/release/sanitized profile passes 6 diagnostic,
  14 cleanup, 10 stack, 10 context, 25 scheduler and 14 owned groups, with exact
  fatal truncation/lifetime/guard/admission checks and identical layout measurements.
  ASan/UBSan/LSan and the expired-fiber-local negative diagnosis pass.
- Conformance remains 10 passed, 13 unsupported, 0 failed in debug/release. Fixtures
  and REQUIRED were unchanged. Prior ownership/diagnostic audits stay in the step log.
- Prior ELF evidence found x86-64 PIE, only libc.so.6 in DT_NEEDED and GLIBC_2.34.
  It was not repeated; baseline-host execution, bundled distribution, full panic
  artifacts/replay and v0.0.1 remain unqualified. Git whitespace passes.
- Preserve the unchanged vendored fcontext.hpp EOF exception and checksum.

## Next steps

1. Add verified initialized element places and exclusive access before indexed
   mutation, slices or owned collections. Preserve evaluation order, root identity,
   last-use checks and all-input bounds; test conflicts and cleanup boundaries.
2. Extend remaining effectful/non-scalar contextual constraints in list_context.rs
   without replaying effects or weakening budgets. Keep unproved cases B001 and
   compare candidate decisions with ordinary single-context checking.
3. Define generated payload/diagnostic layouts and scope cleanup using runtime
   mark/close while parent storage lives. Retain owning outcomes, drain all reports
   and preserve interleaved child/result/local cleanup before cancellation/unwinding.
4. Extend static/intrinsic borrow sources and verified projections only with updated
   lifetime contracts/no-return assumptions. Add source identities and richer
   diagnostic evidence/events without claiming full replay from bootstrap text.
5. Implement required evaluation/effects/budgets/specialization and the project/module
   graph, enabling fixtures only after actual compiler support; continue into Meowy
   libraries, tooling and distribution qualification.
6. Keep the combined gate and both trackers/logs current. Strict conformance needs
   zero unsupported cases and still covers only part of full v0.0.1 qualification.
