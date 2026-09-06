# Compiler handoff and work tracker

Updated: 2026-09-06. Guarded reference results and native cleanup integration pass.
Full v0.0.1 remains incomplete. This milestone is complete; no workers or failing checks remain.
This file tracks compiler implementation; `../STATUS.md` tracks the wider project.
Baseline commits: `54080b1` (compiler), `c82354f` (tests/examples), `f2feea2` (docs).
Union commits: `8209163` (compiler/guard behavior), `177169e` (native coverage/example),
`f749708` (handoff). Reference commits: `71a7baf` (compiler), `064e305` (coverage).
Repository work: `122b022` (schema examples), `6f6a0c1` (verification tooling).
Guarded results: `5e7ad41`. Native cleanup prototype: `e0987be`. Runtime gate: `1ba3c65`.

## Current objective

Completed: guarded bare-reference block results preserve every origin and exclude
proven discarded emissions. Native and release examples pass. The independent
`../runtime/` cleanup prototype and combined repository gate also pass. Next:
CFG loan liveness, aggregate/function borrow contracts and native context/unwind
qualification before mutable/exclusive loans, owners or generated cleanup.
The existing reference remains authoritative. Do not change fixtures to make tests pass.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. Run `cargo test --manifest-path compiler/Cargo.toml` from the repository root.
4. Run `python3 compiler/tests/conformance.py`; 14 unsupported cases are currently expected.
5. Consult the validation log below before claiming any gate passed.
6. After every compiler work step, update `Next steps` and add a checkpoint to
   `Step log` before continuing. Include investigations, edits, checks, and decisions.

No dependency downloads or remote services are needed by the current Rust design.
The bootstrap uses installed LLVM/Clang/LLD 22.1.8 and Rust 1.98.1. It does not yet
provide a bundled sysroot or a qualified distribution. Never claim the documented
Linux 5.4/glibc 2.31 execution baseline from this workstation build.

## Step log

### 2026-09-06 — Complete guarded borrowing and runtime handoff

- Completed: compiler behavior/coverage is `5e7ad41`, runtime prototype is
  `e0987be`, and repository integration is `1ba3c65`. Refreshed supported behavior,
  validation, remaining work and continuation order; this is the final docs split.
- Validation: all 14 combined checks passed with sanitizers enabled outside ptrace;
  93 Rust and 25 Python tests pass, plus native cleanup and release example checks.
  Conformance remains 9 passed, 14 unsupported, 0 failed in both profiles.
  Final documentation links and diff whitespace checks passed.
- Blockers: no active workers, incomplete code or failing milestone check. Predicate
  reassignment remains conservative; full loan dataflow and stack unwinding remain
  future work rather than implied capabilities of this milestone.
- Next steps: implement CFG loan liveness and aggregate/function origin contracts;
  qualify runtime context/unwind interaction, then progress module/library loading.

### 2026-09-06 — Commit the independent native cleanup prototype

- Completed: `e0987be` adds runtime API, implementation, behavior/fatal tests,
  pinned-tool runner, documentation and working rules outside the compiler.
- Validation: 14 normal cases and 2 fatal subprocesses pass in debug, release
  and ASan/UBSan/LSan; 5 Python regressions and staged whitespace checks pass.
- Boundary: compiler linkage, task contexts and DWARF unwinding remain pending;
  payload lifetimes and cross-stack cleanup ordering are explicit caller contracts.
- Next steps: commit repository runtime verification, then complete tracker maps,
  validation evidence and the concrete CFG/context/library continuation plan.

### 2026-09-06 — Commit guarded reference results

- Completed: `5e7ad41` commits emission proof IDs, guarded origin propagation,
  source/native regressions and the borrowed-results example together. The changed
  ancestor-result expectation is included with its behavior change.
- Validation: staged whitespace checks passed; full compiler/runtime gate and
  release example already passed on these source changes.
- Next steps: commit the independent native cleanup prototype, then its repository
  verification integration and the final remaining-work handoff.

### 2026-09-06 — Verify the optimized compiler and prepare split commits

- Completed: the release compiler built and ran `examples/borrow-results.mwy`
  under the release output profile, printing `11`, `22`, `42`, `true` exactly.
