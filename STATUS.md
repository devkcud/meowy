# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Native panic file labels passed the compiler gate. Multi-file P001/P002/P003/P006 failures
name their canonical source file and local byte range. Names are escaped and embedded
in the executable; copied panic evidence retains the actual failing site. One-file
output remains byte-identical. Runtime helpers, P006 mapping, checked-operation
mapping and driver integration are committed separately, followed by documentation.

Relative value imports retain canonical identities, private scopes and ordered
initialization. Function/type exports, package manifests/aliases, module references/
captures and multi-file documentation remain gated. See [the module slice](compiler/MODULES.md).

Carried initialization, shared/exclusive scalar borrows and indexed writes retain
their ownership and reservation rules. Standalone documentation remains complete
for its bounded slice. Net/HTTP/TLS implementation still needs module/type/I/O/task
foundations. The full v0.0.1 release remains incomplete.

## Actual validation

- Runtime probes, source-map tests and five CLI groups passed in debug/release.
  All 82 backend tests passed; focused formatting and Clippy passed.
- Evidence covers all four panic codes, source ranges, escaped names, nested failures,
  retained cleanup causes, operand order, aliases and unchanged one-file output.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1278 Rust
  tests, 20 Python tests, 79 standalone examples and one multi-file example.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Local-link,
  catalog/schema and whitespace checks passed; full release qualification remains open.
- The compiler's native runtime helpers were exercised. Separate prototype
  runtime/sanitizer and editor gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Native panic file labels passed the compiler gate; richer module exports remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

Commit workflow: plan dependency-ordered slices before implementation, commit each
validated slice, and apply the size review threshold in [AGENTS.md](AGENTS.md).

1. Plan separate annotated function/type export slices with canonical identity,
   public annotations, privacy and initialization-order tests.
2. Preserve ownership proofs; keep package policy, module captures and borrowed
   exports gated until their corresponding rules are implemented. Keep STATUS
   concise, commit validated slices, never recreate STEP logs, and do not push.
