# Allocator return lifetime bounds

Ordinary functions returning allocator values now apply the conservative
[public lifetime contract](../docs/reference/memory.md#lifetimes). A returned handle
is bounded by every active borrow-carrying input, even when the body simply returns
memory.heap. Immutable copies and further calls retain those bounds. The former
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
| Mutable allocator bindings or assignment | Existing unbounded static values work; storing active bounds is B001 |
| Field/indexed assignment and list construction/append | Reject active bounds before discarding their facts; existing unbounded static heap lists remain supported |
| Calls returning allocator lists, including behind references | B001 when input constraints need to be attached to unmodeled element paths |
| Restart headers containing allocator bounds beneath mutable references | B001 until header source/activity proof includes those bound paths; unbounded static allocator variant activity remains supported |

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
mutation/list/header gates and fanout exhaustion. Native tests verify full execution,
non-freezing lifetime bounds, skipped call entry and distinct E303/B001 diagnostics
in debug/release. Existing reference and restart regressions remain required.

Next represent bounded allocator mutation/list/header state and dynamic string-view
origins in these same passes. Keep owning construction gated until the
[initialized-state/drop schedules](OWNING_HIR.md) prove normal, Leave, Restart and
panic exits. Runtime task cancellation, native unwinding and release qualification
remain separate work.
