# Compiler handoff and work tracker

Updated: 2026-09-06. Immutable reference records and pinned runtime contexts pass.
Full v0.0.1 remains incomplete. No active workers, incomplete code or failing checks remain.
Implementation: `0b24669`; native coverage/example: `7816524`; runtime/tooling: `3dbed45`.
This file tracks the compiler; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).

## Current objective

Completed: guarded component origins and backwards liveness support immutable
records carrying shared references in their primary, named and nested components.
Projection reads only selected reference leaves; whole copies and full-record
comparisons consume every leaf. Retained component escapes report E303 and live
owner writes report E302. Record/scalar equality and formatting preserve primary
projection without silently discarding fields from record comparisons.
Next: reference unions and function contracts, exclusive access and generated cleanup.
The independent runtime now qualifies a bounded pinned context prototype; generated
programs still use the scalar runtime. Scheduling and DWARF remain separate work.
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
| Names, types, flow | `src/check.rs`, `src/flow.rs` | Scalar/record unions, narrowing, guarded composition and full-shape equality; 22 checker and 5 guard tests |
| Shared storage and loans | `src/borrow.rs`, `src/loans.rs`, `OWNERSHIP.md` | Guarded component origins, E303 escapes and per-leaf E302 liveness; 11 origin and 12 loan groups |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM to ELF pipeline including records, references and tagged unions; 14 focused backend tests |
| CLI and diagnostics | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Native builds, safe output replacement and diagnostic rendering |
| Tests and examples | `tests/`, `examples/`, `README.md` | 39 native groups, 4 harness tests and 9 runnable examples |

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
| Ownership | Reference unions/function contracts, exclusive borrows, reassignment, moves, partial initialization, captures, cleanup | Caller lifetime substitution, use-after-move/borrow rejection and exact-once cleanup |
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
  mutable reference carriers, carrier-address/reborrow operations, reference unions
  and reference-bearing signatures remain B001. See `OWNERSHIP.md` for staging.
- Every retained reference component must outlive its receiving block, even when
  a later consumer ignores it. A direct safe-field projection may leave a local
  carrier; returning that whole carrier must validate every contained reference.
  Discarded emissions retain operand effects without escaping on discarded paths.
- A copy reads all references in the copied value. Direct field/scalar-primary
  projection reads only selected leaves. Reference formatting still needs explicit
  dereference. Equality of two records preserves their full shape through blocks;
  scalar operands retain contextual primary widths and primary projection.
- Loan liveness preserves guards within an iteration and clears correlations on
  restart. Predicate assignments invalidate prior facts. Safe programs requiring
  stronger correlations can be conservatively rejected with E302/E303. Earlier
  mutated-predicate audit found 15 conservative E303 results in 288 combinations,
  with no unsafe acceptance; see the step log. Shared reads may overlap loans.
- Origin alternatives/components per result are capped at 4,096. Persistent facts
  and CFG origin storage each cap 262,144 entries; graph nodes, values, live entries
  and work are also bounded. Exhaustion reports B001, including repeated component
  copies of the same physical owner. Missing proofs never imply safe access.
- Record fields are immutable. Whole mutable records without references can be
  replaced, invalidating field proofs. Nullable primary/field joins and scalar/record
  union predicates work; optional reference fields still require reference unions.
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
- `../runtime/` has bounded cleanup, guarded mappings and worker-pinned contexts
  with explicit lifetime and ASan fiber hooks. There is no scheduler, automatic
  cancellation/join, Meowy personality, landing pads or compiler integration.
  Suspended contexts cannot release storage; caller synchronization is required.
- Timeout supervision kills/reaps the entire compiler/application process group.
  B001 cannot hide a later fault or count as an expected language rejection.

## Validation evidence

- Final `python3 -B tools/verify.py --all`: all 14 selected checks pass outside
  ptrace supervision. The gate pins the Cargo target/output and freshly built compiler.
