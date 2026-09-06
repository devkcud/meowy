# Task and channel APIs

[Library index](README.md)

## Tasks

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

See [task and channel semantics](../tasks-and-channels.md) for submission, joins,
capture, endpoint closure, and cancellation.
