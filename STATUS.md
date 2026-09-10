# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Exported type aliases passed the compiler gate. `-><Name>:` publishes a type separately
from data/functions, and importers use `<module.Type>`. Facades preserve transparent
aliases, existing nominal types, record permissions and callable signatures.
Standalone documentation retains type roles and required-doc checks. See
[the type-export slice](compiler/MODULES.md#exported-type-aliases) and
[runnable geometry facade](compiler/examples/type-modules/main.mwy).

Four small implementation/test commits are complete, followed by a separate
documentation handoff. Literal imports still require top-level immutable
bindings. Broader import locations, package policy, runtime module captures and
borrowed exports remain separate.

Carried initialization, shared/exclusive scalar borrows, indexed writes and native
file-site diagnostics retain their boundaries. Net/HTTP/TLS still needs broader
module/generic-type/I/O/task foundations; full v0.0.1 remains incomplete.

## Actual validation

- Parser, namespace and exported-type identity suites passed, with formatting/Clippy
  checked per slice. Evidence covers privacy, duplicates, transparent/nominal types,
  callable aliases, mutable fields, references and the geometry facade.
- Four library and six native exported-type groups pass, including standalone docs
  generation and public-doc policy. Native examples run in debug/release.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1300 Rust
  tests, 20 Python tests, 79 standalone and three multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Exported type aliases passed the compiler gate; broader import locations remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

Commit workflow: plan dependency-ordered slices before implementation, commit each
validated slice, and apply the size review threshold in [AGENTS.md](AGENTS.md).

1. Plan bounded literal import discovery beyond top-level bindings, preserving
   canonical graph identity, initialization order, source diagnostics and scope gates.
2. Preserve ownership proofs; keep package policy, generic type evaluation, runtime
   module captures and borrowed exports separate. Keep STATUS concise, commit
   validated slices, never recreate STEP logs, and do not push.
