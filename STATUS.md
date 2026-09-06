# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `75785b7` adds immutable reference unions, optional fields, guarded
  extraction and member-preserving retagging. Active variants control E302 loans
  and E303 escapes. Record constructors preserve contextual widths and nullable
  defaults; union equality still requires the same normalized union type.
- Native coverage/example: `4ef3268`; `compiler/examples/optional-borrows.mwy`
  demonstrates nullable fields and unions of different reference types.
- Runtime/tooling: `d3e41aa` adds a bounded single-worker scheduler over guarded
  contexts. Admission is fallible, pumping uses round-robin selection, cleanup may
  suspend, and slots remain occupied until a settled task is explicitly joined.
- All 14 combined checks pass: 136 Rust tests, 32 Python tests, 844 local links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Each runtime
  debug/release/sanitized profile passes 14 cleanup, 10 stack, 10 context and
  11 scheduler cases with exact fatal/guard/admission probes. ASan/UBSan/LSan pass;
  the expired-fiber-local probe produces the required ASan diagnostic.
- The optimized compiler/example passes with exact stdout. Its independent union
  oracle passes all 588 cases: 356 accepted, 232 E302, no unexpected results.
  Ten directed semantic probes and eight additional native runs also pass.
- No active workers, incomplete code or failing checks remain. Conformance is
  9 passed, 14 unsupported, 0 failed. Generated programs still use the scalar
  runtime; structured child cancellation/join and DWARF are not implemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded shared loans through immutable records and unions | Function borrow contracts, exclusive access, moves and cleanup |
| Runtime | Bounded single-worker scheduling, cleanup and settled-only join | Structured child joins, waiting, capture/result ownership and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add verified function/call borrow contracts in compiler origin and loan analysis.
   Preserve component/variant identity and the documented all-input lifetime bound;
   verify accepted returned views, E303 escapes and E302 writes at callers.
2. Add explicit read, reborrow and cleanup edges before exclusive loans or reference
   reassignment. Improve predicate-assignment and loop precision while preserving
   resource bounds and explicit B001 for missing proofs.
3. Extend `runtime/` scheduling with structured child admission, waiting joins and
   capture/result ownership. Add cooperative cancellation before parent storage
   release, then pinned unwind support, Meowy personality and landing pads. Verify
   cleanup that waits for children and interleaved partial-result cleanup order.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 -B tools/verify.py --all` after integrations. LSan needs an environment
   where it can inspect processes. `--strict` still fails for 14 unsupported catalog
   cases; neither the bootstrap gate nor this host qualifies a complete release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
