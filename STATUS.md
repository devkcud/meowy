# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current handoff

- Compiler: `3acfc97` adds verified direct-function shared-borrow contracts,
  including inferred results, recursion and record/union carriers. Actual pointer
  origins are separate from the conservative all-input lifetime bounds. Ignored
  and transitive inputs still protect returned views; violations report E302/E303.
- Native coverage/example: `cb04d32`; `compiler/examples/borrow-functions.mwy`
  demonstrates returned views, scalar-only projections and final-use writes.
- Runtime/tooling: `7b9dc05` adds parent-owned child admission and waiting joins.
  Parents yield their worker while retaining their stack. Child release failures
  retain ownership for retry; missing explicit joins cause a private fatal error.
- All 14 combined checks pass: 154 Rust tests, 33 Python tests, 845 local links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Each runtime
  debug/release/sanitized profile passes 14 cleanup, 10 stack, 10 context and
  18 scheduler cases with exact fatal/guard/admission/unjoined-child probes.
  ASan/UBSan/LSan pass, including the required expired-fiber-local ASan diagnosis.
- The optimized compiler/example passes with exact stdout. Its independent function
  oracle passes all 384 cases: 216 accepted, 168 E302, no unexpected results.
  Twelve directed semantic probes and ten additional native runs also pass.
- No active workers, incomplete code or failing checks remain. Conformance is
  9 passed, 14 unsupported, 0 failed. Generated programs still use the scalar
  runtime; automatic scope-exit joins, cancellation and DWARF are not implemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Shared loans through local records/unions and direct function contracts | Reborrows, exclusive access, moves and generated cleanup |
| Runtime | Fixed-slot parent/child ownership and explicit worker-yielding joins | Owned captures/results, automatic scope-exit joins, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend compiler origins and contract substitution for reborrows, static references
   and verified intrinsic sources before enabling those capabilities. Preserve
   all-input bounds, E303 escape checks and E302 caller conflicts.
2. Add explicit reads, moves, initialized-slot tracking and cleanup edges before
   exclusive loans, reference reassignment or owned collections. Improve predicate
   and loop precision without weakening proof/resource bounds.
3. Add owned capture/result storage and compiler-generated scope-exit joins to the
   parent/child scheduler while borrowed locals are still alive. Add cancellation
   and pinned unwind support, preserving child completion before parent cleanup
   and interleaved partial-result cleanup order. Do not join after C++ locals expire.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 -B tools/verify.py --all` after integrations. LSan needs an environment
   where it can inspect processes. `--strict` still fails for 14 unsupported catalog
   cases; neither the bootstrap gate nor this host qualifies a complete release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
