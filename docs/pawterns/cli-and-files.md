# CLI applications and file boundaries

[Pawterns](README.md)

The tool begins as “just print this thing.” Then someone asks for `--help`, pipes
it into another program, and gives it a filename called `--help`. Time to give
the little creature some manners. Parsing, output, and filesystem effects remain
separate steps, so each can be exercised without summoning the others.
These recipes use a normal project with `main.mwy` as its entry;
see [Your first project](first-project.md) for the manifest and commands.

## Give your tiny tool proper manners

Let's accept a name and a repetition count, provide help, and stop a broken pipe
from masquerading as a bad argument. Three greetings is plenty; this is a hello
world, not an endurance event.

This is a complete `main.mwy`. The application owns argv for as long as the parsed
name is borrowed. No handler runs until help, version, and usage results have been
handled explicitly.

```meowy
cli : @"cli"
fmt : @"fmt"
io : @"io"
memory : @"memory"
process : @"process"

app : cli.command({
    -> name : "greet"
    -> summary : "Greet someone a small number of times."
    -> version : "0.1.0"
    -> options : [
        cli.option<uint8>({
            -> name : "count"
            -> long : "count"
            -> short : "n"
            -> help : "Number of greetings, from 1 through 3."
            -> default : 1
            -> choices : [1, 2, 3]
        })
    ]
    -> arguments : [
        cli.argument<string>({
            -> name : "name"
            -> help : "Name to greet."
        })
    ]
})

status <int32> := 0
'main {
    output := io.stdout()
    errors := io.stderr()
    arguments : process.arguments(memory.heap)
    | arguments <error> | {
        written : fmt.write_line(errors.write, arguments)
        | written <error> | { status = 1; 'main.leave() }
        status = 1
        'main.leave()
    }

    parsed : cli.parse(app, arguments.slice())
    | parsed <cli.Help> | {
        written : cli.write_help(parsed, output.write, 80)
        | written <error> | status = 1
        'main.leave()
    }
    | parsed <cli.Version> | {
        written : cli.write_version(parsed, output.write)
        | written <error> | status = 1
        'main.leave()
    }
    | parsed <cli.UsageError> | {
        written : cli.write_error(parsed, errors.write)
        | written <error> | { status = 1; 'main.leave() }
        status = 2
        'main.leave()
    }

    remaining := parsed.options.count
    'greet {
        | remaining == 0 | 'greet.leave()
        written : fmt.write_line(output.write, "Hello, {parsed.arguments.name}!")
        | written <error> | { status = 1; 'main.leave() }
        remaining = remaining - 1
        'greet.restart()
    }
}
-> status
```

From that project's directory:

```sh
$ meowy check
$ meowy run -- --count 2 Mira
Hello, Mira!
Hello, Mira!
$ meowy run -- --help
$ meowy run -- --count 0 Mira
```

The final command produces a usage error and status `2`; zero violates the
declared choices. A noninteger count, a value exceeding `uint8`, or a missing name
is also a usage error. Help/version and successful greetings use status `0`.
Argv acquisition and output failures use `1`. The emitted entry status decides
the process result; the parser never exits it.

The first `--` above separates `meowy run` options from application arguments.
To greet a name that starts with a dash, use the application's separator too:

```sh
$ meowy run -- -- --help
Hello, --help!
```

Command metadata and parse results are statically shaped, with no allocated
string-keyed option map. `process.arguments(memory.heap)` owns argument storage;
`parsed.arguments.name` borrows it. Formatting streams without creating a joined
greeting string. It can still fail after some bytes reach stdout.

For a small parser fixture, replace the argument acquisition and parse expression
with `argv : ["--count", "2", "Mira"]` followed by
`parsed : cli.parse(app, argv.slice())`, leaving the result handling intact.
Parsing literal arguments itself needs no heap allocator or live process input.

