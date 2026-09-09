# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Declared fixed-capacity, reference-free carried lists are implemented and passed
all compiler checks. The whole-slot proof retains initialized length and payload
across inner restarts. Copies, reads, whole replacement/addition, nested lists and
records, empty lists, primaries, owner resets and Leave have source/native evidence.
Shape checks remain bounded to 256 type parts and 32 levels. Existing list capacity,
layout and one-based initialized-length checks remain unchanged.

Borrowing original list-containing carried storage and indexed writes/reservations
remain explicitly gated. Independent copies, completed results and plain scalar/
record sibling slots retain their existing borrowing rules. See the
[carried list contract](compiler/OWNERSHIP.md#carried-reference-free-lists) and
[runnable example](compiler/examples/carried-lists.mwy). No backend, runtime or
dependency changes were needed.

The [pointer syntax migration](compiler/BORROW_SYNTAX.md) is complete: prefix
`&`/`&!`/`*` bind before postfix selection; `.&`/`.&!`/`.*` operate on the selected
field. Group complete indexed/call targets as needed. Plain carried records retain
shared projections and local exclusive scalar-field borrows; exclusive loans and
all descendants must end before every restart edge.

Standalone documentation tooling remains complete for its bounded bootstrap slice.
Net/HTTP/TLS remain specified library work: module/type/I/O/task foundations and
capability-typed lifecycle implementation precede executable peers and adapters.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 604 library and 565 native Rust tests (1169 total), 16 tooling
  and 4 compiler-harness Python tests, and 74 examples in debug and release.
- All 10 focused carried-list source/shape/proof groups and 7 native groups passed.
  Native checks include retained-length bounds, once-only initialization, copies,
  owner reset, Leave, partial panics, mutability and remaining capability gates.
- Empty-list proof still requires exactly one completed emission. Shape/depth/work,
  capacity and layout limits reject unsupported inputs without skipping proof.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases are not successful language rejections or full release qualification.
- Repository contracts cover local links, 23 catalog records and 7 schemas/6
  examples. Local-link and whitespace checks passed; external links were not fetched.
- Editor and separate runtime/sanitizer gates were not rerun. No editor, runtime,
  backend or dependency files changed in this slice.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Carried reference-free lists retain whole values; their storage borrows and indexed writes remain gated. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Pointer syntax passed Vim/Neovim in the preceding slice; unchanged here. |
| Standard library | Net specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Qualify shared borrows of carried list storage and element projections through
   `compiler/src/loans/transitive.rs::referenced`, `loans/emission_init.rs` and
   `borrow/carried.rs::storage`. Require active, initialized containing storage,
   actual initialized-length checks, exact sources and owner-lifetime evidence.
   Preserve separate gates for exclusive indexed acquisition and indexed writes.
2. Add source/native coverage for shared aliases across inner restarts, nested
   lists/fields, old copies, last use, owner completion/reset, Leave and conflicts;
   run the compiler gate before relaxing that shared-acquisition boundary.
3. Continue module graphs/library foundations before executable net peers/TLS;
   broader carried unions/references/owning cleanup and exclusive header carriage
   remain separate. Use the new pointer syntax in all further source work.
4. Keep STATUS concise after logical steps, commit cohesive validated changes and
   never recreate STEP logs or push.