- Rust: 80 library and 39 native groups pass, including both output profiles.
  All-target Clippy with `-D warnings` and `cargo fmt --check` pass.
- Python: 16 tooling, 11 runtime and 4 compiler-harness regressions pass. Local
  documentation validation checks 841 links; schemas/catalog and Vim/Neovim pass.
- Runtime per debug/release/sanitized profile: 14 cleanup cases plus 2 fatal probes;
  10 stack cases plus real kernel ENOMEM and 2 exact guard faults; 10 context cases
  plus fatal cleanup after resume. ASan/UBSan/LSan normal runs pass, and the expired
  fiber local produces the required ASan stack-use-after-return diagnostic.
  This proves host prototype behavior, not task-overflow recovery or structured joins.
- Conformance: 9 passed, 14 unsupported, 0 failed in debug/release. Passed cases:
  `compact_min`, `minimum_parenthesized`, `invalid_separator`, `forward_group`,
  `forward_interrupted`, `conditional_field`, `scalar_projection`, `function_equality`,
  `reference_identity`. This is not full conformance.
- Independent review verified record origins, projections, full copies, pending
  slots, equality effects and scalar-block contexts. The optimized compiler passes
  all 300 guarded component oracle cases: 204 accepted, 96 E302, zero conservative
  or unexpected results. The temporary oracle is supplemental; committed native
  and source regressions preserve the behavior coverage.
- The optimized compiler builds `examples/borrowed-records.mwy` in release and its
  executable prints exactly `11\n44\n33\n44\nvalue\n0\n1\n2\n3\n` with empty stderr.
- Existing coverage retains 10,000 deterministic malformed/Unicode parser inputs,
  depth/budget stress cases, bounded backend FFI, integer boundaries, short circuiting,
  tagged unions, nullable records and `/dev/full`. These are bounded regression tests.
- Prior ELF inspection found x86-64 PIE with only `libc.so.6` in DT_NEEDED and a
  GLIBC_2.34 requirement. This was not repeated this milestone; glibc 2.31 and
  minimum-kernel execution remain unqualified. Earlier validation is in the step log.
- Git checks preserve one upstream blank-at-EOF in the unchanged vendored
  `fcontext.hpp`; its checksum matches the pinned import. Project files pass normal
  whitespace checks. No other source/license formatting exception is required.

## Next steps

1. Extend `src/borrow.rs` component identities and `src/loans.rs` bundles through
   reference-carrying unions/coercions, preserving active-variant origins. Verify
   optional reference fields, guarded extraction, E303 escapes and E302 live writes.
2. Define function input/result borrow contracts in `OWNERSHIP.md`, `src/check.rs`
   and origin analysis. Apply the documented conservative all-input lifetime bound
   and substitute caller origins; verify accepted nested returns and rejected escapes
   before enabling reference-bearing signatures or parameter/receiver borrow roots.
3. Add explicit reads, reborrows, moves, initialization and cleanup edges before
   exclusive loans, reference reassignment or owned collections. Improve predicate
   and loop precision while preserving B001 bounds. Test final-use access, read/write
   conflicts, temporary-owner rejection and exact-once cleanup on every exit.
4. Add required evaluation, effects, logical budgets and constrained specialization,
   then enable the type-helper, compile-effect/budget and callable fixtures.
5. Build bounded scheduler admission and worker-owned queues over `../runtime/`
   contexts. Implement structured cancellation/join before storage release, then
   pinned unwind support, Meowy personality and landing pads. Verify a suspended
   child borrowing a parent local and cleanup that waits before releasing that local;
   preserve interleaved partial-result/local cleanup order.
6. Implement the project/module graph for native adapters and real Meowy library
   sources, then tools/artifacts and distribution qualification from the table above.
7. Keep the combined gate green and expand the conformance harness REQUIRED set only
   when supported. `--strict` with zero unsupported cases is the catalog's language
   gate; even that catalog covers only part of v0.0.1 qualification.
