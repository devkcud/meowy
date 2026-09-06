# Compiler handoff and work tracker

Updated: 2026-09-06. Contextual list candidates and unary union typing pass the final gate.
Full v0.0.1 remains incomplete; no unfinished implementation or active workers remain.
Inference: `1817ea4`; unary fix: `c6bf80a`; native coverage/example: `50283a5`.
Runtime diagnostic design: `541747f`; existing native batch behavior: `4feecf8`.
This file tracks the compiler; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).

## Current objective

Completed: list literals can select one compatible expected list alternative using
capacity, scalar range, typed elements and fresh list/record shapes. Pure contextual
literals may be deferred; effectful typed values are checked once in source order.
Proved ambiguity or no fit reports E207; all capacities being too small is E103.
Unary operators now preserve the operand type before expected-union injection.

Context-dependent effects/nested constraints that still need an unavailable proof
remain B001 and need annotations. Element places/mutation, aliases, slices,
reference/owned elements and general required evaluation remain unavailable.
The runtime owning-diagnostic lifecycle is documented, not implemented. Generated
programs still use the scalar runtime; generated scope exits, owning messages,
cancellation, automatic propagation and DWARF remain pending.
The unrelated untracked `examples/meow.mwy` is untouched and excluded from this work.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. Run `cargo test --manifest-path compiler/Cargo.toml` from the repository root.
4. Run `python3 compiler/tests/conformance.py`; 13 unsupported cases are currently expected.
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
| Names, types, flow | `src/check.rs`, `src/list.rs`, `src/list_context.rs`, `src/flow.rs` | Record/list contexts, checked extents and bounded candidate probes; 27 checker, 6 list/context and 5 guard groups |
| Shared storage and loans | `src/borrow_value.rs`, `src/borrow_contract.rs`, `src/borrow.rs`, `src/loans.rs`, `OWNERSHIP.md` | Scoped origins/bounds, direct call contracts and E302/E303 checks; 14 origin, 19 loan, 8 contract and 2 value-budget groups |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM to ELF pipeline including bounded lists, records, references and tagged unions; 17 focused backend tests |
| CLI and diagnostics | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Native builds, safe output replacement and diagnostic rendering |
| Tests and examples | `tests/`, `examples/`, `README.md` | 83 native groups, 4 harness tests and 15 covered examples |

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
- Candidate probes do not change flow proofs or replay a live checker. Pure
  contextual literals may wait while typed expressions are checked once. Raw source
  types are borrowed and conservatively retain potentially narrowed members; both
  actual/expected type walks, field searches and shape comparisons are charged.
  At most 256 list candidates are considered. Work exhaustion is B001.
- Unresolved context-dependent effects and nested candidate constraints remain B001;
  add an annotation rather than guessing. Unary operands keep their natural or
  unique literal type before the result enters an expected union. Numeric widening,
  unsigned negation and grouped signed-minimum rules remain checked.
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
- Interpolation streams at print/panic boundaries and does not allocate a string.
  Arithmetic/panic report P002/P006 and exit; recovery, source spans and operand
  evidence remain pending. Signed-minimum remainder by -1 is zero; division panics.
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
  close retains the first diagnostic. Diagnostic text must outlive all waits,
  retries and batch processing; no owned text, cancellation, automatic propagation,
  generated scope exits or native unwinding is supplied.
- Timeout supervision kills/reaps the entire compiler/application process group.
  B001 cannot hide a later fault or count as an expected language rejection.

## Validation evidence

- Final `python3 -B tools/verify.py --all`: all 14 checks pass on the hardened source,
  with explicit Cargo target/output and the actual compiler binary. Runtime sanitizer
  execution ran outside sandbox/ptrace supervision; metadata checks remain distinct
  from compiler execution and release qualification.
- Rust: 114 library and 83 native groups pass. New coverage includes 3 candidate
  groups, 1 unary checker group and 6 native groups. Clippy `-D warnings` and
  formatting pass. The optimized compiler builds/runs `examples/list-unions.mwy`
  in release with empty stderr and exact stdout `20\nmeowy\n300\n2\n`.
- Python: 16 tooling, 14 runtime and 4 compiler-harness tests pass. Documentation
  checks 851 local links; schemas/catalog and Vim/Neovim pass.
- Runtime per debug/release/sanitized profile: 14 cleanup groups plus 2 fatal probes;
  10 stack groups plus kernel ENOMEM and 2 guard faults; 10 context groups plus fatal
  resumed cleanup; 25 scheduler groups plus admission/fatal/child/scope probes;
  13 owned groups plus 3 exact fatal cleanup probes. ASan/UBSan/LSan and the
  expired-fiber-local negative diagnosis pass. Runtime behavior did not change.
- Conformance remains 10 passed, 13 unsupported, 0 failed in debug/release.
  `mixed_list` is required; unsupported core.Type, slices, callables and FFI remain
  visible. Reference fixtures and conformance requirements were unchanged.
- The semantic audit passed 1,080 scalar/capacity comparisons and 16 record/nested/
  narrowing comparisons after the unary fix. It found no remaining mismatch.
  Final declared-type/shape work charging was added afterward and passed focused
  stress plus the complete gate. Temporary audit snapshots supplement committed tests.
- Candidate tests cover unique and ambiguous capacities/types/ranges, later typed
  elements, nested literals, fresh record defaults, typed record-primary copies,
  original E201/E107/E216 errors, effects, early exits, bounded work and annotations
  for explicitly unavailable contexts. Unary tests preserve widths and failure codes.
- Existing suites retain bounded parser/Unicode, origin/loan, native layout,
  arithmetic, safe output and process-supervision regressions. The runtime ownership
  investigation was read-only; constructor-time copying, storage limits and P008
  snapshot independence still need implementation and native evidence.
- Prior ELF evidence found x86-64 PIE, only libc.so.6 in DT_NEEDED and GLIBC_2.34.
  It was not repeated; glibc 2.31, minimum-kernel execution, bundled distribution
  and full v0.0.1 remain unqualified. Prior evidence remains in the step log.
- Git whitespace passes. `examples/meow.mwy` is unrelated untracked work and was
  left untouched. Preserve the checksum-matching vendored fcontext.hpp EOF exception.

## Next steps

1. Extend `src/list_context.rs` for remaining compound/context-dependent elements
   and nested constraints. Keep once-only checking and actual/expected/source work
   charging; verify unique candidates, genuine ambiguity, flow changes and escapes.
2. Implement constructor-time owning panic text following `../runtime/README.md`.
   Choose explicit storage/overflow policy and measure fixed slot/report growth;
   test overwrite, copy independence, slot reuse, retries and both P008 causes.
3. Add initialized element places, moves and cleanup edges before indexed mutation,
   exclusive loans/reborrows, reference reassignment or owned collections. Preserve
   reset/resource bounds and test suspended parent loans and exact-once cleanup.
4. Extend borrow contracts for static/intrinsic sources and verified projections;
   preserve all-input lifetime bounds and update no-return assumptions first.
5. Add general required evaluation, effects, budgets and specialization; enable
   type-helper and compile-effect/budget fixtures only after real compiler support.
6. Generate payload layouts, move/drop and runtime mark/close calls while parents
   live. Drain every failure batch and preserve diagnostic lifetimes and retries;
   then add cancellation and pinned unwind support with interleaved cleanup tests.
7. Build the project/module graph for adapters and Meowy libraries, then tooling,
   artifacts and distribution qualification. Keep both trackers and step logs current.
8. Keep the combined gate green and require conformance cases only when supported.
   `--strict` needs zero unsupported cases and still covers only part of v0.0.1.
