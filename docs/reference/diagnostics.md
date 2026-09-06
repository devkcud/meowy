# Diagnostics and failure behavior

[Documentation index](../README.md)

meowy distinguishes a rejected program, a recoverable failure value, and a panic.
The distinction is part of an API's contract and must not change with build profile.

## Static errors

A static error means the source cannot satisfy a language rule. Diagnostics
identify the operation, the declaration or constraint that it violates, and the
relevant types or ownership path.

| Rejected operation                                  | Reason                                             |
| --------------------------------------------------- | -------------------------------------------------- |
| Assigning text to an integer binding                | Binding types do not change on reassignment        |
| `value<>!<error>` used to conceal a possible error  | Type subtraction is not runtime validation         |
| Emitting twice into a possibly shared primary slot  | Emission initializes once per path                 |
| Missing a required field on a completing path       | Result construction must be complete               |
| Appending a known fourth item to `<T[3]>`           | Inline capacity cannot grow                        |
| Returning a reference to a local list               | Referent is destroyed before the caller can use it |
| Using an owner after sending it through a channel   | The successful transfer moved the owner            |
| Joining a task twice or returning it from its owner | A task has one scoped result owner                 |
| Expanding conflicting fields into one record        | Field selection must have one static meaning       |

Static errors use the [diagnostic presentation](#diagnostic-presentation) below.
The [CLI guide](../cli/README.md) explains how to inspect, repair, and reproduce
them without losing the original failure.

## Recoverable failures

Parsing invalid text, failing to allocate, or sending to a closed receiver produces
an error union where the API promises one. Callers can match, propagate, or map
that result. Error values do not automatically jump out of a scope.

```meowy
strings : @"strings"

parse_age <uint8><error> : (text <string>) {
    -> strings.to_uint8(text)
}
```

The explicit result permits both the integer and the allocation-free parse error.
For errors retaining large owned inputs, keep the concrete error type in the union
or explicitly box its payload before erasing it to `<error>`.

An ignored owned failure is still released correctly. APIs returning a rejected
message or unchanged collection preserve ownership in their error variant so a
caller can retry. Application policy decides whether to retry, report, or stop.

## Panics

Dynamic bounds violations, checked arithmetic violations, failed assertions, and
`debug.panic` raise a panic. A task boundary converts a recoverable child panic
into `tasks.Panicked` after cleanup. An uncaught root panic terminates the program.
Panics are not an alternative control flow for handling expected invalid input.

The compiler diagnoses a provable violation statically; the corresponding runtime
check still exists when the violation depends on runtime input. Optimization may
remove a proven redundant check, but release builds cannot silently introduce
unchecked arithmetic or indexing.

Unsafe memory violations have no recovery guarantee. A panic while already
releasing resources is fatal. Cancellation is cooperative task unwinding with a
typed outcome, not a panic caused by arbitrary thread interruption.

## Diagnostic presentation

An error is a small explanation with enough source context to act on it. The
primary span marks the failing operation; secondary spans show the declaration,
constraint, or ownership transfer that explains the failure. Repairs follow the
error they address, and their commands remain usable after the run ends.

The following is a worked terminal transcript. Version, session, and machine
details are illustrative. Create this **invalid** `main.mwy` in a scratch project
to follow its source locations:

```meowy
list <string[3]> := ["A", "B", "C"]

list = list.add("D")

debug : @"debug"

age <int32> : "twenty"
debug.print(age)

names <string[2]> := ["pato", "ari"]

names = names.add("cornelius")
```

```text
$ meowy run main.mwy

meowy v0.0.1
running main.mwy

1 - error[E103]: list capacity exceeded
  --> main.mwy:3:13
   |
 1 | list <string[3]> := ["A", "B", "C"]
   |      ----------- limited to 3 items
 2 |
 3 | list = list.add("D")
   |             ^^^^^^^^ cannot add a fourth item
   |
   = attempted to insert `"D"`

fix 1.2 [recommended]: increase capacity from 3 to 4
   |
 1 | list <string[4]> := ["A", "B", "C"]
   |              ~
   |
   = `meowy err fix 1.2`
   . reserves inline storage for one more string view

fix 1.1: remove this append
   |
 3 - list = list.add("D")
   |
   = `meowy err fix 1.1`
   . introduces behavioral changes: `"D"` will not be added


2 - error[E207]: incompatible assignment
  --> main.mwy:7:15
   |
 7 | age <int32> : "twenty"
   |     -------   ^^^^^^^^ expected <int32>, found <string>
   |
   = `"twenty"` cannot initialize an <int32> binding
   = if age is numeric, supply the intended integer or parse validated input

fix 2.1: use a string binding
   |
 7 | age <string> : "twenty"
   |      ~~~~~~
   |
   = `meowy err fix 2.1`
   . introduces behavioral changes: age holds text instead of an integer


3 - error[E103]: list capacity exceeded [same as 1]
  --> main.mwy:12:15
   |
10 | names <string[2]> := ["pato", "ari"]
   |       ----------- limited to 2 items
11 |
12 | names = names.add("cornelius")
   |               ^^^^^^^^^^^^^^^^ cannot add a third item
   |
   = attempted to insert `"cornelius"`

fix 3.2 [recommended]: increase capacity from 2 to 3
   |
10 | names <string[3]> := ["pato", "ari"]
   |               ~
   |
   = `meowy err fix 3.2`
   . reserves inline storage for one more string view

fix 3.1: remove this append
   |
12 - names = names.add("cornelius")
   |
   = `meowy err fix 3.1`
   . introduces behavioral changes: `"cornelius"` will not be added


3 errors emitted; 3 self-contained replay capsules saved
session: proj-1788649910
cache: /home/pato/.cache/meowy/projects/8c43d1/entries/51bc09/proj-1788649910/
phase: type checking; program was not started

reproduce:
   `meowy err reproduce 1`   reproduce error 1 [E103]
   `meowy err reproduce 2`   reproduce error 2 [E207]
   `meowy err reproduce 3`   reproduce error 3 [E103]

hint: run `meowy err fix all --diff` to inspect all 5 available fixes.
      run `meowy err fix all --diff --recommended` to preview 2 recommended fixes.
      run `meowy err explain 1` for the full explanation.
      run `meowy err cleanup --diff` to inspect saved artifacts before removal.

hint: quite a lot, huh? run `meowy err summary` to view less information.

found a compiler bug? `meowy err report 2 --diff` previews a report;
`meowy err report 2` submits the reviewed artifact (requires login).
```

`running` names the requested action; the phase line makes clear whether execution
was reached. The type checker continues after independent failures, so one run
can report all three errors. A failed build never executes an older binary.

The two append-removal repairs are explicit alternatives when those writes are
unwanted. They never enter a recommended batch. When growth is intended, a vector
is another design choice, but it needs an allocator and allocation-failure paths;
the tool gives that guidance without inventing a partial patch. `<string[]>` is a
borrowed immutable slice, so removing the capacity annotation is not a growth fix.

### Labels and identities

| Display          | Meaning                                                        |
| ---------------- | -------------------------------------------------------------- |
| `1`              | Occurrence number within a saved session                       |
| `E103`           | Stable diagnostic code for a rule                              |
| `main.mwy:3:13`  | One-based source line and display column of the primary span   |
| `^`              | Primary source span                                            |
| `-` under source | Related declaration or explanatory span                        |
| `~` under source | Replacement text in a suggested edit                           |
| `3 - source`     | A line removed by a suggested edit                             |
| `=`              | Supporting fact or command                                     |
| `.`              | Behavioral or storage consequence of a repair                  |
| `fix 1.2`        | Candidate 2 for occurrence 1 in that session                   |
| `[recommended]`  | Preferred applicable candidate, with its effects still visible |
| `[same as 1]`    | Same diagnostic code as an earlier occurrence                  |

These markers remain meaningful without color. Multiline spans show the relevant
lines; omitted source uses an explicit gap marker. Tabs expand for display and
markers align to rendered columns. Saved edits use UTF-8 byte ranges rather than
terminal columns, so wide characters cannot shift a patch.

Occurrence IDs are assigned in stable source order, independent of parallel
compiler workers. Candidate IDs are assigned before ranking and do not change
when a recommendation is printed first. `[same as 1]` does not merge occurrences:
each keeps its own spans, fixes, and artifact. IDs remain stable within a session;
another run may number them differently. See [session selection](../cli/README.md#saved-sessions).

The [language server](lsp.md#diagnostics-in-a-changing-buffer) reuses rule codes
and evidence with versioned buffer snapshots. Its positions use the negotiated
protocol encoding, not terminal display columns. Live diagnostics have no saved
occurrence numbers and do not replace the last CLI session. Explicit
[editor capture](lsp.md#inspect-facts-and-capture-a-failure) freezes unsaved inputs
and creates a separately selected checking capsule; numbering begins in that
saved session.

### Diagnostic codes

The [code catalog](diagnostic-codes.md) covers source, types, ownership, concurrency,
build inputs, native interfaces, replay tools, and runtime panics. `E103` is the
bounded-list capacity rule; `E207` is an incompatible initializer or assignment.
Codes identify rules across runs, while occurrence numbers identify one failure
in one session. An ordinary returned `<error>` remains a program value and does
not by itself create a compiler diagnostic session.

## Summary and explanation

The summary counts the same occurrences and candidates as the full presentation.
It never invents a recommendation for an error that needs a design decision.

```text
$ meowy err summary

meowy v0.0.1

3 errors from the last run (session proj-1788649910):

1 - E103: list capacity exceeded
    main.mwy:3:13
    2 fixes available (1.2 recommended)

2 - E207: incompatible assignment
    main.mwy:7:15
    1 fix available (review required)

3 - E103: list capacity exceeded (same as 1)
    main.mwy:12:15
    2 fixes available (3.2 recommended)

3 errors - 5 fixes - 2 recommended - 3 replay capsules

`meowy err reproduce <id>` - `meowy err explain <id>` - `meowy err fix <id.fix>`
```

`explain` connects the rule to the exact saved source, without recompiling the
current working tree:

```text
$ meowy err explain 1

Error 1 (E103) occurred because `list` was declared as a
<string[3]>, which allows at most three string values.

At line 3, list.add("D") requires a fourth slot. The declared
type provides only three, and all three are already occupied.

The capacity was declared here:
  main.mwy:1:6
  list <string[3]>

The operation that exceeded it occurred here:
  main.mwy:3:13
  list.add("D")

There are two patch candidates:
  1.2  Increase the inline capacity to 4. Recommended.
       This reserves storage for one more string view.
  1.1  Remove the append if the extra item is unwanted.
       This changes the program's behavior.

For runtime growth, construct a collections.Vector with an
explicit allocator and handle allocation failure. This needs
a manual rewrite; a borrowed <string[]> cannot grow.

Reproduce the saved type-checking failure with:
  meowy err reproduce 1
```

## Repair contracts

A candidate includes its ID, affected source hashes and byte ranges, replacement
text, applicability conditions, behavioral effects, and recommendation reason.
A diagnostic may have no edit candidates and still provide useful guidance.
Only concrete, applicable edits count as fixes in the summary.

Increasing inline capacity uses more storage and changes the concrete list type.
It can be recommended for a local list when all uses accept that capacity and
no public layout or signature is changed. Changing to a vector introduces
allocation and allocation failure. Narrowing a union requires a matching
control-flow proof. A recommendation is a ranked choice, not proof of user intent.

Tools must not invent a numeric age from `"twenty"`, substitute zero to silence
a type error, replace a failed allocation with a null owner, or delete an error
alternative to make a program type-check. A value inferred from context needs an
explicit language or API rule establishing that value; an English interpretation
or an unrelated nearby literal is insufficient. The worked example keeps the
text and offers a clearly labeled type change without recommending it.

`--diff` previews patches without writing source. A recommended batch chooses at
most one candidate per occurrence and reports skipped errors. Explicit candidate
IDs select exactly those edits. Conflicting alternatives cannot be applied
together; overlapping edits must either be identical and deduplicated, or stop
the batch before any source file changes.

Before applying, the tool verifies every affected file against the saved source
hash and validates the combined edits in a temporary source tree. Existing
unselected errors may remain, but a candidate must remove its targeted error
without introducing a new error in the rechecked affected graph. Stale source or
failed validation stops the entire batch. Changes are staged with a recovery
journal; a write failure rolls back completed writes, and interrupted recovery
must finish before another repair can run. The CLI must not claim multi-file
filesystem writes are inherently atomic.

Applying a fix leaves the original session immutable. A subsequent `check` or
`run` produces a new session. The [CLI repair workflow](../cli/README.md#preview-and-apply-repairs)
shows the exact selection, preview, and application steps.

## Replay capsules

A failure saves an executable artifact, called a **replay capsule**. Running it
revisits the failing phase with its original inputs and opens the diagnostic's
supporting evidence. The capsule contains its replay runner, exact compiler and
required tools, source graph, diagnostics, and all captured inputs. It does not
need the original checkout, an installed meowy compiler, a populated build cache,
or a dependency download.

For a static error, the executable drives the preserved compiler up to the
rejected phase. For a runtime failure, it also contains the built program and
its runtime inputs. Both are executable capsules; a static rejection does not
pretend to have produced an application binary. A compiler crash preserves the
compiler executable, invocation, and inputs so it can replay the crash itself.

`meowy err reproduce 1 --verbose` selects and runs the capsule. Exporting it with
`meowy err export 1 --output error-1.replay` produces one relocatable executable;
`./error-1.replay --verbose` provides the same replay without the meowy CLI.
It also supports `--inspect`, which displays stored evidence without rerunning
the failing phase, and `--trace AREA`, which selects the evidence described below.

Isolation means the replay uses a fresh scratch workspace, a private temporary
directory, an allowlisted environment, and bundled build inputs. Absolute source
paths are remapped into that workspace while diagnostics retain their original
locations. The capsule's declared host OS, architecture, CPU, and kernel ABI still
have to be compatible; an executable is not portable across arbitrary machines.
Replaying a cross-compilation failure runs the bundled host compiler against the
saved target configuration. Application execution requires a compatible target
host as well. Missing host requirements are reported before replay begins.

Each occurrence has its own manifest and entry point. The cache may share
immutable payload blobs, but an exported executable embeds the entire closure of
its dependencies. It cannot rely on sibling capsules or paths in the old cache.
Isolation does not mean automatic source minimization: the saved source graph can
contain several errors, with replay focused on the selected one.

The cache root is `$XDG_CACHE_HOME/meowy` when that variable names an absolute
directory, otherwise `$HOME/.cache/meowy`. Under `projects/`, a digest of the
canonical project root separates checkouts. An entry-path digest separates runs
of different programs. A session ID, such as `proj-1788649910`, is unique within
that entry, with a collision suffix when necessary. The manifest retains original
paths for diagnostics; path text is not used as a collision-prone directory key.

| Preserved input                                                                  | Purpose                                                      |
| -------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| Source graph, manifest, and lockfile when present                                | Replay the original input, even after local edits            |
| Compiler version, build identity, executable, required libraries, and invocation | Replay with the toolchain that produced the failure          |
| Target, profile, CPU features, and native input digests                          | Preserve the selected representation and build inputs        |
| Failure phase, code, spans, related notes, and fix candidates                    | Identify the occurrence independently of terminal formatting |
| Declared environment inputs and captured standard streams                        | Record the relevant inputs and observed output               |
| Backend IR, objects, linker commands, and executable, when produced              | Investigate lowering, linking, or runtime failures           |
| Runtime arguments and replayable input, when available                           | Revisit a failure that actually reached execution            |
| Replay runner, per-file digests, and artifact schema version                     | Verify and unpack the executable's contents                  |

Artifacts list backend IR, object files, linker commands, application binaries,
and statistics only when those phases produced them. A type-checking rejection
has its replay executable and compiler payload, but no application executable
or linker run. Missing inputs, interrupted capture, a capture-budget limit, or a
cache write failure must be reported as incomplete. The original diagnostic
still prints even when a complete capsule cannot be saved.

### What happened under the hood

The evidence view connects source-level explanations to compiler and runtime
facts. `--verbose` chooses the useful views for that failure; `--trace AREA`
selects a view explicitly and is repeatable. The same selectors work with
`meowy err inspect`, `meowy err reproduce`, and the exported capsule.

| Area        | Evidence                                                                                         |
| ----------- | ------------------------------------------------------------------------------------------------ |
| `types`     | Required and inferred types, union narrowing, generic substitutions, and the failed constraint   |
| `ownership` | Moves, loan origins and last uses, invalidating operations, and cleanup obligations              |
| `layout`    | Selected target size, alignment, field offsets, capacity, and padding                            |
| `lowering`  | Available lowered IR, source mappings, pass identity, and native call boundary                   |
| `tasks`     | Parent/child IDs, submission, admission, joins, cancellation, deadlines, and cleanup events      |
| `channels`  | Endpoint ownership, send/receive events, capacity, occupancy, and closure                        |
| `clocks`    | Monotonic/civil readings, clock domains, timer slots, wakeups, and zone/calendar rule identities |

Every view labels evidence as **recorded**, **derived from saved inputs**,
**derived from recorded events**, or **unavailable**. Recorded observations are
immutable. Derived evidence is produced by the bundled toolchain against the
captured inputs and marked with that analysis run's identity. A replay must not
present a new scheduler order as the original one.
Optimized-away values, absent passes, unrecorded payloads, and missing events are
shown as unavailable rather than guessed.

For `E103`, the useful chain is the declared capacity, the proven length at the
append, the required fourth slot, and the layout cost of increasing capacity.
For an ownership error, it is the value's owner, the operation that moved it, and
the later use. For a blocked channel, task and endpoint IDs connect the waiting
send to queue occupancy and the receiver's state. These views explain costs and
causes without adding names or keywords to the language grammar.

Layout numbers belong to the recorded target and compiler, not a permanent ABI
for ordinary meowy records. Byte views must distinguish initialized values from
padding or uninitialized slots; replay never reads those bytes as valid values.
Runtime events carry a sequence ID and task identity. Causal edges establish
ordering; timestamps alone do not prove that one task observed another's write.

### Replay fidelity

Reproduction checks the capsule's payload digests, tool identities, and host
requirements, then reruns the recorded phase. The failing child process may exit
nonzero while the replay command succeeds: success means the saved failure was
matched. A static match checks code, phase, and original source span. A compiler
crash additionally checks its saved crash signature. Runtime matching uses the
panic or failure identity, relevant source location, and process outcome; an
unrelated nonzero exit is insufficient.

Replay disables recursive failure capture. Its output and newly derived evidence
belong to a separate replay result; it neither rewrites the capsule nor creates an
endless chain of capsules for the same failure.

Normal runs preserve the executable and captured inputs. For schedule-sensitive
failures, `meowy run --record-replay` additionally records runtime scheduling
decisions and supported external-input boundaries. Replay supplies those inputs
and enforces that event sequence. This mode has explicit instrumentation, I/O,
and storage costs, which are recorded in the capsule along with capture limits
and truncation. It does not change ownership, bounds, or cleanup semantics.

The manifest distinguishes a **closed** recording, with all required inputs
captured, from one containing **external dependencies** or an **incomplete**
capture. Arbitrary FFI, shared files, external services, and uninstrumented
threads are not made deterministic by copying an executable. For recorded
boundaries, replay consumes the event log instead of repeating external effects.
If an uncaptured effect is required, replay stops and identifies it; the explicit
`--allow-live-io` option permits a best-effort rerun and labels it as such.

Recorded standard-library clock reads and timer deliveries advance a virtual
clock during replay. Replay does not wait out the original elapsed interval or
substitute today's civil clock. The `clocks` view connects scheduled deadlines,
observed wakeups, and coalesced ticker slots with task events. Clock resolution
and suspend behavior are captured properties; sequence and causal edges still
establish ordering. Missing required timing events make a recording incomplete.

Unicode, zone, calendar, and pseudorandom rule versions and their data digests are
build inputs. A replay uses the bundled versions. Date conversions derived from
those inputs are labeled derived, while an observed clock read is recorded.
See the [standard library](stdlib/README.md) for the individual effect boundaries.

A replay runner never silently substitutes the host compiler, downloads a
dependency, uses the live project files, or treats a different error as a match.
It reports `matched`, `diverged`, or `unavailable`, with the reason and replay
mode. Reproduction does not replace the last-run session or modify source. Build
input isolation and recorded I/O are not a general OS sandbox for arbitrary native
code. See the [CLI reproduction workflow](../cli/README.md#reproduce-a-failure).

Reports start with a local preview of the exact outgoing files and metadata.
Environment capture uses declared build inputs, not a dump of the user's shell.
Credentials are omitted; source, arguments, paths, and output are reviewable
because they may contain project data. If an omitted input prevents replay, the
report states that limitation. Reporting never uploads silently after a failure.
