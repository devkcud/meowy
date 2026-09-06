# Send bounded messages

[All programs](../README.md) · [Manifest](mod.mwy) · [Entry](main.mwy) · [Workers](workers.mwy)

A producer sends 1 through 4 through a queue holding two messages, while a consumer
sums them. Both workers share one monotonic deadline. The parent joins both and
checks both outcomes before treating the sum as complete.

```sh
cd docs/programs/channel
meowy check
meowy run
```

Normal output:

```text
Sum: 10
```

The manifest supplies two executor workers, four admission slots, and explicit
stack storage. Channel construction separately uses `memory.heap` for two uint32
messages. Queue slots are reused; no message allocates storage.

The parent moves each endpoint into exactly one child and retains no sender clone.
When the producer closes the last sender, the consumer drains buffered messages
before observing Closed. Joining the producer first is safe because the consumer
is already running and can relieve backpressure.

If a child cannot start, releasing its captures closes its endpoint. Closing the
receiver wakes a blocked send with its rejected value. A consumer failure, producer
failure, or allocation failure gives status 1. A producer failure with a successful
consumer result is explicitly printed as `Partial sum`, never as a complete sum.
Success gives status 0. Both child owners are joined before the parent returns.

The worker functions keep endpoint owners local and use lexical cleanup even on
early exits. See [endpoint closure](../../reference/tasks-and-channels.md) for the
meaning of drained closure, rejection, cancellation, and mandatory joins.
