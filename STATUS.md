# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `faaa08b` gives immutable reference-free emitted names actual result-slot
  aliases and shared borrows. SlotAlias and private metadata retain declared
  mutability; backend backing must match that flag and accept the lexical type.
- Immutable aliases retain constant, variant and initialized-length facts. Their
  roots reject direct, field and element writes with E305, including mutable fields
  inside an immutable value. Mutable aliases retain unknown activity and last-use
  conflict checks; reference-bearing emitted names keep their prior copied origins.
- Existing Source::Slot ownership covers nested names, outer target lifetimes,
  discarded cells, restart and E303 publication. Exact cells or concrete union
  members can be borrowed; narrower subunion reads still convert while their
  borrowed views remain B001. Initializers execute once and ordinary copies remain
  independent.
- `496b529` adds eight native groups, `compiler/examples/immutable-slots.mwy`
  and README evidence. Three semantic and five backend groups check facts,
  initialized lengths, write rejection, actual pointers, widening and discarded
  opposite-mutability backing. Runtime ABI and dependencies are unchanged.
- All 14 combined checks pass: 342 Rust tests, 35 Python tests, 862 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs immutable-slots with exact output. Twelve independent
  fact/lifetime/regression cases and source review pass. No unfinished source work,
  active workers or failing checks remain; modular organization is preserved.
- Reference-bearing emitted-slot addresses, mutable reference carriers, exclusive
  references, owned cleanup and full release qualification remain open. Next is
  immutable reference-bearing slot identity with explicit contained-origin proofs.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Immutable and mutable reference-free emitted-storage borrows | Reference-bearing slots, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Model immutable reference-bearing named emissions as slots while preserving each
   copied component's actual origins, input bounds and active variant facts. Keep
   storage borrowing B001 until physical owner and contained-reference dependencies
   are both represented; verify projected reads, calls, discard/restart and escapes.
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
