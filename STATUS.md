# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Organization: `8c8e90a` separates backend lowering/tests, `360c8db` separates
  checker responsibilities, and `e3a0803` separates list-context orchestration,
  effectful blocks, isolated probes and tests. Entry files are now backend.rs 587,
  check.rs 184 and list_context.rs 212 lines. Existing public interfaces and
  function behavior are preserved. Both AGENTS files recommend cohesive modules
  without imposing a hard line-count limit.
- Compiler: `932297a` admits exact same-owner primitive local types in effectful
  suffix probes, including mutable and nonconstant values. Scratch values remain
  unknown; initializers, narrowed types and live guard/borrow IDs are never copied.
  Ordinary scalar deferral remains constant-only, preserving runtime read order.
- A conditional scratch failure can retain an uncertain candidate only inside a
  symbolic short-circuit RHS. Grouped conditions use their actual lowered span;
  ordinary live checking still decides a sole candidate. Runtime overflow checks,
  unconditional errors and candidate-dependent structural diagnostics are preserved.
- Coverage/example: `89b530c` adds six native groups and
  `compiler/examples/dynamic-lists.mwy`; three unit groups cover unknown types,
  guarded failures, scopes and explicit inference boundaries.
- All 14 combined checks pass: 266 Rust tests, 35 Python tests, 857 local
  links, editors, schemas/catalog, formatting, Clippy, build and conformance.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- Refactor proofs preserve all existing backend/checker/list-context test groups.
  The optimized compiler runs dynamic-lists with exact output. Runtime
  debug/release/sanitizer checks pass unchanged; generated cleanup remains pending.
- No active workers, unfinished code or failing checks remain. Remaining work
  includes aggregate/emitted-name/cross-element inference, mutable fields,
  exclusive references, owned elements, modules, cancellation and DWARF.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Modular backend/checker and runtime-valued suffix inference | Broader constraints, ownership and remaining source organization |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Continue organization where it improves the next change: remaining large
   `compiler/src/{borrow,loans,parser}.rs` and `compiler/tests/native.rs` are next.
   Extract ownership/grammar/test responsibilities with matching before/after checks;
   keep this a recommendation rather than a file-size gate.
2. Define mutable field shapes and exclusive-reference contracts before permitting
   field/reference write targets. Keep initialized Copy writes distinct from the
   move/drop state required by owned elements, slices and removal.
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
