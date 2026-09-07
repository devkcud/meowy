# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented explicit storage lifecycle and demand-driven forward availability
  in `d72b413` on the existing CFG. Canonical cells and function/block/branch/statement scopes are
  separate from reference value IDs. Returning initialization, scope endings and
  value-taking versus inspection are recorded as events.
- Availability tracks ready, moved, uninitialized and ended alternatives under guards.
  It preserves short circuits, named Leave, statement temporaries, target-owned aliases,
  restart and panic. Only demanded states are retained; empty scopes keep events without
  unnecessary state. The existing 1,000-matcher acceptance case passes within unchanged
  limits after an initial budget regression was corrected.
- All current HIR source types remain Copy. Internal tests explicitly vary cell Copy
  metadata to prove E301/E309 transitions, reinitialization and ended-scope protection;
  this does not enable source-level moves or `&!`. Permission enforcement, parent
  suspension, indirect stores and generated cleanup remain open.
- All ten compiler checks pass: 592 Rust tests (334 library, 258 native), 20 Python
  tests, 879 links, schemas/catalog, formatting, Clippy, build and conformance.
  All 37 examples execute in debug/release. Fifteen lifecycle groups extend the
  existing access/provenance coverage; no reference fixture or backend code changed.
- Shared provenance remains in `ebc8ebe`, access records in `698e4b1`, and terminal
  expiry in `7906333` with native/example evidence in `324e9ae`. Opaque ancestry and
  unresolved regions cannot authorize access. See the
  [exclusive-reference design](compiler/EXCLUSIVE_REFERENCES.md) for the remaining slice.
- Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
  Runtime/editor suites were not rerun; their historical evidence remains in the
  compiler handoff. Complete v0.0.1 qualification is still open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Lifecycle, guarded availability and shared provenance | Mode-aware exclusive permissions and scalar indirect stores |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Integrate explicit exclusive-reference mode through `compiler/src/hir.rs`,
   `check/`, `borrow/`, `loans/` and the backend scalar-store path. Use the existing
   value-taking/inspection events and Copy classification for real source moves;
   enforce permissions and parent suspension against guarded loan provenance and
   normalized regions. Capture indirect targets once and retain demand only through
   returning stores. Enable the designed ordinary-scalar slice only after native
   debug/release and exact-code tests pass; preserve opaque call/restart, carrier,
   alias/reference-cell and owned-payload gates.
2. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before cancellation and unwinding.
3. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
4. Extend aggregate/emitted-name/cross-element constraints in the list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
5. Preserve current compiler evidence; do not rerun green checks without a new change
   or concern. Run `python3 -B tools/verify.py --all` after wider integrations. LSan
   needs process inspection. Strict conformance still has 13 unsupported cases;
   this bootstrap gate and host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
