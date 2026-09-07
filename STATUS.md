# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `6ecda19` enables whole-carrier and reference-cell shared borrows with
  bounded transitive pointee summaries. Flat Deref paths distinguish each borrowed
  cell from the references and variant activity reachable through it.
- Direct dereference copies may outlive their outer cell while contained pointees
  remain alive. Nested field/reborrow operations load stored pointers once. Mutable
  reference-free pointee reads keep fresh unknown activity; immutable carrier
  snapshots preserve nullable and variant facts.
- Conditional loan transfers carry nested dependencies backward through borrows,
  copies, bindings, emissions and block returns only when later contents are used.
  Actual pointer/value reads stay eager. Whole copies use every contained reference;
  selected fields and pointer/tag inspection avoid unrelated pointee loans.
- Recursive function contracts select guarded input snapshots and attach all active
  input bounds at every returned reference layer. Public call bounds survive later
  dereferences. Reference construction is capped at 64 layers; summary parts,
  candidate fanout, paths and transfer work retain the existing shared budgets.
- `fda28b9` adds eight native groups, `compiler/examples/transitive-borrows.mwy`
  and README evidence. Thirteen new library groups cover origins, transfers and
  call contracts. New focused modules keep state, contract and loan work separate.
- All 14 combined checks pass: 381 Rust tests, 35 Python tests, 864 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs transitive-borrows with exact output. Twelve
  independent cases and source review pass; no unfinished source work, active
  workers or failing checks remain. Runtime ABI and dependencies are unchanged.
- Shared temporary-owner borrows, exclusive references, mutable reference carriers,
  reference-bearing lists, owned cleanup and full release qualification remain open.
  Next is statement-scoped storage for shared borrows of reference-free Copy temporaries.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Bounded nested references and whole-carrier shared borrows | Copy temporary lifetimes, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Materialize reference-free Copy temporary owners with explicit statement lifetime
   and storage identities. Preserve once-only evaluation through calls, dispatch,
   reborrows and matcher condition/body boundaries. Verify same-statement acceptance,
   later-use E303, last-use E302 and leave/restart/panic paths before enabling them.
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
