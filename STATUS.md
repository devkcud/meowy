# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Exported types are in progress as separate syntax, namespace, re-export identity,
integration and documentation commits. Aliases retain their existing type identity;
the compiler handoff records the commit plan.

Annotated module functions and explicitly typed re-exports passed the compiler gate.
Imports and facades retain canonical function IDs, private helper boundaries and
ordered initialization. Calls preserve recursion, shared/exclusive permissions,
returned-reference bounds and callee panic locations. Data/function namespace
collisions are checked, including record spreads. See
[the module slice](compiler/MODULES.md#annotated-function-exports) and
[runnable facade](compiler/examples/function-modules/main.mwy).

Five small implementation/regression commits are complete, followed by a separate
documentation handoff. Type exports are the next slice. Runtime module
data captures, borrowed exports and package/manifest support remain gated.

Carried initialization, shared/exclusive scalar borrows, indexed writes and native
file-site diagnostics retain their existing boundaries. Standalone documentation
is unchanged. Net/HTTP/TLS still needs module/type/I/O/task foundations; the full
v0.0.1 release remains incomplete.

## Actual validation

- Existing function/checker tests and focused export groups passed. Evidence covers
  public signatures, identity, private names, scope, collisions and import cycles.
- Four cross-module call groups passed in debug/release, including the facade,
  recursion, all-input bounds, exclusive calls, storage escape and callee panic sites.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1288 Rust
  tests, 20 Python tests, 79 standalone and two multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Annotated function exports passed the compiler gate; exported types are next. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

Commit workflow: plan dependency-ordered slices before implementation, commit each
validated slice, and apply the size review threshold in [AGENTS.md](AGENTS.md).

1. Plan exported-type syntax and namespace slices with private alias and canonical
   identity tests; keep package policy separate.
2. Preserve ownership proofs and keep runtime module captures/borrowed exports
   gated until their rules are implemented. Keep STATUS concise, commit validated
   slices, never recreate STEP logs, and do not push.
