# Meowy project status

Updated: 2026-09-12. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Eligible immutable named file exports now supply computed-type inputs through imported
fields, copied integers, subrecord aliases and re-exports. Reads preserve exact integer
widths and complete ancestor evidence. Checking/building never executes initializers;
runtime keeps dependency order and initializes shared modules once.

Three implementation/test commits are complete, followed by this documentation
handoff. Primary/composed/conditional export inputs, helper purity, non-integer/mutable
scratch and full E220 accounting remain separate. Ordinary runtime captures and
borrowed-export gates remain in force. See [the supported slice](compiler/COMPUTED_TYPES.md).

Net/HTTP/TLS still needs broader generic-type/I/O/task foundations. Full v0.0.1
release qualification remains incomplete.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including 1401 Rust
  tests, 20 Python tests, and existing examples in debug/release.
- Native integration verified initialization order, silent check/build, source
  diagnostics, ancestor effects and repeated-read work limits.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local links, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Named immutable imported inputs pass; primary exports and helper purity remain gated. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan primary immutable integer file-export eligibility using the concrete files
   and validation in the compiler handoff. Preserve checked initializer provenance,
   exact widths, work charging, initialization and runtime capture gates.
2. Preserve package, borrowed-export and ownership gates. Plan and commit validated
   slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never recreate STEP logs,
   and do not push.
