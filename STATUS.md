# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `5e7ad41` adds guarded bare-reference block results and native coverage.
  All possible origins survive aliasing; retained local escapes are rejected.
- Runtime: `e0987be` adds the independent explicit cleanup prototype, tests and
  rules. Task stacks, DWARF unwinding and compiler integration remain pending.
- Tooling: `1ba3c65` adds runtime verification to `--runtime` and `--all`.
- The previous implementation gate passed all 14 checks: 93 Rust tests, 25 Python
  tests, 829 links, schemas/catalog, editors, formatting/Clippy/build and conformance. Cleanup passed
  14 cases plus 2 fatal probes in each debug/release/sanitized profile. LSan required
  outside-sandbox execution because ptrace blocks its inspection.
- Release example output is exactly `11`, `22`, `42`, `true`. Conformance remains
  9 passed, 14 unsupported, 0 failed. The implementation milestone is complete.
- Tracker split verified: historical entries and checklists are preserved exactly;
  834 local links and diff whitespace checks pass. No implementation changed.

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
