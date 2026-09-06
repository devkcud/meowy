# Compiler handoff and work tracker

Updated: 2026-09-06. Nullable-union milestone completed and verified.
Full v0.0.1 remains incomplete. No implementation workers or failing milestone checks remain.
This file is the restart point; all implementation work is under `compiler/`.
Baseline commits: `54080b1` (compiler), `c82354f` (tests/examples), `f2feea2` (docs).
Union commits: `8209163` (compiler/guard behavior), `177169e` (native coverage/example).

## Current objective

Completed: union/nullable values and guarded flow analysis for conditional emissions.
`conditional_field` passes; native cases cover both output profiles and preserve
E204/E205 initialization diagnostics. Next: design storage places and ownership
analysis before enabling references, borrows, captures or owned collections.
The existing reference remains authoritative. Do not change fixtures to make tests pass.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. Run `cargo test --manifest-path compiler/Cargo.toml` from the repository root.
4. Run `python3 compiler/tests/conformance.py`; 15 unsupported cases are currently expected.
5. Consult the validation log below before claiming any gate passed.
6. After every compiler work step, update `Next steps` and add a checkpoint to
   `Step log` before continuing. Include investigations, edits, checks, and decisions.

No dependency downloads or remote services are needed by the current Rust design.
The bootstrap uses installed LLVM/Clang/LLD 22.1.8 and Rust 1.98.1. It does not yet
provide a bundled sysroot or a qualified distribution. Never claim the documented
Linux 5.4/glibc 2.31 execution baseline from this workstation build.

## Step log

### 2026-09-06 — Complete the nullable-value handoff

- Completed: implementation committed as `8209163`; integration coverage and
  example committed as `177169e`. Updated supported behavior, test counts, limits
  and the ordered ownership/borrow next steps. This documentation is the final split.
- Validation: 77 Rust tests and 4 harness tests pass; conformance is 8 passed,
  15 unsupported, 0 failed. Clippy, formatting, release build and example execution pass.
- Blockers: none for starting ownership design; full loop fixed points, resource
  qualification and all other release gaps remain explicitly listed below.
- Next steps: trace reference-identity and temporary-owner fixtures through HIR,
  specify storage/borrow invariants, and record that design before implementation.

### 2026-09-06 — Commit nullable-union compiler behavior

- Completed: committed normalized union HIR, inline LLVM storage/conversions,
  guard analysis, branch narrowing, emission inference and their unit tests.
- Validation: staged whitespace check passed; full runtime/checking evidence is
  recorded in the preceding checkpoints.
- Blockers: none.
- Next steps: commit the native source regressions, nullable example and required
  conformance case, then finalize the ownership-oriented continuation plan.

### 2026-09-06 — Verify the release example and prepare feature commits

- Completed: the release compiler ran `examples/nullable.mwy` and printed exactly
  `meowy`, `null`, `meowy`, `guest`, each on its own line.
- Validation: final tracked diff passed whitespace checks. Changes are limited to
  union/checker/guard code, its tests/example, and compiler documentation.
- Blockers: none.
- Next steps: commit the compiler feature with its unit tests, then native/conformance
  coverage, and finally the completed handoff with ownership work queued next.

### 2026-09-06 — Pass the final union validation gate

- Completed: final combined checks passed: 53 library tests, 24 native tests,
  4 harness tests, Clippy, formatting and the release compiler build.
- Conformance: 8 passed, 15 explicitly unsupported, 0 failed in debug/release;
  `conditional_field` is required. No reference fixtures were changed.
- Blockers: none for this milestone; loop fixed points and ownership remain future work.
- Next steps: smoke-run the release compiler's nullable example, inspect the final
  diff, and commit compiler behavior, integration coverage and updated handoff separately.

### 2026-09-06 — Finish semantic checks and refresh supported behavior

- Completed: all 53 library and 24 native tests passed; independent review found
  no further union correctness issue after the field-key fix. Updated README with
  union support and conservative literal/restart boundaries.
