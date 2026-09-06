# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: immutable shared-reference places, identity, dereference and bounded
  origin checks pass. [Storage design](compiler/OWNERSHIP.md) records the limits.
- Outside compiler: repository verification and 14 regressions are committed as
  `6f6a0c1`; schema example identity corrections are committed as `122b022`.
- `python3 -B tools/verify.py --all` passed all 12 checks: 90 Rust tests, 18 Python
  tests, 818 local links, catalog/schema integrity, Vim/Neovim, formatting, Clippy,
  compiler build and conformance. Conformance is 9 passed, 14 unsupported, 0 failed.
- Compiler behavior is committed as `71a7baf`, source/native coverage as `064e305`.
  Release compiler smoke execution passed. No active workers, incomplete edits or
  known failing checks remain; this documentation is the final handoff split.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalars, records, unions and immutable shared references work | Guarded borrow flow, moves and cleanup |
| Runtime | Scalar output and fatal panic only | Task stack/unwind prototype and cleanup ABI |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend `compiler/src/borrow.rs` with guarded result origins and CFG last-use
   liveness, then verify mutation/exclusive-loan conflicts and returned-view scopes.
2. Prototype native task-stack/unwind cleanup alongside ownership. Record the
   storage/cleanup ABI and test leave/restart/panic cleanup before scheduler work.
3. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep project-level tools and runtime work visible here.
4. Use `python3 tools/verify.py --all` after semantic integrations. `--strict` is
   still expected to fail for 14 unsupported catalog cases; it is not release proof.

## Step log

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
