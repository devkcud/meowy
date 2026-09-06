# Compiler handoff and work tracker

Updated: 2026-09-06. Parameter/dispatch borrows and explicit task scope closing pass.
Full v0.0.1 remains incomplete. No active workers, incomplete code or failing checks remain.
Implementation: `393471c`; native coverage/example: `efe7d6e`; runtime/tooling: `d7d1758`.
This file tracks the compiler; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).

## Current objective

Completed: reference-free parameter and dispatch self storage can be borrowed
within its local scope. Their addresses never gain caller/static lifetime. Shared
and reference-carrier dispatch retain original sources, active variants and
inherited bounds. Chains such as `&holder.view.field` evaluate the reference-valued
prefix once and then reborrow the original referent; holder addresses remain B001.
E303 rejects copied-storage escapes and E302 protects active shared loans.
Next: exclusive access, new reference sources, moves and generated cleanup. The
runtime now explicitly closes marked child scopes while parent locals still live,
releasing owned results and preserving failure/progress reports across retries.
Compiler-generated scope exits, cancellation, full diagnostic attachment and DWARF
remain pending. Generated programs still use the scalar runtime.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. Run `cargo test --manifest-path compiler/Cargo.toml` from the repository root.
4. Run `python3 compiler/tests/conformance.py`; 14 unsupported cases are currently expected.
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
| Names, types, flow | `src/check.rs`, `src/flow.rs` | Reference unions, narrowing and contextual record composition; 26 checker and 5 guard tests |
| Shared storage and loans | `src/borrow_value.rs`, `src/borrow_contract.rs`, `src/borrow.rs`, `src/loans.rs`, `OWNERSHIP.md` | Scoped origins/bounds, direct call contracts and E302/E303 checks; 14 origin, 19 loan, 8 contract and 2 value-budget groups |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM to ELF pipeline including records, references and tagged unions; 14 focused backend tests |
| CLI and diagnostics | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Native builds, safe output replacement and diagnostic rendering |
| Tests and examples | `tests/`, `examples/`, `README.md` | 68 native groups, 4 harness tests and 13 runnable examples |

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
| Collections | Bounded lists, arrays, slices, maps, vectors, allocators | Extent/count/bounds cases and allocation failures |
| Runtime | Owned allocations, recoverable panics, unwinding, tasks, channels, timers, cancellation | Generated cleanup, structured joins, one-worker progress and sanitizer coverage |
| Modules/projects | Relative imports, manifests, exports, aliases, root locks, dependency graph | Worked projects, offline locked builds and revision identity |
| Native ABI | Clang ABI adapters, explicit native artifacts, record classification | Separate C fixtures, argument/return layout and safety boundaries |
| Standard library | Meowy sources and APIs beyond foundational bootstrap output | API-to-test coverage, 16 worked projects and pinned data |
| Tooling | Test command, LSP, gatostyle/fmt, inspection and repair | Shared diagnostics, negotiated positions, safe rewrites and isolated tests |
| Artifacts/replay | Canonical readers, sessions, capsules, integrity, runtime events/replay | Schema/budget validation and replay after moving sources |
| Distribution | Bootstrap recipes, source inventory, vendoring, bundled tools/sysroot | Offline rebuild, dependency/license inventory and descriptor digests |
| Target qualification | Baseline host, static/shared closure, LTO, DWARF 5/unwind information | ELF inspection, minimum-host execution, reproducibility and sizes |

## Known limits to preserve

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
  handle child failure counts even when close itself returns ok. Only the first
  diagnostic is retained, with caller-provided backing lifetime. No cancellation,
  automatic panic propagation, generated scope exits or native unwinding is supplied.
- Timeout supervision kills/reaps the entire compiler/application process group.
  B001 cannot hide a later fault or count as an expected language rejection.

## Validation evidence

- Final `python3 -B tools/verify.py --all`: all 14 selected checks pass outside
  ptrace supervision. The gate pins Cargo target/output and the freshly built compiler.