- Validation: Clippy passed. Formatting initially flagged two recently added
  checker assertions; the owner formatted them. The final combined gate is rerunning.
- Limits: a separate 65,536-parameter stress case timed out after 60 seconds;
  general frontend resource/performance qualification remains open.
- Next steps: finish the combined gate, build the release compiler and run the
  nullable example, then commit implementation, integration coverage and handoff separately.

### 2026-09-06 — Close union integration defects

- Completed: field proofs now include storage-type identity and retain original
  tags through narrowed subsets. Discarded-only emission slots preserve operand
  effects without forcing stores into a different completed result type.
- Validation: 13 backend tests and the native restart regression pass in both
  profiles. Guard-budget stress produces B001 rather than a compiler fault.
- Blockers: no confirmed unresolved defect; final source regression checks are next.
- Next steps: run all Rust/native/harness/conformance checks, refresh supported
  behavior and remaining-work documentation, then split the finished feature commits.

### 2026-09-06 — Pass conditional-field conformance and resolve composition edges

- Completed: `conditional_field` now passes both checking profiles; the catalog is
  8 passed, 15 unsupported, 0 failed. It is now a required bootstrap case.
- Validation: 18 checker tests pass. The restart-default native case also passes
  after preserving contextual record fields and skipping stores for discarded-only
  emissions; their operand effects still execute.
- Finding: review found field-refinement keys colliding across different record
  variants sharing a field name. Checker owner is correcting variant identity.
- Next steps: cover same-named fields in distinct union variants, rerun the full
  native suite, then complete Clippy/format/conformance validation and feature commits.

### 2026-09-06 — Exercise nullable source programs

- Completed: ran the complete native source suite with the union implementation.
- Validation: 21 of 22 tests passed, including normalization/retagging, optional
  values, equality, narrowing, mutation invalidation and compressed matcher flow.
- Failure: primary record composition passes only the outer primary expectation
  into a nested block, wrongly rejecting its named field with E207. Checker owner
  is fixing this without manufacturing missing inner fields or bypassing E204/E205.
- Next steps: verify partial record construction and restart defaults, review guard
  soundness, then run the full compiler/conformance gates.

### 2026-09-06 — Verify HIR and native union lowering independently

- Completed: normalized HIR set regression passed independently; backend tests now
  cover scalar/record union payloads, tag conversion, equality, defaults, restart
  clearing and exactly-once evaluation. Guard module formatting/Clippy checks passed.
- Validation: all 11 current backend tests passed, including native union execution
  in debug and release. Full checker semantics and end-to-end regressions remain pending.
- Blockers: none; whole-crate compilation is available again after checker integration.
- Next steps: run the new native source suite and fix any remaining checker/backend
  mismatches before expanding the required conformance set.

### 2026-09-06 — Add native union coverage and validate guard primitives

- Completed: added `examples/nullable.mwy` and native cases for nullable defaults,
  union retagging, equality/formatting, local/field narrowing, short circuiting,
  complementary emissions, mutation invalidation and restart defaults.
- Validation: the standalone guard engine passed five tests, including truth tables,
  correlation, 16,384 independent variables and resource exhaustion. New native tests
  are not yet run; checker integration is still incomplete.
- Blockers: whole-crate compilation temporarily fails while the checker replaces
  old path references. Backend union operations are implemented and adding focused tests.
- Next steps: finish checker integration, run the new native suite, fix concrete
  failures, and promote `conditional_field` to required conformance only after it passes.

### 2026-09-06 — Assign independent implementation work

- Completed: assigned `src/backend.rs` to union lowering, `src/check.rs` to guarded
  inference/narrowing, and `src/flow.rs` to a canonical boolean-guard engine.
  The primary writer owns `src/hir.rs`, integration tests, documentation and STATUS.
- Decision: guard operations expose a bounded-resource failure to the checker;
  union storage recursively writes active fields into zeroed memory to avoid
  reading inactive/padding bytes. No new dependencies or public syntax are needed.
