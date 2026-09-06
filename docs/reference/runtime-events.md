# Runtime event operation registry

[Recording profile](replay-recording.md) · [Schema](../schemas/runtime-event.schema.json)

Every row fixes the complete `arguments` and `outcome.data` keys for that operation.
Unknown keys are rejected. The schema enforces these shapes conditionally on
`operation`; no operation inherits an arbitrary API-shaped JSON dictionary.
Unless a row is `unsupported` or an external environment snapshot, `outcome.kind`
is `result`. `resource` is the logical owner/endpoint receiving the operation, or
null for an operation without one. Decimal strings shown as u64/i64 must fit those
ranges. The shared event `task` field is the actor, never the selected child.

| Operation                   | arguments                                                                          | outcome.data                                                                                                                  |
| --------------------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `task.admit`                | `child`: integer; `callable`: string; `capture_count`: integer; `stack_bytes`: u64 | `admitted`: boolean; `error`: error or null                                                                                   |
| `task.dispatch`             | `worker`: integer                                                                  | empty record                                                                                                                  |
| `task.suspend`              | `reason`: join/channel/io/timer/checkpoint; `waiting_on`: string or null           | empty record                                                                                                                  |
| `task.resume`               | `worker`: integer; `reason`: ready/io/timer/cancel                                 | empty record                                                                                                                  |
| `task.join`                 | `child`: integer                                                                   | `state`: completed/cancelled/timeout/panicked/spawn_failed                                                                    |
| `task.cancel`               | `child`: integer; `reason`: explicit/deadline/parent/group                         | `accepted`: boolean                                                                                                           |
| `task.checkpoint`           | empty record                                                                       | `cancelled`: boolean; `reason`: explicit/deadline/parent/group or null                                                        |
| `task.cleanup`              | `owner`: string; `cause`: scope/panic/cancellation                                 | empty record                                                                                                                  |
| `task.settle`               | `state`: completed/cancelled/timeout/panicked/spawn_failed; `result_type`: string  | empty record                                                                                                                  |
| `channel.create`            | `capacity`: u64; `message_type`: string                                            | `channel`: string or null; `sender`: string or null; `receiver`: string or null; `error`: error or null                       |
| `channel.clone`             | `channel`: string                                                                  | `sender`: string                                                                                                              |
| `channel.send`              | `channel`: string; `message`: string; `blocking`: boolean                          | `state`: sent/full/closed/cancelled                                                                                           |
| `channel.receive`           | `channel`: string; `blocking`: boolean                                             | `state`: item/empty/closed/cancelled; `message`: string or null                                                               |
| `channel.close`             | `channel`: string; `endpoint`: sender/receiver                                     | empty record                                                                                                                  |
| `clock.monotonic`           | empty record                                                                       | `domain`: string or null; `nanoseconds`: i64 or null; `error`: error or null                                                  |
| `clock.civil`               | empty record                                                                       | `timestamp`: timestamp or null; `error`: error or null                                                                        |
| `clock.sleep`               | `domain`: string; `deadline_ns`: i64                                               | `wake_ns`: i64 or null; `error`: error or null                                                                                |
| `clock.timer`               | `domain`: string; `deadline_ns`: i64                                               | `wake_ns`: i64 or null; `error`: error or null                                                                                |
| `clock.ticker`              | `domain`: string; `scheduled_ns`: i64; `period_ns`: u64                            | `wake_ns`: i64 or null; `skipped`: u64; `error`: error or null                                                                |
| `process.arguments`         | empty record                                                                       | `program`: string or null; `arguments`: [string] or null; `error`: error or null                                              |
| `process.working_directory` | empty record                                                                       | `path`: string or null; `error`: error or null                                                                                |
| `io.stdin.read`             | `capacity`: u64                                                                    | `count`: u64; `eof`: boolean; `error`: error or null                                                                          |
| `io.stdout.write`           | `count`: u64; `bytes_digest`: digest                                               | `count`: u64; `error`: error or null                                                                                          |
| `io.stderr.write`           | `count`: u64; `bytes_digest`: digest                                               | `count`: u64; `error`: error or null                                                                                          |
| `fs.open`                   | `path_base64`: string; `mode`: 'ReadOnly'                                          | `handle`: string or null; `error`: error or null                                                                              |
| `fs.open_native`            | `directory`: string; `name_base64`: string; `mode`: 'ReadOnly'                     | `handle`: string or null; `error`: error or null                                                                              |
| `fs.file.read`              | `capacity`: u64                                                                    | `count`: u64; `eof`: boolean; `error`: error or null                                                                          |
| `fs.file.seek`              | `offset`: i64; `origin`: Start/Current/End                                         | `position`: u64 or null; `error`: error or null                                                                               |
| `fs.file.metadata`          | empty record                                                                       | `metadata`: metadata or null; `error`: error or null                                                                          |
| `fs.file.close`             | empty record                                                                       | `error`: error or null                                                                                                        |
| `fs.entries`                | `path_base64`: string                                                              | `directory`: string or null; `error`: error or null                                                                           |
| `fs.directory.next`         | empty record                                                                       | `state`: entry/end/error; `name_base64`: string or null; `kind`: file/directory/symlink/other or null; `error`: error or null |
| `fs.directory.close`        | empty record                                                                       | empty record                                                                                                                  |
| `env.snapshot`              | empty record                                                                       | `values`: {LANG, LC_ALL, LC_CTYPE, TZ}; `omitted_names`: [string]; `error`: error or null                                     |
| `random.secure_fill`        | `count`: u64                                                                       | `error`: error or null                                                                                                        |
| `memory.allocate`           | `allocator`: string; `allocation`: string or null; `bytes`: u64; `alignment`: u64  | `allocation`: string or null; `succeeded`: boolean                                                                            |
| `memory.resize`             | `allocator`: string; `allocation`: string or null; `bytes`: u64; `alignment`: u64  | `allocation`: string or null; `succeeded`: boolean                                                                            |
| `memory.release`            | `allocator`: string; `allocation`: string                                          | empty record                                                                                                                  |
| `unsupported`               | `api`: string; `site`: string                                                      | `reason`: string                                                                                                              |