- Validation: all changed code and new runtime files have been reviewed; final
  whitespace checks pass. No worker or unresolved implementation defect remains.
- Next steps: commit compiler behavior with its source tests/example, runtime
  prototype with its tests/rules, tooling integration, and final handoff separately.

### 2026-09-06 — Pass the full compiler and cleanup integration gate

- Completed: `python3 -B tools/verify.py --all` passed all 14 checks outside ptrace
  supervision, with sanitizer checks enabled and unchanged source.
- Validation: 64 library + 29 native Rust tests; 16 tooling + 5 runtime + 4 compiler
  Python tests; 14 cleanup cases and 2 fatal probes in each debug/release/sanitized
  profile. Formatting, Clippy, build, 829 links, schemas and both editors pass.
- Conformance: 9 passed, 14 explicitly unsupported, 0 failed in debug/release.
  These results do not qualify task stacks, DWARF unwinding or full v0.0.1.
- Next steps: release-smoke the borrowed-results example, inspect all changes,
  commit compiler/runtime/tooling independently and finalize the continuation plan.

### 2026-09-06 — Audit borrow proofs and rerun sanitizer gate with required access

- Validation: 10 targeted borrow probes matched expected outcomes; accepted cases
  executed in debug/release. Another 288 predicate combinations found no unsafe
  acceptance. Stable proofs matched exhaustive lifetime safety; reassigned
  predicates conservatively rejected 15 safe combinations after facts were lost.
- Environment failure: combined verification passed repository/editor/runtime
  runner checks and native debug/release, then LeakSanitizer failed under sandbox
  ptrace. The unchanged full command is rerunning outside that supervision.
- Next steps: finish the combined gate, retain the predicate-mutation proof limit
  in the handoff, run the release example and prepare focused commits.

### 2026-09-06 — Pass focused native checks and finish prototype review

- Completed: guarded references pass native selection/alias, discarded-result and
  corrected local-escape tests. Added the borrowed-results example and refreshed
  compiler/storage documentation. Tooling now checks runtime docs and `--runtime`.
- Validation: 16 tooling tests pass. Independent runtime review found no concrete
  defect; 14 cleanup cases plus 2 exact fatal probes pass in each debug/release/
  ASan/UBSan/LSan profile. Five runtime runner tests pass.
- Boundary: runtime transfer moves only cleanup obligations. Partial emissions
  need deferred transfer or cross-stack order metadata before compiler integration.
- Next steps: finish borrow proof edge review, run the complete combined gate,
  release-smoke the example, then split compiler/runtime/tooling/handoff commits.

### 2026-09-06 — Validate guarded origins and integrate runtime verification

- Completed: the new origin pass preserves all alternative roots and passes its
  eight focused source-test groups. Complementary reference results execute in
  both profiles. Added explicit `--runtime` and expanded `--all` verification.
- Failure: a native rejection fixture emitted into an inner block, accidentally
  producing a nullable outer result and E222. Corrected its named emission target
  so it exercises the intended retained local escape.
- Runtime validation: 13 cleanup cases and 2 fatal probes pass in debug/release
  and ASan/UBSan; 5 runner tests pass. LeakSanitizer required outside-sandbox
  execution because ptrace blocks its inspection. Independent review is active.
- Next steps: rerun the corrected native cases, verify tooling/runtime integration,
  review origin soundness and run the combined gate after remaining edits settle.

### 2026-09-06 — Connect emission proofs and add source regressions

- Completed: each HIR emission now receives a unique proof ID, including generated
  record components and unreachable writes. Block completions retain the original
  guard arena. Backend lowering ignores proof IDs; code generation is unchanged.
- Coverage: added native programs for complementary reference selections, aliases,
  named leave, restart-discarded locals, panic effects and retained local escapes.
- Validation: integration waits for the new guarded-origin pass. Tests have not
  yet run against the changed API; runtime behavior tests are also being built.
- Next steps: finish origin propagation, run focused compiler/native checks and
  integrate the runtime verification entrypoint once its checks pass.

### 2026-09-06 — Choose guarded result identities and cleanup boundaries

