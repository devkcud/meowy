# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Straight-line integer block initializers can now supply computed-type inputs. Eligible
blocks use immutable integer bindings and one primary emission, including nested
blocks. Checked values survive scope exit; statements after emission still contribute
errors and work. Required reads do not change runtime initialization or capture rules.
See [the supported slice](compiler/COMPUTED_TYPES.md) and
[block-seed example](compiler/examples/computed-types.mwy).

Three small implementation/test commits are complete, followed by this documentation
handoff. Fields/imported data, helper purity, mutable/non-integer scratch, control flow
and language E220 accounting remain separate. Effects and mutable dependencies never
become eligible merely because their results could be folded.

Documented relative file graphs, ordered initialization, type/function exports and
native source diagnostics retain their existing rules. Net/HTTP/TLS still needs
broader generic-type/I/O/task foundations; full v0.0.1 remains incomplete.

## Actual validation

- Value/provenance and block compatibility checks passed, plus three focused block
  library and four new native groups covering values, effects, source spans and budgets.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1367 Rust
  tests, 20 Python tests, 80 standalone and five multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Straight-line integer block eligibility passes; field/helper/imported-data forms remain gated. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan immutable integer record-field projection eligibility, preserving the whole
   initializer's effects, concrete field identity, mutability and diagnostic/work
   provenance. Keep imported data and helpers separate initially.
   Concrete files and validation are listed in the compiler handoff.
2. Preserve package, runtime-capture, borrowed-export and ownership gates. Plan and
   commit validated slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never
   recreate STEP logs, and do not push.
