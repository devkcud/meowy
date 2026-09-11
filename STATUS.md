# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Immutable integer record fields can now supply computed-type inputs directly or
through copied integers. Flat record evidence preserves checked field identity,
exact widths and the whole initializer's errors/work; selecting a field cannot hide
an effect or invalid sibling. Eligible fields work in function type expressions while
runtime captures stay gated. See [the supported slice](compiler/COMPUTED_TYPES.md)
and [field-capacity example](compiler/examples/computed-types.mwy).

Five small implementation/test commits are complete, followed by this documentation
handoff. Nested records, references, imported data, helper purity, non-integer/mutable
scratch and full E220 accounting remain separate. Checking/building never executes
initializers; ordinary runtime record behavior remains unchanged.

Documented relative file graphs, ordered initialization, type/function exports and
native source diagnostics retain their existing rules. Net/HTTP/TLS still needs
broader generic-type/I/O/task foundations; full v0.0.1 remains incomplete.

## Actual validation

- Seven record evidence/projection and five new native groups passed, alongside
  initializer/record compatibility, source-mapping, budget and documentation checks.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1381 Rust
  tests, 20 Python tests, 80 standalone and five multi-file examples in debug/release.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local-link, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Flat immutable integer record fields pass; nested records and imported data remain gated. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan bounded nested immutable integer-record eligibility and checked field paths.
   Preserve complete ancestor initializer errors/effects/work and existing shape,
   mutation and reference restrictions. Keep imported data/helpers separate.
   Concrete files and validation are listed in the compiler handoff.
2. Preserve package, runtime-capture, borrowed-export and ownership gates. Plan and
   commit validated slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never
   recreate STEP logs, and do not push.