- Decision: give each HIR emission a unique ID; keep write/completion guards in
  private proof tables and pass the existing guard arena to borrow checking.
  Preserve every possible origin, filter discarded paths, and validate lifetimes
  only for completed bare-reference results. Aggregates/signatures remain B001.
- Runtime: caller-owned cleanup entries, checked scope/token generations and
  explicit reverse cleanup are implemented; behavior/sanitizer checks are pending.
  No C++ exception, task-stack or scheduler behavior is being claimed.
- Ownership: root owns HIR/frontend/native integration; borrow worker owns borrow.rs;
  runtime worker owns runtime/. No new behavior has passed its full gate yet.
- Next steps: connect proof identities, propagate guarded origins, add source
  regressions and verify runtime cleanup edges before integration.

### 2026-09-06 — Resume guarded borrows and native cleanup work

- Completed: confirmed a clean tree and read working rules, current handoffs,
  ownership boundaries and compiler architecture. Prior milestone checks are the
  baseline; new behavior is not yet verified.
- Active work: root owns compiler integration and both STATUS files; a reviewer
  is tracing guarded borrow-result dataflow. A separate worker owns `runtime/`
  cleanup/unwind prototyping outside the compiler.
- Next steps: settle the guard metadata and result-origin design, implement safe
  immutable block-result transfers, and validate an explicit native cleanup ABI.

### 2026-09-06 — Complete reference and repository handoff

- Completed: native/example coverage and required `reference_identity` conformance
  committed as `064e305`, following compiler behavior `71a7baf`. Repository tooling
  is `6f6a0c1`; schema identity corrections are `122b022`. This handoff is the final
  focused documentation/rules commit, including the tooling cache ignore.
- Validation: all 12 combined checks passed; 90 Rust and 18 Python tests pass;
  catalog execution is 9 passed, 14 unsupported, 0 failed in both profiles.
  The release compiler example printed `true`, `false`, `42`, `meowy` exactly.
  Final documentation link and diff whitespace checks passed.
- Blockers: no incomplete implementation edits, active workers or failing checks.
  Full language/library/runtime and distribution gaps remain recorded below.
- Next steps: add guarded result-origin analysis and CFG loan liveness, alongside
  native unwind/cleanup prototyping; then build manifest/module/library integration.

### 2026-09-06 — Commit shared-reference compiler behavior

- Completed: `71a7baf` adds typed storage addresses, pointer lowering, frontend
  capability checks and lexical borrow-origin validation with unit coverage.
- Validation: staged whitespace checks passed; 90 Rust tests, conformance and
  release execution were verified before committing.
- Next steps: commit native reference cases/example and required conformance
  coverage, then finish the cross-project handoff and remaining-work priorities.

### 2026-09-06 — Verify release output and finish review

- Completed: the optimized release compiler built successfully and ran the
  references example with exact stdout `true`, `false`, `42`, `meowy`.
- Validation: final diff whitespace checks pass; independent review found no
  unresolved storage/provenance defect. All 12 combined checks already passed.
- Handoff: tooling worker confirmed ownership of the remaining cache-ignore file;
  it will be included with repository working rules and documentation.
- Next steps: commit compiler implementation, native/example coverage, then the
  final trackers/rules with concrete guarded-borrow and outside-compiler work queued.

### 2026-09-06 — Pass the combined compiler and repository gate

- Completed: all 12 checks in `python3 -B tools/verify.py --all` passed, using the
  pinned host target and fresh compiler binary. No implementation worker remains.
- Validation: 64 library + 26 native tests; 4 compiler harness + 14 tooling tests;
  formatting, Clippy, compiler build, 818 local links, schema integrity, catalog,
  Vim and Neovim. Conformance: 9 passed, 14 unsupported, 0 failed, both profiles.
- Blockers: none for this milestone. Full borrow flow, owners, modules, runtime
  unwinding, library implementation and release qualification remain open.
- Next steps: build/run the release compiler's references example, inspect the
  final diff and commit compiler behavior, source coverage and handoff separately.

### 2026-09-06 — Pass reference identity conformance

- Completed: unchanged `reference_identity` passes in debug/release and is now
  required by the bootstrap harness. Added a runnable reference example and
  documented the narrow storage/lifetime capability boundaries.
