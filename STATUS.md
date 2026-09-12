# meowy project status

Updated: 2026-09-12. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Active documentation work

Standardize the project name as `meowy` and use the existing spaced gatostyle
baseline for documentation. Preserve tokens, literal/comment contents, newlines,
intentional compact-syntax demonstrations, exact diagnostics and reference fixtures.
Baseline repository verification passed all four checks. The convention guide and
AGENTS rules passed local-link checks; standalone reader examples are included. Compiler implementation
work remains at the handoff below while this documentation task is active.

Commit plan:
1. Add a documentation convention guide and AGENTS rules; connect the docs index.
2. Normalize schema titles and generated/user-facing project labels in separate slices.
3. Format introductory/guide, language-reference, library, cookbook and compiler
   documentation in related reviewable groups, including their inline code fragments.
4. Apply the same presentation to linked standalone examples as requested;
   preserve tests/tooling/generated source and explicit compact fixtures.
5. Verify lexical preservation, links, repository checks and the compiler gate;
   update both handoffs with actual validation and the implementation next step.

## Current milestone

Compiler documentation now lives under [compiler/docs/](compiler/docs/README.md).
Only `AGENTS.md` and `STATUS.md` remain as Markdown files at the compiler root.
All 17 pages retain their content with corrected links; layout and Cargo metadata checks pass.

Eligible scalar integer primary exports now supply computed-type inputs through a
module name, aliases, arithmetic copies and primary/named re-exports. They retain
original emission evidence, exact widths and transitive work. Runtime HIR and constant
folding remain unchanged; checking/building never executes module initialization.

Named immutable integer/record export inputs remain supported. Integer
primaries of record-valued modules, composed/conditional emissions, helper purity and
full required evaluation remain separate. See [the supported slice](compiler/docs/COMPUTED_TYPES.md).

Net/HTTP/TLS still needs broader generic-type/I/O/task foundations. Full v0.0.1
release qualification remains incomplete.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all ten checks passed, including 1412 Rust
  tests, 20 Python tests, and existing examples in debug/release.
- Nine native primary-input groups passed, including silent checking/building,
  once-only initialization, runtime failure, source spans and repeated-read work.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local links, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Named inputs and scalar integer primaries pass; mixed-record primaries remain gated. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan integer primary projections of record-valued file modules using the concrete
   files and validation in the compiler handoff. Preserve record identity, exact widths,
   initializer evidence, work charging and runtime capture gates.
2. Preserve package, borrowed-export and ownership gates. Plan and commit validated
   slices using [AGENTS.md](AGENTS.md); keep STATUS concise, never recreate STEP logs,
   and do not push.
