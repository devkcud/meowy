# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `94b9160` adds guarded CFG loan liveness for shared references to mutable
  ordinary locals. Live overlapping writes report E302; final-use assignments work.
  Reference temporaries, aliases, block results and named loops are covered.
- Native coverage/example: `e557d63`; `compiler/examples/borrow-liveness.mwy` prints
  `41`, `42`, `7`, `9`, `0`, `1`, `2`, `3` on separate lines in both profiles.
- Runtime/tooling: `7a5d286` adds bounded Linux stack mappings with two guard pages,
  explicit release, retained failure ownership and integrated native checks.
- All 14 combined checks pass: 106 Rust tests, 28 Python tests, 836 local links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Runtime passes
  14 cleanup cases plus 2 fatal probes and 10 stack cases plus kernel ENOMEM and
  2 exact guard faults per debug/release/sanitized profile. LSan ran outside ptrace.
- The optimized compiler/example passes. Its independent guard audit passed all
  196 combinations: 114 accepted and 82 E302, with no unexpected results.
- No active workers, incomplete code or failing checks remain. Conformance is
  9 passed, 14 unsupported, 0 failed. Task switching and DWARF remain unimplemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded shared loans protect mutable ordinary locals | Aggregate/function borrows, exclusive access, moves and cleanup |
| Runtime | Scalar runtime plus separate cleanup and guarded stack prototypes | Pinned context switching, DWARF unwinding and generated cleanup |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend the compiler CFG with read, reborrow and cleanup edges before enabling
   exclusive loans or reference reassignment. Improve predicate-assignment and
   loop precision; verify E302 conflicts and accepted final-use access in both profiles.
2. Add reference-carrying aggregate/function contracts, preserving every origin
   and the documented conservative all-input lifetime bound at calls and returns.
3. Connect guarded stack allocation to the planned pinned context wrapper,
   qualify registers and sanitizer switching hooks, then add DWARF integration.
   Verify cancellation/join before releasing borrowed storage and preserve
   cross-stack partial-result cleanup ordering.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 tools/verify.py --all` after integrations. It now includes sanitizer
   checks and needs an environment where LSan can inspect processes. `--strict`
   still fails for 14 unsupported catalog cases; neither gate is full release proof.
