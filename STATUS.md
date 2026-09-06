# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `406c817` adds shared reborrows and concrete field references through
  shared inputs. Parent expressions evaluate once and point into original storage.
  Function contracts include compatible referent fields and preserve inherited
  all-input lifetime bounds; conflicts and escapes remain E302/E303.
- Native coverage/example: `21086be`; `compiler/examples/reborrows.mwy` verifies
  original field addresses, function-returned views and final-use writes.
- Runtime/tooling: `979c8e8` adds explicit owned capture/result storage over fixed
  caller buffers, actual relocation and exactly-once release. Accepted admission
  failures drop captures; failed joins preserve owned results for retry.
- All 14 combined checks pass: 163 Rust tests, 33 Python tests, 847 local links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Each runtime
  debug/release/sanitized profile passes 14 cleanup, 10 stack, 10 context,
  18 scheduler and 12 owned-value cases with exact fatal/guard/admission probes.
  ASan/UBSan/LSan pass, including the required expired-fiber-local diagnosis.
- The optimized compiler/example passes with exact stdout. Its reborrow guard
  oracle passes all 150 cases: 114 accepted, 36 E302, no unexpected results.
- No active workers, incomplete code or failing checks remain. Conformance is
  9 passed, 14 unsupported, 0 failed. Generated programs still use the scalar
  runtime; automatic scope-exit joins, cancellation and DWARF are not implemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Shared loans, concrete reborrows and direct function contracts | Exclusive access, static/intrinsic sources, moves and generated cleanup |
| Runtime | Fixed-slot child waits and explicit owned capture/result transfers | Generated payload layouts, scope-exit joins, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend origins and function contracts for static references, verified intrinsic
   sources and additional addressable projections before enabling them. Preserve
   all-input bounds and avoid false no-return assumptions for newly valid sources.
2. Add explicit reads, moves, initialized-slot tracking and cleanup edges before
   exclusive loans, reference reassignment or owned collections. Improve predicate
   and loop precision without weakening proof/resource bounds.
3. Generate payload layouts and explicit move/drop operations for runtime owned
   values, then scope-exit joins while borrowed locals still live. Add cancellation
   and pinned unwind support, preserving child completion before parent cleanup
   and interleaved partial-result order. Do not join after C++ locals expire.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 -B tools/verify.py --all` after integrations. LSan needs an environment
   where it can inspect processes. `--strict` still fails for 14 unsupported catalog
   cases; neither the bootstrap gate nor this host qualifies a complete release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
