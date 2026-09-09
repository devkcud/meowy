# Errors worth keeping

[Pawterns](README.md)

“Something went wrong” is technically a message. It is also a terrible gift to
the person debugging your program at midnight. Give callers the facts: a concrete
type to match, a code to recognize in logs, and a payload worth inspecting.
We'll start with a stubborn port setting and work up to owning an erased error.

Use the manifest and commands from [your first project](first-project.md).
Recipes one and two share `ports.mwy`; recipe two replaces `main.mwy` and adds
`port-input.mwy`. Recipe three supplies a separate, complete entry file.
`debug.print` keeps the demonstrations short and panics on output failure; see
[explicit output](../reference/stdlib/text-and-data.md#owned-strings-and-formatting)
when a writer failure also needs handling.

## Bad ports deserve better than “something went wrong”

**Problem:** this application's configuration forbids port zero. Tell the caller
which value was rejected without making it scrape numbers out of your prose.

Complete `ports.mwy`:

```meowy
errors : @"errors"

<PortDetails> : <{
    requested <uint16>
}>

invalid_port : errors.define<PortDetails>({
    -> code : "settings.invalid_port"
    -> message : "Port must be between 1 and 65535"
})

-> <InvalidPort> : <invalid_port.Error>

-> validate <uint16><InvalidPort> : (port <uint16>) 'result {
    | port == 0 | {
        'result -> invalid_port.make({ -> requested : port })
        'result.leave()
    }
    -> port
}
```

Complete `main.mwy`:

```meowy
debug : @"debug"
errors : @"errors"
ports : @"./ports.mwy"

'example {
    port : ports.validate(0)
    | port <ports.InvalidPort> | {
        details : port.payload()
        debug.print(errors.code(&port))
        debug.print(errors.message(&port))
        debug.print("Rejected port: {details.requested}")
        'example.leave()
    }
    debug.print("Accepted port: {port}")
}
```

Expected output:

```text
settings.invalid_port
Port must be between 1 and 65535
Rejected port: 0
```

The argument's `uint16` representation already limits its upper bound. The
application checks the remaining rule: zero is forbidden here. Calling with
`8080` prints `Accepted port: 8080`. Both are handled results, so rejection does
not automatically give the process a failing exit status.

The error stores `PortDetails` inline; defining or constructing it needs no heap
allocation. Its descriptor contains static metadata, while `requested` records
the value for this occurrence. `payload()` borrows those facts. The borrow cannot
outlive or overlap movement of its error owner.

Export the generated type alias and reuse it through the module. Defining another
error with the same code, message, and payload shape creates a different nominal
type. Match the type in program logic; reserve stable codes for logs or a wire
protocol you explicitly design. A plain record named `InvalidPort` would not gain
error behavior just from its name.

See [error definition rules](../reference/stdlib/errors.md#define-and-construct-an-error)
and the complete [custom-errors project](../programs/custom-errors/README.md),
which applies this pattern to an age requirement.

## Keep the parser's receipts

**Problem:** someone enters `"nope"` in the port field. Someone else enters `"0"`.
Both need a response, but the parser and the application have different complaints.
Keep those complaints distinct, and preserve the original evidence.

Keep `ports.mwy` from the first recipe. Complete `port-input.mwy`:

```meowy
errors : @"errors"
ports : @"./ports.mwy"
strings : @"strings"

<InputDetails> : <{
    text <string>
    cause <strings.ParseError>
}>

invalid_text : errors.define<InputDetails>({
    -> code : "settings.invalid_port_text"
    -> message : "Port must be a decimal integer from 0 to 65535"
})

-> <InvalidText> : <invalid_text.Error>

-> parse <uint16><ports.InvalidPort><InvalidText> : (text <string>) 'result {
    port : strings.to_integer<uint16>(text)
    | port <strings.ParseError> | {
        'result -> invalid_text.make({ -> text : text; -> cause : port })
        'result.leave()
    }
    -> ports.validate(port)
}
```

Replace `main.mwy` with:

```meowy
debug : @"debug"
errors : @"errors"
input : @"./port-input.mwy"
ports : @"./ports.mwy"
strings : @"strings"

describe <null> : (failure <input.InvalidText><ports.InvalidPort>) 'result {
    debug.print(errors.code(&failure))
    | failure <input.InvalidText> | {
        details : failure.payload()
        debug.print("Unparsed input: {details.text}")
        cause : details.&cause
        | *cause <strings.ParseError> | {
            debug.print("Cause: strings.ParseError")
        }
        'result.leave()
    }
    details : failure.payload()
    debug.print("Rejected port: {details.requested}")
}

show <null> : (text <string>) 'result {
    port : input.parse(text)
    | port <error> | {
        describe(port)
        'result.leave()
    }
    debug.print("Accepted port: {port}")
}

show("nope")
show("0")
show("8080")
```

Expected output:

```text
settings.invalid_port_text
Unparsed input: nope
Cause: strings.ParseError
settings.invalid_port
Rejected port: 0
Accepted port: 8080
```

The broad `port <error>` predicate selects both concrete failure alternatives.
It does not convert their storage to the common `<error>` descriptor. `describe`
therefore receives the same typed union and reads metadata without boxing. Its
specific matcher narrows the payload before accessing it.

`cause` is an ordinary typed field; there is no automatic traversal into nested
errors. The example explicitly borrows and tests that field. The wrapper also
borrows the original input through `text`, so it cannot outlive an input owner.
Here every input is a static literal. For input coming from a builder, finish
handling the failure while that builder is alive, or copy the text into an
explicit owner before retaining it independently.

**Exercise the boundary:** try `"65536"` and `" 8080 "`. The decimal parser
rejects out-of-range input and surrounding whitespace. If trimming is desired,
call `strings.trim` deliberately and decide whether the payload should preserve
the original spelling or the trimmed view.

See [typed causes](../reference/stdlib/errors.md#add-context-while-preserving-a-typed-cause)
and [broad propagation](../reference/stdlib/errors.md#match-specifically-catch-broadly-and-propagate).
The [custom-errors validation module](../programs/custom-errors/validation.mwy)
uses the same separation between parsing and application policy.

## Box the error without losing the evidence

**Problem:** a boundary stores common `<error>` values, but this failure owns text
you want back. Now imagine allocating its box also fails. Errors reporting errors:
the sequel nobody ordered. Keep ownership recoverable on both paths.

This is a separate, complete `main.mwy`; it does not use the previous modules:

```meowy
debug : @"debug"
errors : @"errors"
memory : @"memory"
strings : @"strings"

rejected_name : errors.define<strings.Owned>({
    -> code : "settings.rejected_name"
    -> message : "The setting name is unavailable"
})
<RejectedName> : <rejected_name.Error>

'example {
    text : strings.copy("root", memory.heap)
    | text <memory.AllocationFailure> | {
        debug.print(text)
        'example.leave()
    }

    original : rejected_name.make(text)
    boxed : errors.box(original, memory.heap)
    | boxed <errors.BoxFailure<RejectedName>> | {
        retained : boxed.take_value()
        name : retained.take_payload()
        debug.print("Box allocation failed; retained {name.view()}")
        'example.leave()
    }

    erased <error> : boxed.value
    debug.print(errors.code(&erased))
    'inspect {
        found : errors.view<RejectedName>(&erased)
        | found <null> | 'inspect.leave()
        name : found.payload()
        debug.print("Rejected: {name.view()}")
    }

    | erased <RejectedName> | {
        recovered : errors.take<RejectedName>(erased)
        name : recovered.take_payload()
        debug.print("Recovered: {name.view()}")
    }
}
```

When both allocations and output succeed:

```text
settings.rejected_name
Rejected: root
Recovered: root
```

`strings.copy` creates the text owner; `make` moves it into the concrete error
without allocating again. `errors.box` then attempts a separate allocation for
the erased boundary. Success returns a non-error `errors.Boxed` record whose
`value` field owns the descriptor. Failure retains the original error in
`BoxFailure`, so the recovery branch can take back the text rather than losing it
while trying to report another failure.

The inspection scope borrows the hidden concrete error without consuming it.
After those borrows end, the explicit matcher proves the type for
`errors.take<RejectedName>`. Taking consumes the descriptor and releases its box;
`take_payload()` then consumes the recovered error and transfers the text owner.
Normal cleanup eventually releases that text exactly once.

Prefer a concrete result union when the caller already knows its alternatives.
A non-null generated payload needs explicit boxing for this common representation,
even when the payload is small. For a generated `<null>` payload, use
`errors.erase(definition.make(null))` without allocating. Merely reading a code,
printing an error, or testing `<error>` requires neither operation.

Boxing does not extend hidden borrows or make an erased value transferable to a
task. Keep concrete errors across task boundaries when their transfer capability
matters. Returning or printing these handled errors also does not create a replay
capsule; an uncaught panic is a separate failure path.

See [erasure and extraction](../reference/stdlib/errors.md#choose-inline-storage-or-explicit-erasure),
[ownership](../reference/memory.md), and [diagnostic capture](../reference/diagnostics.md).
Compare the [custom-errors project](../programs/custom-errors/README.md): it retains
concrete alternatives throughout and therefore needs no error-box allocation.
