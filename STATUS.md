# meowy project status

Updated: 2026-09-12. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Documentation conventions

Write the project name as `meowy`. Reader examples follow
[the documentation conventions](docs/guide/documentation-style.md), including
linked standalone examples. Tests, internal tooling and generated source retain
independent layouts. Explicit compact-syntax demonstrations and exact identifiers,
protocol bytes, output text and reference fixtures remain preserved.

Documentation and 82 standalone examples were standardized. The compiler lexer
verified unchanged tokens, strings, ordinary comments and statement newlines;
the embedded documentation example also keeps its code tokens and expected output.
The final audit checked 107 fences in changed Markdown pages and 111 changed inline
fragments. Audit: `/tmp/meowy-doc-style-audit.json`. No formatter was added to the repo.

Current commit series (each slice was checked before commit):

| Scope | Commits |
| --- | --- |
| Convention, schema titles, visible labels, README location | `fe4b40e`, `07e5cb3`, `3f85f56`, `6ac39cd` |
| Introductory, language, library and compiler guides | `fa99c10`, `44af745`, `f41ae28`, `cccbdb5`, `dd2d704`, `efe7fcf` |
| Reference, storage, lifetime and list examples | `7669847`, `ccee0d6`, `14ed656`, `70d23fb` |
| Alias, carried and exclusive examples | `b2af393`, `db4e3a1`, `71a6560`, `37b6648`, `acfb6b1` |
| Module and documented examples | `7325f2f`, `a9dfeea`, `e9b41c4` |

This final handoff records the complete validation and resumes the implementation
next steps below. Git retains the earlier compiler feature and documentation moves.

## Current milestone

The compiler entry guide is [compiler/README.md](compiler/README.md); detailed
guides live in `compiler/docs/`. The compiler root keeps `README.md`, `AGENTS.md`
and `STATUS.md`. Links and Cargo metadata follow this layout.

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
- Formatting preservation, lowercase labels, the restored README/Cargo path and
  embedded documentation example execution passed. Latest compiler gate log:
  `/tmp/meowy-doc-style-gate.log`.
- Conformance: 10 passed, 13 unsupported, 0 failed. Local links, catalog/schema and
  whitespace checks passed; full release qualification remains open.
- Compiler language behavior, runtime implementation, reference fixtures and
  dependencies are unchanged. Editor and separate runtime/sanitizer gates were not rerun.

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
