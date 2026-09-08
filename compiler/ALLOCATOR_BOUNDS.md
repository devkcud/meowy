# Allocator return lifetime bounds

Ordinary functions returning allocator values now apply the conservative
[public lifetime contract](../docs/reference/memory.md#lifetimes). A returned handle
is bounded by every active borrow-carrying input, even when the body simply returns
memory.heap. Copies, supported rebindings and further calls retain those bounds. The former
blanket signature gate is removed for the supported shapes below.

The actual allocator-producing intrinsic is still the static heap. This change
models public lifetime constraints on that value; it does not add arena/custom
allocator constructors or origins for dynamically owned allocator contexts. Allocator
parameters have no physical context-origin facts in this static-only slice; dynamic
factories will require explicit symbolic context sources in the input/call model.

## Propagation and consumption

The borrow engine's existing State stores these constraints in `bounds`, separate
from physical reference `origins`. Call substitution attaches active input origins
and inherited bounds to allocator components of the result. Input evaluation order
is unchanged, and full argument validation occurs only after every argument returns.
An argument Leave skips entry; an expired argument at actual entry reports E303 even
when the function returns only a scalar.

Allocator parameters are available throughout their synchronous invocation. The
caller substitutes its actual input constraints into the public return contract;
callee-local borrows still require proof in every body, including uncalled bodies.
No shared physical loan is invented for the returned handle. A lifetime bound
requires its source storage to remain alive; it does not by itself prevent replacing
a scalar in that still-live storage. Actual references retain their normal access
restrictions. Typed loan bundles preserve that distinction through copies, block
results, branches and shared pointee transfers.

```meowy
memory : @"memory"

select <memory.Allocator> : (context <&int32>) {
    -> memory.heap
}

context := 7
handle : select(&context)
context = 8
copy : handle
```

Both uses of the returned allocator occur while context's cell remains alive.
Returning `select(&local)` from local's own block, or consuming a handle from
`select(&1)` in a later statement, reports E303. The
[allocator-bounds example](examples/allocator-bounds.mwy) executes the supported
case in debug/release without changing the runtime allocator representation.

## Supported shapes and explicit limits

| Operation | Current boundary |
| --- | --- |
| Immutable allocator bindings, copying and forwarding | Retain all active input bounds |
| Immutable record fields and closed unions | Retain component paths and variant guards; scalar projections and proven null alternatives do not retain unrelated allocator bounds |
| Shared borrows/reborrows of allocator cells and carriers | Preserve contained bounds beneath the reference; direct dereference drops only the cell access, while function results also retain their public input bounds |
| Direct mutable memory.Allocator bindings and assignment | Retain guarded bound versions through branches, Leave and Restart; overwrite-before-read can discard expired bounds |
| Mutable record/union/list carriers and mutable emitted aliases | Active allocator bounds remain B001; existing unbounded static storage works |
| Field/indexed assignment and list construction/append | Reject active bounds before discarding their facts; existing unbounded static heap lists remain supported |
| Calls returning allocator lists, including behind references | B001 when input constraints need to be attached to unmodeled element paths |
| Restart headers for direct allocator handles and shared references containing allocator components | Preserve bound paths and variant activity; reference components still require physical origin coverage |

## Mutable versions and restart

Direct mutable allocator bindings reuse the reference version/branch machinery.
The RHS finishes before assignment commits; earlier writes survive a later Leave,
while an unfinished outer store does not occur. Copies made before replacement
keep their own constraints. Replacing a handle with memory.heap clears its current
bounds without repairing an older copy or reading the overwritten value. A live
reference to the handle cell still prohibits replacement with E302.

Restart headers use the existing bounded canonical replay and exact predecessor
snapshots. Allocator components may have an empty bound set, so their CFG versions
retain explicit empty entries. Header shape distinguishes those optional bound
paths from physical reference paths, which still require origin coverage. Empty
bounds cannot stand in for a missing reference proof.

Ancestor sources survive an inner restart. Ended Local, Temporary and emitted Slot
sources become terminal Expired identities; reinitializing the same source site
does not revive an older handle. Overwriting the handle or pointer before reading
its expired contents is allowed. An actual expired read reports E303. Header
widening remains conservative and does not prove arbitrary relationships between
iteration counters and prior assignments. The
[mutable-allocators example](examples/mutable-allocators.mwy) exercises the supported
loop and empty/bounded transitions in debug/release.

Tag-only inspection remains distinct from consuming allocator contents. A scalar
projection from a temporary carrier can leave its block without carrying an
unselected allocator field. Retaining the whole emitted carrier still validates
all active bounds. A call through a reference to a carrier validates its contained
constraints at entry; borrowing a cell does not hide an expired allocator bound.

All bound expansion uses the existing part, path, fact and shared work budgets.
Input-by-result fanout beyond those limits reports B001 without publishing partial
call facts. No list capacity is expanded into per-element state. These B001 cases
are bootstrap boundaries, not successful language-conformance rejections.

## Evidence and next work

Checker tests cover transitive/all-input bounds, E303 escapes and temporary expiry,
shared carrier/reborrow paths, inactive nullable results, scalar projection,
guarded mutation, Leave, restart expiry, required reference coverage, remaining
carrier gates and fanout exhaustion. Native tests verify full execution,
non-freezing lifetime bounds, skipped call entry and distinct E303/B001 diagnostics
in debug/release. Existing reference and restart regressions remain required.

Next represent bounded aggregate/list/emitted-alias mutation and dynamic string-view
origins in these same passes. Keep owning construction gated until the
[initialized-state/drop schedules](OWNING_HIR.md) prove normal, Leave, Restart and
panic exits. Runtime task cancellation, native unwinding and release qualification
remain separate work.
