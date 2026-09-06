# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `e8a6157` lets pure unary/binary scalar expressions constrain expected
  list candidates. It reuses ordinary checked typing in bounded isolated state,
  preserving intermediate widths, grouped negation, typed constants, floating
  behavior and short circuits. Effectful expressions are still checked once.
- Candidate probing uses each expression's original reach. An earlier nonreturning
  element cannot incorrectly remove a candidate because of later arithmetic.
  Probes never import live guard identities or turn typed constants into literals.
- Coverage/example: `15a5ff6` adds six native groups and
  `compiler/examples/compound-lists.mwy`. Four new unit groups cover exact scalar
  behavior, lexical identities, saved reach and charged constant/probe work.
- All 14 checks pass: 212 Rust tests, 35 Python tests, 852 local links, editors,
  schemas/catalog, formatting, Clippy, build and conformance. An independent corpus
  passes 408 differential comparisons with zero mismatches: 129 unique accepts,
  151 ambiguities and 128 no-fit rejections across 994 compiler checks.
- The optimized compiler builds/runs the new example with exact stdout.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- Runtime behavior is unchanged: owning message snapshots remain `d92f94c`, and
  generated P002/P006 evidence remains `eb65cbd`. Full runtime debug/release/
  sanitized profiles pass, including message lifetime/truncation, P008, guard faults
  and the expired-fiber-local negative diagnosis. No release qualification is implied.
- No active implementation workers, unfinished code or failing checks remain.
  Complex effectful/captured/non-scalar contexts still need explicit annotations or
  future support. Element ownership, generated task cleanup, cancellation and DWARF
  remain unimplemented; the full v0.0.1 release is incomplete.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Pure-compound list inference, shared borrows and failure evidence | Element places, moves/cleanup and remaining contextual constraints |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add verified initialized element places and exclusive access before list mutation,
   slices or owned elements. Preserve root/field identity, evaluation order and
   all-input lifetime bounds; verify conflicts, last use and cleanup boundaries.
2. Extend remaining effectful/non-scalar contextual constraints in
   `compiler/src/list_context.rs` without replaying effects or weakening work limits.
   Keep annotations/B001 where a candidate is unproved; test unique and ambiguous
   cases against ordinary checking.
3. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before adding cancellation and unwinding.
4. Add richer diagnostic source identities, related evidence and artifacts/events;
   current bounded snapshots and byte-span text do not implement complete replay.
5. Build the manifest/module graph for Meowy libraries and documented projects.
   Keep runtime, editor and library progress visible here.
6. Run `python3 -B tools/verify.py --all` after integrations. LSan needs process
   inspection. Strict conformance still has 13 unsupported cases; neither this
   host nor the bootstrap gate qualifies the complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
