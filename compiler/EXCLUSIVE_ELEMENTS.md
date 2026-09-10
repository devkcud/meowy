# Exclusive scalar borrows through indexed storage

`&!(items[index])` and `rows[index].&!value` borrow initialized scalar storage
through owned bounded lists: ordinary locals, named record fields, nested indexed owners and exact-backed
emitted aliases. Elements may be boolean, integer or float. This implements a
bounded slice of the existing [collection](../docs/reference/collections.md) and
[memory](../docs/reference/memory.md) contracts, preserving one-based positions and
initialized-length checks. Whole-list exclusive references remain unavailable.

## Owner authority and reservation

`ExclusivePath` HIR stores an owned Place (local view plus leading named-field
path) and the complete WriteStep sequence. Its scalar leaf may be a list element
or a mutable record field beneath one or more indexes. It is separate from shared
ElementBorrow and names no parent reference. `check/indexed.rs` resolves the bounded
path; `Proofs::exclusive_path_type` independently validates it in both analyses.
A named field supplies its own mutability; an index inherits its list slot's flag.
The final slot must be mutable, so immutable lists can expose mutable record
fields without permitting element replacement. Projected owners must be
reference-free Copy records. A mutable binding holding a shared reference does
not grant owned access. Nested paths retain the same owner/shape proof. Temporary/reference-derived roots and non-scalar pointees remain gated.
An explicit value copy into a new mutable local is a valid independent owner.

Emitted aliases require exact backing for the complete list or containing record,
including unrelated fields and list capacity. Slot annotations can contextualize
list construction before borrowing; incompatible guarded completion layouts remain
B001. Proven discarded emissions retain their original layout. Alias proofs carry
explicit mutable/exclusive intent and completed backing evidence.

The graph records an owner read and a private storage reservation before the index.
The reservation names the selected list region, including its named-field prefix,
but has no loan identity or authority. Sibling storage outside the outermost selected
list remains disjoint.
It permits reads while blocking conflicting writes/exclusive acquisitions during
returning index evaluation. This protects the captured address and initialized length.

After the index returns, a fresh root exclusive loan names Source::Local or
Source::Slot with mixed Field/Element projections ending at the selected scalar.
Field acquisition retains the final Field projection and demands enclosing list
reservations; it adds no synthetic index or extra element projection.
Its authority comes from selected-slot and owner proof, not from the reservation or a shared
pointer. The final acquisition demands the reservation;
afterward the reservation is dead. A non-returning index creates no exclusive loan
and has no artificial future reservation demand. Existing work/storage limits cover
these records and their analyses.

Nested paths reserve each enclosing collection before its index. Each returned
intermediate index demands all reservations already captured because its bounds
check and address calculation occur even if a later index exits. Final acquisition
demands all of them, then ends them together. Cancellation within an index skips
that index's bounds/acquisition demand and every later evaluation; demand from an
earlier completed index remains. Thus an outer mutation followed by a returning
outer index is rejected even if the inner index always leaves. Mutating the owner
inside a non-returning inner index is allowed after earlier checks have completed.

## Evaluation and bounds

The backend captures the actual owner address and initialized length before evaluating
each index exactly once, checking an outer position before evaluating the next.
Intermediate bounds diagnostics retain their own index-path spans. It reuses the
existing signed/unsigned conversion and P001 bounds helper, then computes the
selected address only on the valid path. No list copy,
allocation, runtime ABI change or LLVM alias promise is added for the borrow itself.

Known invalid constant positions report E101 using available length/capacity facts.
A position within capacity but beyond an unknown mutable initialized length still
reaches runtime P001. Negative signed, zero, huge unsigned, empty and zero-capacity
cases retain the existing diagnostics. Index effects occur before runtime bounds
failure. Leave/panic during the index skips bounds, acquisition and any later store,
while preserving completed earlier effects and pointer-holder versions.

## Conflict and lifetime boundary

Element projections conservatively overlap every other element of the same list.
Owner capture/read, owner replacement, list length access and direct element operations
use whole-collection regions in this first slice. They may therefore conflict with a
live exclusive element even when an index or metadata field could later be proved
independent. This is not per-index disjointness or a complete two-phase borrow model.

Reference moves, shared/exclusive reborrows, parent suspension, direct scalar calls,
guarded returns and anonymous block results retain existing authority and lifetimes.
After the final use, the owner can be accessed again. Local owners cannot escape their
scope through an element reference (E303). Emitted pointers may outlive their lexical
alias while its target block lives, but cannot escape that target. Moving a holder
remains E301/E309; conflicting access is E302 and borrowing an immutable selected slot is E305.

## Evidence and next work

Fifteen native groups execute accepted programs in debug/release and check exact
rejections. They cover scalar layouts, once-only index effects, read/write reservations,
conservative overlap, moves/children, call/block transfer, explicit copied owners,
cancellation/conditional exits, captured stores, signed/unsigned initialized bounds,
zero capacity, scopes, wider-root exclusions and short-circuit acquisition.

Six graph groups prove reservation authority is empty and ends at acquisition,
non-returning indices create no loan or future demand, and mutable-owner proof is
required independently of storage shape in both origin and loan analysis. They also
verify canonical Local/Slot field paths, cancellation, and missing alias/field proof.
Fourteen additional native groups cover projected/alias storage, sibling regions,
mutable paths, whole-owner backing, target scopes, cancellation, bounds, guarded views
and captured stores. Reference fixtures are unchanged. Full conformance still has
13 unsupported cases.

The [exclusive elements example](examples/exclusive-elements.mwy) reads an index while
reserving its owner, mutates the actual element through a call and cancels a later
acquisition without retaining the owner reservation.

The [projected elements example](examples/exclusive-projected-elements.mwy) carries
an emitted list-element pointer past its lexical alias, mutates a sibling list
during index evaluation, and writes the actual target storage through a call.

Fifteen nested-path native groups cover mixed fields/indexes, per-level bounds and
cancellation, completed-index reservation demand, actual emitted layouts, guarded
views, calls, captured stores, integer widths and empty lists. Three further graph
groups prove reservation chains and independently checked intermediate owner paths.
The [nested elements example](examples/exclusive-nested-elements.mwy) demonstrates
ordered index effects, a disjoint outer sibling and cancellation of an inner index.

Sixteen indexed-field native groups cover scalar widths, mixed paths, mutable
boundaries, collection conflicts, disjoint shared-field views, exact alias backing,
call/child/move transfer, target scope, guarded views, cancellation and prefix bounds
spans. Three graph groups prove the exact leaf region, reservation expiry, cancelled
acquisition demand and required field mutability in both analysis passes.
The [indexed fields example](examples/exclusive-indexed-fields.mwy) returns an
emitted field pointer across its alias scope and mutates the actual target via a call.

Generated payload/diagnostic layouts and scope cleanup are the next integration
boundary. Local exclusive scalar paths in carried lists now use the
[carried-element restart proof](EXCLUSIVE_RESTARTS.md#carried-list-elements), including
whole-slot initialization before capture and completed acquisition. Reference-derived
and temporary roots, owning elements, whole-list exclusive values, ordinary local
exclusive roots in reset graphs and exclusive header carriage remain separate.
