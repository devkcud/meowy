# Teach the suite to complain usefully

[Pawterns](README.md) · Previous: [Errors worth keeping](errors.md) · Next: [Text and data](text-and-data.md)

Future-you will change a helper at 01:47, say “surely that's equivalent,” and go
to bed. Leave that person a few well-placed tripwires. The goal is useful failure:
what changed, which input exposed it, and how to see it happen again.

These four recipes build one small test project. For a ready-made application
plus the same patterns, use the [testing project](../programs/testing/README.md).
The [testing reference](../reference/stdlib/testing.md) defines the full contract.

## Four rows, one very opinionated calculator

**Use this when:** a pure helper has several boundary cases and you want each
assertion to report the input that betrayed you.

Create a directory with this complete `mod.mwy`. A test-only project does not
need an application entry:

```meowy
-> build : {
    -> profile : "debug"
}

-> test : {
    -> version : 1
    -> paths : ["./tests"]
    -> processes : 1
    -> timeout_ms : 30_000
    -> output_bytes : 65_536
    -> seed : 2026
}
```

Complete `arithmetic.mwy`:

```meowy
errors : @"errors"

<OddDetails> : <{
    requested <uint32>
}>

odd_input : errors.define<OddDetails>({
    -> code : "arithmetic.odd_input"
    -> message : "An exact half needs an even input"
})

-> <OddInput> : <odd_input.Error>

-> half_even <uint32><OddInput> : (value <uint32>) 'result {
    | value % 2 != 0 | {
        'result -> odd_input.make({ -> requested : value })
        'result.leave()
    }
    -> value / 2
}
```

Create `tests/arithmetic_test.mwy`:

```meowy
arithmetic : @"../arithmetic.mwy"
testing : @"testing"

<Row> : <{
    input <uint32>
    expected <uint32>
}>

-> tests : testing.suite({
    -> halves_table : testing.case(() {
        rows <Row[4]> : [
            { -> input : 0; -> expected : 0 },
            { -> input : 2; -> expected : 1 },
            { -> input : 42; -> expected : 21 },
            { -> input : 65_534; -> expected : 32_767 }
        ]
        index <usize> := 1

        'rows {
            | index > rows.size() | 'rows.leave()
            row : rows[index]
            actual : arithmetic.half_even(row.input)
            | actual <arithmetic.OddInput> | {
                testing.fail("Even input was rejected in row {index}")
            }
            expected : row.expected
            testing.equal(&actual, &expected, "Wrong half in row {index}")
            index = index + 1
            'rows.restart()
        }
    })
})
```

Run from that project directory:

```sh
meowy test --list
meowy test
```

The listing contains `tests/arithmetic_test.mwy::tests::halves_table`. The run
passes one case containing four table rows. A table row is not a separate process
or selectable case: name separate `testing.case` values when that separation
matters. The loop and rows are ordinary language values.

`testing.equal` borrows both values, compares their full value, and reports a
`P005` panic on mismatch. It stops that case rather than adding a sad boolean to
a drawer you might forget to open. Set row three's expected value to `20` to
exercise the failure: actual `21`, expected `20`, message `Wrong half in row 3`.
Restore `21` afterward. The numeric assertions use the same concrete `uint32`
type; they do not quietly convert one operand to make a comparison work.

The matcher plus `testing.fail`, which never returns, proves that `actual` is a
number afterward. An assertion does not itself refine a union. This distinction
keeps “I checked it” from becoming an unchecked cast wearing a tiny badge.

The exported Suite value makes the runner collect the case. `tests`, `Row`, and
`testing` remain ordinary names. Renaming the exported suite changes its case
IDs; merely editing a body does not. A function named `test_half` without a
Suite export is just a function waiting for somebody to call it.