A completed boundary receives its sequence number when its result is observable.
A blocking call first records suspension/resumption, then its completed boundary
event; the scheduler consumes those events before the result is supplied. Cleanup
events describe completion of one owner cleanup; a panic/termination during it
instead appears in the saved failure and any preceding events. All event clocks
are virtual on replay.

`error` is null for success, otherwise `{type, code, message, native_code, details}`.
Its nominal type identity and code/message come from the captured distribution;
native_code is a nullable signed 64-bit decimal string. `details` is the tagged
wire value defined below. A reader cannot fabricate a different library error from
that record: the bundled runtime validates and constructs its matching native error
representation. An error outcome leaves optional success fields null; channel and
task states use their listed outcomes rather than a fabricated io error.

Read payload members concatenate to `count` bytes; EOF and partial-progress error
may coexist as permitted by the API. Write arguments hash the full attempted bytes;
write result count names the committed prefix, and its payload members contain
that prefix for visible replay. Successful entropy fills have exactly the requested
payload bytes; refusal records the error and the documented cleared destination.
Argument/working-directory decoding errors carry error details with native bytes
as tagged byte values when required. Filesystem paths/native names use strict padded
base64 of the validated native bytes. Directory names may contain non-UTF-8 bytes.
Metadata timestamps are Unix seconds plus 0..999999999 nanoseconds. Permission
bits preserve Linux mode permission/special bits in 0..4095. No inode or address
becomes a resource identity.

A channel message ID identifies one attempted owner transfer, assigned in send-entry
order. `sent` and matching `item` events refer to the same ID. A failed attempt
does not transfer it. Channel creation returns all three logical IDs or an error.
Clock delivery success supplies wake values; failure supplies null wake values.
Allocation success returns a logical allocation ID; resize keeps its old ID when
it succeeds and leaves the original allocation unchanged when it fails. A failed
allocation has null new allocation; allocator identity and byte/alignment requests
must agree. Runtime shortage during a recorded success is unavailable replay.

## Tagged wire values

`details` never serializes target memory layout. Null and boolean have their own
tags. Integers carry an explicit primitive type and canonical decimal string,
including the full signed/unsigned 128-bit range. The selected target fixes usize/
isize width; readers reject out-of-range values and noncanonical leading signs/zeros.
Float32/float64 carry exactly 8/16 lowercase hexadecimal digits of IEEE-754 bits,
most-significant nibble first; this preserves negative zero, infinities, and NaN
payloads without JSON floating point or host endian dependence. Text is valid UTF-8;
bytes reference inventory payloads. Records carry a compiler-stable type identity
plus unique named fields in UTF-8 byte order; variants add the selected alternative
and its tagged value. Resource values carry type and logical ID, never an address.
The bundled distribution must recognize the type/field shape; an unknown type is
unavailable evidence, not a guessed record. Source nominal identities include the
canonical package/module/definition identity.

Mutex/atomic synchronization, source raw-pointer creation/address/cast/equality/
arithmetic/dereference and unsafe raw-memory operations, direct host calls, and
any externally observing or
synchronizing API absent from this registry
are unsupported recording boundaries in profile 1. Reaching one emits `unsupported`
with the resolved API and source-site ID, marks external dependencies, and preserves
its observed effects only as best-effort evidence. This limits replay completeness;
it does not reject or change a valid program using those APIs.