- Rust: 104 library and 68 native groups pass in the required profiles. All-target
  Clippy with `-D warnings` and `cargo fmt --check` pass.
- Python: 16 tooling, 14 runtime and 4 compiler-harness regressions pass. Local
  documentation validation checks 848 links; schemas/catalog and Vim/Neovim pass.
- Runtime per debug/release/sanitized profile: 14 cleanup cases plus 2 fatal probes;
  10 stack cases plus kernel ENOMEM and 2 guard faults; 10 context cases plus fatal
  resumed cleanup; 22 scheduler cases plus admission/fatal/child/scope probes;
  13 owned-value cases plus 3 exact fatal cleanup probes. ASan/UBSan/LSan normal
  runs pass; the expired fiber-local negative probe is diagnosed as required.
- Conformance: 9 passed, 14 unsupported, 0 failed in debug/release. Passed cases:
  `compact_min`, `minimum_parenthesized`, `invalid_separator`, `forward_group`,
  `forward_interrupted`, `conditional_field`, `scalar_projection`, `function_equality`,
  `reference_identity`. This is not full conformance.
- The final optimized compiler passes all 108 dispatch/carrier guard cases:
  66 accepted, 42 E302, no conservative or unexpected results. Nineteen directed
  source checks and ten additional native profile runs passed on the preceding
  debug snapshot. Temporary oracle/provenance files supplement committed tests.
- The optimized compiler builds `examples/scope-borrows.mwy` in release with empty
  stderr and exact stdout `false\n7\n8\ntrue\n9\n10\n`.
- Native/source tests verify parameter/const-self copy identity, earlier argument
  copies, original shared receivers, nullable carriers, inherited E302/E303 bounds,
  effectful reference prefixes and early leaves. Reference fixtures were not changed.
- Scope-close tests cover exact mark capacity, nesting, stale/foreign marks, older
  children, cleanup-local borrows, owned-result release, all consumed failure counts
  and retry progress. Private unclosed-scope errors and fatal P008 result-drop panics
  require exact evidence. Plain joins retain their owned-result protection.
- Existing coverage retains 10,000 deterministic malformed/Unicode parser inputs,
  depth/budget stress, bounded backend FFI, arithmetic boundaries, short circuiting,
  tagged unions, nullable records and `/dev/full`. These are bounded regressions.
- Prior ELF inspection found x86-64 PIE with only `libc.so.6` in DT_NEEDED and a
  GLIBC_2.34 requirement. It was not repeated this milestone; glibc 2.31 and
  minimum-kernel execution remain unqualified. Historical evidence is in the log.
- Current Git whitespace checks pass. Preserve the prior unchanged upstream
  blank-at-EOF in vendored `fcontext.hpp`; its checksum still matches the import.

## Next steps

1. Extend `src/borrow_contract.rs`, source identities and CFG proofs for static
   references, verified intrinsic sources and additional addressable projections
   before enabling them. Keep all-input bounds and update no-return assumptions
   where new sources become valid; test E302/E303 and address identity.
2. Add explicit reads, moves, initialized-slot tracking and cleanup edges before
   exclusive loans/reborrows, reference reassignment or owned collections. Preserve
   reset/resource bounds and improve predicate precision; verify final-use access,
   parent-loan suspension, temporary-owner lifetime and exact-once cleanup.
3. Add required evaluation, effects, logical budgets and constrained specialization,
   then enable the type-helper, compile-effect/budget and callable fixtures.
4. Generate owned payload layouts, move/drop operations and runtime mark/close calls
   before parent locals die. Handle close failure reports according to scope-exit
   rules, then add cancellation, diagnostic attachments and pinned unwind support.
   Verify panic/cancellation while joining and interleaved result/local cleanup;
   explicit native close currently waits without cancelling children.
5. Implement the project/module graph for native adapters and real Meowy library
   sources, then tools/artifacts and distribution qualification from the table above.
6. Keep the combined gate green and expand the conformance harness REQUIRED set only
   when supported. `--strict` with zero unsupported cases is the catalog's language
   gate; even that catalog covers only part of v0.0.1 qualification.
