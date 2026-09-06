# Foundational library contracts

[Documentation index](../README.md)

These modules define the small API vocabulary used by the guide and worked
programs. They are imported with `@"name"` and share the language's versioned
foundational-library contract. This chapter specifies behavior and failure types;
it does not imply a particular package host, release tag, or native dependency.

In tables, `T` is a compile-time type parameter. `&T` borrows, `&mut T` borrows
exclusively, and an unborrowed non-copyable argument transfers ownership. Intrinsic
member operations on collections, tasks, and channel endpoints borrow their
receiver unless explicitly described as consuming it.

## Output and text

| API                               | Result                        | Contract                                                                |
| --------------------------------- | ----------------------------- | ----------------------------------------------------------------------- |
| `debug.print(value)`              | `<null>`                      | Prints a formattable value and a newline; output failure panics         |
| `debug.panic(value)`              | `<never>`                     | Raises a recoverable panic with formatted diagnostic context            |
| `strings.to_uint8(text <string>)` | `<uint8><strings.ParseError>` | Parses decimal digits with an optional leading `+`, range 0 through 255 |
| `string.size()`                   | `<usize>`                     | UTF-8 byte length                                                       |
| `string.bytes()`                  | `<uint8[]>`                   | Borrows the string's bytes                                              |

The parser rejects empty input, whitespace, separators, signs other than a single
leading `+`, fractional numbers, trailing characters, and out-of-range values.
`strings.ParseError` is an allocation-free concrete `<error>` with a static code
and message; formatting can add the original input without storing an owned copy.

`debug.print` borrows its argument and supports primitives, errors, and aggregates
whose fields are formattable. An aggregate's default display uses its primary
value. Runtime string interpolation at this call streams directly to output.
Production I/O that needs
recoverable write errors uses a writer API rather than the diagnostic printer.

## Numbers and bits

| API                                  | Result                    | Contract                                                                      |
| ------------------------------------ | ------------------------- | ----------------------------------------------------------------------------- |
| `numbers.convert<T>(value)`          | `<T><numbers.RangeError>` | Exact, range-checked numeric conversion                                       |
| `numbers.checked_add(a <T>, b <T>)`  | `<T><numbers.RangeError>` | Reports integer overflow as a value                                           |
| `numbers.wrapping_add(a <T>, b <T>)` | `<T>`                     | Integer addition modulo the width                                             |
| `numbers.truncate<T>(value)`         | `<T><numbers.RangeError>` | Discards a floating fractional part, then checks range                        |
| `numbers.wrapping<T>(value)`         | `<T>`                     | Explicit integer low-bit conversion, interpreted in destination signedness    |
| `bits.shl(value <T>, count <usize>)` | `<T>`                     | Shift left, discarding high bits                                              |
| `bits.shr(value <T>, count <usize>)` | `<T>`                     | Logical right shift for unsigned integers; sign-extending for signed integers |

Both shift functions panic when `count` is at least the bit width. A constant
invalid shift is a static error. Conversions and arithmetic are compiler-known
operations: when the input type's full range proves conversion cannot fail, the
result excludes `RangeError`. This permits explicit widening without a redundant
runtime failure branch. `truncate` still rejects NaN, infinities, and values whose
truncated result is out of range.

## Storage and values

| API                                                    | Result                         | Contract                                                               |
| ------------------------------------------------------ | ------------------------------ | ---------------------------------------------------------------------- |
| `memory.heap`                                          | Allocator handle               | Selects system-managed dynamic storage                                 |
| `memory.size_of<T>()`                                  | `<usize>`                      | Compile-time storage size for the target                               |
| `memory.align_of<T>()`                                 | `<usize>`                      | Compile-time storage alignment for the target                          |
| `memory.read<T>(pointer <*T>)`                         | `<T>`                          | Unsafe read of initialized aligned storage; requires `T : memory.Copy` |
| `memory.offset_bytes<T>(pointer <*T>, offset <isize>)` | `<*T>`                         | Unsafe address calculation within the same allocation or one past it   |
| `values.take_primary(value)`                           | Primary type of the input      | Consumes the aggregate, releases its fields, and transfers its primary |
| `dynamic.box(value, allocator)`                        | `<any><dynamic.BoxFailure<T>>` | Erases a non-null owner; failure retains the original `value`          |
| `dynamic.take<T>(value <any>)`                         | `<T>`                          | Consumes and extracts a box after a proven matching type test          |

