# Meowy project status

Updated: 2026-09-10. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Local exclusive named Boolean/integer/float fields inside list-containing carried
records are implemented and passed the compiler gate. Direct acquisition reaches
the existing exact field/source and restart-frontier proof. Whole-slot
initialization, selected-field permission, owner lifetime and no live exclusive
ancestry across reset remain
required. Disjoint list replacement and certified shared list headers coexist with
these local scalar loans. See [the proof](compiler/EXCLUSIVE_RESTARTS.md#carried-record-fields)
and [runnable example](compiler/examples/exclusive-carried-list-fields.mwy).

Exclusive list/indexed acquisition and indexed writes/reservations remain gated.
Shared whole-list and element views retain their prior support through inner
restarts while the result owner lives. Carried list initialization retains whole
length/payload with 256-part/32-level shape bounds. No backend, runtime, dependency,
syntax or binding-rule changes were needed.

Standalone documentation tooling remains complete for its bounded bootstrap slice.
Net/HTTP/TLS remain specified library work; module/type/I/O/task foundations and
capability-typed lifecycle implementation precede executable adapters.

## Actual validation

- Focused compiler checks passed 12 source/graph groups and eight native groups;
  native cases execute in debug and release. Coverage includes list siblings,
  nested fields, copies, children, calls, conflicts, shared headers, owner resets,
  Leave, final use and preserved capability gates.
- Graph checks preserve containing-slot Acquire and exact storage identity, and
  reject missing initialization, inactive owners and post-completion acquisition.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 624 library and 582 native Rust tests (1206 total), 16 tooling
  plus 4 compiler-harness Python tests, and 77 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  cases are not successful language rejections or full release qualification.
- Local links, catalog/schema and whitespace checks passed. External links were
  not fetched. Editor and separate runtime/sanitizer gates were not rerun.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Local exclusive scalar fields in list-containing carried records passed the compiler gate; indexed paths remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Investigate exclusive scalar list elements in `loans/elements.rs`, including
   initialization, reservations, bounds, source identity and reset frontiers.
   Qualify indexed writes in `loans/control.rs` as a separate slice afterward.
2. Preserve shared-header certificates, call/input opacity, old-copy loans and
   owner expiry. Continue module graphs/library foundations; broader carried shapes,
   owning cleanup and exclusive header carriage remain separate.
3. Keep STATUS concise after logical steps, commit cohesive validated changes and
   never recreate STEP logs or push.
