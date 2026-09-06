# Compiler handoff and work tracker

Updated: 2026-09-06. Continuing with union/nullable values and conditional fields.
Full v0.0.1 remains incomplete. Reference and implementation review is in progress.
This file is the restart point; all implementation work is under `compiler/`.
Baseline commits: `54080b1` (compiler) and `c82354f` (integration tests/examples).

## Current objective

Extend the working bootstrap with union/nullable values and flow analysis for
conditional emissions. Make `conditional_field` pass, execute present/absent-field
cases in debug and release, and preserve E204/E205 initialization diagnostics.
The existing reference remains authoritative. Do not change fixtures to make tests pass.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. Run `cargo test --manifest-path compiler/Cargo.toml` from the repository root.
4. Run `python3 compiler/tests/conformance.py`; 16 unsupported cases are currently expected.
5. Consult the validation log below before claiming any gate passed.
6. After every compiler work step, update `Next steps` and add a checkpoint to
   `Step log` before continuing. Include investigations, edits, checks, and decisions.

No dependency downloads or remote services are needed by the current Rust design.
The bootstrap uses installed LLVM/Clang/LLD 22.1.8 and Rust 1.98.1. It does not yet
provide a bundled sysroot or a qualified distribution. Never claim the documented
Linux 5.4/glibc 2.31 execution baseline from this workstation build.

## Step log

### 2026-09-06 — Commit integration tests and prepare the feature split

- Completed: integration tests/examples committed as `c82354f`; compiler baseline
  is `54080b1`. Documentation and tracking rules are the third focused commit.
- Validation: both staged diffs passed whitespace checks; working code is unchanged.
- Blockers: none. Read-only design reviews are complete.
- Next steps: add normalized union HIR/coercions/tests, implement LLVM storage and
  guarded checker flow independently, then add native nullable-field regressions.

### 2026-09-06 — Commit the compiler implementation

- Completed: committed the verified Rust frontend/checker/driver, LLVM bridge,
  runtime, toolchain pins and inline unit tests as the first focused commit.
- Validation: staged diff passed whitespace checks; baseline validation is recorded below.
- Blockers: Git writes require the approved escalation because `.git` is read-only
  inside the sandbox. Source edits remain within the workspace.
- Next steps: commit integration tests/examples, then documentation and this tracker;
  start union changes only after these baseline commits are complete.

### 2026-09-06 — Verify the baseline and select guarded union lowering

- Completed: baseline passed 33 library tests, 12 native tests, 4 harness tests,
  Clippy, formatting, and 7 conformance cases (16 explicitly unsupported).
- Decision: use normalized tagged unions with inline maximum-size payload storage,
  explicit coercion/type-test HIR, and shared boolean guards for emission reachability.
  Keep physical local types separate from types proven inside matcher branches.
- Blockers: restart back-edges need conservative handling until full loop dataflow;
  do not accept possible repeated emissions into surviving outer blocks.
- Next steps: commit the verified baseline in three focused pieces, then implement
  HIR/backend union support and guarded checking with native regression cases.

### 2026-09-06 — Establish the next implementation and commit boundaries

- Completed: confirmed conditional slots must join emitted types and add null on
  missing paths; unions require a discriminant and payload storage. Existing HIR
  has neither unions nor dynamic type tests. Started separate checker/backend reviews.
- Validation: inspected the full untracked compiler tree and existing commit style.
- Blockers: none. Shared HIR details are being reviewed before concurrent edits.
- Next steps: refresh baseline checks and commit the existing compiler, integration
  tests/examples, and documentation separately; then implement the reviewed union design.

### 2026-09-06 — Resume nullable-value work

- Completed: reread `AGENTS.md`, the tracker, and current Git status. Existing
  compiler work remains uncommitted and will be preserved.
- Validation: prior results are recorded below; no tests rerun in this step.
- Blockers: none. The union representation and flow-analysis changes need review.
- Next steps: trace the reference fixture through checking and LLVM lowering,
  agree on shared representation, and refresh the baseline tests before editing.

### 2026-09-06 — Verify tracking changes

- Completed: checked both changed documentation files for whitespace errors and
  verified that this file contains `Step log` and `Next steps` sections.
- Validation: all documentation checks passed; no compiler tests rerun for this
  documentation-only change. This resolves the pending check in the previous entry.
- Blockers: none for the next investigation.
- Next steps: follow item 1 below to review nullable emission rules and trace
  `conditional_field`; then record findings and update the implementation steps.

### 2026-09-06 — Require a handoff after every step

- Completed: inspected the existing tracker and rules; updated `AGENTS.md` to
  require a checkpoint and current next steps after every compiler work step.
- Validation: documentation inspected; final diff check pending. Compiler behavior
  is unchanged, so previous runtime results remain the latest execution evidence.
- Blockers: none for the next investigation; full-language gaps remain listed below.
- Next steps: review nullable-field rules and trace `conditional_field` through
  `src/check.rs`, `src/hir.rs`, and `src/backend.rs`; record the proposed representation
  and verification cases before implementing it.

## Implementation map

