# Runtime recording profile 1

[Diagnostics](diagnostics.md#replay-fidelity) · [Artifact formats](artifact-formats.md)

This chapter fixes the capture policy for the initial distribution. It adds no
manifest settings. `meowy run --record-replay` enables event recording; ordinary
`run` preserves build inputs, argv, available streams and the final failure but
does not promise deterministic external observations. The test harness enables
profile 1 event recording for every executing case, within the fixed limits below;
listing/build-only tests create no runtime log. No test recording override exists.

## Supported boundaries

| Operation                                                                                                                                                                                                     | Recording and replay behavior                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Task admission, scheduling at runtime checkpoints, joins, cancellation and cleanup; channel creation/send/receive/close                                                                                       | Record stable task/resource IDs, action, outcome and causal predecessor IDs; replay releases the same task at each checkpoint                                           |
| Monotonic/civil clock reads, sleep, timer/ticker delivery                                                                                                                                                     | Record returned readings/errors, clock domains and delivered/coalesced slots; replay consumes them on a virtual clock                                                   |
| `process.arguments`, `process.working_directory`                                                                                                                                                              | Capture exact native argument/path bytes and decoding outcome; replay supplies that snapshot                                                                            |
| Standard input reads through `io`                                                                                                                                                                             | Capture requested count, returned bytes/progress/error/EOF; replay supplies the recorded result                                                                         |
| Standard output/error writes through `io`, `fmt`, `debug`                                                                                                                                                     | Capture attempted bytes and committed progress/error; replay verifies the attempt, reproduces the visible transcript and supplies the saved outcome                     |
| `fs.open`/`fs.open_native` with `fs.ReadOnly`, file read/seek/metadata/close, `fs.entries` and directory next/cleanup                                                                                         | Record operation inputs, returned bytes/metadata/entry order/errors and logical handle IDs; replay supplies virtual read-only handles, without opening those host paths |
| `env.snapshot` and environment lookups through that snapshot                                                                                                                                                  | The fixed allowlist below determines whether the snapshot is closed; lookups use its captured bytes                                                                     |
| Cryptographic entropy through `random`                                                                                                                                                                        | Record requested byte count and returned bytes/error; replay supplies them                                                                                              |
| Built-in allocator allocation/resize/release                                                                                                                                                                  | Record requested size/alignment, logical allocation ID and success/refusal; replay reproduces a refusal and obtains real backing storage for a recorded success         |
| Deterministic pure operations, seeded generators, parsing, Unicode/calendar lookup                                                                                                                            | Recompute with bundled code/rule data; no input event is required                                                                                                       |
| Filesystem write-capable opens/writes/mutations/sync, networking/DNS, child-process spawn/wait, mutex/atomic synchronization, source raw-pointer operations, arbitrary FFI, uninstrumented threads/native I/O | Unsupported in profile 1; mark an external dependency before crossing the boundary; default replay stops there                                                          |

Record requested paths and arguments as bytes after the API's own validation.
Pure validation failures need no external event. Native handles, pointers and
memory addresses are never persistent resource IDs. Open/read failures are events
too. Read-only filesystem replay returns recorded observations even if the
original path no longer exists. A later write to that virtual handle is an
unsupported boundary, not a write to the original path.

Library helpers such as `fs.read_all` record their primitive reads and allocator
operations, without duplicating those effects as helper-level events. Failure to
obtain memory or runtime resources required to supply a recorded successful result
is unavailable replay (`E704`), not evidence that the original program failed that
allocation. Runtime-internal recording storage is outside the application's
allocation event stream and charged to its capture budget.

Source-level raw-pointer construction, `memory.address`, pointer casts/equality/
arithmetic/dereference and unsafe raw-memory access are unsupported boundaries.
Replay allocation addresses and address reuse may differ, so logical allocation
IDs cannot certify those observations. Hidden pointer implementation inside a
supported intrinsic is not a source observation. Safe references to live storage
retain their ordinary identity behavior; a reference cannot observe freed storage.

Supported standard-stream replay remains visible on the documented streams.
Reproducing the transcript is the one deliberate replay output effect; it does
not claim a successful original write when that write recorded partial progress
or an error. Test/case replay instead labels captured streams on stderr. A replay
output failure is a runner failure, separate from the saved application's result.

An unsupported operation may execute in the original recorded run, but the
recording cannot be `closed`. `--allow-live-io` permits the replay to perform such
effects using recorded arguments and labels the whole rerun `best_effort`; it
does not relabel that capture closed. It must never silently reuse an original
credential or create a live handle from a recorded virtual handle. If a later
live operation needs a handle that only exists virtually, replay is unavailable
and names the acquisition that also needs a live rerun. A fresh best-effort rerun
may execute the complete affected resource acquisition/lifetime live, explicitly
reporting which earlier recorded events it no longer enforces.

## Environment and budgets

Compiler, dependency, and replay subprocesses get a fresh temporary directory,
distribution tool locations and fixed `LANG=C`, `LC_ALL=C`, `TZ=UTC`.
They do not inherit the caller's shell environment. Tool commands may consult
their documented cache-location, terminal and transport authentication inputs;
those are not semantic build inputs and credentials never enter artifacts.

The original application retains its documented inherited environment and working
directory. Runtime capture includes environment values only for `LANG`, `LC_ALL`,
`LC_CTYPE`, and `TZ`, including presence/absence. A successful full `env.snapshot`
that includes any other name is an external dependency: store the omitted names,
not their values, and do not claim closed replay of that snapshot. Recording does
not change what the original application sees. No application-environment secret
is restored through the compiler's fixed environment. Best-effort live environment
access uses the replay caller's environment and reports that difference. Source,
argv, file contents, output, and entropy can themselves contain sensitive data;
they are exact captured inputs, not a claim of automatic secret detection.

| Fixed limit                                | Profile 1 value and behavior                                                           |
| ------------------------------------------ | -------------------------------------------------------------------------------------- |
| Captured project source/native/data inputs | 256 MiB total uncompressed bytes per session                                           |
| Runtime event log and input payloads       | 64 MiB per application/case recording                                                  |
| Captured stdout plus stderr                | 8 MiB per application recording; tests use the smaller of this and `test.output_bytes` |
| Individual event payload                   | 8 MiB; a larger I/O result is represented by ordered chunks of that event              |
| Replay runner/toolchain payload            | No capture truncation; the distribution closure is required for an executable capsule  |

The limits are charged in exact byte counts before compression/deduplication.
An input captured in several events is charged each time to the event budget;
one session's shared source/tool blobs are stored once. A limit truncates only
capture, never the application's read/write or return value. Record the first
omitted event/input and observed counts, mark `incomplete`, and preserve the
original failure (`E707`). Test output-budget enforcement remains a test failure
under its separate supervision rules. Toolchain-copy/disk failure likewise marks
capture incomplete, without changing the application's status. No background
upload, environment dump, or automatic cache eviction is part of capture.

## Event matching and result identity

Events use the [event schema](artifact-formats.md#runtime-events) and exact
[operation registry](runtime-events.md): monotonically
increasing integer `seq`, nonnegative `task`, operation name, argument digest,
logical resource ID where applicable, causal predecessor IDs, and an outcome
whose payload digests identify exact bytes. Each operation fixes its complete
argument/result field set. IDs begin at one except the root task
is zero. Creation/submission order assigns resource/task IDs; replay never assigns
them from host thread IDs or addresses. Profile 1 schedule events occur at task
dispatch/suspend/resume, supported I/O, channel operations, task joins and
cancellation checkpoints; they do not claim to record every machine instruction.

Argument digests use the exact canonical JSON encoding defined in
[artifact formats](artifact-formats.md#canonical-json) for scalar metadata and
separate byte-payload digests. Pure formatting and value serialization happen
before the boundary. Each operation identifies its API name and action; e.g.
`fs.file.read` carries logical handle, requested count and output buffer capacity,
with result count/bytes/error/EOF. Ownership handles are compared by logical ID;
paths by exact validated bytes; numeric values by their exact serialized value.

Replay must match operation, task, resource, argument digest and required causal
predecessors before supplying the saved result. The first mismatch, extra event,
unconsumed required event at termination, or changed final failure is `diverged`
(`E706`), with expected and actual evidence. A missing/truncated payload or
unsupported required boundary is `unavailable` (`E704`), not divergence. A closed
recording must consume every required event and match the final failure's code,
phase, original byte span, panic message where present, and exit/signal outcome.
Timing is virtual; a host timestamp or different progress message is not a match
key. Compiler-crash matching uses signal/exception kind plus normalized pass and
top symbolized compiler frame, with unavailable components marked explicitly.

Capture is supervised by `meowy run`/the test runner. A directly launched deployed
executable prints its minimal runtime diagnostic; it does not promise to recover
the absent compiler/source closure or create a complete capsule. Fatal termination
can leave incomplete/no evidence as documented. Recorded I/O and build isolation
are not an OS sandbox, and no schedule capture makes an invalid data race valid.