- Validation: catalog execution is 9 passed, 14 unsupported, 0 failed. Independent
  native probes passed for null/union addresses, optional fields, recursive local
  borrows and iteration-local references. Full combined gate remains next.
- Decision: E303 is limited to final direct emissions with proven completing
  local result storage; guarded/discarded and other uncertain transfers use B001.
- Next steps: finish new origin regressions, format and run `tools/verify.py --all`,
  then split compiler behavior, source coverage and completed handoff commits.

### 2026-09-06 — Commit repository verification tooling

- Completed: `6f6a0c1` adds repository verification, local-link checking, 14 tests
  and usage documentation. Compiler execution uses an explicit target/output path.
- Validation: all six repository/editor checks and staged whitespace checks pass.
- Next steps: finish discarded-emission diagnostic coverage, document the bounded
  reference support, run the combined gate and commit the compiler feature/tests.

### 2026-09-06 — Commit verified schema identity corrections

- Completed: `122b022` corrects the three distribution references in schema examples.
- Validation: existing schema integrity/rejection checks and staged whitespace
  checks passed. Five focused shared-reference checker/backend/native tests pass.
- Next steps: commit repository tooling independently, finish conservative origin
  diagnostics, and run the combined compiler/repository gate before compiler commits.

### 2026-09-06 — Review discarded reference emissions

- Completed: backend identity/dereference test passes in both profiles. All six
  borrow-origin unit tests and Clippy pass. Fixed a rejection fixture to construct
  its record before union injection so it reaches the intended address boundary.
- Finding: origin validation could report E303 for reference emissions discarded
  by restart/panic or unreachable guards. Emission is not return. The origin-pass
  owner is limiting E303 to proven escapes and using B001 for uncertain transfers.
- Next steps: verify discarded/unreachable emissions, rerun reference checks,
  then run the full combined repository/compiler gate and commit focused slices.

### 2026-09-06 — Integrate origin checks and run first native cases

- Completed: borrow.rs now validates lexical storage, aliases, branches and scoped
  control transfers before backend lowering. Shared identity/dereference tests
  passed in the checker, backend and native source suite.
- Failure: one rejection regression used invalid mutable syntax (`!:`); the
  language uses `:=`. Corrected that test input; no implementation rule changed.
- Validation: three focused tests passed; the scope rejection case needs rerunning.
  Repository tooling now has 14 passing tests and pins its compiler build/run
  target directory, avoiding stale binaries under custom Cargo environments.
- Next steps: rerun reference and origin cases, inspect control-flow handling,
  run full checks and commit the independently verified schema/tooling changes.

### 2026-09-06 — Add shared-reference behavior coverage

- Completed: added source/native cases for copied references, distinct addresses,
  nested record field storage, dereference, local function use and shadowing.
  Rejection coverage includes local escapes and unsupported ownership boundaries.
- Validation: HIR/backend diff checks passed; Cargo verification awaits borrow.rs.
  Baseline Cargo process completed successfully, including doc tests.
- Outside compiler: all six repository/editor checks now pass after correcting
  three stale distribution digests. This is static/editor evidence; the new
  compiler milestone is not yet validated.
- Next steps: complete origin-pass integration, format and run focused reference
  checks, then the full Cargo and repository verification command.

### 2026-09-06 — Implement the bounded reference frontend

- Completed: added shared-reference type handling, physical local/record address
  resolution, scalar/record dereference and immutable reference aliases. Mutable,
  temporary, emitted and parameter storage remain explicit capability boundaries.
- Integration: backend/HIR worker added typed pointer operations; origin-pass
  worker is checking lexical ownership and escaped emissions. Compilation is
  pending the completed pass. Root owns frontend and native tests.
- Outside compiler: 13 new tooling tests, 816 local links, 23 catalog records and
  both editor checks pass. Existing schema examples contain a stale distribution
  digest; tooling worker owns the three affected examples and is correcting it.
- Next steps: integrate origin validation, exercise references in both profiles,
  verify E303 and unsupported cases, then run the full compiler/repository gates.

### 2026-09-06 — Bound shared-reference storage before implementation

