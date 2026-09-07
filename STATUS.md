# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented direct exclusive borrows of mutable emitted Bool/Int/Float aliases
  (`94192da`; contract/example `ec45d2e`).
  Exclusive intent is checked against the completed backing type; it must be exact.
  Shared union-member borrowing retains its existing compatibility rule.
- Canonical Slot roots and target-block lifetime preserve alias identity beyond
  lexical scope, including guarded aliases and cancelled typed fallback storage.
  Mutations, moves, children, calls/blocks, bounds and captured stores reuse the
  existing passes. Self-containing shared views and direct escapes report E303.
- All ten compiler checks pass: 697 Rust tests (343 library, 354 native), 20 Python
  tests, 43 debug/release examples, formatting, Clippy, build, schemas/catalog and
  919 local links. Fourteen new native groups and canonical/backing evidence pass,
  including cancelled storage with a different completed field type.
- Immutable scalar aliases report E305; widened backing, record aliases/projections,
  exclusive carriers and restart bodies remain gated. No backend, ABI, dependency
  or reference fixture changed. Conformance remains 10 passed/13 unsupported/0
  failed; runtime/editor suites were not rerun.
- Standard library/module loading, generated cleanup and complete v0.0.1
  qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Exclusive scalar locals, fields and emitted storage | Emitted record-field borrows and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Design scalar-field exclusive borrows through mutable emitted record aliases.
   Reuse `compiler/src/check/references.rs`, `compiler/src/check/aliases.rs` and
   canonical Slot projections; require exact backing, reference-free Copy records,
   mutable crossed fields and target-block lifetime. Prove disjoint siblings,
   ancestor conflicts and self-escape rejection before removing the projection gate.
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