| Component | Files | Current state |
| --- | --- | --- |
| Workspace and interfaces | `Cargo.toml`, `rust-toolchain.toml`, `src/ast.rs`, `src/hir.rs`, `src/lib.rs` | Builds offline with no external Rust dependencies |
| Lexer and parser | `src/lexer.rs`, `src/parser.rs` | Implemented for the documented bootstrap subset; malformed-input and depth checks pass |
| Names, types, flow | `src/check.rs` | Scalar, record, function and emission analysis implemented; 12 focused tests pass |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM → ELF pipeline; 6 focused backend tests pass |
| CLI and diagnostic rendering | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Implemented and exercised through native tests |
| Native tests, examples, documentation | `tests/`, `examples/`, `README.md` | 12 native tests, 4 harness tests and 4 runnable examples |

Agents share this checkout. After a sudden stop, file existence is not proof that
a component compiles. The interfaces are `parser::parse`, `check::check`,
`backend::emit_ir`, and `backend::emit_object`. `src/lib.rs` composes the frontend.
Rust modules use explicit visibility where Rust allows it. Follow the user's
short-name, immutable-value, and no-comment conventions.

## First milestone checklist

- [x] Inspect reference, conformance catalog, and available toolchain.
- [x] Create an isolated implementation directory and pin the Rust version.
- [x] Establish syntax, typed representation, and diagnostic interfaces.
- [x] Add `AGENTS.md` with project rules and persistent handoff requirements.
- [x] Lossless tokens, original byte offsets, malformed-input recovery.
- [x] Contextual punctuation parser, interpolation, compact/spaced equivalence.
- [x] Lexical names, distinct type namespace, aliases of resolved intrinsics.
- [x] Scalar typing, expected literals, numeric domains and overflow diagnostics.
- [x] Static block fields, emission initialization, emission without return.
- [x] Functions, recursion, forward groups, dispatch, named scope control.
- [x] LLVM verification, object emission, native linking, streaming output.
- [x] Checked arithmetic and short circuiting in debug and release.
- [x] CLI check/build/run, byte-span diagnostics, protected/atomic output paths.
- [x] Native integration tests, selected conformance, explicit unsupported cases.
- [x] README with exact commands and honest supported-feature boundaries.

## Still outside this compiler

Items here must not be silently accepted with different semantics. `B001` is an
internal bootstrap capability diagnostic, not an assigned reference error code.
The supported portion of the language uses the reference's established `E...`
codes. The bootstrap's JSON diagnostic stream is not a release artifact schema.

| Area | Work remaining | Required evidence |
| --- | --- | --- |
| Full frontend | Complete grammar, stable item IDs, recovery CST/editor integration, all type forms | Conformance, compact syntax properties, malformed UTF-8 and parser fuzzing |
| Type system | Unions/narrowing, subtraction, callable environments, generics/capabilities, type queries, nominal identity | All type and callable fixtures plus negative boundary cases |
| Required evaluation | Type-producing helpers, effect analysis, cycle checks, logical budgets, specialization | E211/E219/E220 cases and determinism/budget tests |
| Ownership | Moves, borrows, references, partial initialization, capture analysis, generated cleanup | Use-after-move and borrow rejection; exact-once cleanup on every exit |
| Collections | Bounded lists, arrays, slices, maps, vectors, allocators | Extent/count/bounds cases and allocation-failure behavior |
| Runtime | Owned allocations, recoverable panics, unwinding, tasks, channels, timers, cancellation | Task/unwind prototype, one-worker progress, cleanup and sanitizer tests |
| Modules/projects | Relative imports, manifest policy, exports, aliases, root locks, dependency graph | Worked projects, offline locked builds, duplicate/revision identity tests |
| Native ABI | Clang ABI adapters, explicit native artifacts, record classification | Separate C fixtures, argument/return layout, safety-boundary checks |
| Standard library | Meowy library sources and documented APIs beyond foundational bootstrap output | API-to-test coverage, all 16 worked projects, pinned Unicode/time/calendar data |
| Tooling | Test command, LSP, gatostyle/fmt, inspection and repair workflows | Shared diagnostics, negotiated positions, safe rewrites and isolated test cases |
| Artifacts/replay | Canonical schemas/readers, sessions, capsules, integrity, runtime events/replay | Schema validation, corruption/budget tests, replay after moving original sources |
| Distribution | Maintained bootstrap recipes/xtask, pinned source inventory, vendoring, bundled tools/sysroot | Offline rebuild, dependency/license inventory, descriptor digests |
| Target qualification | Baseline host, static/shared closure, LTO modes, DWARF 5 and unwind information | ELF inspection, minimum-host execution, reproducible builds and size measurements |

## Known bootstrap limits worth preserving during handoff

- Only direct, noncapturing functions are lowered. A function declaration can be
  aliased, but first-class function-pointer storage and anonymous/indirect calls remain unavailable.
- Record fields are immutable. Nullable primary/field joins report B001 rather
  than receiving an invented representation. Full predicate-based flow refinement is still needed.
- Interpolation streams only at print/panic boundaries. It does not construct a
  heap string or a general-purpose standalone string value.
