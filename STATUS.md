# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Shared carried-record borrows are implemented and passed the compiler gate. The existing
Acquire event proves active, initialized whole-slot storage for direct and field
borrows; the physical solver retains projection, conflict and owner-lifetime
checks. Shared references and projected reborrows survive inner restarts and alias
scope exit while their result owner lives. Exclusive record-storage borrows remain
gated. Eight new source/proof groups and five native groups cover this boundary.

Declared reference-free record result slots now retain initialization across inner
restarts. Nested scalar/unit records, copies, mutable fields, whole-record writes,
owner resets and Leave use the existing whole-slot initialization proof. Shape
eligibility is bounded to 256 type parts and 32 levels. Ordinary copies and supported
scalar sibling loans retain their existing rules. No backend, runtime or dependency
changes were needed.

The bounded [documentation slice](COMPILER.md#documentation-completion-slice) is
complete for standalone bootstrap sources: structural attachment, derived
signatures, checked links, diagnostics, local API pages and checked/opt-in examples.
HTTP and TLS remain specified library work, not executable implementations.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  formatting, Clippy, build, repository contracts and compiler regression coverage.
- 1123 Rust tests passed: 577 library and 546 native. The 71 compiler examples run
  in debug and release. Compiler/tool harness coverage remains 20 Python tests.
- The focused `--lib carried_record` run passed all 17 record groups, including
  eight new source/proof groups. Five new native groups cover shared record borrows
  and their execution/diagnostic boundaries in both profiles.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases are not successful language rejections or full release qualification.
- Final documentation check: 1057 local links in 101 Markdown files, 0 failures.
  `git diff --check` passed; external links were not fetched.
- Vim/Neovim and standalone documentation CLI execution passed in the preceding
  documentation slice; they were not rerun for this compiler-only change.
- The separate runtime/sanitizer gate was not rerun. No runtime/backend code changed.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Carried plain-record initialization and shared projections are supported; local exclusive scalar-field borrows are next. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Documentation fences supported in Vim/Neovim; no changes in this slice. |
| Standard library | HTTP/TLS contracts exist; module and library foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Qualify local exclusive borrows of scalar fields in carried records through
   `compiler/src/borrow/carried.rs`, `compiler/src/loans/exclusive_restarts.rs` and
   the existing place/path proof. Require an active, initialized containing slot,
   an exact projected source and no live exclusive loan or descendant across a
   restart edge. Keep whole-record and non-scalar exclusive paths gated. Add
   source/native cases, then run the compiler gate.
2. Preserve certified shared-header coverage, genuine call/input uncertainty,
   scalar exclusive backedge boundaries, old-copy loans and no synthetic reads.
   Nullable, union, list, reference-bearing and owning carried slots remain separate.
3. Continue module graphs and library foundations before executable HTTP/TLS;
   standalone documentation completion is not a prerequisite for more doc polish.
4. Keep root/compiler STATUS concise with actual evidence and concrete next steps;
   commit cohesive validated changes, never recreate STEP logs and do not push.
