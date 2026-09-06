# Compose values without losing their meaning

[Pawterns](README.md) · Previous: [Your first project](first-project.md) · Next: [Ownership](ownership.md)

A number is useful. A number that remembers which sensor produced it is easier
to debug at midnight. Meowy's blocks can carry both without making you choose.

Each recipe below is an independent complete `main.mwy`. Use the minimal
manifest from [your first project](first-project.md#print-a-greeting-from-a-project).

## Carry metadata through a calculation

Let the result keep its name tag. The numeric value goes in the primary slot,
and its origin travels beside it as a field.

```meowy
debug : @"debug"

<Reading> : <{
    -> <int32>
    sensor <string>
}>

<Checked> : <{
    <Reading>
    acceptable <boolean>
}>

label <Reading> : (value <int32>, sensor_name <string>) {
    -> value
    -> sensor : sensor_name
}

check <Checked> : (reading <Reading>) {
    -> reading
    -> acceptable : reading >= -40 && reading <= 85
}

reading : 21.(label, "rack-a").(check)
debug.print("{reading} from {reading.sensor}")
| reading.acceptable | debug.print("Within range")

scalar <int32> : reading
debug.print(scalar)
```

The output is:

```text
21 from rack-a
Within range
21
```

`check` emits the complete incoming value as its primary. Primary composition
retains `sensor` while adding `acceptable`; scalar formatting and comparison use
the numeric primary. `scalar` copies that copyable primary without taking the
metadata away from `reading`.

The aggregate stores its fields inline, and the literal sensor name has static
storage. This does not authorize arbitrary record narrowing: a function expecting
`Reading` cannot silently discard the extra `acceptable` field of `Checked`.
Build a deliberate smaller record when you need a smaller public shape. Likewise,
emitting another `sensor` field while composing `reading` is a collision, not an
override.

See [primary composition](../reference/values-and-blocks.md#primary-composition)
and the [composition project](../programs/composition/README.md).

## Choose exactly one result

Matchers are generous: every matching arm gets a turn. That is excellent for
collecting observations and awkward when you meant “pick one answer, please.”
Give that selection an explicit exit.

```meowy
debug : @"debug"
strings : @"strings"

classify <string> : (text <string>) 'result {
    parsed : strings.to_uint8(text)
    | parsed <error> | {
        'result -> "not a byte-sized age"
        'result.leave()
    }

    | parsed < 18 | {
        'result -> "minor"
        'result.leave()
    }

    -> "adult"
}

debug.print(classify("12"))
debug.print(classify("24"))
debug.print(classify("twenty"))
```

This prints `minor`, `adult`, then `not a byte-sized age`. Leaving the error arm
proves that `parsed` is a byte at the later comparison. Leaving the minor arm
ensures the result is emitted once.

**Invalid variation:** deleting either `'result.leave()` permits execution to
reach another primary emission. `->` initializes a result slot; it never acts as
a return statement. Simply printing an error also leaves the error alternative
in the later flow type.

This recipe deliberately turns a parse failure into a static description. When
callers need the error's identity or offending input, use
[typed failures](errors.md) instead. Parsing here allocates no payload and all
returned strings borrow literals.

See [matchers and flow analysis](../reference/values-and-blocks.md#matchers-and-flow-analysis)
and [named scopes](../reference/values-and-blocks.md#named-scopes-and-cleanup).

## Change presentation without changing type meaning

Your spacebar may take the day off. Type predicates and proven ascriptions still
have different jobs, and the surrounding grammar keeps them straight.

```meowy
debug : @"debug"

increment <int32> : (value <int32>) { -> value + 1 }
double <int32> : (value <int32>) { -> value * 2 }

ordinary : double(increment(20))
pipeline : 20.(increment).(double)
debug.print(ordinary == pipeline)

value <int32><null> : 42
|value<int32>|debug.print(value.{->self<int32>})
```

The output is `true` followed by `42`. The matcher's `value<int32>` tests the
union alternative. Inside its body, `self<int32>` has a proof from that test;
it does not parse or convert anything. Dispatch evaluates its receiver once.

For a complete program with no spaces outside strings, use this separate file:

```meowy
debug:@"debug";value<int32><null>:42;|value<int32>|debug.print(value.{->self<int32>})
```

That file prints `42`. Semicolons preserve the statement boundaries. The
[minimal gatostyle preset](../guide/gatostyle.md#presets-are-starting-values)
can remove horizontal spacing, but readability and preferred call forms remain
project choices. When owners or borrowed parameters enter a pipeline, changing
the shape can change lifetimes and requires more care than this scalar example.

See [angle brackets in context](../reference/syntax.md#angle-brackets-in-context)
and [dispatch](../reference/values-and-blocks.md#dispatch).

## Give a pure helper a small regression driver

Future-you will eventually “simplify” this helper. Leave a small tripwire for
that occasion: an entry that checks both branches and complains if either changes.

Save this complete file as `checks.mwy` beside the project's normal entry:

```meowy
debug : @"debug"

fallback <:T> : (value <T><null>, alternative <T>) 'result {
    | value <null> | {
        'result -> alternative
        'result.leave()
    }
    -> value
}

empty : fallback<int32>(null, 7)
present : fallback<int32>(3, 7)

| empty != 7 | debug.panic("The empty case must use its fallback")
| present != 3 | debug.panic("The present case must keep its value")
debug.print("2 cases passed")
```

From that project directory:

```sh
meowy check checks.mwy
meowy run checks.mwy
```

The successful run prints `2 cases passed`. An explicit entry chooses this file
while retaining the project's other manifest settings. For a real helper, export
it from a module and import that same module in both the application and driver,
so the check exercises the code the application uses.

`meowy check` alone does not execute these case comparisons. A mismatch reaches
`debug.panic` during the driver run and participates in the normal
[failure inspection workflow](toolchain-workflow.md#inspect-and-replay-a-deliberate-failure).
Choose boundary and failure cases as well as happy paths; fixed inputs make the
result repeatable, but two cases are not proof for every possible input.
