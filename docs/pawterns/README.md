# Pawterns

[Documentation index](../README.md) · [Language tour](../guide/README.md) · [Worked projects](../programs/README.md)

A cookbook for meowy and the tools around it. Start with a greeting. Give it a
module. Teach it to handle bad input. Eventually, give it a channel and discover
that “Hello, world!” can wait forever if everybody is being polite in the wrong
order. We'll fix that too.

These are recipes for getting things done: code to use, a result to expect, and
the small detail that saves you an afternoon. The reference holds the full rules;
Pawterns puts them to work. There are 35 recipes across 11 chapters.

## Pick something to cook

| Chapter                                                   | What's cooking                                                                                                |
| --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| [Your first project](first-project.md)                    | A greeting, a helper module, and the helper's suspiciously rapid promotion to a package.                      |
| [Compose values](composition.md)                          | Metadata that stays attached, one-result branches, compact syntax, and a regression driver for future-you.    |
| [Borrow the bytes, keep the owner](ownership.md)          | Caller-owned buffers, greetings that survive their own function, and lists that mean it when they say “full.” |
| [Errors worth keeping](errors.md)                         | Custom failures, the parser's receipts, and boxing an error without throwing away its identity.               |
| [Text and data](text-and-data.md)                         | Visually identical text, borrowed map keys, and JSON with limits and output that waits its turn.              |
| [CLI apps and files](cli-and-files.md)                    | A tiny tool with proper manners, short writes, and an eight-byte file preview.                                |
| [Time and calendars](time-and-calendars.md)               | One deadline, tomorrow's calendar day, and a trip through Chinese date labels.                                |
| [Tasks and channels](tasks-and-channels.md)               | Racing squares, moving whole batches, and three tasks trying to fit in one seat.                              |
| [Deadlocks and shutdown](deadlocks-and-shutdown.md)       | A greeting that actually arrives, the spare sender keeping everyone late, and cooperative cancellation.       |
| [Make the tools pull their weight](toolchain-workflow.md) | LSP setup, gatostyle preferences, portable failure evidence, and repeatable checks.                           |
| [Pack light](lean-binaries.md)                            | Reachable stdlib code, older CPUs, and a container with a firm memory budget.                                 |

## Take a route

**First afternoon:** follow [your first project](first-project.md), then
[composition](composition.md) and [ownership](ownership.md). Add
[typed errors](errors.md) once the program needs to tell a caller what went wrong.
Set up [editor help](toolchain-workflow.md#let-the-editor-read-the-same-project-you-do)
whenever you'd rather have the source point at the mistake.

**Make a useful little tool:** combine [CLI parsing and I/O](cli-and-files.md)
with [text/data](text-and-data.md) or [time/calendars](time-and-calendars.md).
Keep the entry in charge of process status. Give the reusable work ordinary
inputs and results so you can exercise it without borrowing the whole terminal.

**Let it do several things at once:** read [tasks and channels](tasks-and-channels.md)
and [shutdown](deadlocks-and-shutdown.md) together. The second chapter is part of
the recipe, not the chapter you save for after the first mysterious hang.
Then [measure the owners and binary](lean-binaries.md) before shipping it to a
smaller machine.

## Rescue the afternoon

| What's happening                                                   | Jump here                                                                                                |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| My matcher keeps going after emitting its answer.                  | [Choose exactly one result](composition.md#choose-exactly-one-result)                                    |
| The function returns a string view into storage it just destroyed. | [Build a greeting that survives](ownership.md#build-a-greeting-that-survives-its-own-function)           |
| I need to tell malformed input apart from a domain rule.           | [Keep the parser's receipts](errors.md#keep-the-parsers-receipts)                                        |
| Writing failed, but some bytes already escaped.                    | [Make a short write visible](cli-and-files.md#make-a-short-write-visible-before-involving-a-filesystem)  |
| “Tomorrow” is disagreeing with my 24-hour timer.                   | [Tomorrow is not always 24 hours away](time-and-calendars.md#tomorrow-is-not-always-24-hours-away)       |
| The tasks finished in a different order.                           | [Keep the answers in order](tasks-and-channels.md#let-the-squares-race-keep-the-answers-in-order)        |
| Everyone owns their memory correctly and nobody is moving.         | [Hello, world. Goodbye, wait cycle.](deadlocks-and-shutdown.md#hello-world-goodbye-wait-cycle)           |
| The producer finished, but the consumer never sees closure.        | [Find the spare sender](deadlocks-and-shutdown.md#the-spare-sender-keeping-everyone-after-hours)         |
| Yesterday's failure vanished from the terminal.                    | [Inspect and replay it](toolchain-workflow.md#inspect-and-replay-a-deliberate-failure)                   |
| I imported time and want to know what stayed in the binary.        | [Use a duration without every calendar](lean-binaries.md#use-a-duration-without-inviting-every-calendar) |

## Before you paste

A **complete `main.mwy`** replaces that project's entry; separate recipes do not
all belong in one enormous file. Start with the small manifest in
[your first project](first-project.md#print-a-greeting-from-a-project) unless the
recipe supplies another. Task recipes include an explicit executor. A **fragment**
names the existing file or section it changes, and manifest additions go into the
existing block rather than duplicating it.

Shell commands run from the directory stated by the recipe. A `$` marks a shell
prompt in mixed transcripts; omit it when typing the command. Plain command
blocks can be copied directly. Expected output describes the example's behavior,
with task admission, timing, and external I/O qualifications stated alongside it.

**Invalid** snippets demonstrate rejected source. **DO NOT RUN** snippets show
wait cycles on paper, with a working repair beside them. Deliberate panic recipes
say so before the command. You can learn why a greeting hangs without donating
your afternoon to a hung greeting.

The snippets keep allocator choices, returned error values, and resource lifetimes
visible. Diagnostic `debug.print` is handy at the workbench and can itself panic
on output failure; writer-based recipes handle that boundary explicitly.
When you want several recipes assembled into one application, use the
[worked projects](../programs/README.md). When you need the exact contract, follow
the reference links at the recipe's end.
