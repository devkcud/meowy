# Errors and custom failures

[Library index](README.md) · [Types](../types.md) · [Failure behavior](../diagnostics.md)

`@"errors"` defines application error types and reads error values without hiding
their payload, ownership, or allocation cost. Define a failure once, construct it
with typed facts, return it in a union, and match the value at the point where
the application can recover. Errors do not throw, print, exit, or unwind a scope
by themselves.

Use concrete alternatives such as `<uint8><TooYoung>` when callers need the
failure's data. Use `<error>` at a deliberately erased boundary, after checking
whether the concrete representation fits or needs explicit boxing. A type
predicate recognizes an error without performing that storage conversion.

The [custom-errors project](../../programs/custom-errors/README.md) contains a
complete module, entry, and manifest. It parses three inputs, defines two custom
failures, preserves a parser error as a typed cause, and inspects the results
without allocating.

## Define and construct an error

```meowy
errors : @"errors"

<AgeDetails> : <{
    age <uint8>
    minimum <uint8>
}>

too_young : errors.define<AgeDetails>({
    -> code : "signup.too_young"
    -> message : "Age must be at least 18"
})

<TooYoung> : <too_young.Error>

check_age <uint8><TooYoung> : (age <uint8>) 'result {
    | age < 18 | {
        'result -> too_young.make({
            -> age : age
            -> minimum : 18
        })
        'result.leave()
    }

    -> age
}
```

`errors.define<P>(spec)` is a pure compile-time constructor. It produces a
definition value with a concrete nominal type `<definition.Error>` and a
`definition.make(payload <P>)` function. The function creates that error at
runtime, storing `P` inline. Neither defining the type nor constructing its inline
payload asks for an allocator. `P` must be a concrete, sized type; recursive
payloads follow the ordinary indirection rule.

The definition accepts exactly two required fields:

| Field     | Requirement                                                                                      | Purpose                                                                                   |
| --------- | ------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| `code`    | Static string of at least two dot-separated components; each component matches `[a-z][a-z0-9_]*` | Stable application identifier, for example `signup.too_young` or `settings.invalid_port`. |
| `message` | Nonempty static UTF-8 string                                                                     | Human explanation shared by all values of this error type.                                |

Runtime interpolation cannot supply these fields. Put changing facts in `P`,
then format those facts through a writer. Invalid descriptors, including unknown
fields or malformed codes, produce `E217`; a runtime-dependent definition also
violates the compile-time type rule `E211`.

Each definition site and complete compile-time instantiation creates one nominal
error identity within its canonical module. The instantiation includes `P` and
the descriptor's code/message values: a pure helper using the same site with
different static metadata produces different types, while repeating the same
instantiation preserves identity. Two independently declared definitions
remain different types even if their metadata and payload shapes agree. Reusing
the definition through a binding alias, type alias, or repeated module import
preserves its identity. Definitions cannot create fresh types from runtime input.
Matching uses the resolved type identity, never a comparison of message text.

The generated error is opaque: a record with `code`, `message`, or matching
payload fields cannot forge it. `errors`, `define`, `make`, and `Error` are ordinary
resolved names. Aliasing the intrinsic preserves its behavior; declaring an
unrelated value with the same spelling does not create error types.

For an error with no changing facts, choose `<null>` explicitly:

```meowy
stopped : errors.define<null>({
    -> code : "worker.stopped"
    -> message : "The worker has stopped"
})

failure : stopped.make(null)
```

`make(null)` still supplies its one parameter. No omitted argument, exception
declaration, reserved word, or special return syntax is involved.

## Read the code, message, and payload

Continuing the age example:

```meowy
debug : @"debug"

result : check_age(16)

| result <TooYoung> | {
    details : result.payload()
    debug.print(errors.code(&result))
    debug.print(errors.message(&result))
    debug.print("Received {details.age}; minimum is {details.minimum}")
}
```

This prints:

```text
signup.too_young
Age must be at least 18
Received 16; minimum is 18
```

