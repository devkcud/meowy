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

Documentation and 82 standalone examples retain the standardized readable layout.
The prior token/literal preservation audit is `/tmp/meowy-doc-style-audit.json`;
Git preserves its completed commit series. Compiler guides remain in `compiler/docs/`.

## Current milestone

The compiler entry guide is [compiler/README.md](compiler/README.md); detailed
guides live in `compiler/docs/`. The compiler root keeps `README.md`, `AGENTS.md`
and `STATUS.md`. Links and Cargo metadata follow this layout.

Eligible direct integer primary exports now supply computed inputs even when their
modules also export named fields. Arithmetic, integer-annotated required reads,
copies and scalar re-exports preserve exact widths and initializer evidence.
Aliases and type queries keep the complete record type. Named effects remain
independent of primary eligibility, and check/build never execute initialization.

Behavior/test commits: `533bf22` (copy evidence), `fdc2cd7` (required projections),
`5ee7044` (staging, startup and transitive evidence). Whole-record computed inputs,
composed/conditional export evidence and helper purity remain separate.
See [the supported slice](compiler/docs/COMPUTED_TYPES.md).

Net/HTTP/TLS still needs broader generic-type/I/O/task foundations. Full v0.0.1
release qualification remains incomplete.

## Actual validation

- All 20 primary native groups passed, including debug/release execution, silent
  check/build, startup ordering, width/effect/capture gates and repeated work.
- `python3 -B tools/verify.py --compiler`: all ten checks passed, including 1423
  Rust tests, 20 Python tests, fmt, Clippy, build and existing examples.
  Log: `/tmp/meowy-mixed-primary-gate.log`. The new guide example also passed
  debug/release execution.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug/release. Local links,
  catalog/schema and whitespace checks passed; full release qualification is open.
- Runtime implementation, reference fixtures and dependencies are unchanged.
  Editor and separate runtime/sanitizer gates were not rerun; release remains open.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Named inputs and direct integer primaries pass, including mixed modules. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan record-composing module re-export evidence using `statements.rs`, `exports.rs`
   and input lookup paths. Preserve individual initializer evidence, record identity,
   privacy, work and startup order; keep conditional exports and helper purity separate.
2. Preserve package, borrowed-export and ownership gates. Commit validated slices
   using [AGENTS.md](AGENTS.md), keep STATUS concise, and do not push.