- Runtime arithmetic and explicit panic report P002/P006 and exit. Their diagnostics
  still need source locations and operand evidence, and recoverable unwinding is absent.
- Integer remainder at signed minimum divided by -1 is zero; signed division at
  the same boundary panics. Preserve this distinction in both profiles.
- Float32 literals are parsed directly to float32, avoiding double rounding through float64.
- Do not infer a whole block's constant from its last emission: earlier paths may
  leave with another value. A regression covers this previous bug.
- The flow checker caps active paths at 4096 with B001; replace path enumeration
  with suitable CFG/dataflow analyses as the language grows.
- The compiler embeds its runtime archive but depends on the installed LLVM shared
  library and exact native tool paths. It is not relocatable/offline distribution packaging.
- LLVM IR is verified before and after optimization. The bridge copies explicitly
  bounded source bytes into an owned LLVM buffer; preserve that FFI boundary.
- Broad fuzzing, full source/input resource limits, canonical artifact diagnostics,
  rich related spans, source-level debug information, and failure capsules are pending.
- The parser limits recursive descent and caps constructed AST depth at 256 with
  B001. A 20,000-term expression previously overflowed the checker stack; regression
  coverage now verifies graceful rejection. Keep the depth guard before recursive passes.
- Conformance supervision kills the whole compiler/application process group on
  timeout. A B001 result cannot hide a later compiler fault or count as a strict pass,
  even when its message is empty. Harness regressions protect these distinctions.

## Validation log

- 2026-09-06: repository was clean before implementation; no prior compiler existed.
- 2026-09-06: detected Rust 1.98.1, LLVM/Clang/LLD 22.1.8, CMake 4.4.3,
  Ninja 1.13.2, Python 3.14.7 on the current host.
- Final `cargo test --locked --manifest-path compiler/Cargo.toml`: 33 library tests
  and 12 native integration tests passed. Native tests execute both output profiles.
- Final `python3 -m unittest discover -s compiler/tests -p 'test_*.py'`: 4 harness tests passed.
- Final `cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings`
  and `cargo fmt --manifest-path compiler/Cargo.toml --check` both passed.
- `python3 compiler/tests/conformance.py`: 7 passed, 16 unsupported, 0 failed, both profiles.
  Passed: `compact_min`, `minimum_parenthesized`, `invalid_separator`, `forward_group`,
  `forward_interrupted`, `scalar_projection`, `function_equality`. This is NOT full conformance.
- Parser tests retain 10,000 deterministic malformed/Unicode inputs and long-chain
  stress regressions. These are smoke coverage, not comprehensive fuzz qualification.
- Backend focused validation: bounded FFI inputs, invalid LLVM syntax/SSA, ELF emission,
  integer boundaries, string/numeric output, short circuiting and `/dev/full` all passed.
- Checker focused validation: 12 tests passed, including float32 rounding, block
  path constants, signed remainder, forward groups and the 4096-path limit.
- The reference catalog validator passed all 23 records; this was metadata validation only.
- `cargo build --locked --release --manifest-path compiler/Cargo.toml` succeeded.
  The release compiler built and ran `examples/records.mwy` with expected stdout.
- Actual `readelf -h/-d/--version-info` inspection of `build/records` found ELF64
  x86-64 PIE, interpreter `/lib64/ld-linux-x86-64.so.2`, and only `libc.so.6` in
  DT_NEEDED. It requires GLIBC_2.34; the documented glibc 2.31 target is NOT qualified.
  No LLVM, Rust, or C++ runtime library appears in that executable's dependency list.

## Next steps

1. Add normalized `Type::Union`, explicit coercion and type-test expressions in
   `src/hir.rs`. Implement inline tagged storage, tag remapping, null defaults,
   equality and formatting in `src/backend.rs`; verify both output profiles.
2. Replace enumerated paths with shared boolean reach guards in `src/check.rs`
   and a focused `src/flow.rs` module. Preserve complementary predicate facts,
   invalidate mutable-place proofs on assignment, and handle restart edges safely. Make
   `conditional_field` pass without weakening emission initialization. Add native
   tests for present and absent fields in both profiles, retain E204/E205 rejection
   coverage, and run the Rust and conformance suites before marking it supported.
3. Introduce storage places, moves/borrows, capture summaries, and cleanup lowering
   before enabling references or owned collections. Target `reference_identity`,
   `slice_equality`, `temporary_borrow`, and `named_owner_borrow` with real semantics.
4. Add required evaluation, effects, logical budgets and constrained specialization.
   Then enable the type-helper, compile-effect/budget, and callable fixtures.
5. Prototype task stack/unwind/cleanup alongside ownership, as `../COMPILER.md`
   requires, before committing to a scheduler/runtime ABI.
6. Implement project/module graphs, native adapters and Meowy standard-library
   layers, then tools/artifacts and distribution qualification from the table above.
7. Keep the bootstrap gate green; expand the harness's REQUIRED set when a case
   becomes supported. Only `--strict` with zero unsupported cases passes the catalog's
   full-language gate, and even that catalog is only part of v0.0.1 qualification.
