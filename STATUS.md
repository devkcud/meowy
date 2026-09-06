# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `228d808` resolves list candidates for an unlabeled element block with
  a context-independent prefix and a terminal pure emission suffix. It checks the
  prefix once in the ordinary block frame, uses the resulting bindings/reach to
  choose one type, then finishes that same frame. No application effect is replayed.
- Closed scalar/list/record results and immutable primitive constants can constrain
  the choice. Prefix locals retain ordinary defaults and shadowing. Probes use a
  minimal scratch checker; live guards, borrow IDs and effects are never copied.
- Proved ambiguity reports E207; unresolved cross-element, emitted-name or mutable/
  nonconstant constraints stay B001. Structural E203/E205/E206 trial errors
  are retained: another valid candidate survives, and a shared structural failure
  is reported only when all trials agree. Duplicate/forwarding regressions pass.
- Coverage/example: `1337785` adds five native groups and
  `compiler/examples/effectful-lists.mwy`. Two unit groups exercise prefix types,
  reach, source errors, required checks and the explicit inference boundaries.
- All 14 combined checks pass: 257 Rust tests, 35 Python tests, 856 local
  links, editors, schemas/catalog, formatting, Clippy, build and conformance.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- The optimized compiler runs the effectful-lists example with exact output.
  Independent effect/ownership reviews found no remaining blocker. Runtime
  debug/release/sanitizer checks pass unchanged; generated cleanup is still pending.
- No active workers, unfinished code or failing checks remain. General effectful
  constraints, mutable fields, exclusive references, owned/reference elements,
  module/library integration, cancellation and DWARF remain future work.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Nested list writes and bounded effectful context inference | Broader constraints, mutable fields, exclusive references and moves/cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend `compiler/src/list_context.rs` beyond immutable constant suffix leaves.
   Establish how concrete nonconstant prefix-local types can constrain candidates
   without transferring guard IDs or stale predicates. Keep unresolved cross-element
   constraints explicit; test once-only effects, shadowing, diagnostics and budgets.
2. Define mutable field shapes and exclusive-reference contracts before permitting
   field/reference write targets. Keep initialized Copy writes distinct from the
   move/drop state required by owned elements, slices and removal.
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
