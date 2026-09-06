# Nested commands for exact durations

[Program collection](../README.md) · [Source](main.mwy) · [Project settings](mod.mwy)

`span` has a real command tree: `duration` selects a group, then `inspect` or
`scale` selects an operation. The parser returns nested records with distinct
generated types. Ordinary matchers dispatch each level; no command callback
or flattened string-keyed result is hidden inside the parser.

From the repository root:

```sh
cd docs/programs/duration-cli
meowy check
meowy run -- duration inspect --unit 1h 1h30m
```

The application prints:

```text
Duration: 1h30m
Nanoseconds: 5400000000000
Whole units: 1
Remainder: 30m
```

`inspect` takes a `time.Duration` positional and a `time.Duration` option. The
unit defaults to `time.Second`; it must be positive before `split` can run.
`scale` takes a duration and a signed `int64` factor:

```sh
meowy run -- duration scale 250ms 6
```

```text
1.5s
```

The root's global `--raw` flag is accepted before or after selecting descendants.
Its result stays in `parsed.options.raw`, while the selected operation's values
live in that child's typed arguments/options record:

```sh
meowy run -- --raw duration scale 250ms 6
meowy run -- duration scale --raw 250ms 6
```

Both print:

```text
1500000000
```

Ask for help at any level; each command describes its own children or arguments:

```sh
meowy run -- --help
meowy run -- duration --help
meowy run -- duration inspect --help
meowy run -- --version
```

The source first narrows `parsed.command` to
`<app.commands.duration.Parsed>`, then narrows its `command` field to the selected
`inspect` or `scale` type. Description names determine these types at compile
time. Help/version and usage errors leave the entry scope before normal command
handling; they are values, not library-triggered process exits.

Try the failure paths:

```sh
meowy run -- duration inspect --unit 0 1s
meowy run -- duration scale 1month 2
meowy run -- duration scale 1w 9223372036854775807
```

These respectively reject a nonpositive splitting unit, invalid duration syntax,
and signed nanosecond overflow. Multiplication uses `try_scale`, so an external
factor that exceeds the duration range returns a handled `time.RangeError`.
`1d` always means 24 elapsed hours; calendar days belong to the date library.

For negative duration positionals, `--` makes the argument boundary explicit:

```sh
meowy run -- duration scale -- -1s 2
```

```text
-2s
```

The only chosen heap allocator is for the process argument owner. The command
description, parsed records, duration values, and arithmetic results are inline;
formatting streams into explicit standard-output/error writers. No intermediate
owned string or timer is involved.

Help, version, and completed operations return status `0`. Parse/validation and
arithmetic-range errors return `2`; argument-loading or output failures return
`1`. The root and `duration` group both require a child, so an omitted operation
is a usage error.

See [typed CLI trees](../../reference/stdlib/cli.md#subcommands-and-global-options)
and [fixed durations](../../reference/stdlib/time-and-date.md#fixed-durations).
