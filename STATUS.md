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

Direct top-level file-module compositions now forward eligible integer primaries
and named integer/record inputs through facades. Each export retains its original
source identity and initializer evidence; required reads charge retained work at
every hop. Projected records keep complete ancestor evidence. Runtime composition,
record identity, privacy, capture gates and startup order remain intact.

Commits: `eb3840e` (source IDs/work), `0a3dd81` (composition evidence), `36cf2c2`
(staging and transitive-work integration). Whole-record module inputs, nonmodule/
conditional compositions and helper purity remain separate.
See [the supported slice](compiler/docs/COMPUTED_TYPES.md).

Net/HTTP/TLS still needs broader generic-type/I/O/task foundations. Full v0.0.1
release qualification remains incomplete.

## Actual validation

- All nine native composition groups and the library identity test passed, covering
  debug/release execution, silent check/build, startup order, repeated work,
  ancestor effects and width/privacy/capture boundaries.
- `python3 -B tools/verify.py --compiler`: all ten checks passed, including 1433
  Rust tests, 20 Python tests, fmt, Clippy, build and existing examples.
  Log: `/tmp/meowy-composed-inputs-gate.log`. The facade guide example also passed
  debug/release execution.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug/release. Local links,
  catalog/schema and whitespace checks passed; full release qualification is open.
- Runtime implementation, reference fixtures and dependencies are unchanged.
  Editor and separate runtime/sanitizer gates were not rerun; release remains open.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Primary/named inputs retain evidence through direct module composition. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan eligible local-record compositions using export and record-input paths.
   Preserve complete ancestor evidence, work and runtime effects for every selected
   field; keep module namespace eligibility distinct from whole-record eligibility.
2. Preserve package, borrowed-export and ownership gates. Commit validated slices
   using [AGENTS.md](AGENTS.md), keep STATUS concise, and do not push.
