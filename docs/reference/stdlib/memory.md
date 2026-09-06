# Storage and owned values

[Library index](README.md)

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

For error-specific erasure, [errors.box](errors.md#choose-inline-storage-or-explicit-erasure)
preserves an error's nominal tag and returns either an owned descriptor in a
success record or `errors.BoxFailure<E>` retaining the original error. Common
metadata inspection and concrete error construction need no such allocation.

The [memory reference](../memory.md) defines ownership, borrowing, raw pointer
preconditions, string lifetimes, and deterministic cleanup. These APIs do not
relax those rules.
