# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented shared authority provenance in `ebc8ebe`: graph-local LoanIds distinguish
  separate
  acquisitions, copies keep identity, and reborrows retain guarded parent alternatives.
  Branches, short circuits, named Leave and copied pointee contents preserve ancestry.
  These identities remain separate from physical sources and immutable value IDs.
- Graph values now preserve actual origins separately from public lifetime bounds.
  Existing liveness still visits both roles; normalized access regions use only actual
  origins. Named-field paths agree across direct reads and reborrowed views while
  canonical slots, lexical views, primary/variant paths and terminal expiry remain.
- Calls and reachable restart bodies retain opaque authority. Restart-erased tag
  correlations may leave explicit unresolved-region guards; these cannot authorize
  access. Missing actual origins in non-restarting graphs remain B001 even for calls.
  No exclusive mode, consuming operation or forward initialization check is enabled.
- All ten compiler checks pass: 577 Rust tests (319 library, 258 native), 20 Python
  tests, 879 links, schemas/catalog, formatting, Clippy, build and conformance.
  All 37 examples execute in debug/release. Fifteen new groups cover identity,
  guarded parents, bounds, opacity, region resolution, inputs, expiry and budgets.
- Existing shared E302/E303 behavior remains intact; the first full gate exposed
  three restart-region regressions that were corrected without changing expectations.
  The [exclusive-reference design](compiler/EXCLUSIVE_REFERENCES.md) now records the
  implemented provenance foundation and the remaining permission/availability work.
- Access records remain implemented in `698e4b1`, terminal expiry in `7906333`
  with native/example evidence in `324e9ae`. No backend, runtime ABI, dependency or
  reference fixture changed. Metadata growth stays within the existing logical caps.
- Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
  Runtime/editor suites were not rerun; their historical evidence remains in the
  compiler handoff. Complete v0.0.1 qualification is still open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded shared provenance and actual-source access regions | Storage lifecycle, forward availability and exclusive permissions |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add explicit storage lifecycle/consuming-use events and forward initialized/moved
   state on the existing CFG in `compiler/src/loans/state.rs`, `control.rs`, `values.rs`
   and a focused `init.rs`. Separate physical storage availability from reference
   value definitions and backward demand. Prove initialization, replacement, child
   copies, parent suspension, guarded joins and exact Leave behavior before enabling
   the designed scalar `&!` slice. Preserve opaque call/restart boundaries and reject
   inferred exclusive ancestry crossings until their contracts are implemented.
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