- Completed: traced locals, result-slot copies, dispatch receivers and union
  projections. Independent review confirmed proof paths are not physical places.
- Decision: first support immutable ordinary local storage and its record fields,
  shared-reference copies, identity and dereference. Track borrow origins before
  lowering. Mutable roots, exclusive loans, temporary owners and interprocedural
  borrow contracts remain B001. Named emitted bindings cannot yet be borrowed.
- Validation: baseline 53 library and 24 native tests passed. Final Cargo process
  completion still needs polling. Reference fixtures remain unchanged.
- Outside compiler: a worker is implementing repository verification using the
  existing catalog/schema/editor checks plus a tested local-link checker.
- Next steps: record the storage contract in `OWNERSHIP.md`, implement typed
  addresses and origin checks, and test escaped locals and both native profiles.

### 2026-09-06 — Resume after split commits and widen project work

- Completed: confirmed prior work is already split into six focused commits and
  the working tree is clean. Read compiler rules, setup and ownership contract.
- Scope: the user also authorized implementation outside `compiler/`; repository
  verification tooling will progress alongside storage/borrow foundations.
- Validation: no new code yet. Fixture paths are `../docs/conformance/sources/`,
  not `cases/`; correct that path in further investigation.
- Next steps: inspect storage lowering and reference fixtures, record a sound
  bounded implementation design, and delegate independent repository tooling.

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
| Names, types, flow | `src/check.rs`, `src/flow.rs` | Scalar/record unions, branch narrowing and guarded emissions; 21 checker and 5 guard tests pass |
| Shared storage/borrow origins | `src/hir.rs`, `src/borrow.rs`, `OWNERSHIP.md` | Immutable local/record places, guarded block results and all-root escape checks; 8 source-test groups pass |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM → ELF pipeline including tagged unions; 14 focused backend tests pass |
| CLI and diagnostic rendering | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Implemented and exercised through native tests |
| Native tests, examples, documentation | `tests/`, `examples/`, `README.md` | 29 native tests, 4 harness tests and 7 runnable examples |

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

## Completed shared-reference milestone

- [x] Record physical storage and lexical lifetime boundaries before implementation.
- [x] Add local/record-field places, pointer identity and copyable dereference.
- [x] Track immutable reference aliases and reject proven final local escapes.
- [x] Keep guarded/discarded transfers and unavailable ownership rules explicit.
- [x] Pass reference identity and native source checks in debug/release.
- [x] Pass the combined repository gate and release compiler example.

## Completed guarded-result and cleanup-protocol milestone

- [x] Reuse frontend guard proofs with unique emission IDs and block completions.
- [x] Propagate all possible bare-reference origins through blocks and aliases.
- [x] Verify complementary selection, named exits and discarded local emissions.
- [x] Prototype explicit native cleanup outside the compiler with bounded storage.
- [x] Verify initialized-only/reverse cleanup, transfer and fatal P008 evidence.
- [x] Include native debug/release/sanitizer checks in repository verification.

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
| Ownership | Mutable/exclusive borrows, returned/temporary views, moves, partial initialization, captures and cleanup | Use-after-move and borrow rejection; exact-once cleanup on every exit |
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

- Shared borrowing is limited to immutable ordinary local roots and concrete
  record fields. Parameter/receiver/emission places, mutable roots, temporary
  owners, exclusive loans, reference-carrying aggregates/signatures and reference
  reassignment are B001. `OWNERSHIP.md` records the remaining stages.
- Bare-reference block results support guarded alternatives and named exits.
  Every retained origin must outlive its receiving block. Discarded emissions keep
  operand effects but do not escape. Reference-carrying aggregates are still B001.
- Assigning predicates invalidates prior facts. Audit found 15 safe mutated-predicate
  combinations conservatively rejected with E303; stronger relation/dataflow
  tracking is pending. No unsafe acceptance was found in 288 bounded combinations.
- `../runtime/` proves an explicit cleanup protocol only. Generated programs still
  use the scalar runtime. Task stacks, Meowy personality/landing pads, child joins
  and cross-stack partial-result ordering must be implemented before integration.

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
- Final combined `python3 -B tools/verify.py --all` passed all 14 checks outside
  ptrace supervision. It pins the Cargo target/output path and freshly built compiler.
