# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Bounded nested immutable records now supply computed-type inputs through checked
field paths and projected subrecord aliases. Every read retains the complete original
ancestor initializer's errors and work, including unselected siblings. The shape limit
is 256 total fields and 32 record levels. See
[the supported slice](compiler/COMPUTED_TYPES.md) and
[nested field example](compiler/examples/computed-types.mwy).

Four small implementation/test commits are complete, followed by this documentation
handoff. Imported data, references, helper purity, non-integer/mutable scratch and full
E220 accounting remain separate. Checking/building never executes initializers;
ordinary runtime storage, captures and initialization order remain unchanged.

Documented relative file graphs, type/function exports and native source diagnostics
retain their existing rules. Net/HTTP/TLS still needs broader generic-type/I/O/task
foundations; full v0.0.1 remains incomplete.

## Actual validation

- Nested construction/path checks and four new native groups passed, alongside
  initializer/record compatibility, source-mapping, budget and documentation checks.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1392 Rust
  tests, 20 Python tests, 80 standalone and five multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Nested immutable record paths pass; imported data and helper purity remain gated. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan explicit eligibility metadata for exported immutable integer/record data.
   Preserve declaration/ancestor provenance, initialization, privacy and file spans;
   keep synthetic module bindings, runtime captures and helper purity separate.
   Concrete files and validation are listed in the compiler handoff.
2. Preserve package, runtime-capture, borrowed-export and ownership gates. Plan and
   commit validated slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never
   recreate STEP logs, and do not push.