`memory.AllocationFailure` describes a failed allocation without requiring a new
allocation to report it. `dynamic.BoxFailure<T>` is a concrete error that also
retains the input owner. An allocator handle retained by a constructor must
outlive the constructed owner. `memory.Copy` is a structural capability, not an
opt-in promise that can override ownership restrictions.

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

## Time and tasks

| API                                                  | Result                   | Contract                                                            |
| ---------------------------------------------------- | ------------------------ | ------------------------------------------------------------------- |
| `time.ms(value <uint64>)`                            | `<time.Duration>`        | Constructs a non-negative millisecond duration                      |
| `time.after(duration)`                               | `<time.Instant>`         | Adds a duration to the monotonic clock; overflow panics             |
| `job.deadline(instant)`                              | `<null>`                 | Sets or shortens a task's deadline before join                      |
| `&group.deadline(instant)`                           | `<null>`                 | Sets a group deadline before submission                             |
| `job.cancel()`, `ticket.cancel()`, `&group.cancel()` | `<null>`                 | Nonblocking, idempotent cancellation request                        |
| `job.status()`, `ticket.status()`                    | Task state value         | Snapshot, never an ownership or completion proof                    |
| `tasks.checkpoint()`                                 | `<null>` on continuation | Acknowledges pending cancellation by unwinding to the task boundary |

`tasks.Outcome<T>` is `T` plus `Cancelled`, `Timeout`, `Panicked`, and
`SpawnFailed`. These error types use allocation-free descriptors when needed.
A panic diagnostic may carry an explicitly runtime-owned trace buffer; failure
to allocate that trace must still preserve the basic panic code.

All tasks require an executor supplied by the program runtime. Its worker count,
allocator, stack budget, and admission limit are explicit runtime configuration,
not inferred from a group or channel capacity. Programs that do not use tasks do
not require an executor. Executor exhaustion produces `SpawnFailed`, and the
language does not promise one operating-system thread per task.

`tasks.Send` means ownership can transfer to another task; `tasks.Sync` means a
shared borrow can be used by another task. See the concurrency reference for
capture and scope rules. A group parameter uses `<tasks.Group<T, N>>` behind an
exclusive reference; it grants submission access only and cannot outlive its
owner or be retained by a child. `group.submit(function)` takes a transferable
zero-argument callable and returns a borrowed ticket, following the same admission
and capture rules as a labeled submission. It cannot seal or join the group.

## Channels

| API                                               | Result                                          | Contract                                                       |
| ------------------------------------------------- | ----------------------------------------------- | -------------------------------------------------------------- |
| `channel.bounded<T>(allocator, capacity <usize>)` | Endpoint record or `<memory.AllocationFailure>` | Allocates a queue for a transferable, owned message type       |
| `sender.clone()`                                  | `<channel.Sender<T>>`                           | Creates another sender owner without growing the queue         |
| `sender.send(value <T>)`                          | `<null><channel.Rejected<T>>`                   | Waits, then transfers the value or returns it in the rejection |
| `sender.try_send(value <T>)`                      | Also `<channel.Full<T>>`                        | Never waits; a full result retains the value                   |
| `receiver.receive()`                              | `<channel.Item<T>><channel.Closed>`             | Waits for an item or drained closure                           |
| `receiver.try_receive()`                          | Also `<channel.Empty>`                          | Never waits                                                    |
| `sender.close()`, `receiver.close()`              | `<null>`                                        | Consumes the endpoint and applies its closure rules            |

The endpoint record has `sender <channel.Sender<T>>` and
`receiver <channel.Receiver<T>>`. `Item<T>`, `Rejected<T>`, and `Full<T>` expose
`value <T>`. Rejection/full errors retain payloads inline, as collection errors do.
`Closed` and `Empty` are allocation-free concrete errors. Endpoint clone count
overflow panics instead of wrapping. Sender and receiver operations need exclusive
access to that endpoint; each producer uses its own sender owner.

## Native access

`ffi.extern<Signature>("C", symbol)` resolves a declared link-time symbol as an
unsafe function pointer. The signature must contain only ABI-compatible types.
`ffi.c_int` and related aliases reflect the build target. There is no implicit
marshalling of strings, records, callbacks, or unions. See
[native interfaces](modules-and-ffi.md#native-interfaces).
