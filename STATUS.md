# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `c325099` extends statement-owned Copy temporaries to references,
  records and unions carrying references. The real initializer state is preserved
  beneath Deref, retaining origins, bounds and active variants independently of the
  temporary cell's StatementId. Existing HIR, contracts and lifetime rules are reused.
- Materialization reads directly stored reference values; deeper pointee summaries
  remain conditional on later demand. Ordinary borrows and temporary materialization
  share a bounded loan helper with different direct-use inputs. Initializers still
  evaluate once, including computed whole-carrier field projections.
- A direct dereference copies contents while the cell lives. The copy can outlive
  that cell if its original pointees/bounds survive; retained temporary-cell addresses
  still expire. Public call bounds remain attached after dereference. Tag-only
  inspection and full entered-call validation retain their existing distinctions.
- `fb72c97` adds eight native groups, `compiler/examples/reference-temporaries.mwy`
  and README evidence. Three origin, two loan and four backend groups cover cell
  identity, direct copies, nullable activity, eager/deferred reads, E302/E303,
  call bounds and leave/restart/panic. No HIR, ABI or dependency expansion was needed.
- All 14 combined checks pass: 419 Rust tests, 35 Python tests, 866 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs reference-temporaries with exact output. Nine
  independent checks and six profile executions pass; no unfinished source work,
  active workers or failing checks remain. Temporary/summary/depth budgets remain.
- Mutable reference bindings/carriers, reference-bearing lists, exclusive references,
  owned cleanup and full release qualification remain open. Next is ordinary local
  mutable shared-reference bindings with explicit value versions and cell-loan checks.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Reference-bearing Copy temporaries and bounded shared borrowing | Mutable shared-reference locals, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Model ordinary mutable shared-reference locals with a fixed &T type and explicit
   origin versions. Reassignment must change future reads while existing copies keep
   their old pointees/bounds; a live borrow of the reference cell must block writes.
   Verify branch joins and restart flow before enabling them; keep unproved flows B001.
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
