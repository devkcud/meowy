# Exclusive borrows of emitted scalar storage

After `->name:=value` initializes a boolean, integer or float slot, `&!name` may
borrow its actual storage exclusively. Mutable emitted reference-free Copy records
also permit scalar-field paths such as `&!row.inner.n`, with every crossed field
mutable. This extends the [field contract](EXCLUSIVE_FIELDS.md)
and follows the existing [emission](../docs/reference/values-and-blocks.md) and
[memory rules](../docs/reference/memory.md). No new HIR operation, backend addressing,
allocation or runtime cleanup ABI is introduced.

## Initialization and representation

The emitted name enters scope after its initializer returns. Its initializer can
still resolve an outer binding of the same name. An unknown or not-yet-introduced
name reports E201. Immutable aliases or crossed fields report E305. The pointee
must be Bool/Int/Float, either a direct scalar alias or a named path through a
reference-free Copy record. Indexed/union/reference paths, non-scalar exclusive
pointees and exclusive-reference carriers remain gated.

`Alias.exclusive` records the first exclusive-borrow span. When the target block's
completed field type is known, `check/aliases.rs` requires it to equal the alias's
declared owner type. For a field borrow this means the complete record, including
unselected fields and their mutability. Widening to a union reports B001 even when
the local owner is an exact union member or the selected scalar field is unchanged. Shared alias borrowing retains its existing compatible-member
support; the new restriction is specific to exclusive access.

If the target result is discarded, existing emission/completion proof permits its
scalar or record fallback cell. That cell already has the declared local type. Discarding a
result does not skip its initializer or undo successful mutation and other effects.
No union payload is reinterpreted as exclusive scalar storage.

## Identity and lifetime

Each borrowed source retains `Source::Slot` target, canonical root, lexical view and
projection. Different guarded aliases for the same target field share physical
identity but keep distinct loans. Initialization and availability use the canonical
cell; its scope belongs to the target block, not the alias's smaller lexical scope.
Sibling slots remain independent. Direct name writes and indirect stores therefore
participate in the same conflict checks.

A pointer can leave the alias's lexical scope while the target remains alive. It
cannot survive target completion or escape into its own constructing result.
Supported direct escapes and self-containing shared views report E303; exclusive
reference carriers remain B001 rather than gaining carrier support. Reinitializing
a moved pointer holder after target completion is allowed when its new owner lives.

The emitted scalar or record owner remains Copy. Only the exclusive reference holder moves. Parent
suspension, reborrows, guarded choices, direct calls, returned references, anonymous
block results and all-input lifetime bounds reuse the existing authority machinery.
Conflicts report E302, definite moved uses E301 and uncertain availability E309.

## Scoped effects

Indirect stores capture their target before RHS evaluation. Replacing the pointer
holder does not redirect that store. A later Leave or panic can skip the final store
while preserving earlier mutations and moves. Own-target Leave publishes an
initialized result; ancestor Leave can discard it and retain only completed effects.

Exclusive restart bodies remain gated, including iteration-local scalar aliases.
Named record fields use the same canonical Slot root plus concrete field indexes.
Sibling and primary access are disjoint from a named descendant; ancestor and
whole-owner access still overlap. Indexed/union/reference paths require further
authority and region contracts. Existing shared slot, record and collection support is
unchanged; generated destruction and owning payloads remain separate work.

## Evidence and next work

Fourteen native groups run accepted cases in debug/release and check exact primary
rejections. They cover actual result storage, widths, siblings, canonical conflicts,
lexical versus target lifetime, guarded views, moves/children, call/block transfer,
captured stores, Leave/cancellation, escapes, backing types, initialization and panic.
A graph group checks shared canonical storage with distinct guarded loans and target
scope ownership. A checker group verifies the strict exclusive backing error while
retaining a valid shared union view.

The [exclusive slots example](examples/exclusive-slots.mwy) returns a slot pointer
from an inner lexical scope, passes a child through a call and mutates discarded
storage. Reference fixtures are unchanged. Full conformance still has 13 unsupported
cases and does not qualify a complete language release.

Fourteen additional native groups cover emitted record-field mutation, nested
mutability, sibling/primary access, ancestor conflicts, target lifetime, guarded
views, moves/children/calls, exact whole-record backing, captured stores, cancelled
mixed layouts, escapes/bounds, copied records, sibling collections and panic.
A graph test verifies canonical nested projections across distinct guarded views;
a checker test rejects unrelated-field widening while preserving a shared view.

The [projected slots example](examples/exclusive-slot-fields.mwy) reads the primary
and writes a sibling while carrying a nested-field pointer out of the alias scope.
[Scalar list elements](EXCLUSIVE_ELEMENTS.md) now support ordinary local,
record-field and emitted storage with exact backing, canonical regions, selected-list
reservations and target lifetimes, including nested indexed owners and scalar field
leaves after indexes. Whole-record exclusive pointees, non-Copy carriers and cleanup
remain separate work.