| API                            | Result                  | Contract                                                                                                |
| ------------------------------ | ----------------------- | ------------------------------------------------------------------------------------------------------- |
| `errors.define<P>(spec)`       | Compile-time definition | Exposes `<definition.Error>` and `definition.make(payload)`.                                            |
| `definition.make(payload <P>)` | `definition.Error`      | Copies a copyable payload or moves an owner into inline storage; no allocation.                         |
| `errors.code(value <&E>)`      | `string`                | Borrows metadata from a statically proven error type or error-only union. Does not erase or consume it. |
| `errors.message(value <&E>)`   | `string`                | Borrows the common human explanation under the same conditions.                                         |
| `failure.payload()`            | `&P`                    | Shared borrow of a generated error's concrete payload.                                                  |
| `failure.take_payload()`       | `P`                     | Consumes a generated error and transfers its payload, leaving no error owner behind.                    |

`E` in the metadata functions can be a generated error, a library error, the
common `<error>` descriptor, or a union containing only error alternatives.
These are compiler-checked intrinsic constraints; arbitrary records with similarly
named fields do not qualify. No temporary `<error>` box is constructed to read
metadata. A success/error union must first be narrowed to its error alternatives.

`payload` and `take_payload` are generated for `errors.define` types. Existing
library errors keep the payload accessors documented by their own modules;
metadata inspection does not invent a common payload shape for them.

The payload borrow cannot outlive or overlap destruction/movement of its error
owner. Copyable fields such as `details.age` may be copied through that borrow;
an owned string, channel endpoint, or collection cannot be moved out of it.
End dependent borrows before `take_payload()`. Metadata strings reside in static
storage, but the inspection functions retain the conservative returned-borrow
contract: their views are bounded by the input borrow at the call site.