See [discovery](../reference/stdlib/testing.md) and
[proven union narrowing](../reference/types.md#unions-and-narrowing).

## An error is a result; a panic has flipped the table

**Use this when:** rejection is part of the API, but a separate operation is
supposed to panic. Tell the suite which kind of complaint you ordered.

Keep the preceding files and add this complete `tests/failures_test.mwy`:

```meowy
arithmetic : @"../arithmetic.mwy"
debug : @"debug"
errors : @"errors"
testing : @"testing"

-> tests : testing.suite({
    -> odd_input : testing.case(() 'case {
        result : arithmetic.half_even(7)
        | result <arithmetic.OddInput> | {
            details : result.payload()
            actual : details.requested
            expected <uint32> : 7
            testing.equal(&actual, &expected, "Keep the rejected input")
            testing.text_equal(errors.code(&result), "arithmetic.odd_input",
                "Keep the public error code stable")
            'case.leave()
        }
        testing.fail("Odd input unexpectedly received an exact half")
    })

    -> explicit_panic : testing.panics({
        -> code : "P006"
        -> message : "The calculator ate a sock"
    }, () {
        debug.panic("The calculator ate a sock")
    })
})
```

Run `meowy test --filter failures_test`. Both cases pass. The first receives a
normal error value, matches its concrete type, checks its borrowed payload, and
leaves the case successfully. The second passes because the callback actually
panics with `P006` and the exact requested message.

Neither passing case creates a failure occurrence. If `half_even(7)` starts
returning a number, the first case fails with `P005`. If the expected-panic body
returns normally, the second fails with `T001`. A different panic code or message
does not count as “close enough”; the diagnostic keeps the actual panic and the
expectation that it failed to meet. Case initialization and module cleanup are
outside that expected-panic callback boundary.

This is deliberately an explicit runtime panic. A statically invalid expression
still fails checking before any case can run; `testing.panics` cannot turn source
errors into passing tests. Likewise, a handled `tasks.Panicked` result is a value
you inspected, not a panic escaping the callback.

To reserve a named case without pretending it works, wrap a Case in
`testing.skip("A concrete reason", case_value)`. The descriptor and body are
still checked; the body is not executed. The worked project's
[arithmetic suite](../programs/testing/tests/arithmetic_test.mwy) keeps an
explicitly skipped exhaustive sweep this way. A green run with one skipped case
means one case still did no work. The cat did not check under that sofa.

See [custom errors](errors.md) and the
[diagnostic code catalog](../reference/diagnostic-codes.md).

## Tiny fixtures, no haunted temporary directory

**Use this when:** an I/O algorithm accepts a writer and you need to exercise
partial success. A local three-byte buffer is very good at refusing byte four.

Add this complete `tests/io_test.mwy` to the preceding project:

```meowy
io : @"io"
testing : @"testing"

-> tests : testing.suite({
    -> partial_write : testing.case(() {
        buffer <uint8[3]> := []
        attempt : 'write {
            writer := io.buffer_writer(&!buffer)
            -> io.write_all(writer.write, "meow".bytes())
        }

        expected_count <usize> : 3
        actual_count : attempt.count
        testing.equal(&actual_count, &expected_count, "Keep committed progress")
        | attempt.error <null> | testing.fail("A fourth byte cannot fit")
        testing.bytes_equal(buffer.slice(), "meo".bytes(),
            "The first three bytes survive the capacity failure")
    })
})
```

Run `meowy test --filter partial_write`. The case passes: three bytes were
committed, an `io.Error` reports capacity failure, and the buffer contains `meo`.
The result is not “nothing happened” merely because a later byte could not fit.

The inner scope is doing work. Its writer holds an exclusive borrow of `buffer`;
ending that scope releases the borrow before `bytes_equal` reads the contents.
Returning the small `io.Write` record does not return the writer or extend its
borrow. Everything in this fixture is local or static, with no heap allocation
or host file involved.

For the read side, `io.bytes_reader(bytes)` supplies an equally small borrowed
cursor. Keep its source alive while exercising the read callable. For a combined
copy test, give `io.copy` that reader, a buffer writer, and explicit initialized
scratch; check both progress counts, not just the absence of an error.

`bytes_equal` compares initialized bytes and lengths, including embedded zero
bytes. `text_equal` compares exact UTF-8 bytes, so visually identical Unicode
spellings can differ. When normalization is part of your API, normalize in the
code under test and assert that promised result. The comparison should not
quietly clean the evidence for you.

Each case runs in a fresh process, which separates module state. It does not
invent a private host filesystem or network. Two cases that explicitly open the
same path can still fight over it. These buffer fixtures avoid that shared state
by construction; a real filesystem test needs an explicit path and cleanup
policy appropriate to that test.

See [I/O callables](../reference/stdlib/io-and-system.md#readers-and-writers)
and [partial output](cli-and-files.md#make-a-short-write-visible-before-involving-a-filesystem).

## Make hello world clock out

**Use this when:** a producer transfers an owned message and the consumer must
finish after the last sender closes. “It printed hello” is only half the story.

Add this `executor` record inside the existing `build` block in `mod.mwy`:

```meowy
-> executor : {
    -> workers : 2
    -> max_tasks : 2
    -> stack_bytes : 65_536
    -> allocator : "system"
}
```

Keep the manifest's `test.processes : 1` and `test.timeout_ms : 30_000`.
Create this complete `tests/channel_test.mwy`:

```meowy
channel : @"channel"
memory : @"memory"
strings : @"strings"
testing : @"testing"

drain <usize> : (receiver <channel.Receiver<strings.Owned>>) {
    input := receiver
    count <usize> := 0

    'messages {
        item : input.receive()
        | item <channel.Closed> | 'messages.leave()
        text : item.value
        testing.text_equal(text.view(), "meow", "The receiver owns the greeting")
        count = count + 1
        'messages.restart()
    }

    again : input.receive()
    | again <channel.Item<strings.Owned>> | {
        testing.fail("A drained channel produced another message")
    }
    -> count
}

-> tests : testing.suite({
    -> owned_message_and_close : testing.case(() {
        endpoints : channel.bounded<strings.Owned>(memory.heap, 1)
        | endpoints <memory.AllocationFailure> | testing.fail("Queue allocation failed")
        text : strings.copy("meow", memory.heap)
        | text <memory.AllocationFailure> | testing.fail("Greeting allocation failed")

        output := endpoints.sender
        consumer : >> drain(endpoints.receiver)
        sent : output.send(text)
        output.close()
        received : << consumer

        | sent <channel.Rejected<strings.Owned>> | {
            testing.fail("The receiver rejected the owned greeting")
        }
        | received <error> | testing.fail("The consumer did not finish successfully")
        expected <usize> : 1
        testing.equal(&received, &expected, "Consume exactly one owned message")
    })
})
```

Run `meowy test --filter owned_message_and_close`. With successful allocation and
task admission, it passes. The text moves from parent to channel to receiver;
the receiver checks the bytes, releases the owner, and drains to closure. A
second receive verifies that drained closure stays closed. The parent closes
its only sender before joining and then checks that exactly one message arrived.

There are no scheduling sleeps and no assertion about which worker gets there
first. The queue, sender closure, and join establish the events that matter.
`test.processes` controls concurrent case processes; `build.executor` supplies
tasks within each case. Turning the former up does not repair missing task
configuration or increase a case's admission budget.

**Deliberately break it:** remove `output.close()`. The consumer receives its
message and then waits for more while the parent waits in `<< consumer`, still
owning an open sender. Hello world has clocked in for an eternal shift. The
watchdog eventually reports `T002` and terminates the case. Restore the close.
That watchdog covers initialization through cleanup; it is not a task deadline,
an exact elapsed-time guarantee, or proof that every timeout is a deadlock.
Slow hosts and blocking native calls can also exceed a budget.

After an actual failed run, inspect the test summary. If the failure is labeled
occurrence `1`, these commands select the test session from the project directory:

```sh
meowy err summary --test
meowy err inspect 1 --test --verbose
meowy err reproduce 1 --test --verbose
```

The report keeps the case ID separate from that occurrence number. Replay uses
the captured case and supervision policy; abrupt termination may leave incomplete
evidence, and a scheduling-dependent problem is not made deterministic merely
by saving its binary. A panic in a child becomes an outcome at the join. In this
example, the parent treats any failed outcome as a failed assertion.

For generated inputs in a case, begin with `random : @"random"` at module scope
and `generator := random.seeded(testing.seed())` inside its callback. The seed
is derived from the manifest's base seed and stable case ID and is recorded with
the run. It reproduces the generator's input stream under its recorded algorithm;
it does not freeze the OS scheduler, wall clock, or network. Keep generated inputs
bounded, handle generator failures, and record enough context to identify the
iteration. More random work is not a substitute for the specific closure test
that catches this bug immediately.

See [the spare sender problem](deadlocks-and-shutdown.md),
[task cleanup](../reference/tasks-and-channels.md#scope-exit), and
[the complete testing project](../programs/testing/README.md).
