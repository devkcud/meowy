# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `b7ddf0c` adds inline bounded lists with typed/inferred literals,
  initialized length, one-based copy indexing, `.size()`, value-returning `.add()`,
  equality and whole-value replacement. Elements are copyable and reference-free;
  whole-list borrows retain existing E302/E303 rules.
- Native coverage/example: `abd1774` adds nine native groups and
  `compiler/examples/bounded-lists.mwy`. Typed extent arithmetic preserves overflow
  checks; runtime P001/P003 includes operands and source byte spans.
- Runtime/tooling: `4feecf8` adds bounded batches of every consumed child failure.
  Full buffers leave the next failed child pending; retries preserve cumulative
  counts and ticket identities. Summary-only closing remains available.
- All 14 combined checks pass: 187 Rust tests, 34 Python tests, 849 local links,
  schemas/catalog, editors, formatting/Clippy/build and conformance. Each runtime
  debug/release/sanitized profile passes 14 cleanup, 10 stack, 10 context,
  25 scheduler and 13 owned-value cases with exact fatal/guard/admission probes.
  ASan/UBSan/LSan pass, including the required expired-fiber-local diagnosis.
- The optimized compiler builds and executes the bounded-lists example with exact
  stdout. Conformance is 10 passed, 13 unsupported, 0 failed in debug/release;
  `mixed_list` now passes with its assigned E207 rejection.
- No active workers, incomplete implementation files or failing checks remain.
  Generated programs still use the scalar runtime; automatic scope-exit joins,
  cancellation and DWARF are not implemented. The earlier isolated ASan probe
  timeout did not recur in the full outside-sandbox gates; no probe was suppressed.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Copyable bounded lists and scoped shared borrows | List union-context inference, element places, exclusive access, moves and cleanup |
| Runtime | Explicit marked scope close with bounded failure batches | Generated scope-exit code, owned diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend `compiler/src/list.rs` with single-evaluation inference across multiple
   expected list alternatives. Keep literal widths, exact capacities and explicit
   B001 boundaries; verify unique candidates and genuine ambiguity separately.
2. Add initialized element places, exclusive loans and move/drop state before
   list element mutation, slices or owned elements. Extend static/intrinsic borrow
   sources without weakening all-input lifetime bounds or resource budgets.
3. Generate payload layouts, move/drop operations and marked task scope closing
   while parent locals remain live. Drain every failure batch before reusing its
   buffer; add owned diagnostic storage, cancellation and pinned unwind support.
   Preserve interleaved child/result/local cleanup. Native close currently waits.
4. Build the manifest/module graph needed for real Meowy library sources and the
   documented projects. Keep runtime, editor and library progress visible here.
5. Use `python3 -B tools/verify.py --all` after integrations. LSan needs an environment
   where it can inspect processes. `--strict` still fails for 13 unsupported catalog
   cases; neither the bootstrap gate nor this host qualifies a complete release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
