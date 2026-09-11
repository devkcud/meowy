# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Proven immutable integer initializers can now supply computed-type inputs. Eligibility
tracks checked literals, aliases and arithmetic separately from constant folding,
retains original failure spans, and charges dependency work on every required read.
Eligible lexical reads work inside functions while runtime captures remain gated.
See [the supported slice](compiler/COMPUTED_TYPES.md) and
[eligible-seed example](compiler/examples/computed-types.mwy).

Three small implementation/test commits are complete, followed by this documentation
handoff. Block/field/imported-data initializers, helper purity, mutable/non-integer
scratch, control flow and language E220 accounting remain separate. Checking/building
does not execute initializers; ordinary execution preserves application effects.

Documented relative file graphs, ordered initialization, type/function exports and
native source diagnostics retain their existing rules. Net/HTTP/TLS still needs
broader generic-type/I/O/task foundations; full v0.0.1 remains incomplete.

## Actual validation

- Six initializer library and four new native groups passed, alongside computed-type
  compatibility checks. Coverage includes provenance, budgets and initialization boundaries.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1358 Rust
  tests, 20 Python tests, 80 standalone and five multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Immutable integer initializer eligibility passes; block/helper/imported-data forms remain gated. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan eligibility for straight-line integer block initializers with local eligible
   bindings and one primary emission. Preserve effect/mutability restrictions,
   original diagnostics and dependency work; keep helpers/imported data separate.
   Concrete files and validation are listed in the compiler handoff.
2. Preserve package, runtime-capture, borrowed-export and ownership gates. Plan and
   commit validated slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never
   recreate STEP logs, and do not push.