See [typed CLI descriptors](../reference/stdlib/cli.md),
[the nested duration CLI](../programs/duration-cli/README.md), and
[the calendar CLI's entry boundary](../programs/calendar-cli/main.mwy).

## Make a short write visible before involving a filesystem

Before testing against a flaky disk, make a buffer fail on purpose. Eight bytes
of room and an eleven-byte message will do nicely. A failed write can still have
written something, which is inconvenient but very much worth knowing.

This complete `main.mwy` intentionally sends eleven ASCII bytes into eight bytes
of caller-owned storage. The writer's exclusive borrow ends before inspection.

```meowy
debug : @"debug"
io : @"io"
strings : @"strings"

buffer <uint8[8]> := []
attempt : 'write {
    writer := io.buffer_writer(&!buffer)
    -> io.write_all(writer.write, "hello meowy".bytes())
}

debug.print("Committed: {attempt.count} bytes")
| attempt.error <io.Error> | debug.print("Output is incomplete")
prefix : strings.from_utf8(buffer.slice())
| prefix <error> | debug.panic(prefix)
debug.print(prefix)
-> 0
```

Expected output:

```text
Committed: 8 bytes
Output is incomplete
hello me
```

`io.write_all` handles partial progress; it cannot invent capacity after a writer
reports failure. The count reports the already committed prefix. Blindly retrying
the whole message against an external writer can duplicate that prefix. A retry
policy must account for the count and the destination's actual state.

This fixture uses ASCII so its eight-byte prefix is valid UTF-8. A byte writer
may split a multibyte character, and `strings.from_utf8` can then fail. Keep
arbitrary binary prefixes as bytes instead of treating them as repaired text.

The buffer and writer cursor are inline; no heap allocation or OS operation is
needed to exercise this failure. The final debug prints are the only output
effects. Even when writing to stdout, `write_all` does not guarantee atomicity
against other writers: its individual host writes can interleave.

Use an in-memory writer similarly to test help rendering or JSON encoding. The
[typed JSON recipe](text-and-data.md#decode-a-bounded-schema-then-finish-encoding-before-writing)
finishes encoding in memory before exposing bytes to an external sink.
See [the read/write progress contract](../reference/stdlib/io-and-system.md#readers-and-writers).

## Copy a bounded preview and preserve cleanup evidence

You want a preview, not the whole enormous file. Copy at most eight bytes into a
new destination, keep scratch tiny, and still listen when sync or close has bad
news. “The copy loop finished” is only one part of the story.

Start from a working copy of the [file-copy project](../programs/file-copy/README.md).
This is a project variation: keep its manifest, argv/path handling, file ownership,
and result reporting. In `main.mwy`, change the command summary to:

```meowy
-> summary : "Copy at most the first eight bytes into a new file."
```

Replace the existing `scratch` and `copied` bindings with this block. It uses the
`bytes`, `io`, `source`, and `destination` bindings already present in that file.

```meowy
scratch := bytes.filled<4>(0)
copied : 'preview {
    preview := io.limited(source.read, 8)
    -> io.copy(preview.read, destination.write, scratch.slice_mut())
}
```

Keep these operations immediately afterward, before any fallible reporting:

```meowy
synced : destination.sync()
destination_closed : destination.close()
source_closed : source.close()
```

Keep the original checks of `copied.error`, `synced`, `destination_closed`, and
`source_closed` too. The adapter's borrowed read callable expires when `'preview`
ends, allowing `source` to be consumed by `close`. `destination.write` is borrowed
only during copying.

Run from the copied project's directory with an unused destination:

```sh
$ meowy check
$ meowy run -- message.txt preview.txt
Copied: 8 bytes
```

`preview.txt` contains exactly `meowy sa`, with no newline. The eight-byte limit
is the intended output bound. Reaching it is successful completion of this
bounded stream, not evidence that the original source reached EOF. A source
shorter than eight bytes succeeds with its actual shorter length.

`path.parse` only validates borrowed path text. It does not create a file, check
permissions, expand `~`, resolve aliases, or establish directory confinement.
The subsequent `fs.open` calls perform the effects. Keeping `fs.CreateNew` rejects
an existing destination, including the same path as the source, without truncating
it. Do not replace this mode with `ReplaceContents` just to make reruns convenient.

The recipe uses four initialized scratch bytes, scalar progress counters, a
small inline adapter, two file handles, and the original allocated argv owner.
It never stores the full input. An empty bounded list would not be useful read
scratch: reads need initialized elements to fill, which `bytes.filled` supplies.

I/O failure can leave a partial new file. Sync requests the host's durability
operation; it does not erase a prior transfer failure or independently sync the
directory. A failed close still consumes its owner and must not be retried. The
original project attempts sync and both closes before reporting their outcomes,
so a reporting failure does not skip those explicit cleanup attempts.

For whole-file copying, remove the `io.limited` adapter and restore the original
`io.copy(source.read, destination.write, scratch.slice_mut())`. For replacing a
configuration file, inspect `fs.replace` and its explicit replacement/durability
options instead of assuming a streaming copy is an atomic update.

See [paths](../reference/stdlib/io-and-system.md#paths-are-data),
[file ownership and replacement](../reference/stdlib/io-and-system.md#files-and-directories),
and [the complete cleanup sequence](../programs/file-copy/main.mwy).
