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

Record predicates now accept eligible immutable boolean locals and exact-width integer
comparisons. Boolean aliases and scoped scratch retain values, source errors and work;
failed predicates preserve record error paths without choosing a branch. Short-circuiting,
module staging, runtime HIR and ordinary flow/capture/ownership checks remain intact.

Commits: `e6632e3` (typed evidence), `6924169` (comparisons), `5fe7946` (boolean
bindings), `cd680d3` (scoped scratch). Integration checks and the guide are complete.
Boolean block initializers and boolean field/export inputs remain separate.
See [the supported slice](compiler/docs/COMPUTED_TYPES.md#conditional-record-initializers).

Net/HTTP/TLS still needs broader generic-type/I/O/task foundations. Full v0.0.1
release qualification remains incomplete.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all ten checks passed, including 1462
  Rust tests, 20 Python tests, fmt, Clippy and build.
- Debug/release integration probes pass for transitive predicate work, skipped
  operands, separate roots, silent check/build and dependency initialization.
  The updated guide example prints `7` in both profiles.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug/release. Local links
  and catalog/schema checks passed. Log: `/tmp/meowy-predicate-inputs-gate.log`.
- Runtime implementation, reference fixtures and dependencies are unchanged.
  Editor and separate runtime/sanitizer gates were not rerun. Full release qualification
  remains open; record-field shape limits are not native ownership-budget guarantees.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Boolean locals and integer comparisons retain record predicate evidence. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan boolean block-initializer evidence with complete tail work/errors and runtime
   staging. See the [compiler handoff](compiler/STATUS.md#next-steps).
2. Preserve package, borrowed-export and ownership gates. Keep boolean field/export
   inputs, required boolean scratch, conditional module exports and helper purity
   separate. Commit validated slices using [AGENTS.md](AGENTS.md); do not push.
