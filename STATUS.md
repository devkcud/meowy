# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

The bounded [documentation slice](COMPILER.md#documentation-completion-slice) is
complete for standalone bootstrap source files. It supports structural attachment,
compiler-resolved value/type links, derived signatures, `doc check/build`, safe
local API pages, checked/reject examples and explicit bounded example execution.
E801-E805 identify documentation failures; unknown language/library features remain
unsupported rather than passing a documentation check.

The [documentation example](compiler/examples/documentation.mwy) executes normally
and documents its own small API. Its doc check ran one example, and doc build
produced a local API preview without executing application initialization.
Full LSP/rename, package documentation, authored assets and public index formats
are deferred. Stop expanding documentation now and return to core compiler features.
[HTTP](docs/reference/stdlib/http.md) and [TLS](docs/reference/stdlib/tls.md) remain
library contracts, not implemented protocol support.

## Actual validation

- All ten compiler checks passed: 560 library + 536 native = 1096 Rust tests,
  20 Python tests, 69 examples in debug/release, formatting, Clippy, build and
  repository contracts. One stored-interpolation fixture was corrected to the
  existing debug-output path; no test failures remain.
- New coverage checks attachment, links, checked signatures, missing public docs,
  example expectations, opt-in execution, time/output limits, deterministic HTML,
  active-content escaping and protected output replacement.
- Vim and Neovim suites passed. A direct documentation example check/build also
  passed. Documentation checks are not HTTP/TLS qualification or a full release.
- Local documentation validation passed: 1050 links in 101 Markdown files and Git
  whitespace checks. External links were not fetched by the local checker.
- Conformance remains 10 passed, 13 unsupported, 0 failed. The native runtime and
  sanitizer gates were not rerun; their prior evidence is preserved in Git.
- Markdown tooling adds pinned pulldown-cmark 0.13.4 with locked transitive crates.
  Generated programs do not link those compiler tooling dependencies. Runtime/backend
  code and reference conformance fixtures are unchanged.

## Area handoff

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Standalone documentation plus existing ownership/restart/alias proofs | Declared reference-free record-slot initialization |
| Runtime | Private ownership/cleanup and bounded task prototypes | Source integration, cancellation and cleanup qualification |
| Standard library | Partial foundational support; broad contracts including HTTP/TLS | Module/ownership prerequisites, then independent library/protocol tests |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim doc fences pass; no full LSP server | Reuse checked documentation when LSP infrastructure is built |
| Tools | Compiler gates and standalone doc commands work | Preserve bounded tooling; no further documentation polish now |
| Distribution | Host bootstrap with pinned Rust/LLVM tools | Bundled sysroot, notices, reproducibility and minimum-host qualification |

## Next steps

1. Resume declared reference-free record-slot initialization across inner restarts
   in `borrow/carried.rs`, `check/statements.rs` and `loans/emission_init.rs`. Require
   bounded shapes and full-slot availability. Keep nullable/union/reference-bearing
   carried initialization gated; qualify storage borrowing and projections separately.
2. Preserve certified shared-header coverage, call/input uncertainty, exclusive
   backedge boundaries, old copies, owner expiry and no synthetic reads while
   expanding compiler support. Extend list contexts and owning cleanup only with
   their corresponding proof obligations.
3. Build the manifest/module graph and source library foundations. HTTP/TLS need
   separate transport/provider/resource tests, not another documentation milestone.
4. Keep STATUS concise and current. Run relevant gates when behavior changes,
   update actual evidence and commit cohesive work. Do not push or publish without
   explicit authorization, and do not count unsupported conformance as success.
