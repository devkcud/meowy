# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Shared borrows of carried reference-free list storage and element projections are
implemented and passed the compiler gate. Whole-list views, nested lists/record
fields, reborrows and call-returned views can survive inner restarts and alias scope
exit while the result owner lives. Acquire still requires an active, initialized
containing slot. Indexing checks the current initialized length; source identity,
parent authority, last use and owner expiry retain their existing rules.

The list-storage gate now applies to exclusive acquisition rather than every
Acquire. Exclusive borrows of list-containing carried storage and indexed writes/
reservations remain gated, including exclusive scalar-field borrows from records
containing lists. See [the contract](compiler/OWNERSHIP.md#shared-carried-list-borrows)
and [runnable example](compiler/examples/carried-list-borrows.mwy). No backend,
runtime, dependency, syntax or binding-rule changes were needed.

Carried list initialization retains whole length/payload with 256-part/32-level
shape bounds. Binding `:` prevents replacement of its slot while mutable fields
retain independent permissions; shared references remain read-only. Use tight prefix
`&`/`&!`/`*`, selected-field `.&`/`.&!`/`.*` and grouping for full indexed/call targets.
Plain carried records retain local exclusive scalar-field loans that end before reset.

Standalone documentation tooling remains complete for its bounded bootstrap slice.
Net/HTTP/TLS remain specified library work; module/type/I/O/task foundations and
capability-typed lifecycle implementation precede executable adapters.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 622 library and 580 native Rust tests (1202 total), 16 tooling
  plus 4 compiler-harness Python tests, and 76 examples in debug and release.
- All 10 focused source/proof groups and seven native groups passed. Graph evidence
  preserves containing-slot Acquire, physical Slot/Field/Element paths and parent
  reference identity, and rejects early/inactive/post-completion acquisition.
- Native cases cover retained whole/element views, nested fields, current-length
  bounds, once-only index effects, canceled acquisition, owner resets, old copies,
  calls, final use, disjoint writes and mixed shared/exclusive headers.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases are not successful language rejections or full release qualification.
- Repository contracts cover local links, 23 catalog records and 7 schemas/6 examples.
  Local-link and whitespace checks passed; external links were not fetched.
- Editor and separate runtime/sanitizer gates were not rerun. Their code is unchanged.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Shared carried-list views/projections are supported; exclusive list-containing storage and indexed writes remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Qualify local exclusive named scalar fields in list-containing carried records
   through `borrow/carried.rs::storage`, `loans/transitive.rs::referenced` and
   `loans/exclusive_restarts.rs::exclusive_restart_source`. Require active whole-slot
   initialization, exact field identity, selected-slot permission and no live
   exclusive loan/descendant across reset. Keep indexed acquisition/writes gated.
2. Add source/native coverage for list sibling replacement, field conflicts, parent
   suspension, owner resets and last use, then run the compiler gate. Investigate
   exclusive indexed elements and indexed-write reservations as separate slices.
3. Preserve shared-header certificates, call/input opacity, old-copy loans and
   owner expiry. Continue module graphs/library foundations; broader carried shapes,
   owning cleanup and exclusive header carriage remain separate.
4. Keep STATUS concise after logical steps, commit cohesive validated changes and
   never recreate STEP logs or push.
