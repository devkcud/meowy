# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Broader literal import discovery passed the compiler gate. Imports are found throughout
the parsed AST, including functions, inactive branches, interpolation and type
operands. Dependencies retain source order and initialize once before entry execution.
Function-local aliases may use exported functions/types while runtime module-data
captures remain gated. See [the module slice](compiler/MODULES.md) and
[runnable example](compiler/examples/scoped-imports/main.mwy).

Three small implementation/test commits are complete, followed by a separate
documentation handoff. Package policy, general type evaluation, borrowed module
storage and multi-file documentation remain separate.

Type/function exports, ownership and native source diagnostics retain their existing
rules. Net/HTTP/TLS still needs broader generic-type/I/O/task foundations; the full
v0.0.1 release remains incomplete.

## Actual validation

- Three graph discovery groups, seven scoped-import groups and a nested/inline
  initializer group passed. Native execution runs in debug/release where applicable.
- Evidence covers order, types/calls, captures, privacy, missing paths/cycles,
  inactive dependencies, budgets, initializer failure and source-output protection.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1311 Rust
  tests, 20 Python tests, 79 standalone and four multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Broader literal imports passed the compiler gate; runtime module-data captures remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

Commit workflow: plan dependency-ordered slices before implementation, commit each
validated slice, and apply the size review threshold in [AGENTS.md](AGENTS.md).

1. Plan compilation/checking of documentation-bearing module graphs, preserving
   per-file spans, attachment/signature/link validation and scope privacy.
2. Keep package policy, general type evaluation, runtime module-data captures and
   borrowed exports separate. Preserve ownership proofs, keep STATUS concise,
   commit validated slices, never recreate STEP logs, and do not push.
