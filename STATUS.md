# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

The [borrow and dereference syntax migration](compiler/BORROW_SYNTAX.md) is
complete and passed compiler/native/editor checks. Prefix `&`, `&!` and `*` bind
before following field selection, indexing, calls and type suffixes. Dotted `.&`,
`.&!` and `.*` operate on the immediately selected field. Parentheses group a
complete target: `&items[i]` means `(&items)[i]`; `&(items[i])` borrows the element.
Likewise `*object.field` means `(*object).field`; `object.*field` dereferences the
selected field. Prefix chains inherit that boundary and parentheses reset it.

Repository source strings, examples, documentation and editor fixtures now follow
that grammar. Existing program intent was preserved with explicit grouping or
selected-field forms. The implementation reuses the existing AST/HIR and ownership
passes; no runtime, backend, dependency or compatibility-mode changes were needed.
Try [pointer-syntax.mwy](compiler/examples/pointer-syntax.mwy) for executable examples.

Carried reference-free records retain whole-slot initialization across inner
restarts; shared projections survive while the result owner lives. Local exclusive
Boolean/integer/float field loans and all descendants must end before every restart.
Whole-record/non-scalar exclusive paths and wider carried shapes remain separate.

Standalone documentation tooling remains complete for its bounded bootstrap slice.
The net/HTTP specification merge is complete: `@"net"` owns transports and peer
composition; `net.http` supplies sender/receiver adapters. Networking, HTTP and TLS
remain specified library work, not executable implementations.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 594 library and 558 native Rust tests (1152 total), 16 tooling
  and 4 compiler-harness Python tests, and 73 examples in debug and release.
- All 21 parser tests pass, including 7 new syntax groups. Six native acceptance
  groups exercise the new distinctions in both profiles. Vim and Neovim suites pass.
- Old/new normalized ASTs matched 962 migrated Rust snippets and 37 changed
  standalone sources. Unsupported documentation examples were checked separately;
  generated fragments also retain their native behavior/diagnostic regressions.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases are not successful language rejections or full release qualification.
- Final local-link and whitespace checks passed: 1083 links in 103 Markdown files
  and `git diff --check`. Repository contracts also cover 23 catalog records and
  7 schemas/6 examples. External links were not fetched.
- The separate runtime/sanitizer gate was not rerun; no runtime/backend code changed.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | New pointer syntax and migrated sources pass; carried plain records/shared projections/local exclusive fields remain supported. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Vim/Neovim fixtures cover dotted borrow/dereference, grouped targets and unchanged logical/type operators. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Use the new syntax for all further source work. Select fields with `.&`, `.&!`
   and `.*`, and group full indexed/call targets when the operation applies after
   selection/evaluation. Keep source-byte expectations derived from actual text.
2. Resume investigation of declared fixed-capacity, reference-free carried list
   slots through `compiler/src/borrow/carried.rs`, `compiler/src/check/statements.rs`
   and `compiler/src/loans/emission_init.rs`. Prove initialization, copies,
   replacement and owner-reset behavior before expanding eligibility. Qualify
   indexed acquisitions/reservations separately; broader carried shapes remain gated.
3. Continue module graphs and library foundations before executable net peers/TLS;
   implement capability-typed configuration, bounded lifecycle and raw adapters
   before HTTP sender/receiver adapters, following COMPILER.md.
4. Preserve ownership/header/frontier evidence, keep STATUS concise after logical
   steps, commit cohesive validated changes and never recreate STEP logs or push.
