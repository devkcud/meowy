# Anonymous scalar-reference block results

Ordinary blocks may produce one bare shared or exclusive reference to a boolean,
integer or float. This extends [guarded reference returns](REFERENCE_RETURNS.md)
using the existing [emission rules](../docs/reference/values-and-blocks.md) and
[memory contract](../docs/reference/memory.md). No new syntax, allocation, runtime
ABI or generated cleanup is introduced.

## Emission and transfer

An anonymous emission initializes the block's primary slot. Emitting an exclusive
holder moves it immediately, even though execution continues after the emission.
The old holder is unavailable until reassigned. A completed block transfers that
same loan identity to its consumer; it does not create fresh access permission.
Expected shared scalar-reference types reborrow without consuming the parent.

The existing guarded copy links carry reference identities through result slots,
normal completion, nested blocks, function arguments/results and exact-target Leave.
Availability remains separate: definite moved use is E301 and a feasible moved path
at a join is E309. Duplicate initialization is E205; a required primary missing on
a completing path is E204. No user-visible movable reference cell is added.

Retained slot demand lasts through block completion, including effects after an
emission. A shared child permits compatible parent reads but suspends writes;
an exclusive child suspends overlapping parent reads too. Conflicts remain E302.
Ending a holder's local cell does not end the scalar owner, but an owner declared
inside the completed block cannot escape through its result: E303.

## Scoped exits and cancellation

Leaving the result's own target completes its initialized value. Leaving an
ancestor or panicking can cancel a nested result. The emitted operand's completed
moves and effects still happened; cancellation does not restore a moved holder.
A cancelled result has no artificial future pointer use. Its borrowed scalar may
be accessed again when no other live loan needs it.

`loans/control.rs::scalar_emission` permits compatible anonymous scalar-reference
slots and anonymous scalar-reference emissions proved cancelled by their emission
and target-completion guards. Missing cancellation evidence is B001. Named fields
and dispatch results are not admitted through this exception. Explicit dispatch
BlockIds preserve that gate even when the receiver itself is a scalar.

Indirect stores evaluate a block-valued target once before their RHS. The captured
loan stays demanded through a returning store; Leave/panic skips the final store.
Earlier RHS holder replacement and movement remain visible. Short-circuit skip
paths do not move a holder, and complementary guards preserve availability.

## Remaining boundaries

Exclusive-bearing records/unions, named results, reference cells, dispatch results
and bodies containing exclusive values plus a resolved restart remain gated.
These restrictions also apply to shared values retaining exclusive ancestry.
Existing purely shared carriers, cells and restart behavior remain supported.
Function signature restrictions and conservative all-input lifetime bounds are
unchanged. Wider pointees and owning payload cleanup need separate contracts.

## Validation

Fifteen native block groups exercise moves/reinitialization, expected shared
conversion, retained demand, guarded choices, named Leave, cancellation, RHS changes,
captured targets, function integration, widths, lifetimes, initialization, dispatch
boundaries, short circuits and panic in debug/release. Two graph groups prove
identity preservation across guarded same-address acquisitions and rejection of
missing cancellation evidence. Reference fixtures remain unchanged.

The [reference blocks example](examples/reference-blocks.mwy) demonstrates an emitted
child, continued execution, parent resumption and cancellation preserving a move.

The short-circuit matrix also exposed an independent operator bug: a skipped block
has type Never, but a containing scalar operator previously rejected it. The scalar
checker now propagates non-returning unary/non-boolean binary operands while retaining
operand checking and evaluation order. Three native control groups exercise skipped
shared/numeric blocks, panic prefixes and scoped Leave without exclusive features.
