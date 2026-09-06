# A ticker with a slow consumer

[Worked projects](../README.md)

Observe three ticks from a 100 ms ticker, deliberately waiting 250 ms after the
first observation. The second call demonstrates what happens when a consumer
falls behind: the ticker reports the most recent due slot and counts omitted
slots in `skipped`.

From the repository root:

```sh
cd docs/programs/ticker
meowy check
meowy run
```

[mod.mwy](mod.mwy) selects [main.mwy](main.mwy). Copy this directory elsewhere and
the same commands work without a repository-level manifest.

Successful output contains three lines, each with the observation number,
scheduled and observed elapsed durations, and skipped-slot count. For example,
the shape of a line is:

```text
tick 2: scheduled=<elapsed duration>, observed=<elapsed duration>, skipped=<count>
```

Those angle-bracketed descriptions are placeholders, not literal program output.
Exact times and skip counts depend on clock resolution and scheduling. The 250 ms
pause spans at least two 100 ms periods, so the second observation skips at least
one slot. Additional delays can skip more, including before the first observation.
`scheduled` stays on the ticker's original schedule; `observed` records when the
caller received the tick. Both are measured from the same monotonic reading taken
just before ticker construction.

The program counts **observations**, so it finishes after three lines even when
more than three periods elapsed. A ticker coalesces missed slots; it does not
build a growing queue or launch a callback for each missed tick. Use a channel
and an explicit producer when every produced message must be retained instead.

The ticker constructor exposes its allocation through `memory.heap`. The owner
is move-only, each wait requires exclusive access, and `close()` releases its
state on the normal path. A failure leaves the named `main` scope, which drops
and disarms the owner automatically. No task is spawned and no executor is
configured; these waits block the calling host thread.

Allocation and invalid-period errors, a stopped ticker, and output errors are
handled with status 1. Error reporting uses `debug.print`, whose own output
failure panics. The fixed positive period avoids an invalid interval in this
example, and the program never explicitly stops the ticker before its last wait.
Success emits status 0.

See [owned timers and tickers](../../reference/stdlib/time-and-date.md#owned-timers-and-tickers)
and [monotonic clocks](../../reference/stdlib/time-and-date.md#clocks-deadlines-and-waits).
