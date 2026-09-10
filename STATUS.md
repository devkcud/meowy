# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Indexed writes to carried lists passed the compiler gate. Whole-slot initialization and
active ownership are checked before capture and at the completed store. Existing
reservations protect each returning index/RHS phase, and captured addresses survive
RHS index-variable changes. Scalar and aggregate leaves retain their layouts,
lengths, selected-field permissions and old copies. See
[the proof](compiler/OWNERSHIP.md#carried-indexed-writes) and
[runnable example](compiler/examples/carried-writes.mwy).

Leave/Restart/panic cancels unfinished stores while preserving completed effects.
Disjoint shared headers and local exclusive siblings keep their support. Whole-list
exclusive values, reference/temporary-derived writes, broader carried shapes and
exclusive header carriage remain separate. No backend, runtime, dependency, syntax
or binding-rule changes were needed.

Standalone documentation tooling remains complete for its bounded bootstrap slice.
Net/HTTP/TLS remain specified library work; module/type/I/O/task foundations and
capability-typed lifecycle implementation precede executable adapters.

## Actual validation

- Nine source groups, four graph groups and eight native groups passed; native
  cases execute in debug/release. Evidence covers initialization, reservations,
  layouts, captured indices, bounds, cancellation, owner reset, last use and conflicts.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 651 library and 598 native Rust tests (1249 total), 16 tooling
  plus 4 compiler-harness Python tests, and 79 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  cases are not successful language rejections or full release qualification.
- Local links, catalog/schema and whitespace checks passed. External links were
  not fetched. Editor and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Carried indexed writes passed the compiler gate; whole-list exclusive values remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Start the bounded file-module foundation through `check/names.rs`, `lib.rs` and
   `driver.rs`: canonical relative-file identity, exports, source diagnostics, cycle
   rejection and once-only ordered initialization. Preserve explicit package and
   manifest gates; the compiler handoff identifies the governing contracts.
2. Preserve shared-header certificates, call/input opacity, old copies and owner
   expiry. Broader carried shapes, owning cleanup and exclusive header carriage
   remain separate. Keep STATUS concise, commit validated changes, and never
   recreate STEP logs or push.
