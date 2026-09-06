# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `1817ea4` adds expected-list union inference using capacity, literal
  range, typed elements and fresh list/record shapes. Pure contextual literals may
  wait for typed values; effectful expressions are checked once. Genuine ambiguity
  and no fit report E207; all capacities being too small reports E103.
- Unary typing: `c6bf80a` applies `!`, `-` and `~` before expected-union injection,
  preserving numeric widths, grouped literal bounds and ordinary assignment.
- Native coverage/example: `50283a5` adds six native groups and
  `compiler/examples/list-unions.mwy`. Type, field lookup and shape work is charged;
  limits remain explicit B001 instead of guessing a contextual type.
- Runtime: `541747f` documents constructor-time owning diagnostic snapshots and
  their storage/retry/P008 checks. This was a read-only design investigation;
  runtime messages still borrow storage. Existing batch behavior is `4feecf8`.
- All 14 checks pass: 197 Rust tests, 34 Python tests, 851 local links, schemas,
  catalog, editors, formatting/Clippy/build and conformance. Runtime debug/release/
  sanitized profiles pass 14 cleanup, 10 stack, 10 context, 25 scheduler and
  13 owned-value groups, with fatal/guard/admission and expired-fiber-local probes.
- The optimized compiler runs the list-unions example with exact stdout.
  The semantic audit passed 1,096 comparisons before final resource-accounting
  hardening; the final gate and large-record budget test pass after that hardening.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- No unfinished implementation or active workers remain. The unrelated untracked
  `compiler/examples/meow.mwy` appeared during work and was left untouched, outside
  these commits and validation. Generated programs still use the scalar runtime;
  automatic scope-exit joins, cancellation and DWARF remain unimplemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Bounded list candidate inference and scoped shared borrows | Remaining contextual constraints, element places, moves and cleanup |
| Runtime | Failure batches; owning-diagnostic design documented | Owning message storage, generated scope exits, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend `compiler/src/list_context.rs` for the remaining context-dependent compound
   elements and nested constraints. Preserve once-only checking, source/type work
   limits and original diagnostics; never infer ambiguity from an unproved case.
2. Implement the owning diagnostic value described in `runtime/README.md`. Capture
   text before its callback storage expires, choose visible capacity/overflow rules,
   and measure the cost across outcomes, 16 scope records and caller report buffers.
   Verify slot reuse, retries, callback-local overwrite and both P008 causes.
3. Add initialized element places, exclusive loans and move/drop state before list
   mutation, slices or owned elements. Extend static/intrinsic borrow sources
   without weakening all-input lifetime bounds or resource budgets.
4. Generate payload layouts and marked task scope closing while parent locals live.
   Drain every failure batch; preserve interleaved child/result/local cleanup before
   adding cancellation and pinned unwind support. Native close currently waits.
5. Build the manifest/module graph for real Meowy library sources and documented
   projects. Keep runtime, editor and library progress visible here.
6. Run `python3 -B tools/verify.py --all` after integrations. LSan needs process
   inspection. `--strict` still fails for 13 unsupported catalog cases; this host
   and the bootstrap gate do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
