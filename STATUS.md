# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Immutable integer calculations inside computed-type blocks passed the compiler gate.
Local bindings retain exact widths and signedness, and checked arithmetic can compute
list capacities without creating runtime storage. Documentation and exported types
reuse the results. See [the supported slice](compiler/COMPUTED_TYPES.md) and
[updated capacity example](compiler/examples/computed-types.mwy).

Three small implementation/test commits are complete, followed by this documentation
handoff. Runtime initializer eligibility, mutable/non-integer scratch, helper calls,
control flow and language E220 accounting remain separate. Runtime inputs and known
debug effects are rejected without executing initializers.

Documented relative file graphs, ordered initialization, type/function exports and
native source diagnostics retain their existing rules. Net/HTTP/TLS still needs
broader generic-type/I/O/task foundations; full v0.0.1 remains incomplete.

## Actual validation

- Fourteen focused library/parser and nine matching native groups passed, including
  widths/overflow, work limits, static-input boundaries, exports and documentation.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1348 Rust
  tests, 20 Python tests, 80 standalone and five multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Integer scratch in computed-type blocks passes; runtime initializer eligibility remains gated. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan explicit eligibility evidence for immutable integer runtime bindings before
   allowing them as required type inputs. Reuse existing constant/purity analysis;
   distinguish pure initializers from effects, parameters and mutable state.
   Concrete files and validation are listed in the compiler handoff.
2. Preserve package, runtime-capture, borrowed-export and ownership gates. Plan and
   commit validated slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never
   recreate STEP logs, and do not push.
