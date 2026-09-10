# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Documentation-bearing relative file graphs passed the compiler gate. Ordinary
check/build/run validate each file's attachments, signatures, parameter/module links
and public/private boundaries. Imported links resolve exported functions, data and
types through facades and aliases. Diagnostics retain the owning file/local spans.
See [the supported module slice](compiler/MODULES.md) and
[documented facade example](compiler/examples/documented-modules/main.mwy).

Five small implementation/test commits are complete, followed by this documentation
handoff. Multi-file doc commands, site generation, public indexes/coverage and example
graph resolution remain separate. Ordinary compilation does not execute examples.

Broader AST import discovery, ordered initialization, type/function exports, ownership
and native source diagnostics retain their existing rules. Net/HTTP/TLS still needs
broader generic-type/I/O/task foundations; full v0.0.1 remains incomplete.

## Actual validation

- Documentation source/model, graph scope/privacy/link and six native boundary groups
  passed. The documented facade runs in debug/release with one dependency initialization.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1327 Rust
  tests, 20 Python tests, 79 standalone and five multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Documented relative file graphs pass; runtime module-data captures remain gated. |
| Documentation tooling | Standalone sites/examples complete; graph compile checking works; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Return to compiler type construction: audit `compiler/src/check/names.rs::type_value`
   against the required-evaluation contract, then plan the smallest bounded extension
   with accepted/effect/budget rejection fixtures. Details are in the compiler handoff.
2. Preserve package, runtime-capture, borrowed-export and ownership gates. Plan and
   commit validated slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never
   recreate STEP logs, and do not push.
