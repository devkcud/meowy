# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented terminal expired source identities for restart-carried shared
  references and public lifetime bounds. Ended Local/Slot/Temporary sources cannot
  revive when the same static storage site runs again. Safe overwrite-before-read
  is accepted; actual expired use reports E303.
- Live ancestor-owned sources, including enclosing-statement temporaries, survive
  inner restarts. Structural component paths and active variants remain distinct.
  Header transfers add no read; raw predecessor snapshots retain physical loans
  and current-iteration E302 conflicts. Generated storage, runtime ABI and dependencies
  are unchanged. Prior guarded activity is `1df163b` with RestartId `b0c9756`.
- All ten compiler checks pass: 550 Rust tests, 20 Python tests, 873 local links,
  schemas/catalog, formatting, Clippy, build and conformance. All 37 examples run
  in debug/release, including `compiler/examples/expired-restarts.mwy`.
  Independent audit passes twelve checks and ten profile executions.
- New coverage adds three origin, six loan and nine native groups. Obsolete B001
  carriage assertions now check acceptance or actual-use E303. Initial test/gate
  failures and corrections are preserved in the step logs. No failing check remains.
  Implementation is `7906333`; native coverage/example is `324e9ae`. No push or
  publication was requested. The split preserved all 26 non-tracker files exactly;
  Git integrity checks pass. Compiler tests were not rerun for this commit-only step.
- Existing replay/work/storage limits remain. Independent field/owner and temporal
  correlations can widen conservatively. Mutable reference carriers, exclusive/owned
  work and generated cleanup remain open. Conformance is 10 passed, 13 unsupported,
  0 failed in both profiles. Runtime/editor checks were not rerun; historical
  evidence at `f16c30b` remains in the compiler handoff.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded restart headers with terminal source expiry | Exclusive-reference initialization/reborrow model and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Design the first exclusive-reference and initialization slice in
   `compiler/OWNERSHIP.md`, `compiler/src/check/mutation.rs`, `compiler/src/borrow_value.rs`
   and `compiler/src/loans/`. Define move/reborrow state, parent suspension and scope
   exits before enabling source-level exclusive references or mutable reference
   carriers. Preserve first-collection conflicts, canonical slot identity, terminal
   expiry and source-guarded header transfers; prove acceptance and rejection in both
   profiles before relaxing B001.
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