Definitions are immutable and reusable. A generated error has `memory.Copy`
exactly when `P` does, and is transferable only when its payload and retained
storage satisfy the [task transfer rules](../tasks-and-channels.md#transfer-and-synchronization).
Borrowed text inside a payload still borrows its original input; making an error
does not extend that input's lifetime.

## Match specifically, catch broadly, and propagate

A specific matcher exposes the corresponding payload. A broad matcher is useful
for forwarding failures without erasing their concrete variants:

```meowy
age_or_failure <uint8><TooYoung> : (age <uint8>) 'result {
    checked : check_age(age)

    | checked <error> | {
        'result -> checked
        'result.leave()
    }

    -> checked
}
```

`checked<error>` recognizes `TooYoung`, but its storage remains `TooYoung`.
The declared result retains that concrete alternative, so the emission neither
boxes nor drops the payload. A function taking a borrowed error-only union can
inspect its common metadata the same way. Shared references inspect their
referent in a predicate: `| *failure <TooYoung> |` does not move the error.

`->` initializes the selected result slot; it does not return early. The named
`leave()` in the example ends that path, so the final emission sees only the
success alternative. Logging an error and continuing does not establish that
proof. A successful result is never recovered by subtracting `<error>` from a
type without matching the actual value.

An application may deliberately use a plain record as a failure alternative.
The [packet decoder's Truncated](../../programs/packet/codec/header.mwy) and
[JSON report's InvalidSession](../../programs/json-report/report.mwy) do that.
Their callers match those record types explicitly. An ordinary record is not
recognized by `<error>` merely because it is named `Failure` or contains a reason.
Use `errors.define` when broad error matching and common metadata are wanted.

## Add context while preserving a typed cause

Context is data. Put the original error in a typed payload rather than flattening
it into a string:

```meowy
strings : @"strings"

<ParseDetails> : <{
    text <string>
    cause <strings.ParseError>
}>

invalid_age : errors.define<ParseDetails>({
    -> code : "signup.invalid_age"
    -> message : "Age must contain decimal digits from 0 to 255"
})

<InvalidAge> : <invalid_age.Error>

read_age <uint8><InvalidAge><TooYoung> : (text <string>) 'result {
    parsed : strings.to_uint8(text)
    | parsed <strings.ParseError> | {
        'result -> invalid_age.make({
            -> text : text
            -> cause : parsed
        })
        'result.leave()
    }

    -> check_age(parsed)
}

result_with_context : read_age("twenty")
| result_with_context <InvalidAge> | {
    details : result_with_context.payload()
    debug.print("Input: {details.text}")
    debug.print(errors.message(&details.cause))
}
```

`cause` is an ordinary payload field, with a concrete parser-error type. A
different operation can use a different cause type or a closed union. If the
payload borrows external input, that input must outlive the returned error. The
literal in this example has static storage; a view into a temporary builder
would be rejected. To retain dynamic text independently, explicitly construct
`strings.Owned` with an allocator and move that owner into the payload.

Matching the outer error does not also match the cause. Inspect the payload and
match its cause deliberately. There is no automatic search through arbitrary
fields, hidden linked list, or exception ancestry. This keeps traversal order,
ownership, and the number of inspected failures visible in ordinary code.
Several failures can likewise be retained in a concrete bounded list or record;
aggregation does not require allocating an opaque “joined error.”

## Choose inline storage or explicit erasure

The common `<error>` representation is a fixed-size descriptor with a nominal
tag, static metadata, and an optional owned payload box. It is not an inline union
large enough for every possible user payload. A concrete error can be larger,
retain borrowed data, or own resources while still satisfying an `<error>` test.

For generated errors, `P = null` is descriptor-compatible. Every other `P` stays
inline and requires explicit boxing before storage as `<error>`, even when that
particular payload is small. This rule has no target-dependent size cutoff.
Foundational-library errors use this closed compatibility rule, independent of
target layout. The following types are descriptor-compatible:

- `strings.ParseError` and `cli.InvalidValue`;
- `time.Stopped`, `iter.Overflow`, `channel.Closed`, and `channel.Empty`;
- `tasks.Cancelled`, `tasks.Timeout`, `tasks.Panicked`, and `tasks.SpawnFailed`.

Every other foundational-library error is **not descriptor-compatible**, including
new error types unless a later library contract explicitly adds them to this
list. In particular, `io.Error`, `collections.Bounds`, `memory.AllocationFailure`,
date/calendar errors, JSON errors, and generic errors retaining rejected owners
require `errors.box` before storage as `<error>`. Small size, allocation-free
construction, and a particular value having no facts do not change that rule.
The task-runtime descriptors retain their documented runtime-owned evidence;
erasing one transfers that existing evidence and does not allocate another box.

This list governs implicit descriptor-compatible argument/result conversion as
well as `errors.erase`. Error predicates and `errors.code`/`errors.message` work
for every concrete error without erasure. For example, an `io.Error` can be
matched and inspected directly, but `errors.erase` on it is a static error.
The generated-error rule above applies to application and dependency errors;
neither a matching code nor a record shape grants library-error identity.

### Concrete library-error capabilities

Erasure compatibility is independent of copying or task transfer. The following
concrete error families contain only scalar facts, immutable views, or
program-lifetime metadata and have `memory.Copy`, `tasks.Send`, and `tasks.Sync`:

| Module                          | Error types                                                                                                                                       |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `strings`                       | `ParseError`, `InvalidUtf8`, `SliceError`, `InvalidSeparator`, `BufferTooSmall`                                                                   |
| `unicode`                       | `InvalidScalar`                                                                                                                                   |
| `numbers`, `math`               | `numbers.RangeError`, `math.DomainError`, `math.RangeError`                                                                                       |
| `random`, `encoding`, `hash`    | `random.InvalidBound`, `random.EntropyError`, `encoding.InvalidData`, `encoding.BufferTooSmall`, `hash.LimitError`                                |
| `json`                          | `ParseError`, `TypeError`, `NumberError`, `EncodeError`                                                                                           |
| `collections`, `iter`, `memory` | `collections.Bounds`, `iter.Overflow`, `memory.AllocationFailure`                                                                                 |
| `time`                          | `RangeError`, `ParseError`, `FormatError`, `Stopped`                                                                                              |
| `date`                          | `InvalidDate`, `InvalidTime`, `RangeError`, `ClockError`, `UnknownZone`, `InvalidOffset`, `Ambiguous`, `Nonexistent`, `ParseError`, `FormatError` |
| `calendars`                     | `UnknownCalendar`, `CalendarMismatch`                                                                                                             |
| System modules                  | `io.Error`, `path.InvalidPath`, `path.Error`, `fs.Error`, `env.Error`, `process.Error`, `net.InvalidAddress`, `net.Error`                         |
| `cli`                           | `InvalidValue`, `UsageError`, `RenderError`                                                                                                       |
| `channel`, `tasks`              | `channel.Closed`, `channel.Empty`, `tasks.Cancelled`, `tasks.Timeout`, `tasks.SpawnFailed`                                                        |

An immutable view still retains its source lifetime; in particular UsageError's
argv borrows cannot be sent as non-static external borrows through a channel.
The grant does not make those views static. These errors have no ordinary
equality unless their own chapter explicitly grants it; inspect promised facts.

`collections.Full<T,N>`, `Missing<T,N>`, `PushFailure<T>`, `Duplicate<K,V>`,
`InsertFailure<K,V>`, `channel.Rejected<T>`, `channel.Full<T>`,
`dynamic.BoxFailure<T>`, and `errors.BoxFailure<E>` derive each of these three
capabilities from every retained type argument. Their additional numeric facts
and allocation-failure causes impose no further restriction. Retained owners
still move rather than copy whenever their type lacks `memory.Copy`.

`tasks.Panicked` and `process.RunFailure` are move-only and have `tasks.Send` and
`tasks.Sync`; the first owns immutable runtime evidence, and the second owns its
captured byte vectors, completed status/termination evidence, and concrete system
failure facts. They contain no erased application error or non-static external
borrow. No other opaque error obtains a capability implicitly: generated errors
follow their payload rule, and new library types must publish their grants.

### Erasure APIs

For the no-payload definition above, the erased boundary needs no allocation:

```meowy
erased_stop <error> : errors.erase(stopped.make(null))
debug.print(errors.code(&erased_stop))
```

| API                                | Result                                   | Contract                                                                                                                             |
| ---------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `errors.erase(value <E>)`          | `error`                                  | Moves/copies a descriptor-compatible concrete error into the common representation without allocating. Other `E` types are rejected. |
| `errors.box(value <E>, allocator)` | `errors.Boxed` or `errors.BoxFailure<E>` | Explicitly allocate storage for one concrete error; preserve its nominal identity and all payload facts.                             |
| `errors.view<E>(value <&error>)`   | `&E` or `null`                           | Compare the stored nominal identity and borrow the concrete error if it matches; never consume or allocate.                          |
| `errors.take<E>(value <error>)`    | `E`                                      | After a proven matching type test, consume the descriptor, recover the concrete owner, and release box storage if present.           |
| `box_failure.value()`              | `&E`                                     | Borrow the original value retained after allocation failure.                                                                         |
| `box_failure.cause()`              | `&memory.AllocationFailure`              | Borrow the allocation-failure facts without another allocation.                                                                      |
| `box_failure.take_value()`         | `E`                                      | Consume the boxing failure and recover the original value for retry or another handling path.                                        |

`errors.Boxed` is a success record containing `value <error>`. It is deliberately
not an error: `errors.box` distinguishes a successfully boxed original failure
from failure to allocate its box. After excluding `BoxFailure<E>`, move the
record's `value` field into the erased boundary. The record has ordinary field
cleanup and no additional allocation. `BoxFailure<E>` is a concrete inline error
retaining the original `E` and its allocation cause; it must itself stay concrete
unless explicitly boxed. Boxing does not recursively retry a failed allocation.
Its common code is `errors.box_failed` and message is `Could not allocate error
storage`; the original error's metadata remains accessible through `value()`.

Boxing accepts one concrete error type, not a success/error union or an already
erased descriptor. Each successful `box` allocates once, even if `E` could have
used `erase`; callers choose the cheaper operation explicitly. A moved argument
is available only through the success descriptor or the failure's retained value.
The allocator must outlive both the resulting owner and any views into its box.
Boxing borrowed data retains the original borrow requirements too; it does not
copy the referent or make a non-static borrow suitable for a channel.

For example, these named scopes keep every allocation outcome explicit:

```meowy
memory : @"memory"

'inspect {
    original : too_young.make({ -> age : 16; -> minimum : 18 })
    boxed : errors.box(original, memory.heap)

    | boxed <errors.BoxFailure<TooYoung>> | {
        retained : boxed.value()
        debug.print(errors.message(retained))
        debug.print("Keeping the concrete error: boxing could not allocate")
        'inspect.leave()
    }

    erased <error> : boxed.value
    'view {
        found : errors.view<TooYoung>(&erased)
        | found <null> | 'view.leave()
        details : found.payload()
        debug.print("The retained age is {details.age}")
    }

    | erased <TooYoung> | {
        recovered : errors.take<TooYoung>(erased)
        payload : recovered.take_payload()
        debug.print("Recovered inline payload: {payload.age}")
    }
}
```

For `view<E>` and `take<E>`, `E` must be one concrete nominal error type. A success
type, structural payload record, union, or `<error>` itself is not an extraction
target. A type alias of that concrete error is valid and keeps the same identity.
`view` checks the tag and returns a shared reference; it can be used without an
earlier matcher. `take` needs the explicit proof for the same owner, and outstanding
borrows must have ended. Matching an erased descriptor does not unbox it, and a
proven type ascription cannot replace the consuming extraction. A borrowed
descriptor cannot be used to move out the hidden owner.

An erased parameter or result still carries inferred lifetime obligations for
any hidden borrowed payload and allocator. Passing it across a function boundary
does not discard those obligations or turn the hidden data into static storage.

Dropping a common error destroys its active payload exactly once and releases
the retained box with its original allocator. `<error>` is not `memory.Copy` or
`tasks.Send`: its erased payload may own non-copyable or non-transferable state.
Preserve concrete error types across tasks when their transfer capability matters.
No error representation promises a C ABI or a stable serialized memory layout;
use `memory.size_of<T>()` and `memory.align_of<T>()` for the selected target.

## Print errors and choose application policy

The default display of a custom error is `code: message`. For example:

```text
signup.too_young: Age must be at least 18
```

`debug.print(failure)` borrows the error and adds a newline. For recoverable output
failure, use `fmt.write` or `fmt.write_line` with an explicit writer and handle
the returned `io.Error`. Formatting the error's common metadata allocates no
language heap storage. A writer can still allocate according to its own contract,
and a failed streamed write may already have emitted a prefix.

Default display does not inspect private payloads, invoke application getters,
walk causes, or expose the original input automatically. Read the typed payload
and choose which fields to print. To store a changing message, use an explicitly
allocated string builder or caller-owned output buffer; a `<string>` view cannot
outlive its backing storage.

Application codes identify domain failures. They are not meowy compiler codes
such as `E207`, saved occurrence numbers such as `1`, or process exit statuses.
Two nominal types can publish the same application code, so compare types for
in-process handling. Keep codes stable for logs or an explicitly designed wire
protocol; wording may change, and neither a code nor serialized bytes can recreate
a nominal owner without a validating constructor.

Return recoverable failures to the layer that can choose retry, fallback, or an
exit status. Merely constructing, returning, printing, or dropping an error does
not create a compiler diagnostic or replay capsule. An uncaught panic is a
different failure path. The [failure reference](../diagnostics.md) and
[CLI workflow](../../cli/README.md) describe capture and reproduction.
