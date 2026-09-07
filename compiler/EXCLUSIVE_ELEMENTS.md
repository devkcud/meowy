# Exclusive scalar list-element borrows

`&!items[index]` now borrows an initialized scalar element of an ordinary mutable
bounded-list local. Elements may be boolean, integer or float. This implements a
bounded slice of the existing [collection](../docs/reference/collections.md) and
[memory](../docs/reference/memory.md) contracts, preserving one-based positions and
initialized-length checks. Whole-list exclusive references remain unavailable.

## Owner authority and reservation

`ExclusiveElement` HIR stores a local owner ID and one index expression. It is
separate from shared ElementBorrow and names no parent reference. Frontend and
analysis checks require an ordinary mutable scalar-list owner; a mutable binding
holding a shared reference is not an owner proof. Aliases, field/indexed roots,
temporaries, reference roots and non-scalar elements remain gated in this slice.
An explicit value copy into a new mutable local is a valid independent owner.

The graph records an owner read and a private storage reservation before the index.
The reservation names the complete list region but has no loan identity or authority.
It permits reads while blocking conflicting writes/exclusive acquisitions during
returning index evaluation. This protects the captured address and initialized length.

After the index returns, a fresh root exclusive loan names Source::Local with an
Element projection. Its authority comes from mutable-owner proof, not from the
reservation or a shared pointer. The final acquisition demands the reservation;
afterward the reservation is dead. A non-returning index creates no exclusive loan
and has no artificial future reservation demand. Existing work/storage limits cover
these records and their analyses.

## Evaluation and bounds

The backend captures the actual owner address and initialized length before evaluating
the index exactly once. It reuses the existing signed/unsigned conversion and P001
bounds helper, then computes the element address only on the valid path. No list copy,
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
scope through an element reference (E303). Moving a holder remains E301/E309; conflicting
access is E302 and immutable owner borrowing is E305.

## Evidence and next work

Fifteen native groups execute accepted programs in debug/release and check exact
rejections. They cover scalar layouts, once-only index effects, read/write reservations,
conservative overlap, moves/children, call/block transfer, explicit copied owners,
cancellation/conditional exits, captured stores, signed/unsigned initialized bounds,
zero capacity, scopes, wider-root exclusions and short-circuit acquisition.

Three graph groups prove reservation authority is empty and ends at acquisition,
non-returning indices create no loan or future demand, and mutable-owner proof is
required independently of storage shape in both origin and loan analysis. Reference
fixtures are unchanged. Full conformance still has 13 unsupported cases.

The [exclusive elements example](examples/exclusive-elements.mwy) reads an index while
reserving its owner, mutates the actual element through a call and cancels a later
acquisition without retaining the owner reservation.

Next, generalize the owned-list place to existing mutable record fields and exact-backed
emitted storage. Preserve canonical regions, every mutable boundary, actual selected
list capture, reservation lifetime and target scope. Reference-derived, temporary
and nested-index roots, owning elements and exclusive restart bodies remain separate.
