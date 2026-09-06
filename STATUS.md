# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `393471c` adds scope-local borrowing of parameter/self copies,
  shared/reference-carrier dispatch and reference-prefix field reborrows.
  Copied-storage addresses cannot escape; shared receivers keep original
  owners and inherited lifetime bounds. Evaluation remains once-only.
- Native coverage/example: `efe7d6e`; `compiler/examples/scope-borrows.mwy`
  contrasts local copies with shared receiver identity and final-use access.
- Runtime/tooling: `d7d1758` adds explicit marked scope closing. It waits for
  children, releases owned results and reports failures while retaining progress
  across release retries. Marks use 16 fixed records per task and must close
  before the parent's C++ locals disappear.
- All 14 combined checks pass: 172 Rust tests, 34 Python tests, 848 local links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Each runtime
  debug/release/sanitized profile passes 14 cleanup, 10 stack, 10 context,
  22 scheduler and 13 owned-value cases with exact fatal/guard/admission probes.
  ASan/UBSan/LSan pass, including the required expired-fiber-local diagnosis.
- The optimized compiler/example passes with exact stdout. Its dispatch matrix
  passes all 108 cases: 66 accepted, 42 E302, no unexpected results. Nineteen
  directed checks and ten additional native profile runs also pass.
- No active workers, incomplete code or failing checks remain. Conformance is
  9 passed, 14 unsupported, 0 failed. Generated programs still use the scalar
  runtime; automatic scope-exit joins, cancellation and DWARF are not implemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scoped local/parameter/receiver borrows and reference-prefix reborrows | Exclusive access, static/intrinsic sources, moves and generated cleanup |
| Runtime | Explicit marked scope close, child waits and owned transfers | Generated scope-exit code, complete diagnostic propagation, cancellation and DWARF |
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
3. Generate payload layouts, move/drop operations and task scope-close calls while
   parent locals still live. Consume failure reports according to the language
   contract, add cancellation and pinned unwind support, and preserve interleaved
   child/result/local cleanup. Native close currently waits without cancellation.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 -B tools/verify.py --all` after integrations. LSan needs an environment
   where it can inspect processes. `--strict` still fails for 14 unsupported catalog
   cases; neither the bootstrap gate nor this host qualifies a complete release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
