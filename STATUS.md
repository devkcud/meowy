# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `5e7ad41` adds guarded bare-reference block results and native coverage.
  All possible origins survive aliasing; retained local escapes are rejected.
- Runtime: `e0987be` adds the independent explicit cleanup prototype, tests and
  rules. Task stacks, DWARF unwinding and compiler integration remain pending.
- Tooling: `1ba3c65` adds runtime verification to `--runtime` and `--all`.
- The full 14-check gate passed: 93 Rust tests, 25 Python tests, 829 links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Cleanup passed
  14 cases plus 2 fatal probes in each debug/release/sanitized profile. LSan required
  outside-sandbox execution because ptrace blocks its inspection.
- Release example output is exactly `11`, `22`, `42`, `true`. Conformance remains
  9 passed, 14 unsupported, 0 failed. No worker, incomplete edit or failing check
  remains; this handoff is the final documentation split.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded immutable reference results work | CFG loan liveness, aggregate/function borrows, moves and cleanup |
| Runtime | Scalar runtime plus a separate tested cleanup protocol | Context stacks, DWARF unwinding and generated cleanup |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add CFG loan liveness and stronger predicate-assignment relations in the
   compiler before enabling mutable/exclusive references. Verify E302 conflicts
   and accepted mutation after a borrow's final use in both profiles.
2. Add reference-carrying aggregate/function contracts, preserving every origin
   and the documented conservative all-input lifetime bound at calls and returns.
3. Extend `runtime/` with the planned pinned context wrapper and bounded stacks,
   then DWARF personality/landing-pad integration. Verify cancellation/join before
   releasing borrowed storage and cross-stack partial-result cleanup ordering.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 tools/verify.py --all` after integrations. It now includes sanitizer
   checks and needs an environment where LSan can inspect processes. `--strict`
   still fails for 14 unsupported catalog cases; neither gate is full release proof.

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

### 2026-09-06 — Pass all combined verification checks

- Completed: repository tooling exercised its compiler path successfully; all 12
  selected checks passed, including 90 Rust and 18 Python tests plus both editors.
- Conformance: 9 passed, 14 explicitly unsupported, 0 failed in debug/release.
  Static schema/link checks and native execution remain separately identified.
- Next steps: finish release smoke validation, create compiler/test/handoff commits,
  and leave guarded borrowing plus runtime/module work ready for continuation.

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

### 2026-09-06 — Pass repository and editor checks

- Completed: repaired only stale distribution digest fields in three schema
  examples. Added reference source/runtime regressions while integration finishes.
- Validation: `python3 -B tools/verify.py --editor both` passed all six checks:
  13 tooling tests, 816 local links, 23 catalog records, 7 schemas/6 examples with
  integrity/rejection checks, Vim and Neovim. Whitespace checks passed.
- Next steps: validate compiler references, run the combined verification command,
  and commit schema corrections separately from tooling and compiler behavior.

### 2026-09-06 — Implement reference and repository foundations

- Completed: reference frontend and pointer lowering are being integrated with
  the lexical origin pass. Repository tooling, documentation and 13 tests exist.
- Validation: tooling tests, 816 local links, 23 catalog records and both editor
  regressions pass. Full schema verification exposed a pre-existing stale
  distribution digest in mod.lock, capsule and build-report examples.
- Ownership: tooling worker owns those three fixture corrections as well as tools;
  compiler workers own HIR/backend and borrow.rs, root owns frontend/integration.
- Next steps: repair fixture identities using the existing validator, complete
  compiler integration and tests, then rerun the combined verification commands.

### 2026-09-06 — Begin work outside the compiler

- Completed: inspected the repository and existing validation scripts. Added root
  rules and this cross-project handoff. Recorded a bounded shared-reference design.
- Validation: baseline 53 compiler library and 24 native tests pass; new work has
  not yet been validated. Existing docs/editor checks will be reused by tooling.
- Next steps: implement shared-reference lowering/origin checks and repository
  verification, then run focused checks before expanding to the full gates.
