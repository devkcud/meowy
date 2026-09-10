# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Exclusive Boolean/integer/float elements in carried lists passed the compiler gate,
including nested indexes and mutable scalar fields within record elements. The
existing owner/type proof now works with an Element-aware restart source qualifier.
Containing-slot initialization is required before owner capture and again at
acquisition. Reservations preserve evaluation order, bounds and completed-index
demand when a later index cancels. See
[the proof](compiler/EXCLUSIVE_RESTARTS.md#carried-list-elements) and
[runnable example](compiler/examples/exclusive-carried-elements.mwy).

Local exclusive scalar fields and disjoint shared list headers retain their support.
Every exclusive loan/descendant must end before reset. Indexed writes remain gated;
whole-list exclusive values and exclusive header carriage remain separate. No
backend, runtime, dependency, syntax or binding-rule changes were needed.

Standalone documentation tooling remains complete for its bounded bootstrap slice.
Net/HTTP/TLS remain specified library work; module/type/I/O/task foundations and
capability-typed lifecycle implementation precede executable adapters.

## Actual validation

- Eight focused source groups, six graph groups and eight native groups passed;
  native cases execute in debug and release. Coverage includes initialization,
  reservations, mixed paths, mutable fields, parent identity, shared headers, calls,
  bounds, owner resets, cancellation, expiry and retained capability gates.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 638 library and 590 native Rust tests (1228 total), 16 tooling
  plus 4 compiler-harness Python tests, and 78 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  cases are not successful language rejections or full release qualification.
- Local links, catalog/schema and whitespace checks passed. External links were
  not fetched. Editor and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Local exclusive carried scalar elements passed the compiler gate; indexed writes remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Qualify indexed writes in `loans/control.rs` separately: whole-slot initialization,
   reservation order, captured addresses, bounds, canceled RHS, owner expiry and
   final use. Preserve shared-header certificates, call/input opacity and old copies.
2. Continue module graphs/library foundations. Broader carried shapes, owning
   cleanup and exclusive header carriage remain separate. Keep STATUS concise,
   commit cohesive validated changes, and never recreate STEP logs or push.
