# Diagnostics and failure behavior

[Documentation index](../README.md)

meowy distinguishes a rejected program, a recoverable failure value, and a panic.
The distinction is part of an API's contract and must not change with build profile.

## Static errors

A static error means the source cannot satisfy a language rule. Diagnostics
identify the operation, the declaration or constraint that it violates, and the
relevant types or ownership path.

| Rejected operation                                  | Reason                                             |
| --------------------------------------------------- | -------------------------------------------------- |
| Assigning text to an integer binding                | Binding types do not change on reassignment        |
| `value<>!<error>` used to conceal a possible error  | Type subtraction is not runtime validation         |
| Emitting twice into a possibly shared primary slot  | Emission initializes once per path                 |
| Missing a required field on a completing path       | Result construction must be complete               |
| Appending a known fourth item to `<T[3]>`           | Inline capacity cannot grow                        |
| Returning a reference to a local list               | Referent is destroyed before the caller can use it |
| Using an owner after sending it through a channel   | The successful transfer moved the owner            |
| Joining a task twice or returning it from its owner | A task has one scoped result owner                 |
| Expanding conflicting fields into one record        | Field selection must have one static meaning       |

An example diagnostic presentation:

```text
error: bounded list capacity exceeded
  --> main.mwy:2:9
   |
 1 | names <string[3]> := ["Ada", "Dev", "Lin"]
   |       ----------- capacity is three
 2 | names = names.add("Sam")
   |         ^^^^^^^^^^^^^^^^ appending requires a fourth slot
   |
   = choose a larger inline capacity or explicitly allocate a vector
```

This is a diagnostic format example, not captured command output. Diagnostic
identifiers must be stable once assigned; the prose here does not allocate a
numbered error catalog.

## Recoverable failures

Parsing invalid text, failing to allocate, or sending to a closed receiver produces
an error union where the API promises one. Callers can match, propagate, or map
that result. Error values do not automatically jump out of a scope.

```meowy
strings : @"strings"

parse_age <uint8><error> : (text <string>) {
    -> strings.to_uint8(text)
}
```

The explicit result permits both the integer and the allocation-free parse error.
For errors retaining large owned inputs, keep the concrete error type in the union
or explicitly box its payload before erasing it to `<error>`.

An ignored owned failure is still released correctly. APIs returning a rejected
message or unchanged collection preserve ownership in their error variant so a
caller can retry. Application policy decides whether to retry, report, or stop.

## Panics

Dynamic bounds violations, checked arithmetic violations, failed assertions, and
`debug.panic` raise a panic. A task boundary converts a recoverable child panic
into `tasks.Panicked` after cleanup. An uncaught root panic terminates the program.
Panics are not an alternative control flow for handling expected invalid input.

The compiler diagnoses a provable violation statically; the corresponding runtime
check still exists when the violation depends on runtime input. Optimization may
remove a proven redundant check, but release builds cannot silently introduce
unchecked arithmetic or indexing.

Unsafe memory violations have no recovery guarantee. A panic while already
releasing resources is fatal. Cancellation is cooperative task unwinding with a
typed outcome, not a panic caused by arbitrary thread interruption.

## Helpful repairs

A suggested repair must describe its behavioral effect. Increasing inline capacity
uses more storage. Changing to a vector introduces allocation and allocation
failure. Narrowing a union requires a matching control-flow proof.

Tools must not guess that a textual age should become zero, replace a failed
allocation with a null owner, or delete an error alternative to make a program
type-check. Repairs should be reviewable edits with source locations; neither
formatting nor diagnostics may silently change application behavior.
