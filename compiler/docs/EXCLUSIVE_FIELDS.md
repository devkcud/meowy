# Exclusive borrows of scalar record fields

`owner.&!field` now borrows a boolean, integer or float field from an ordinary
reference-free Copy record. Its binding may be immutable; the selected field
must be mutable. Nested named fields and grouping are supported.
This follows the [memory contract](../../docs/reference/memory.md) and extends existing
[scalar exclusive references](EXCLUSIVE_REFERENCES.md). No new reference type,
allocation, backend operation, runtime ABI or LLVM alias promise is introduced.

## Storage and mutability

The root must be an ordinary owned local record whose complete type is Copy and
contains no references. The selected field must be mutable; enclosing record fields
and the root binding need not be replaceable. Unknown names report E201; immutable
selected fields report E305. `check/mutation.rs::record_field` shares field lookup
and each slot's flag with direct assignment, keeping both operations consistent.

`check/references.rs` walks the path with existing structural/work limits and the
256-step write-path cap. The parser and type budgets can stop earlier. The result is
a canonical Place with the original local root and concrete field indexes; the
existing Borrow HIR carries a scalar exclusive type. The owner is not copied into
temporary storage. Copying a record before borrowing creates independent storage.

Temporary/call roots, indexed paths, union-payload paths, reference
indirection and non-scalar exclusive pointees remain B001. This does not enable
exclusive references to whole records, lists, strings or reference cells. Existing
purely shared field and collection operations retain their support.

## Regions and last use

Direct field reads/writes, exclusive acquisitions, reborrows and indirect stores
use the existing normalized access regions and guarded loan identities. Disjoint
named siblings may be read, written or borrowed concurrently. An ancestor/whole-owner
read or replacement overlaps every contained field. Conflicting access reports E302,
including acquisition of an unused reference when an overlapping loan is still live.

Primary projection is distinct from named-field access. `loans/permissions.rs` keeps
the remaining Slot(0) access component and excludes named descendants at that
record boundary. A primary read still overlaps a loan covering the record itself or
an ancestor. No primary marker is converted into a named field index.

Borrow lifetimes follow the complete record owner. A field reference cannot escape
a local record's scope or a function-local copy: E303. Moving a reference holder
transfers its existing authority and makes that holder unavailable (E301/E309);
it does not move the containing record. Parent/child suspension remains unchanged.

## Calls, blocks and stores

Field references pass through direct scalar-reference arguments, guarded returns and
anonymous block results without changing their physical root or projection. Caller
entry checks therefore distinguish sibling field arguments of the same record.
Public all-input bounds retain their conservative write/lifetime protection and
never authorize unrelated storage access.

Indirect stores capture the field pointer once before RHS effects. A sibling can
change on the RHS, and replacing the pointer holder does not redirect the captured
store. Whole-owner replacement conflicts with a returning store. When Leave/panic
skips that store and no other loan remains live, the owner may be replaced; completed
moves/effects are preserved. Guarded selections suspend only their selected regions.

## Evidence and next work

Fourteen native groups execute in debug/release and check exact primary rejection
codes. They cover layouts/widths, siblings and primary projections, nested mutability,
ancestor/whole-owner conflicts, copies, moves/reborrows, calls/blocks, public bounds,
guarded selections, captured/cancelled stores, lifetimes and unsupported paths.
A 16-level nested field path executes; a 300-level source stops at an existing
structural budget. A direct AST test verifies the path cap before root resolution;
a graph test verifies primary disjointness without losing ancestor overlap.

The [exclusive fields example](../examples/exclusive-fields.mwy) keeps two field loans
live while reading the record's primary and mutating through a function call.
Reference fixtures and existing checks are preserved. Full conformance still has
13 unsupported cases; this does not qualify a complete language release.

[Mutable emitted storage](EXCLUSIVE_SLOTS.md) also supports scalar-field paths with
exact whole-record backing, mutable fields, target lifetime and canonical Slot
projections. [Indexed storage](../EXCLUSIVE_ELEMENTS.md) now supports scalar elements
and mutable field leaves such as `rows[i].&!value`, including nested and emitted
owners. It uses complete owned paths and enclosing collection reservations. Owned
carriers and generated destruction remain separate contracts.
