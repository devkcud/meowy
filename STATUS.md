# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `0b24669` supports immutable records with primary, named and nested
  shared-reference components. Projections retain only selected loans; whole
  copies retain every component. Conflicting writes report E302 and escapes E303.
  Record equality preserves full fields while scalar comparisons project primaries.
- Native coverage/example: `7816524`; `compiler/examples/borrowed-records.mwy`
  exercises copies, nested projections, primary references and loop final uses.
- Runtime/tooling: `3dbed45` adds private pinned contexts on guarded stacks using
  checksum-verified Boost.Context sources. Explicit resume/yield/completion,
  worker pinning, register preservation and ASan fiber hooks are exercised.
- All 14 combined checks pass: 119 Rust tests, 31 Python tests, 841 local links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Each native
  debug/release/sanitized profile passes 14 cleanup, 10 stack and 10 context cases,
  with exact fatal/guard probes. ASan detects the expired-fiber-local probe; normal
  cases pass ASan/UBSan/LSan outside ptrace supervision.
- The optimized compiler/example passes with exact stdout. Its independent
  component oracle passes all 300 cases: 204 accepted, 96 E302, no other results.
- No active workers, incomplete code or failing checks remain. Conformance is
  9 passed, 14 unsupported, 0 failed. Generated programs still use the scalar
  runtime; scheduling, automatic cancellation/join and DWARF are not implemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded shared loans and immutable reference records | Reference unions/function contracts, exclusive access, moves and cleanup |
| Runtime | Separate cleanup, guarded allocation and pinned context prototypes | Bounded scheduling, structured joins, DWARF and generated cleanup |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend compiler component origins to reference unions and verified function/call
   contracts. Preserve branch identity and the documented all-input lifetime bound;
   verify accepted projections/returns, E303 escapes and E302 caller writes.
2. Add explicit read, reborrow and cleanup edges before exclusive loans or reference
   reassignment. Improve predicate-assignment and loop precision while preserving
   resource bounds and explicit B001 for missing proofs.
3. Build bounded scheduler admission and worker-owned queues over `runtime/` contexts.
   Add structured child cancellation/join before parent storage release, then
   pinned unwind-library integration, Meowy personality and landing pads. Verify
   cleanup that suspends while joining and interleaved partial-result cleanup order.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 -B tools/verify.py --all` after integrations. LSan needs an environment
   where it can inspect processes. `--strict` still fails for 14 unsupported catalog
   cases; neither the bootstrap gate nor this host qualifies a complete release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Git whitespace checks exclude only that upstream blank-at-EOF
warning; project-owned files require the normal check.
