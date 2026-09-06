# Collection APIs

[Library index](README.md)

## Collections

| API                                          | Result                                              | Contract                                                  |
| -------------------------------------------- | --------------------------------------------------- | --------------------------------------------------------- |
| `collections.vector<T>(allocator, capacity)` | `<collections.Vector<T>><memory.AllocationFailure>` | Allocates a vector with the requested initial capacity    |
| `collections.array<T, N>(literal)`           | `<collections.Array<T, N>>`                         | Constructs exactly `N` inline elements                    |
| `collection.size()`                          | `<usize>`                                           | Number of initialized elements                            |
| `collection.slice()`                         | `<T[]>`                                             | Shared view borrowing the collection                      |
| `collection.slice_mut()`                     | `<collections.MutSlice<T>>`                         | Exclusive view borrowing the collection                   |
| `collection.get(index)`                      | `<&T><collections.Bounds>`                          | Checked shared element borrow                             |
| `collection.get_copy(index)`                 | `<T><collections.Bounds>`                           | Checked read for copyable elements                        |
| `list.add(value)`                            | `<T[N]>`                                            | Consumes and appends within capacity; panics if full      |
| `list.try_add(value)`                        | `<T[N]><collections.Full<T, N>>`                    | On failure, retains the unchanged list and unsent element |
| `list.remove(index)`                         | Record with `list` and `value`                      | Consumes and removes an element; panics if out of bounds  |
| `list.try_remove(index)`                     | Removal record or `<collections.Missing<T, N>>`     | On failure, retains the unchanged list                    |

`Full<T, N>` exposes `list` and `value`; `Missing<T, N>` exposes `list` and the
failed `index`. Their retained payloads use inline generic error storage and do
not allocate. Erasing an error with a non-inline payload to the common `<error>`
representation requires explicit boxing; a predicate `<error>` still recognizes
concrete error types without performing that erasure.

Vector methods that can grow storage report allocation failure. The collection
chapter defines indexing and alias semantics; maps have runtime key lookup and
allocator-backed storage, with no implicit conversion from a list.

## Growable vectors

A Vector owns contiguous initialized elements and retains its explicit allocator.
Its capacity can grow; its length changes only after an operation succeeds.
Allocation-size overflow is an AllocationFailure with a size-overflow cause.

| API | Result | Contract |
| --- | --- | --- |
| `vector.capacity()` | `usize` | Allocated element slots |
| `vector.reserve(capacity <usize>)` | `null` or `memory.AllocationFailure` | Ensure at least that total capacity; leave storage unchanged on failure |
| `vector.push(value <T>)` | `null` or `collections.PushFailure<T>` | Append, growing when necessary; failure retains `value` and the vector is unchanged |
| `vector.pop()` | `collections.Item<T>` or `iter.End` | Remove and transfer the last element, or report an empty vector |
| `vector.clear()` | `null` | Release all elements, retaining allocated capacity |

`Item<T>` has a `value <T>` field, so a null element remains distinct from an empty
result. PushFailure contains the rejected value and its allocation-failure cause.
Reserve, push, pop, and clear exclusively borrow the vector; they do not consume
its owner. Existing element borrows must end before these calls, including when
a particular push happens to fit without moving the buffer.

## Runtime-key maps

`collections.map<K, V>(allocator, capacity, hash, equal)` constructs a
`collections.Map<K, V>` or returns `memory.AllocationFailure`. Capacity is an
initial number of entries, not a permanent limit. The map retains the supplied
allocator and concrete callables:

- `hash` accepts `<&K>` and returns `<uint64>`.
- `equal` accepts two `<&K>` values and returns `<boolean>`.
- Equality must be an equivalence relation, and equal keys must hash equally.
  Both functions must be pure and stable while keys remain in the map.

There is no implicit hash/equality dictionary, even for primitive keys. The
library supplies `collections.hash_string` and `collections.equal_string` for
borrowed string keys. The string hasher's versioned algorithm is for table lookup;
use `hash` and a byte encoding when a persistent digest is required. A map does
not extend the lifetime of borrowed key data.

| API | Result | Contract |
| --- | --- | --- |
| `map.size()` | `usize` | Stored entry count |
| `map.get(key <&K>)` | `<&V><null>` | Borrow a value if the key is present |
| `map.contains(key <&K>)` | `boolean` | Test presence without borrowing the value |
| `map.insert(key <K>, value <V>)` | `null` or `collections.Duplicate<K, V>` or `collections.InsertFailure<K, V>` | Transfer a new entry; reject an existing equal key |
| `map.remove(key <&K>)` | `collections.Entry<K, V>` or `iter.End` | Remove and transfer both stored key and value |
| `map.reserve(capacity <usize>)` | `null` or `memory.AllocationFailure` | Ensure total entry capacity without changing entries on failure |
| `map.entries()` | Concrete borrowed cursor | Yield entries in unspecified order without allocating |
| `map.clear()` | `null` | Release keys/values and retain allocated capacity |

Duplicate and InsertFailure retain the supplied `key` and `value`; neither changes
the map. InsertFailure also exposes its allocation cause. Existing entries are
never silently overwritten or dropped by insert. To replace one, explicitly
remove it and insert the new owner, handling both results. Removal retains the
map's capacity, so reinserting into the freed slot does not need to grow storage.

Entry has `key <K>` and `value <V>` fields. An entry cursor yields a record with
borrowed `key <&K>` and `value <&V>`, or `iter.End`. These borrows remain valid only
while the shared map borrow is alive. Mutation requires ending the cursor and
all entry borrows. Maps promise neither numeric indexing nor insertion order.
Dropping a map releases each remaining key/value exactly once and frees storage.

## Cursors and collection algorithms

`@"iter"` exposes `iter.End`, a copyable end marker, plus allocation-free adapters.
A cursor's `next()` takes exclusive access and returns an item or End. End is not
an `<error>`; fallible cursors such as directory iteration declare their additional
error alternative explicitly. After exhaustion, repeated calls return End.

`iter.slice(values)` borrows a slice and yields `<&T>` or End in position order.
It is not a consuming iterator. The ordinary named loop handles exhaustion:

```meowy
iter : @"iter"
debug : @"debug"
values <int32[3]> : [10, 20, 30]
cursor := iter.slice(values.slice())

'loop {
    item : cursor.next()
    | item <iter.End> | 'loop.leave()
    debug.print(*item)
    'loop.restart()
}
```

`iter.take(next, count <usize>)` borrows a supplied next callable and forwards at
most count items; zero never calls the source. For a source that returns an error,
take forwards that error and becomes exhausted. `iter.enumerate(next)` wraps each
successful item with `position <usize>` and `value`; positions start at one, and
an unrepresentable next position yields `iter.Overflow` and exhausts the adapter.
Adapters do no work until advanced and preserve the source item's borrow lifetime.
There is no hidden task, lookahead, or heap collection.

`collections.sort(slice_mut, compare)` sorts in place without allocating and is
not stable. `compare` explicitly accepts two shared element borrows and returns
`int32` (negative, zero, positive); it must describe a consistent total order.
`collections.stable_sort(slice_mut, compare, allocator)` preserves equal-element
order and returns `null` or AllocationFailure. It acquires scratch before moving
any elements, so allocation failure leaves the input unchanged. Both require an
exclusive slice and move elements without duplicating non-copyable owners.
