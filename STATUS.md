# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Bounded relative file imports passed the compiler gate. Top-level immutable bindings
can import immutable reference-free value exports. Canonical paths/symlinks deduplicate
modules; isolated scopes and ordered dependency initialization preserve private
bindings and initialize diamonds once. Compiler diagnostics map back to the correct
file and local span. See [the slice](compiler/MODULES.md) and
[runnable example](compiler/examples/modules/main.mwy).

Function/type exports, package manifests/aliases, module references/captures and
multi-file documentation remain gated. Native panic messages still print graph byte
offsets; file labels and local runtime sites are the next diagnostic slice.

Carried list initialization, shared/exclusive scalar borrows and indexed writes
retain their validated ownership and reservation rules. Standalone documentation
tooling remains complete for its bounded slice. Net/HTTP/TLS remain specified
library work; module/type/I/O/task foundations precede executable adapters.

## Actual validation

- Eight graph/checker groups and nine native groups passed, including a multi-file
  diamond example. Native cases run in debug/release where applicable.
- Evidence covers paths/symlinks, snapshots/limits, initialization order/failure,
  private/value exports, compiler-error mapping, capability gates and input protection.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 1266 Rust tests, 20 Python tests, 79 standalone examples and
  one multi-file example in debug/release. The final hard-link protection case
  and Clippy also passed.
- Conformance: 10 passed, 13 unsupported, 0 failed. This is not full release
  qualification. Local-link, catalog/schema and whitespace checks passed.
- Editor and separate runtime/sanitizer gates were not rerun; their code is unchanged.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Relative value imports passed the compiler gate; richer exports and packages remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Map native panic sites to source files/local spans, preserving single-file output.
2. Extend annotated function/type exports with canonical identity and private scope
   preservation. Keep package policy, module captures and borrowed exports gated
   until their corresponding initialization/lifetime rules are implemented.
3. Preserve ownership proofs and keep STATUS concise; commit validated changes,
   never recreate STEP logs, and do not push.
