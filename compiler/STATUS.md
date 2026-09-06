# Compiler handoff and work tracker

Updated: 2026-09-06. Reference unions and bounded runtime scheduling pass.
Full v0.0.1 remains incomplete. No active workers, incomplete code or failing checks remain.
Implementation: `75785b7`; native coverage/example: `4ef3268`; runtime/tooling: `d3e41aa`.
This file tracks the compiler; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).

## Current objective

Completed: immutable records and unions carry shared references with guarded
component origins and backwards liveness. Active variants control retained E303
escapes and overlapping E302 writes; absent payloads carry no loan. Injection,
extraction and retagging preserve origins by member type. Tag predicates avoid
payload reads while retaining construction effects. Record constructors select
one compatible union member with contextual widths and nullable defaults.
Union equality still requires the same normalized union type.
Next: function borrow contracts, exclusive access and generated cleanup. The
independent runtime now has bounded scheduling and settled-only explicit join;
generated programs still use the scalar runtime. Structured child lifetimes,
waiting joins, cancellation and DWARF remain separate work.
The language reference is authoritative. Do not change fixtures to make tests pass.

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
| Names, types, flow | `src/check.rs`, `src/flow.rs` | Reference unions, narrowing and contextual record composition; 24 checker and 5 guard tests |
| Shared storage and loans | `src/borrow_value.rs`, `src/borrow.rs`, `src/loans.rs`, `OWNERSHIP.md` | Active variants, scoped proofs, E303 escapes and E302 liveness; 14 origin, 15 loan and 2 value-budget groups |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM to ELF pipeline including records, references and tagged unions; 14 focused backend tests |
| CLI and diagnostics | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Native builds, safe output replacement and diagnostic rendering |
| Tests and examples | `tests/`, `examples/`, `README.md` | 46 native groups, 4 harness tests and 10 runnable examples |

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
| Ownership | Function borrow contracts, exclusive borrows, reassignment, moves, partial initialization, captures, cleanup | Caller lifetime substitution, use-after-move/borrow rejection and exact-once cleanup |
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

- Shared borrow roots are ordinary immutable/mutable locals and concrete record
  fields. Parameter/receiver/emission places, temporary owners, exclusive loans,
  mutable reference carriers, carrier-address/reborrow operations and reference
  signatures remain B001. See `OWNERSHIP.md` for the remaining analysis stages.
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
- `../runtime/` has bounded cleanup, guarded contexts and a single-worker scheduler.
  Fixed slots stay occupied until explicit settled-only join succeeds, including
  failed admission with retained memory. Cleanup can yield before settlement.
  Callback data and panic text are borrowed; pump limits transitions, not CPU time.
  There is no structured child tree, waiting join, capture ownership transfer,
  automatic cancellation, Meowy personality, landing pads or compiler integration.
- Timeout supervision kills/reaps the entire compiler/application process group.
  B001 cannot hide a later fault or count as an expected language rejection.

## Validation evidence

- Final `python3 -B tools/verify.py --all`: all 14 selected checks pass outside
  ptrace supervision. The gate pins the Cargo target/output and freshly built compiler.
- Rust: 90 library and 46 native groups pass, including both output profiles.
  All-target Clippy with `-D warnings` and `cargo fmt --check` pass.
- Python: 16 tooling, 12 runtime and 4 compiler-harness regressions pass. Local
  documentation validation checks 844 links; schemas/catalog and Vim/Neovim pass.
- Runtime per debug/release/sanitized profile: 14 cleanup cases plus 2 fatal probes;
  10 stack cases plus kernel ENOMEM and 2 guard faults; 10 context cases plus fatal
  resumed cleanup; 11 scheduler cases plus kernel admission refusal and fatal task
  cleanup. ASan/UBSan/LSan normal runs pass. The expired fiber local produces the
  required ASan stack-use-after-return diagnostic. Structured child joins and
  task-overflow recovery remain unqualified.
- Conformance: 9 passed, 14 unsupported, 0 failed in debug/release. Passed cases:
  `compact_min`, `minimum_parenthesized`, `invalid_separator`, `forward_group`,
  `forward_interrupted`, `conditional_field`, `scalar_projection`, `function_equality`,
  `reference_identity`. This is not full conformance.
- The final optimized compiler passes all 588 independent union oracle cases:
  356 accepted, 232 E302, zero conservative or unexpected results. Ten directed
  lifetime/restart probes and eight additional native debug/release runs passed
  on the preceding snapshot. Committed source/native tests retain the behavior
  coverage; temporary oracle/provenance files are supplemental evidence.
- The optimized compiler builds `examples/optional-borrows.mwy` in release and its
  executable prints exactly `7
8
missing
8
42
text
` with empty stderr.
- Native constructor checks verify int64 fields/primaries, uint8 limits, nullable
  reference defaults and discarded named fields. Source regressions retain E222
  for mismatched union equality types. Reference fixtures were not changed.
- Resource review verified bounded wide-tag expansion, deep path work, growing
  merges and saturated counters. Source tests reject oversized tag domains and
  repeated inactive-payload copies with B001; lexical assumptions are cached.
- Existing coverage retains 10,000 deterministic malformed/Unicode parser inputs,
  depth/budget stress, bounded backend FFI, integer boundaries, short circuiting,
  tagged unions, nullable records and `/dev/full`. These are bounded regressions.
- Prior ELF inspection found x86-64 PIE with only `libc.so.6` in DT_NEEDED and a
  GLIBC_2.34 requirement. It was not repeated this milestone; glibc 2.31 and
  minimum-kernel execution remain unqualified. Historical evidence is in the log.
- Current Git whitespace checks pass. Preserve the prior unchanged upstream
  blank-at-EOF in vendored `fcontext.hpp`; its checksum still matches the import.

## Next steps

1. Define function input/result borrow contracts in `OWNERSHIP.md`, `src/check.rs`
   and origin analysis. Apply the documented conservative all-input lifetime bound
   and substitute caller origins, preserving components/variants. Verify accepted
   nested returns, E303 escapes and E302 caller writes before enabling reference
   signatures or parameter/receiver borrow roots.
2. Extend `src/loans.rs` with explicit reads, reborrows, moves, initialization and
   cleanup edges before exclusive loans, reference reassignment or owned collections.
   Improve predicate/loop precision while preserving B001 bounds. Test final-use
   access, read/write conflicts, temporary-owner rejection and exact-once cleanup.
3. Add required evaluation, effects, logical budgets and constrained specialization,
   then enable the type-helper, compile-effect/budget and callable fixtures.
4. Extend `../runtime/` scheduler with bounded child admission, waiting joins and
   capture/result ownership. Add cooperative cancellation before parent storage
   release, then pinned unwind support, Meowy personality and landing pads. Verify
   a suspended child borrowing a parent local and cleanup that waits before releasing
   that local; preserve interleaved partial-result/local cleanup order.
5. Implement the project/module graph for native adapters and real Meowy library
   sources, then tools/artifacts and distribution qualification from the table above.
6. Keep the combined gate green and expand the conformance harness REQUIRED set only
   when supported. `--strict` with zero unsupported cases is the catalog's language
   gate; even that catalog covers only part of v0.0.1 qualification.
