# meowy documentation

This documentation defines the language's semantics and the foundational library
contracts used in its programs. The reference is authoritative; the guide teaches
those rules by building small programs, and the design notes explain their intent.

## Learn the language

1. Read the [guide](guide/README.md) for bindings, functions, blocks, and loops.
2. Read [values and blocks](reference/values-and-blocks.md) to understand what
   emissions do, when code executes, and how primary values compose with fields.
3. Read [types](reference/types.md) and [collections](reference/collections.md)
   before defining public data structures.
4. Read [memory](reference/memory.md) for ownership and predictable storage.
5. Read [tasks and channels](reference/tasks-and-channels.md) before introducing
   concurrent work.
6. Follow the [manifest guide](guide/mod.md) to configure `mod.mwy`, local path
   aliases, package dependencies, and the project entry.
7. Use the [CLI guide](cli/README.md) to check and run programs, then inspect and
   reproduce failures from their saved executable capsules.
8. Set a [gatostyle policy](guide/gatostyle.md) for layout, code quality, and the
   expression forms your project prefers.

## Look something up

| Question                                              | Reference                                                         |
| ----------------------------------------------------- | ----------------------------------------------------------------- |
| What does this punctuation mean?                      | [Syntax](reference/syntax.md)                                     |
| Does `->` return? Does `{ ... }` create a function?   | [Values and blocks](reference/values-and-blocks.md)               |
| Can a mutable binding change type?                    | [Types](reference/types.md)                                       |
| How do errors and nullable values narrow?             | [Types](reference/types.md#unions-and-narrowing)                  |
| Where does memory come from, and when is it released? | [Memory](reference/memory.md)                                     |
| Is `T[4]` an array or a capacity limit?               | [Collections](reference/collections.md)                           |
| Who joins a task? What does cancellation guarantee?   | [Tasks and channels](reference/tasks-and-channels.md)             |
| How are native data and functions declared?           | [Modules and FFI](reference/modules-and-ffi.md#native-interfaces) |
| What does the standard library provide?              | [Standard library](reference/stdlib/README.md)                    |
| How do durations, dates, and zones work?              | [Time and date](reference/stdlib/time-and-date.md)                 |
| How do I use Chinese or Hebrew leap months?           | [Calendars](reference/stdlib/calendars.md)                         |
| How do I configure mod.mwy and local import aliases?  | [Manifest guide](guide/mod.md)                                    |
| How do I enforce coding style and code quality?       | [gatostyle](guide/gatostyle.md)                                   |
| Which failures are values?                            | [Diagnostics](reference/diagnostics.md)                           |
| What does a diagnostic code mean?                     | [Code catalog](reference/diagnostic-codes.md)                     |
| How do I check, build, and run a program?             | [Command line](cli/README.md#check-build-and-run)                 |
| How do I review and apply suggested fixes?            | [Repair workflow](cli/README.md#preview-and-apply-repairs)        |
| Can I see the original compiler and runtime evidence? | [Replay capsules](reference/diagnostics.md#replay-capsules)       |
| How do I replay a failure without the project?        | [Export a replay executable](cli/README.md#export-one-executable) |

## Read complete programs

The [worked programs](programs/README.md) cover validation, allocation-free packet
decoding, ordered task results, and a bounded producer/consumer channel. They are
part of the documentation, with explanations of their data and failure paths.

[Design notes](design.md) record the language's main decisions.
The [manifest guide](guide/mod.md) explains project configuration alongside its
[sample manifest](guide/mod.sample.mwy).

## Conventions

Code fences marked `meowy` use language syntax. A snippet explicitly labeled
**invalid** demonstrates a diagnostic. API tables describe contracts; angle
brackets in prose denote types, not command placeholders.

Reference rules use **must** for requirements and **may** for permitted choices.
Target-dependent properties, such as pointer width and C layout, must be supplied
by the selected build target. No sample module version or performance measurement
is implied by a code listing.

The documentation generally uses four-space indentation and omits semicolons at
line ends; compact examples demonstrate other legal presentations. Gatostyle's
layout engine preserves structure, while optional coding-style fixes require
proof that their rewrites preserve behavior. Spaces never select language meaning.

Shell examples use uppercase placeholders such as `PATH` and `TRIPLE`; help
output may use `<id>` for a command argument. Those command placeholders are
separate from meowy's type syntax. Diagnostic transcripts illustrate the format
and contract rather than reporting a particular local run.
