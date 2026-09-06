# Build command-line applications

[Library index](README.md) · [I/O and system services](io-and-system.md)

`@"cli"` builds application command lines from ordinary meowy values. It handles
typed options, positional arguments, subcommands, help, and usage diagnostics.
It is separate from the `meowy` development tool's own [command interface](../../cli/README.md).
Parsing returns a value; it does not print, exit, read the environment, or invoke
a command handler behind the application's back.

## Describe a command

```meowy
cli : @"cli"

app : cli.command({
    -> name : "moon"
    -> summary : "Show a date in another calendar."
    -> version : "0.1.0"

    -> options : [
        cli.option<string>({
            -> name : "calendar"
            -> long : "calendar"
            -> short : "c"
            -> help : "Calendar used for the result."
            -> default : "gregorian"
            -> choices : ["gregorian", "julian", "hebrew", "chinese"]
        }),
        cli.flag({
            -> name : "verbose"
            -> long : "verbose"
            -> short : "v"
            -> help : "Include the selected rule-data version."
        })
    ]

    -> arguments : [
        cli.argument<string>({
            -> name : "day"
            -> help : "Gregorian date in YYYY-MM-DD form."
        })
    ]
})

argv : ["--calendar", "chinese", "2026-02-17"]
parsed : cli.parse(app, argv.slice())

# After excluding Help, Version, and UsageError, these are statically typed:
  parsed.options.calendar  has type string
  parsed.options.verbose   has type boolean
  parsed.arguments.day     has type string
#
```

`cli.command` is a pure compile-time constructor. It checks the description and
exports a concrete `<app.Parsed>` type, rather than returning a string-keyed map
or `any`. `cli.option<T>`, `cli.argument<T>`, and the other descriptor constructors
are ordinary named values too. Aliasing them keeps the same construction rules.
The source grammar gains no annotations or keywords.

Omitted `options`, `arguments`, and `commands` lists are empty. Omitted summary
and description text are empty strings; an omitted version disables the built-in
version option.

Descriptions are closed compile-time data. Command names and result-field names
are known before execution; there is no runtime mutation of the command tree.
Descriptor lists preserve declaration order for usage and help output. Duplicate
names, invalid aliases, inconsistent defaults, and ambiguous command trees are
configuration diagnostics at the description site.

## Options and arguments

Every descriptor has a required `name`, which is an identifier used in the typed
result, and optional `help` text, defaulting to an empty string. Option descriptors
also have `long` and/or `short`: long names use ASCII letters, digits, and hyphens,
start with a letter, and contain no leading `--`; a short name is one ASCII letter.
At least one spelling is required. Result-field names need not match flag names.

| Constructor | Result field | Additional configuration |
| --- | --- | --- |
| `cli.flag(spec)` | `boolean` | `default` defaults to `false` |
| `cli.count(spec)` | `uint32` | Starts at zero and counts occurrences, including short clusters |
| `cli.option<T>(spec)` | `T` or nullable `T` | A typed `default`, or `required : true`; without either, the result is nullable |
| `cli.many<T, N>(spec)` | `T[N]` | Repeated option, in occurrence order; `minimum` defaults to zero |
| `cli.argument<T>(spec)` | `T` or nullable `T` | Required by default; `required : false` makes it nullable |
| `cli.rest<T, N>(spec)` | `T[N]` | Final positional descriptor only; `minimum` defaults to zero |

`option` cannot combine `required : true` and a default. A default must already
have type `T` and pass every declared check. For many/rest, `minimum` is a
compile-time integer in `0..=N`. Optional positionals follow all required ones;
nothing may follow `rest`. There is no implicit unbounded list growth.

For typed value descriptors, `choices` is an optional nonempty list of accepted
values of `T` supporting ordinary equality; absence means no membership restriction. `parse` may supply an
explicit pure callable from `<string>` to `<T><cli.InvalidValue>`. `validate` may
supply a pure callable from `<T>` to `<null><cli.InvalidValue>`. These functions
cannot perform I/O or create side effects during parsing. Their concrete callable
environments remain statically known; there is no implicit callback boxing.