- Cargo tests: 64 library and 29 native tests passed, including both output profiles.
- Repository checks: 16 tooling and 5 runtime Python tests, 829 local links,
  schema/catalog integrity, Vim and Neovim passed. Native cleanup passed 14 normal
  cases plus 2 fatal probes in each debug/release/ASan/UBSan/LSan profile.
- Sandbox LeakSanitizer failed due to ptrace; the unchanged full gate passed with
  the required outside-sandbox execution. No sanitizer check was disabled.
- Final `python3 -m unittest discover -s compiler/tests -p 'test_*.py'`: 4 harness tests passed.
- Final `cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings`
  and `cargo fmt --manifest-path compiler/Cargo.toml --check` both passed.
- Compiler conformance: 9 passed, 14 unsupported, 0 failed, both profiles.
  Passed: `compact_min`, `minimum_parenthesized`, `invalid_separator`, `forward_group`,
  `forward_interrupted`, `conditional_field`, `scalar_projection`, `function_equality`,
  `reference_identity`.
  This is NOT full conformance.
- Parser tests retain 10,000 deterministic malformed/Unicode inputs and long-chain
  stress regressions. These are smoke coverage, not comprehensive fuzz qualification.
- Backend focused validation: bounded FFI inputs, invalid LLVM syntax/SSA, ELF emission,
  integer boundaries, string/numeric output, short circuiting and `/dev/full` all passed.
- Checker focused validation: 21 tests passed, including nullable slot inference,
  complementary predicates, field narrowing, assignment invalidation, record composition
  and restart boundaries. Eight borrow source-test groups cover all candidate
  roots, lexical aliases, retained escapes, function isolation and discarded paths. Five guard tests
  include 16,384 independent variables.
- The reference catalog validator passed all 23 records; this was metadata validation only.
- Release compiler build with the explicit host target succeeded. Its borrowed
  results example prints exactly `11\n22\n42\ntrue\n` in release. The earlier
  references example also passed with `true\nfalse\n42\nmeowy\n`.
  The prior release compiler also ran `examples/nullable.mwy` with exact stdout
  `meowy\nnull\nmeowy\nguest\n`. The earlier records example also passed.
- Actual `readelf -h/-d/--version-info` inspection of `build/records` found ELF64
  x86-64 PIE, interpreter `/lib64/ld-linux-x86-64.so.2`, and only `libc.so.6` in
  DT_NEEDED. It requires GLIBC_2.34; the documented glibc 2.31 target is NOT qualified.
  No LLVM, Rust, or C++ runtime library appears in that executable's dependency list.

## Next steps

1. Add an explicit storage/control-flow graph for `src/borrow.rs` with backwards
   last-use liveness and loop fixed points. Preserve current guards and improve
   predicate assignment relations. Test valid post-use mutation and exact E302
   conflicts before accepting mutable/exclusive roots or reference reassignment.
2. Extend origin sets to reference-carrying record/union components and function
   input/result contracts. Verify all-input lifetime bounds and caller substitution
   with accepted nested returns and E303 rejections; keep unknown origins B001.
3. Add partial initialization, capture summaries and cleanup lowering; then enable
   bounded collections/slices and the owner-borrow fixtures. Test exact-once cleanup
   and temporary-owner rejection. Do not mark unsupported tests as passing rejections.
4. Add required evaluation, effects, logical budgets and constrained specialization.
   Then enable the type-helper, compile-effect/budget, and callable fixtures.
5. Extend `../runtime/` beyond explicit cleanup: qualify the pinned context wrapper
   with bounded stacks/guard pages and sanitizer hooks, then Meowy DWARF personality
   and landing pads. Test a suspended borrowing child that is cancelled and joined
   before parent cleanup. Preserve interleaved partial-result/local cleanup order.
6. Implement project/module graphs, native adapters and Meowy standard-library
   layers, then tools/artifacts and distribution qualification from the table above.
7. Keep the bootstrap gate green; expand the harness's REQUIRED set when a case
   becomes supported. Only `--strict` with zero unsupported cases passes the catalog's
   full-language gate, and even that catalog is only part of v0.0.1 qualification.