- Validation: design checked against union, equality, emission and narrowing rules;
  compiling the whole feature remains pending integration.
- Next steps: add native cases for optional fields/primaries, union tag remapping,
  active-variant comparisons/printing, flow proofs and their invalidation.

### 2026-09-06 — Add the shared union representation

- Completed: baseline split is complete (`54080b1`, `c82354f`, `f2feea2`). Added
  normalized union-set helpers and explicit coercion/type-test nodes in `src/hir.rs`.
- Validation: helper regression added; full compilation is temporarily incomplete
  until backend and checker handle the new variants. Do not claim this intermediate tree passes.
- Blockers: integration work is expected, not a missing input.
- Next steps: backend owner implements tagged storage; checker owner integrates
  union inference/narrowing with a separately implemented boolean-guard engine.

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
| Names, types, flow | `src/check.rs`, `src/flow.rs` | Scalar/record unions, branch narrowing and guarded emissions; 19 checker and 5 guard tests pass |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM → ELF pipeline including tagged unions; 13 focused backend tests pass |
| CLI and diagnostic rendering | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Implemented and exercised through native tests |
| Native tests, examples, documentation | `tests/`, `examples/`, `README.md` | 24 native tests, 4 harness tests and 5 runnable examples |

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

## Completed union milestone

- [x] Review reference contracts and commit the existing bootstrap separately.
- [x] Add normalized union HIR and explicit coercion/type-test operations.
- [x] Implement and independently test shared boolean guards.
- [x] Integrate union typing, reachable slot inference and branch narrowing.
- [x] Verify tagged LLVM storage, conversions, equality, formatting and defaults.
- [x] Pass new native cases and `conditional_field` in debug/release.
- [x] Refresh documentation, run required checks and make focused feature commits.

## Still outside this compiler

Items here must not be silently accepted with different semantics. `B001` is an
internal bootstrap capability diagnostic, not an assigned reference error code.
The supported portion of the language uses the reference's established `E...`
codes. The bootstrap's JSON diagnostic stream is not a release artifact schema.

| Area | Work remaining | Required evidence |
| --- | --- | --- |
| Full frontend | Complete grammar, stable item IDs, recovery CST/editor integration, all type forms | Conformance, compact syntax properties, malformed UTF-8 and parser fuzzing |
| Type system | Literal types/unions, subtraction, callable environments, generics/capabilities, full type queries, nominal identity | All type and callable fixtures plus negative boundary cases |
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
- Record fields remain immutable. Nullable primary/field joins and scalar/record
  union predicates now work. Mutable bindings can replace records; this invalidates
  field proofs. Proof caches include both place and storage type across variants.
- Multiple matching numeric widths in an expected union report E207 rather than
  choosing one. Bind an explicitly typed member first; no width conversion is implicit.
- Interpolation streams only at print/panic boundaries. It does not construct a
  heap string or a general-purpose standalone string value.
- Runtime arithmetic and explicit panic report P002/P006 and exit. Their diagnostics
  still need source locations and operand evidence, and recoverable unwinding is absent.
- Integer remainder at signed minimum divided by -1 is zero; signed division at
  the same boundary panics. Preserve this distinction in both profiles.
- Float32 literals are parsed directly to float32, avoiding double rounding through float64.
- Do not infer a whole block's constant from its last emission: earlier paths may
  leave with another value. A regression covers this previous bug.
- Shared boolean guards replace the former 4096-path enumeration limit. Node,
  cache and work budgets in `src/flow.rs` produce B001 when exhausted. Named-block
  entry conservatively forgets mutable proofs; restart with writes into surviving
  outer result slots reports B001. Full loop fixed points and ownership MIR remain pending.
- The compiler embeds its runtime archive but depends on the installed LLVM shared
  library and exact native tool paths. It is not relocatable/offline distribution packaging.
