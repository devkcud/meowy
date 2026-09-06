# Collect ordered task results

[All programs](../README.md) · [Manifest](mod.mwy) · [Entry](main.mwy) · [Worker](work.mwy)

Three calls from a reusable worker module run in a bounded task group. One join
returns their results in submission order, independently of completion order.

```sh
cd docs/programs/tasks
meowy check
meowy run
```

If all tasks complete within the one-second deadline:

```text
Result 1: 4
Result 2: 9
Result 3: 16
```

The manifest explicitly selects two workers, admission for four unjoined tasks,
and 65,536 bytes per task stack. The group's four result slots are a separate
inline capacity. Neither setting infers the other, and a program starting tasks
must have an executor configured before entry.

Cancellation, a deadline, panic, or admission failure occupies the corresponding
result position with its typed error. The program prints that outcome and returns
status 1 if any task failed, otherwise 0. Outcomes are read through references
because diagnostics can own trace storage; they are never copied out of the list.
The group is joined exactly once, and its result list owns every remaining outcome
until scope cleanup.

See [task lifetimes](../../reference/tasks-and-channels.md) and
[monotonic deadlines](../../reference/stdlib/time-and-date.md#clocks-deadlines-and-waits).
