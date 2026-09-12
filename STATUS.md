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

Record initializers now retain computed inputs through matcher branches proven from
boolean literals, negation and short-circuit logic. Selected conditions/statements
retain work and source errors; skipped branches and operands contribute no evaluation
work or effects. Aliases, local compositions and module forwarding preserve the evidence.
Runtime HIR, ordinary checks, capture gates and initialization order remain unchanged.

Commits: `bfb9678` (record statement accumulator), `c4651f3` (literal-branch evidence).
Integration checks and the supported guide are complete. Nonliteral predicates,
conditional module exports and helper purity remain separate.
See [the supported slice](compiler/docs/COMPUTED_TYPES.md#conditional-record-initializers).

Net/HTTP/TLS still needs broader generic-type/I/O/task foundations. Full v0.0.1
release qualification remains incomplete.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all ten checks passed, including 1452
  Rust tests, 20 Python tests, fmt, Clippy and build.
- Eight focused conditional groups pass, including selected/skipped effects and
  errors, forwarded work, silent check/build and one-time startup. Execution and
  work/staging integration exercise debug/release. The guide example prints `7`.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug/release. Local links
  and catalog/schema checks passed. Log: `/tmp/meowy-conditional-record-gate.log`.
- Runtime implementation, reference fixtures and dependencies are unchanged.
  Editor and separate runtime/sanitizer gates were not rerun. Full release qualification
  remains open; record-field shape limits are not native ownership-budget guarantees.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Literal record branches retain selected evidence through composition/imports. |
| Documentation tooling | Constructed signatures and file graphs are checked; multi-file doc commands/indexes remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; type/I/O/task foundations precede implementation. |
| Runtime and release | File-site runtime helpers passed native probes; platform/distribution qualification remains open. |

## Next steps

1. Plan boolean-local and integer-comparison predicate evidence with source errors,
   widths and repeated-work accounting. See the [compiler handoff](compiler/STATUS.md#next-steps).
2. Preserve package, borrowed-export and ownership gates. Keep conditional module
   exports, integer-block branches and helper purity separate. Commit validated slices
   using [AGENTS.md](AGENTS.md), keep STATUS concise, and do not push.