- LLVM IR is verified before and after optimization. The bridge copies explicitly
  bounded source bytes into an owned LLVM buffer; preserve that FFI boundary.
- Broad fuzzing, full source/input resource limits, canonical artifact diagnostics,
  rich related spans, source-level debug information, and failure capsules are pending.
- A 65,536-parameter stress input timed out after 60 seconds during review; broad
  frontend scaling is not qualified. Compact correlated-guard exhaustion returns B001.
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
- Final `cargo test --locked --manifest-path compiler/Cargo.toml`: 53 library tests
  and 24 native integration tests passed. Native tests execute both output profiles.
- Final `python3 -m unittest discover -s compiler/tests -p 'test_*.py'`: 4 harness tests passed.
- Final `cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings`
  and `cargo fmt --manifest-path compiler/Cargo.toml --check` both passed.
- `python3 compiler/tests/conformance.py`: 8 passed, 15 unsupported, 0 failed, both profiles.
  Passed: `compact_min`, `minimum_parenthesized`, `invalid_separator`, `forward_group`,
  `forward_interrupted`, `conditional_field`, `scalar_projection`, `function_equality`.
  This is NOT full conformance.
- Parser tests retain 10,000 deterministic malformed/Unicode inputs and long-chain
  stress regressions. These are smoke coverage, not comprehensive fuzz qualification.
- Backend focused validation: bounded FFI inputs, invalid LLVM syntax/SSA, ELF emission,
  integer boundaries, string/numeric output, short circuiting and `/dev/full` all passed.
- Checker focused validation: 19 tests passed, including nullable slot inference,
  complementary predicates, field narrowing, assignment invalidation, record composition
  and restart boundaries. Five guard tests include 16,384 independent variables.
- The reference catalog validator passed all 23 records; this was metadata validation only.
- `cargo build --locked --release --manifest-path compiler/Cargo.toml` succeeded.
  The release compiler runs `examples/nullable.mwy` with exact stdout
  `meowy\nnull\nmeowy\nguest\n`. The earlier records example also passed.
- Actual `readelf -h/-d/--version-info` inspection of `build/records` found ELF64
  x86-64 PIE, interpreter `/lib64/ld-linux-x86-64.so.2`, and only `libc.so.6` in
  DT_NEEDED. It requires GLIBC_2.34; the documented glibc 2.31 target is NOT qualified.
  No LLVM, Rust, or C++ runtime library appears in that executable's dependency list.

## Next steps

1. Read `../docs/reference/memory.md` and the `reference_identity`, `temporary_borrow`
   and `named_owner_borrow` fixtures. Trace current locals, field projections and
   scope exits in `src/hir.rs`, `src/check.rs` and `src/backend.rs`. Record storage-place,
   lifetime and cleanup invariants and a concrete MIR/dataflow design before editing.
2. Introduce explicit storage places and borrow/move analysis before accepting `&`
   or dereference syntax. Verify reference identity, immutable/exclusive borrow
   conflicts, mutation invalidation and returned-reference lifetimes with accepted
   native cases and the specified E3xx rejections; preserve current union proofs.
3. Add partial initialization, capture summaries and cleanup lowering; then enable
   bounded collections/slices and the owner-borrow fixtures. Test exact-once cleanup
   and temporary-owner rejection. Do not mark unsupported tests as passing rejections.
4. Add required evaluation, effects, logical budgets and constrained specialization.
   Then enable the type-helper, compile-effect/budget, and callable fixtures.
5. Prototype task stack/unwind/cleanup alongside ownership, as `../COMPILER.md`
   requires, before committing to a scheduler/runtime ABI.
6. Implement project/module graphs, native adapters and Meowy standard-library
   layers, then tools/artifacts and distribution qualification from the table above.
7. Keep the bootstrap gate green; expand the harness's REQUIRED set when a case
   becomes supported. Only `--strict` with zero unsupported cases passes the catalog's
   full-language gate, and even that catalog is only part of v0.0.1 qualification.
