# Reproducible dice rolls

[Worked projects](../README.md)

Roll a six-sided die 600 times using seed 2026, then print a histogram. The
generator and all six counters live inline; the simulation needs no allocator.

From the repository root:

```sh
cd docs/programs/seeded-rolls
meowy check
meowy run
meowy run
```

[mod.mwy](mod.mwy) selects [main.mwy](main.mwy). The second run produces the same
histogram when the seed, call sequence, and `random.Algorithm` identity are
unchanged. Copy the project directory to run it independently.

Output starts with the algorithm identity, seed, and roll count. Six following
lines have the form `face: count`, in face order from 1 through 6. Their counts
sum to 600; they need not be equal. There is deliberately no fixed expected
histogram here: the library contract names its algorithm identity without
requiring a particular sequence across different identities. Save that identity
alongside the seed when preserving a simulation result.

`below(6)` samples uniformly from zero through five without a modulo bias. Adding
one maps those values to the collection's one-based positions. The explicit
`numbers.convert<usize>` then converts the generator's `uint64` result to the
target's collection-position type; type ascription would not perform that
conversion. `get_copy` checks the position before updating its existing counter.
The list initializes all six positions and never grows. Each `uint32` counter
can be at most 600, so this fixed simulation cannot overflow its counters.

The generator is mutable, and each draw advances its inline state. Keep the draw
order when reproducing results: an extra random call changes the subsequent
sequence. It has no global seed or shared hidden generator. Use
`random.secure_fill` for OS cryptographic entropy when unpredictability is
required; a known seed is useful here precisely because it is reproducible.

The program handles invalid bounds, numeric conversion failures, checked lookup
failures, and output errors with status 1. The fixed bound and six initialized
positions keep the first three out of the successful path. Error reporting uses
`debug.print`, whose own output failure panics. Normal completion emits status 0.
Streaming output can leave a partial histogram if the writer fails.

See [randomness and numeric utilities](../../reference/stdlib/text-and-data.md#numbers-mathematics-and-randomness),
[numeric conversions](../../reference/stdlib/text-and-data.md#numeric-conversions-and-shifts),
and [bounded collection access](../../reference/collections.md#indexing-mutation-and-removal).
