# Memory and resource ownership

[Documentation index](../README.md)

meowy uses value storage, explicit allocation, and deterministic cleanup. Safe
code must not read uninitialized storage, use a released allocation, violate
reference exclusivity, or race on ordinary memory. There is no tracing garbage
collector or automatic heap promotion of escaping locals.

[Memory and binary optimization](optimization.md) follows these owners through
frames, heap allocations, static storage, and channel queues. It covers peak
memory budgets, constrained hosts, and the code/data retained in an executable.

## Storage and layout

Scalars, records, closed unions, and bounded lists live inline in their owner.
An owner may be a local frame, another record, static storage, or an explicitly
allocated object. Escape analysis may remove or relocate storage while preserving
its observable lifetime and identity; it cannot introduce an observable allocator
call to make an invalid borrow legal.

| Value                    | Representation contract                                             |
| ------------------------ | ------------------------------------------------------------------- |
| Fixed-width integer      | Stated bit width; signed integers use two's complement              |
| Boolean                  | One byte in ordinary addressable storage, valid values 0 and 1      |
| Null                     | No payload bytes; its enclosing union may need a tag                |
| Reference or raw pointer | Target-width address; safe references are non-null                  |
| String or slice          | Data address and `usize` length                                     |
| Bounded list `T[N]`      | Runtime length and inline capacity for `N` elements                 |
| Record                   | Primary storage, if any, and storage for each field                 |
| Closed union             | Active alternative plus enough aligned storage for that alternative |
| Non-capturing function   | Code pointer                                                        |
| Capturing function       | Code plus an inline capture environment                             |

Only initialized list elements are valid `T` values and are cleaned up. Padding
bytes are unspecified and cannot be inspected as initialized data. Native layout,
field order, tag placement, and calling convention require explicit FFI declarations;
ordinary records and unions do not promise a stable binary ABI.

`memory.size_of<T>()` and `memory.align_of<T>()` are compile-time target queries.
The size includes padding, capacity, and tags as applicable. Large inline values
can exhaust a stack; choosing inline storage is not a guarantee of cheap copies.

## Copies, moves, and borrows

Every resource has one owner. Initialization, assignment, parameter passing, and
emission copy a `memory.Copy` value and move any other value. A move invalidates
the source until it is assigned a new value.

Scalars, shared references, raw pointers, non-capturing function pointers, strings,
and immutable slices are copyable. Records, unions, arrays, and bounded lists are
copyable exactly when every constituent is copyable. Exclusive references,
allocation owners, channel endpoints, and task handles are not copyable. Copying a
borrowed view never extends the referenced storage's lifetime.

```meowy
<Point> : <{
    x <int32>
    y <int32>
}>

point <Point> : { -> x : 2; -> y : 3 }
same : point       # an inline copy #
view : &point      # a shared borrow #
```

`&value` creates a shared reference. `&!value` creates an exclusive reference
and requires a mutable location. Shared references permit concurrent reads;
exclusive references exclude all other access to the same storage until their
last use. Field access through a reference projects a reference or copies a
copyable field. It never moves a resource out of borrowed storage.

`*reference` accesses a safe reference's referent. In a copyable value context it
copies the value; in a predicate, field access, borrow, or formatting context it
inspects the borrowed place. It cannot move a non-copyable owner out of a reference.
Raw pointers use the explicitly unsafe memory operations instead.

Borrowing a field can be disjoint from borrowing another field when the checker
can prove their storage does not overlap. A dynamic index is conservatively
treated as potentially overlapping another index. No safe operation can grow,
move, or destroy a collection while outstanding references depend on its storage.

## Lifetimes

Reference lifetimes follow the borrowed owner. Local uses are inferred through
the last use of a borrow. A returned reference must borrow from an input or static
storage, never a function local.

Returned views use an inferred lifetime contract. The body must prove that every
returned reference points into borrowed input storage, a borrowed receiver or
capture, or static storage. For a public signature, the result is conservatively
bounded by all borrow-carrying inputs and captures; it cannot outlive any of them.
With no such inputs or captures, a returned borrowed view must be static. This
rule is available to callers without inspecting the body or parsing an extra
declaration modifier.

```meowy
bytes <uint8[]> : (text <string>) {
    -> text.bytes()
}
```

Here the returned slice cannot outlive `text`. With multiple borrowed inputs,
the conservative contract may shorten a view's usable lifetime; separate the
operation into smaller functions when an independent view should live longer.
This rule applies to records containing references too. A static string literal
can be returned freely. A string built into a local buffer cannot be returned as
a borrowed view. Return the owning buffer instead, or write into caller-owned
storage and borrow that.

