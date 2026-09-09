# Compiler handoff and work tracker

Updated: 2026-09-09. The bounded standalone documentation slice is complete.
No failing checks or unfinished implementation remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the whole project;
[../COMPILER.md](../COMPILER.md#documentation-completion-slice) records the plan.
Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Completed bounded documentation slice

- Lexer tokens preserve declaration/module fences, maximal bar counts and exact
  opener/body/closer spans. Parser sidecars retain doc trivia and token positions,
  including interpolation rebasing, without changing executable AST layout.
- `documentation/model.rs` attaches doc blocks to supported declarations, parameters,
  fields and modules. E801 rejects orphaned, duplicated or misplaced documentation.
- `check/documentation.rs` reads actual checker scopes/types; it does not infer a
  parallel type system or add runtime reads. Structured value/type links use E802.
  Signatures come from checked values, functions, aliases and field types.
- `documentation/markup.rs` uses pinned CommonMark parsing for prose, link exclusions,
  example metadata and output fences. E803 handles metadata/public-coverage failures.
- `doc check/build` work on explicit standalone source files. Examples check through
  the existing compiler; run examples execute only with `--run-examples`. Failures
  retain the original fence plus example-local coordinates. Unsupported features
  remain unsupported, never matching a requested E-code rejection.
- Example execution uses temporary working directories, closed stdin, an empty
  inherited environment, a default 5000 ms execution timeout and 1 MiB per output
  stream. `--example-timeout-ms` accepts 1..60000. This is not a security sandbox.
  E804 reports expectation or execution-limit failures.
- `doc build` stages an owned `index.html`, refusing unrelated destinations and
  unowned files. HTML is deterministic and responsive; active HTML/unsafe URLs are
  neutralized and images render as alt text. E805 reports publication failures.
- Full package/facade documentation, LSP/rename, assets and public serialized indexes
  remain separate capabilities. The standalone public policy covers top-level
  declarations/exposed members. HTTP/TLS stay unavailable. Do not extend docs now.

## Validation actually performed

- `python3 -B tools/verify.py --compiler`: all ten checks passed.
- Rust: 560 library + 536 native = 1096 tests. Python: 20. Examples: 69 in both
  debug and release. Formatting, Clippy, build and repository contracts passed.
- Eight new library and eight new native documentation groups pass. Existing fence
  regressions now check supported attachment instead of the old blanket B001 gate.
  A fixture initially used unsupported stored interpolation; moving it into the
  supported debug-output path fixed the failure without widening string support.
- Vim and Neovim suites passed. The physical documentation.mwy example also passed
  `doc check --run-examples` (one example run) and `doc build` (zero examples run).
- Local links (1050 across 101 Markdown files) and Git whitespace checks passed.
- Conformance: 10 passed, 13 unsupported, 0 failed. Native runtime/sanitizer gates
  were not rerun. HTTP/TLS, full release, minimum host and portable distribution
  are not qualified by these results.

## Compiler architecture and retained boundaries

| Component | Important files and boundary |
| --- | --- |
| Source and docs | `lexer.rs`, `lexer/comments.rs`, `parser/`, `documentation/`, `check/documentation.rs`; exact spans, one checked binding model |
| Names/types/flow | `check/`, `list_context/`, `flow.rs`; contextual types and bounded flow proofs |
| Result initialization | `borrow/carried.rs`, `loans/emission_init.rs`, `loans/emission_value.rs`; declared scalar slots and pure Boolean-state proof |
| Borrow/value summaries | `borrow/`, `borrow_value.rs`, `borrow_contract.rs`; aliases, current values, union activity, lifetimes and public bounds |
| Restart authority | `loans/exclusive_restarts.rs`, `loans/restart_headers.rs`, `loans/restarts.rs`; all predecessor coverage and no live exclusive ancestry across reset |
| Storage and loans | `loans/storage.rs`, `loans/init.rs`, `loans/authority.rs`, `loans/permissions.rs`; acquisition, moves, parent authority and final-use conflicts |
| Native lowering | Rust frontend and C++ LLVM boundary; generated programs use the private runtime, not compiler Rust crates |
| Documentation CLI | `driver.rs`, `documentation/command.rs`, `documentation/render.rs`; shared input policy, explicit execution and safe output |

Carried initialization currently supports declared non-nullable Boolean, integer,
float and static-string slots. Inner restarts preserve ancestor initialization;
owner resets clear it. Local exclusive scalar loans must end before reset edges.
Shared headers require complete entry/backedge coverage before header-only opacity
can be excluded from conservative ancestry. Genuine call/input uncertainty and
exclusive descendants remain gated. Late-publication certificates, alias backing
conversions, source expiry and old-copy loans remain independent requirements.

## Still outside this compiler

The full manifest/module graph, generic specialization/core.Type evaluation,
capturing and indirect-callable cases, general source-level owned cleanup and FFI
remain incomplete. Wider exclusive restart carriage, HTTP/TLS, full LSP/rename,
package documentation, assets and public documentation index formats are not
enabled by this slice. Unsupported conformance remains explicitly unsupported.

Rust 1.98.1 and LLVM/Clang/LLD 22.1.8 remain pinned. Markdown tooling uses
pulldown-cmark 0.13.4 and its Cargo.lock-pinned transitive dependencies; only the
compiler/tooling links them. Fetching those dependencies is a build prerequisite,
not a runtime network effect. No native runtime/backend dependency was added.

## Resume here

1. Read this handoff and the root/compiler AGENTS instructions. Preserve existing
   user changes. Documentation is finished for this slice; return to compiler work.
2. Prove declared reference-free record-slot initialization in `borrow/carried.rs`,
   `check/statements.rs` and `loans/emission_init.rs` before widening scalar-only
   eligibility. Require bounded shapes and full-slot availability; keep nullable,
   union and reference-bearing carried slots gated. Qualify borrowing and exclusive
   projections separately and preserve every predecessor's actual coverage.
3. Preserve indirect-write Boolean invalidation, acquisition/lifecycle, moves,
   final-use conflicts, old copies, expired sources and the exclusive backedge gate.
   Add source/native boundary cases and run the compiler gate after integration.
4. Extend list contexts, the manifest/module graph and source ownership/library
   support with their own proofs. HTTP/TLS execution requires separate provider,
   transport, cancellation and resource-limit qualification.
5. Keep STATUS focused on current findings, actual checks, blockers and next steps.
   Commit completed validated work by concern; do not push or publish implicitly.
