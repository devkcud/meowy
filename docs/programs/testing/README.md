# Test the helper, the failure, and the goodbye

[All programs](../README.md) · [Manifest](mod.mwy) · [Entry](main.mwy) · [Arithmetic](arithmetic.mwy)

A tiny calculator gets a disproportionately useful suite: table cases, a typed
error, an expected panic, partial writes, dynamic JSON, and a greeting that lets its
consumer go home. All tests import `@"testing"`; no special test keyword or
magic function name sneaks into the language.

From the repository root:

```sh
cd docs/programs/testing
meowy check
meowy run
meowy test --list
meowy test
```

The application prints `Half of 42: 21`. Testing discovers these IDs in order:

```text
tests/arithmetic_test.mwy::tests::exhaustive_sweep
tests/arithmetic_test.mwy::tests::explicit_panic
tests/arithmetic_test.mwy::tests::halves_table
tests/arithmetic_test.mwy::tests::odd_input
tests/channel_test.mwy::tests::owned_message_and_close
tests/io_test.mwy::tests::partial_write
tests/stdlib_test.mwy::tests::custom_cli_parser
tests/stdlib_test.mwy::tests::custom_writer_failure
tests/stdlib_test.mwy::tests::dynamic_json_numbers
tests/stdlib_test.mwy::tests::json_limit_boundaries
```

The listing also labels the skipped sweep and its reason. With successful
allocation, admission, and host operation, the run finishes with **9 passed,
0 failed, 1 skipped, and 0 not started**, returning status 0. The table's four
rows run inside one case; the expected panic passes without publishing a failure
occurrence. The skipped case is an explicit placeholder, not claimed coverage.
Remove the skip only after replacing its deliberately failing body.

| File                                          | What it checks                                                                                                                   |
| --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| [Arithmetic tests](tests/arithmetic_test.mwy) | Four even inputs, the exact odd-input payload and code, an expected `P006`, and the named skip                                   |
| [I/O test](tests/io_test.mwy)                 | A three-byte destination retains `meo` and reports partial progress when asked to write `meow`                                   |
| [Channel test](tests/channel_test.mwy)        | A `strings.Owned` message transfers once, its contents survive, and the receiver observes permanent drained closure              |
| [Stdlib cases](tests/stdlib_test.mwy)         | A custom CLI parser, a refusing I/O adapter, exact dynamic JSON numbers, missing versus null members, and inclusive input limits |

`main.mwy` and the suite import the same arithmetic module. `meowy test` does not
execute the application entry; `meowy check` checks that entry's graph and does
not execute or discover the suite. `meowy test --no-run` checks the discovered
graphs and builds the case harness without running callbacks.

Each active case gets a fresh process and initializes its own dependency graph.
The table and error payload use inline storage. The I/O cursor borrows a local
bounded list; its inner scope ends before assertions borrow the resulting bytes.
That fixture never touches a host file.

The stdlib cases construct `cli.InvalidValue` through `cli.invalid` and `io.Error`
through `io.error`; ordinary record literals cannot forge those nominal errors.
The writer handles an empty span successfully, then rejects nonempty output with
zero committed bytes and an explicit portable kind. Dynamic JSON keeps a number
larger than uint64 as source text, rejects its checked uint64 conversion, and
distinguishes a missing member from JSON null. These cases use caller-owned argv
and document-owned JSON data; no external files or services are required.

The concurrency case allocates its queue and greeting before submitting work.
`send(text)` moves the text owner; the parent cannot reuse it. The receiver owns
and releases each received string. Closing the only sender **before** joining
allows the receive loop to finish, and the explicit join exposes task failure
before the parent checks its result. Removing that close can block until the
case watchdog ends the process; passing a message is not a shutdown protocol.

The manifest permits one case process at a time. Each case executable uses the
explicit two-worker, two-task executor when required. `test.processes` controls
case isolation and concurrency; it does not supply task workers. The 30-second
watchdog covers startup, initialization, callback, joins, and cleanup. It is a
host supervision limit, not a promise that a blocked task can unwind in time.

Run just the channel case, still checking the discovered test graphs first:

```sh
meowy test --filter owned_message_and_close
```

An unexpected assertion failure is `P005`. If a failed run's test summary labels
it occurrence `1`, inspect its saved evidence from this project directory:

```sh
meowy err summary --test
meowy err explain 1 --test
meowy err reproduce 1 --test --verbose
```

Occurrence `1` is a diagnostic identifier, not the first case in the listing.
Watchdog termination reports `T002`; killed execution may leave only supervisor
evidence. The report states what was captured instead of inventing a completed
unwind. The preserved seed identifies generated inputs, when used; it does not
control task scheduling.

See the [testing reference](../../reference/stdlib/testing.md),
[testing Pawterns](../../pawterns/testing.md),
[typed errors](../../reference/stdlib/errors.md), and
[channel shutdown rules](../../reference/tasks-and-channels.md#channels).