Without a custom parser, supported `T` values are `string`, `boolean`, fixed-width
integers, and `time.Duration`. Integer parsing uses decimal text with a sign only
where that integer type permits it, checks range, and never accepts trailing
characters. Boolean text is exactly `true` or `false`. Duration text follows the
[time parser](time-and-date.md#fixed-durations). Additional copyable domain values,
including validated dates, can use a custom parser.

Option values must be `memory.Copy`; string results may borrow argv. Parsers cannot
return views of temporary storage. An `InvalidValue` has a static explanatory
message; `UsageError` adds the option/argument name, input token, and position.
This keeps the common parse path allocation-free without losing useful context.

## Parsing rules

`cli.parse(app, argv <string[]>)` returns
`<app.Parsed><cli.Help><cli.Version><cli.UsageError>`. The slice contains only
arguments, excluding the executable name; it can come from a literal, a test, or
`process.arguments(allocator).slice()` after handling that constructor's errors.
The parser never reads the process argument vector itself.

- Long value options accept `--calendar chinese` and `--calendar=chinese`.
- Short value options accept `-c chinese` and `-cchinese`.
- Short flags/counts can cluster: `-vvv` increments a count three times. The first
  value-taking short option consumes the rest of its token, or the next token.
- Flags take no following token. `--verbose=false` is accepted as an explicit flag
  assignment; `--verbose false` leaves `false` as a positional argument.
- Repeating a scalar option is a usage error. Repeating the same flag value is
  idempotent; if explicit flag values differ, the last occurrence wins. Repeated
  count/many options follow their own rules. Numeric/count overflow is
  a usage error, not a panic.
- `--` ends option and subcommand recognition. Later tokens are positional data,
  including tokens that look like flags or command names.
- Unknown options and excess/missing positional values are errors. There is no
  automatic prefix matching, spelling correction, or shell-string expansion.

A value option consumes the next token even if it starts with `-`, unless that
token is the standalone `--` marker, which reports a missing value. Negative
numeric positional tokens are accepted when the next positional's numeric parser
recognizes them; otherwise use `--` to disambiguate flag-looking data. An option
with an explicit `=value` can always carry such text, including the literal `--`.

Successful parse results expose `path`, `options`, `arguments`, and `command`.
The path has a singleton string-literal type identifying the complete command
path, such as `<"moon convert">`. This discriminant keeps sibling parsed types
distinct even when they declare identical options and arguments. Names in a
command path are space-separated identifiers. Options and arguments are exact
typed records. With no child command selected, `command`
is `null`. Parsing does not normalize original strings, mutate argv, allocate a
hidden result map, or execute a handler. The result's borrowed fields cannot
outlive the argument storage.

## Subcommands and global options

A command description may emit a `commands` list of child command descriptors.
The generated descriptor also exposes children by their `name`, so a child named
`convert` supplies `<app.commands.convert.Parsed>`. A root result's `command`
field is the closed union of its children's parsed types, plus `null` when
omitting a child is permitted. Each child's own `command` field supports further
nesting; there is no flattened bag of unrelated option names.

`require_command : true` requires a child and defaults to `false`. For a command
with children, positional arguments are forbidden on that parent: the next bare
token selects the child. Positionals belong to leaf commands. This keeps command
selection unambiguous without heuristics about whether a filename is a verb.

An option with `global : true` is inherited by descendants, appears in their
help, and may occur before or after selecting a child. Its value remains in the
parsed record of the declaring command. A descendant cannot reuse either of its
flag spellings. Other options belong only to the current command; siblings may
reuse their spellings without a conflict.

The application dispatches explicitly:

```meowy
| parsed.command <app.commands.convert.Parsed> | {
    convert(parsed.command.arguments, parsed.options)
}
```

This is a fragment for an application that declared `convert`. It uses ordinary
matcher narrowing; the CLI library does not add a command-dispatch syntax or
start a task for a command automatically.

## Help, versions, and usage errors

Command descriptions accept `summary`, `description`, and `version` strings.
Only the root carries a version. `help : true` defaults on and reserves `--help`
and `-h` on that command. When a root version exists, `--version` is reserved at
the root. These built-ins return `Help` or `Version`, never exit the process.
Disabling help releases its spellings for explicitly declared options.

A recognized help/version token returns immediately for the current command,
without requiring its remaining positionals or processing later tokens. A usage
error encountered before that token still wins. After `--`, help-looking tokens
are data. Help carries the exact selected command path, so it can be rendered
without reparsing or guessing which subcommand was intended.

| API | Result | Contract |
| --- | --- | --- |
| `cli.write_help(help, write, width <usize>)` | `null` or `io.Error` or `cli.RenderError` | Stream usage, descriptions, options, defaults, positionals, and child summaries |
| `cli.write_version(version, write)` | `null` or `io.Error` | Stream the declared application version and a newline |
| `cli.write_error(error, write)` | `null` or `io.Error` | Stream the offending token, its 1-based argv position, and a help hint |

`write` is an explicitly supplied callable with the [I/O write contract](io-and-system.md#readers-and-writers),
such as a standard-output handle's `write` member. Help uses declared order,
plain text, and a positive width; indivisible words may exceed it. Rendering
streams through the writer without an intermediate heap string. It does not
infer a terminal width, add color, or inspect environment variables.

Applications conventionally send help/version to stdout with status `0`, and
usage errors to stderr with status `2`. Write failures remain observable errors.
The application decides its status and emits it at the entry boundary; parse
helpers never call a hidden `exit()`. User-facing CLI errors do not reuse meowy
compiler diagnostic codes or replace a saved compiler replay session.

## Configuration belongs to the application

The parser reads only argv. Environment variables, configuration files, prompts,
and stored defaults are separate inputs. Load them through the corresponding
stdlib modules, validate them, and apply an explicit precedence rule in ordinary
code. Compile-time command defaults are not an implicit channel for reading the
machine's configuration.

For example, an application can distinguish an absent nullable option from an
explicit empty string, then choose argv, configuration, or a built-in value in
that order. A `flag` with default false deliberately does not preserve an
absent/false distinction; use `option<boolean>` when that distinction matters.

The [complete calendar CLI](../../programs/calendar-cli/main.mwy) shows these choices
together, with an explanation in the [guide](../../guide/time-and-date.md).

This separation also makes tests straightforward: pass a literal argument slice,
inspect the typed result or UsageError, and render help into an in-memory writer.
Completion frontends can inspect the same immutable descriptor; they must never
execute command handlers just to discover an option's name.

The [duration project](../../programs/duration-cli/README.md) demonstrates nested
command types and inherited options.
