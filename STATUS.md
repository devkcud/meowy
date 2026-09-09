# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Binding and field replacement permissions are now independent. `object : { -> field
:= 7 }` permits `object.field = 8` while preventing whole-binding reassignment.
Owned nested record fields use the selected field's permission; immutable enclosing
bindings/record fields do not freeze mutable descendants. Shared-reference access
remains read-only and conflicting loans still reject mutation. List indexing
inherits the containing list slot's permission, while record element fields retain
independent flags. See the [language rule](docs/reference/values-and-blocks.md#mutability)
and [runnable example](compiler/examples/binding-fields.mwy).

The compiler now distinguishes replaceable roots from owned values whose fields
can change. Branch/restart snapshots, reference/allocator bounds, emitted aliases,
refinement invalidation and exclusive paths use the appropriate distinction.
Whole-slot reassignment, immutable selected slots, shared permissions, owner
lifetimes and existing unsupported capabilities remain checked. No backend,
runtime, dependency or pointer-syntax changes were needed.

Carried reference-free lists retain initialized length and payload across inner
restarts; their storage borrows and indexed writes/reservations remain gated.
Plain carried records retain shared projections and local exclusive scalar-field
loans that end before restart. Use tight prefix `&`/`&!`/`*`, selected-field
`.&`/`.&!`/`.*` and grouping for complete indexed/call targets in all new source.

Standalone documentation tooling remains complete for its bounded bootstrap slice.
Net/HTTP/TLS remain specified library work; module/type/I/O/task foundations and
capability-typed lifecycle implementation precede executable adapters.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 612 library and 573 native Rust tests (1185 total), 16 tooling
  plus 4 compiler-harness Python tests, and 75 examples in debug and release.
- Eight new source groups and eight native groups cover field/root permission,
  nested/indexed paths, reference updates, nullable activity, alias snapshots,
  allocator bounds, function/dispatch copies, captured stores and rejection rules.
- Old ancestor-mutability rejections now execute as positive native checks when
  the selected slot is mutable. Actual immutable-slot, shared/conflict and lifetime
  diagnostics remain covered. Local-link and whitespace checks passed.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases are not successful language rejections or full release qualification.
- Repository contracts also cover 23 catalog records and 7 schemas/6 examples;
  external links were not fetched. Editor and separate runtime/sanitizer gates were
  not rerun; no editor or runtime/backend code changed.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Selected-slot mutability is implemented; shared carried-list acquisition is next. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax previously passed Vim/Neovim; no changes in this slice. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Use selected-slot permission for writes/borrows and changing-value metadata for
   snapshots/refinements. Do not equate an immutable binding with an unchanging
   record or weaken shared-reference permissions.
2. Qualify shared carried-list storage/element borrows through
   `compiler/src/loans/transitive.rs`, `loans/emission_init.rs` and
   `borrow/carried.rs::storage`. Require active initialized storage, current length,
   exact sources, parent authority and owner expiry. Preserve the separate gates
   for exclusive indexed acquisition and indexed writes/reservations.
3. Add focused source/native coverage and run the compiler gate before relaxing
   that shared-acquisition boundary. Continue module graphs/library foundations;
   broader carried shapes, owning cleanup and exclusive header carriage stay separate.
4. Keep STATUS concise after logical steps, commit cohesive validated changes and
   never recreate STEP logs or push.
