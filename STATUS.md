# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `745ca2f` adds checked shared element borrows such as `&values[index]`.
  References address original initialized list storage, compose nested list/record
  paths and return through functions while retaining every borrowed-input bound.
  Copied parameter/self storage cannot escape; returning index effects keep the
  parent loan live, including when the resulting reference is discarded.
- Physical addresses use actual indices. Abstract Field/Element lifetime paths
  conservatively overlap indices within the same list region and do not enumerate
  capacity. Bounds remain E101/P001; element writes and exclusive borrows stay B001.
- Derived-reference proof gaps are checked after CFG reachability is known. Dead
  branches need no invented origins; any reachable gap still reports B001. This
  fixes two false rejections found in literal-false/short-circuited expressions.
- Coverage/example: `5c9defb` adds seven native groups and
  `compiler/examples/element-borrows.mwy`. Library coverage adds three backend,
  one list, one contract and two loan groups, including path-budget checks.
- All 14 checks pass: 226 Rust tests, 35 Python tests, 853 local links, editors,
  schemas/catalog, formatting, Clippy, build and conformance. Runtime sanitizer,
  message/lifetime, fatal and guard checks pass unchanged. Conformance remains
  10 passed, 13 unsupported, 0 failed in both profiles.
- The optimized compiler passed the exact element-borrows example, an active E302
  conflict and a zero-capacity P001 check preserving index effects. Independent
  ownership review/retests found no remaining blocker or unsafe acceptance.
- No active implementation workers, unfinished code or failing checks remain.
  Runtime snapshots remain `d92f94c`; generated panic evidence remains `eb65cbd`.
  Indexed mutation, exclusive references, slices, owned/reference elements,
  generated task cleanup, cancellation and DWARF remain future work.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Checked shared list-element borrows and typed failure evidence | Exclusive element access, moves/cleanup and remaining contextual constraints |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend checked element places with exclusive access and initialized/move state
   before indexed writes, slices or owned elements. Define target/index/RHS ordering,
   preserve parent storage while a place is in use, and test overlap/last-use cases.
   Current abstract Element paths do not prove different indices disjoint.
2. Extend remaining effectful/non-scalar contextual constraints without replaying
   effects or weakening budgets. Keep annotations/B001 for unproved candidates.
3. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before cancellation and unwinding.
4. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
5. Build the manifest/module graph for Meowy libraries and documented projects.
   Keep runtime, editor and library work visible here.
6. Run `python3 -B tools/verify.py --all` after integrations. LSan needs process
   inspection. Strict conformance still has 13 unsupported cases; the bootstrap
   gate and this host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
