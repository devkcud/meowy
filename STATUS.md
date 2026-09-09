# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Declared reference-free record result slots now retain initialization across inner
restarts. Nested scalar/unit records, copies, mutable fields, whole-record writes,
owner resets and Leave use the existing whole-slot initialization proof. Shape
eligibility is bounded to 256 type parts and 32 levels. Borrowing original carried
record storage remains B001; ordinary copies and supported scalar sibling loans
retain their existing rules. No backend, runtime or dependency changes were needed.

The bounded [documentation slice](COMPILER.md#documentation-completion-slice) is
complete for standalone bootstrap sources: structural attachment, derived
signatures, checked links, diagnostics, local API pages and checked/opt-in examples.
HTTP and TLS remain specified library work, not executable implementations.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  formatting, Clippy, build, repository contracts and compiler regression coverage.
- 1110 Rust tests passed: 569 library and 541 native. The 70 compiler examples run
  in debug and release. Compiler/tool harness coverage remains 20 Python tests.
- The focused `--lib carried_record` run passed all nine source/shape groups.
  Five new native groups cover execution and diagnostic boundaries in both profiles.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases are not successful language rejections or full release qualification.
- Final documentation check: 1053 local links in 101 Markdown files, 0 failures.
  `git diff --check` passed; external links were not fetched.
- Vim/Neovim and standalone documentation CLI execution passed in the preceding
  documentation slice; they were not rerun for this compiler-only change.
- The separate runtime/sanitizer gate was not rerun. No runtime/backend code changed.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Carried scalar and plain record initialization is supported; shared record-storage projections are next. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Documentation fences supported in Vim/Neovim; no changes in this slice. |
| Standard library | HTTP/TLS contracts exist; module and library foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Qualify shared borrows of carried record storage and field projections in
   `compiler/src/borrow/carried.rs`, `compiler/src/loans/emission_init.rs` and the
   existing source/storage solver. Require active, initialized whole-slot storage
   at acquisition and preserve expiry across inner and owner restarts. Add focused
   source/native cases, then run the compiler gate. Keep exclusive projected loans
   gated until their separate frontier and path proof is qualified.
2. Preserve certified shared-header coverage, genuine call/input uncertainty,
   scalar exclusive backedge boundaries, old-copy loans and no synthetic reads.
   Nullable, union, list, reference-bearing and owning carried slots remain separate.
3. Continue module graphs and library foundations before executable HTTP/TLS;
   standalone documentation completion is not a prerequisite for more doc polish.
4. Keep root/compiler STATUS concise with actual evidence and concrete next steps;
   commit cohesive validated changes, never recreate STEP logs and do not push.
