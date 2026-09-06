# Worked programs

[Documentation index](../README.md)

These small programs share the reference's syntax and library contracts. Their
expected behavior is explained below, including failure paths and storage costs.
All imports resolve to a foundational module or a companion file in this directory.

The [sample manifest](../../mod.sample.mwy) selects `main.mwy` as the entry and
exports the packet module. To select another listing, set `build.entry` to that
file's path. The task and channel programs additionally require an
[executor configuration](../reference/modules-and-ffi.md#build-settings).
The [CLI guide](../cli/README.md#check-build-and-run) covers entry selection and
the check, build, and run commands.

## Validate an age

[age.mwy](age.mwy) parses text once, keeps the parsed byte separate from the input,
and validates both bounds. Its output is:

```text
Welcome
Below the minimum age
Above the maximum age
Enter a whole number from 0 to 255
```

| Input | Result |
| --- | --- |
| `"18"` or `"100"` | Accepted; endpoints are inclusive |
| `"17"` | Below the minimum |
| `"101"` | Above the maximum |
| `"256"`, `"-1"`, `"twenty"`, or `""` | Parse error |

The options contract requires `minimum <= maximum`; the supplied options satisfy
it. All returned text is static borrowed storage. Each rejection emits to the
function's labeled body and leaves it, so no path emits twice. See
[union narrowing](../reference/types.md#unions-and-narrowing).

## Decode a binary header

[packet.mwy](packet.mwy) exports a decoder; [main.mwy](main.mwy) supplies bytes and
formats its result. The custom wire format is four bytes:

| Byte offset | meowy position | Meaning |
| --- | --- | --- |
| 0 | 1 | Version |
| 1 | 2 | Flags |
| 2 | 3 | High byte of payload size |
| 3 | 4 | Low byte of payload size |

The size is big-endian. The input `[1, 0, 1, 44]` produces version `1`, flags `0`,
and payload size `300`. Fewer than four bytes produce `Truncated` with the
required and actual lengths. Extra bytes are permitted; this routine reads the
header only, without validating the availability of the announced payload.

The decoder uses a borrowed slice, explicit widening, and a shift. It allocates
no dynamic storage and does not depend on native record padding, alignment, or
host endianness. It returns a closed structural union, demonstrating that a
domain alternative need not be an exception or an erased error object.

## Collect ordered work

[tasks.mwy](tasks.mwy) submits three squares into a group with capacity four. If
all tasks complete before cancellation, its output is:

```text
Result 1: 4
Result 2: 9
Result 3: 16
```

Positions follow submission order. A deadline, panic, or executor admission
failure occupies the corresponding position with its typed error. Group capacity
is separate from worker count and stack storage. The group is joined once before
results are examined.

The result list is read by reference because task diagnostics can own trace
storage. This avoids copying a non-copyable error payload. All outcomes and their
resources remain owned by the result list until its scope ends.

## Send bounded messages

[channel.mwy](channel.mwy) sends `1`, `2`, `3`, and `4` through a queue that holds
two messages. A producer and consumer run concurrently; the parent joins both.
On normal completion it prints `Sum: 10`.

The parent moves each endpoint into exactly one child and keeps no sender clone.
Producer completion closes the last sender, the consumer drains the queue, and
then `Closed` ends its loop. The parent can join the producer first because a
separately running consumer drains the queue. Both use a common deadline.

If a child cannot start, its captures are released: dropping the sender closes
the send side, and dropping the receiver wakes a blocked producer with rejection.
The parent observes both join outcomes. A partial sum can be printed alongside a
producer error; applications requiring all-or-nothing results must reject that
partial sum explicitly.

The only application allocation is the explicit bounded queue. Task scheduling
has its separately configured runtime storage cost. No message allocates queue
storage, and neither child borrows stack data from a scope that could disappear.