Non-copyable field moves mark that field uninitialized. Remaining fields are
still cleaned up, but the whole record cannot be used until all fields are valid.
Safe field moves are forbidden for opaque resources with a custom cleanup
contract. References cannot outlive a task join, loop iteration, or allocation
that they borrow from.

## Explicit allocation

Dynamic storage takes an allocator:

```meowy
memory : @"memory"
collections : @"collections"

'work {
    buffer : collections.vector<uint8>(memory.heap, 256)
    | buffer <memory.AllocationFailure> | 'work.leave()

    # buffer owns its allocation; leaving this scope releases it #
}
```

`memory.heap` is an explicitly selected system allocator. Library operations that
grow storage accept or retain the allocator chosen at construction. Allocation
failure is a value, not a null pointer or an automatic process abort.

An allocator handle is borrowed by its allocations and must remain valid until
they are released. An arena can use caller-provided storage; resetting it requires
that all values and references backed by that arena have expired. An arena reset
cannot silently skip the cleanup of live resource owners.

Allocation returns initialized owners through safe constructors. Uninitialized
raw storage is confined to unsafe code until every required byte and field has
been initialized. There is no general safe `zeroed<T>()`: zero bits are not a valid
value for every type.

## Strings and formatting

`<string>` is a borrowed immutable UTF-8 view. Its length counts bytes. It cannot
be used as a NUL-terminated C string, and byte positions are not character indices.
Owned text uses a library buffer with an allocator; `.view()` borrows it.

An interpolation at a formatting call, such as `debug.print("value: {x}")`,
streams its pieces to the writer and needs no intermediate owned string. In a
stored string initializer, interpolation must be constant-evaluable or use an
explicit text builder and allocator. String concatenation follows the same rule;
`+` cannot silently allocate a runtime string.

The message arguments of [`testing` assertions](stdlib/testing.md#assertions-borrow-their-evidence)
are also formatting boundaries. They evaluate message expressions once and can
stream failing-test context into bounded diagnostic storage without an owned
intermediate string; this does not change the rules for stored strings.

For example, `"{prefix}-{id}"` is a constant when both inputs are constant values.
For runtime output, use a writer or builder. Plain string literal views live for
the whole program.

## Cleanup

Owners are released in reverse initialization order on normal completion,
`leave()`, `restart()`, and panic unwinding. Emitted owners belong to the result
and survive normal completion. If construction fails, only the slots already
initialized are released.

Cleanup is compiler-generated from the value's ownership structure. Opaque
resource types supply an intrinsic release operation. Cleanup cannot emit values,
restart a scope, or throw a recoverable error. A fallible operation such as flushing
a file must be called explicitly before cleanup. Releasing an already explicitly
closed endpoint has no further effect.

A scope first requests cancellation for unfinished children and joins **all**
children before releasing storage they could borrow. It then releases remaining
locals in reverse order. A deadline does not permit destroying a running child's
stack or borrowed data. This may delay scope exit if a child cannot cooperate.

A recoverable panic unwinds and releases owners. A panic during cleanup is fatal.
Process termination, a fatal trap, and an explicit abort do not promise cleanup.

## Raw pointers and unsafe operations

Raw pointers describe addresses without proving validity. They may be null or
dangling, but reading or writing through them requires a `!{ ... }` block and a
proof of allocation lifetime, bounds, alignment, initialization, and access rights.
Creating a reference from a raw pointer must establish all safe-reference rules.

```meowy
memory : @"memory"

read_word <uint32> : (address <*uint32>) !{
    -> memory.read<uint32>(address)
}
```

The function's `!{ ... }` body preserves the caller's precondition: `address` must
point to a live, aligned, initialized `uint32` that can be read without racing a writer.
Its type is `<!(*uint32) -> uint32>`; calling it requires a `!{ ... }` block even
though it has a meowy body.
Prefer a safe slice parameter when bounds and ownership can be expressed in the type.

Inside an ordinary function, an inner `!{ ... }` block marks a locally justified
operation. It does not grant permission to a separately declared function or a
spawned task: each must establish its own boundary. An intrinsic such as
`memory.read` keeps its calling requirements when assigned another name. A local
binding named `unsafe` has no special meaning.

Pointer arithmetic uses byte offsets (`memory.offset_bytes`); collection indexing
remains one-based. Integer-to-pointer conversion cannot establish provenance or
make an arbitrary address valid. `!{ ... }` never disables integer checks, type
checking, cleanup, or task ownership rules.

For memory shared between tasks, use channels, locks, or atomics. Ordinary shared
mutable storage and raw pointers are not automatically transferable. Volatile
access expresses an observable memory access; it is not a synchronization primitive.
