# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `dddd8ae` enables shared borrows of mutable emitted names, concrete
  fields and initialized list elements. Source::Slot separates target-block
  ownership, canonical slot identity and the original alias's declared pointee type.
  A reference may outlive an inner alias name while its outer target stays active.
- Borrowed storage must match the alias type or contain it as one exact concrete
  union member. Proper subunion views remain B001. Discarded initialized cells are
  explicitly owned by the target's partial result under current Copy-only rules.
- Overlapping writes report E302, including future uses across inner restarts;
  last-use RHS reads and disjoint fields remain accepted. Publication cannot return
  a reference into its own construction storage (E303). Canonical identities also
  protect aliases introduced by different guarded emissions.
- Existing backend address helpers handle these cells and payloads without new
  production lowering or runtime ABI changes. `4a012b0` adds eight native groups,
  `compiler/examples/emitted-borrows.mwy` and README coverage; eight library groups
  cover origin/lifetime, conflicts and native addresses.
- All 14 combined checks pass: 326 Rust tests, 35 Python tests, 861 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs emitted-borrows with exact output. Ten independent
  lifecycle/call-bound cases and source review pass. No unfinished source work,
  active workers or failing checks remain; modular source organization is preserved.
- Immutable emitted-name borrows, mutable reference-bearing fields, source-level
  exclusive references, owned cleanup and full release qualification remain open.
  Next is immutable reference-free emitted storage, preserving immutable facts.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Mutable emitted-storage borrows and unified checked writes | Immutable emitted storage, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Give immutable reference-free emitted names actual slot identities before enabling
   their borrows. Preserve immutable tag/origin facts and reject writes with E305;
   coordinate checker aliases, native cells and target-owned lifetimes. Verify
   outer targets, copies, widened/discarded backing, restart and E303 publication.
2. Preserve first-collection conflict rules and precise slot identity while adding
   capabilities. Shared-reference/temporary write roots, mutable reference-bearing
   fields and source-level exclusive references need explicit initialization and
   cleanup models before being enabled.
3. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before cancellation and unwinding.
4. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
5. Extend aggregate/emitted-name/cross-element constraints in the new list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
6. Run `python3 -B tools/verify.py --all` after integrations. LSan needs process
   inspection. Strict conformance still has 13 unsupported cases; the bootstrap
   gate and this host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
