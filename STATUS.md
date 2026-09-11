# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Bounded computed-type blocks passed the compiler gate. They construct supported
types using local immutable type bindings, aliases and primary emissions, with
isolated scopes and shared bootstrap work limits. No runtime storage or code is
created for the block itself. Exports, documentation and ordinary ownership checking
reuse the constructed type. See [the supported slice](compiler/COMPUTED_TYPES.md)
and [runnable example](compiler/examples/computed-types.mwy).

Four small implementation/test commits are complete, followed by this documentation
handoff. Scalar/mutable scratch, helper calls, control flow, full required evaluation
and language E220 accounting remain separate. Known debug effects reject with E219.

Documented relative file graphs, ordered initialization, type/function exports and
native source diagnostics retain their existing rules. Net/HTTP/TLS still needs
broader generic-type/I/O/task foundations; full v0.0.1 remains incomplete.

## Actual validation

- Nine focused library/parser and five native groups passed, including scope/type
  identity, work limits, effect diagnostics, exports and exact documentation signatures.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1340 Rust
  tests, 20 Python tests, 80 standalone and five multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Bounded computed-type blocks pass; full required evaluation and specialization remain open. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan immutable scalar scratch in required type blocks so a local checked capacity
   can construct a list type. Reuse scalar/extent checking, preserve width/overflow
   and runtime-input/effect boundaries, and keep helper calls/full E220 separate.
   Concrete files and validation are listed in the compiler handoff.
2. Preserve package, runtime-capture, borrowed-export and ownership gates. Plan and
   commit validated slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never
   recreate STEP logs, and do not push.
