# I/O and system services

[Library index](README.md) · [Text and data](text-and-data.md) · [CLI applications](cli.md)

System modules expose effects as ordinary calls with explicit ownership and
failure results. Paths do not perform I/O just by being constructed; opening a
file, starting a process, resolving a name, and connecting a socket do. These
operations are unavailable during pure manifest or compile-time evaluation.

## Readers and writers

I/O algorithms take explicit callables. A read callable accepts an exclusive
initialized `<collections.MutSlice<uint8>>`; a write callable accepts `<uint8[]>`.
Both return small records rather than hiding a partial transfer in an exception:

| Result     | Fields                                                     | Meaning                                                       |
| ---------- | ---------------------------------------------------------- | ------------------------------------------------------------- |
| `io.Read`  | `count <usize>`, `end <boolean>`, `error <io.Error><null>` | A valid prefix of bytes, possibly followed by EOF or an error |
| `io.Write` | `count <usize>`, `error <io.Error><null>`                  | A committed prefix, possibly followed by an error             |

The count is never larger than the supplied span. Read bytes remain valid even
when the result also reports EOF or failure. With a nonempty buffer, a read must
produce bytes, report EOF, report an error, or wait; zero progress without one of
those outcomes is invalid. A write of nonempty input likewise makes progress or
reports an error. Empty spans succeed with zero count without probing for EOF.

`io.Error` is an opaque nominal error with a portable kind, optional native error
code, and static message. `io.Kind` has exactly the ordinary values `io.Closed`,
`Permission`, `NotFound`, `Interrupted`, `WouldBlock`, `TimedOut`, `BrokenPipe`,
`NoProgress`, `InvalidCount`, `LimitExceeded`, and `Other` (all in `io`). Kind values
are `memory.Copy`, `tasks.Send`, and `tasks.Sync`, and support ordinary equality
and formatting; their display is their listed name. `io.Error` has those same
three capabilities, but no ordinary equality; inspect its documented facts.

| API                                                                     | Result            | Contract                                                          |
| ----------------------------------------------------------------------- | ----------------- | ----------------------------------------------------------------- |
| `io.error(kind <io.Kind>, native_code <int64><null>, message <string>)` | `io.Error`        | Construct an inline error; message must be nonempty static UTF-8. |
| `failure.kind()`                                                        | `io.Kind`         | Read the portable classification.                                 |
| `failure.native_code()`                                                 | `int64` or `null` | Read the original host code when available; zero is not absence.  |

Read the message with `errors.message(&failure)`. `errors.code` returns `io.`
followed by the kind's lowercase snake-case spelling, such as `io.broken_pipe`.
The constructor preserves the supplied native code and message without allocating;
it does not fabricate an OS failure or perform I/O. Use `null` when no native
operation failed. A raw OS number is not a meowy diagnostic code. The code/kind
relationship is fixed; callers cannot forge another nominal library error through
this constructor. These errors follow the [erasure classification](errors.md#choose-inline-storage-or-explicit-erasure).

Both `io.error` and `cli.invalid` validate static message metadata through their
resolved intrinsic identity, including aliases: empty messages use `E217`, and
runtime-dependent messages use `E211`. Their other arguments may be runtime
values. Put changing facts in an application-defined error when they are needed;
the portable I/O boundary intentionally retains only the fields above.
Adapter code reports malformed callable results as NoProgress or InvalidCount
instead of looping forever. Library-generated errors have nonempty static
messages; only their kinds and documented facts are compatibility contracts.
The [custom writer source case](../../programs/testing/tests/stdlib_test.mwy)
constructs an `io.Write` record with `io.error`, handles empty input successfully,
and checks the portable kind and retained explanation after `io.write_all` fails.

| API                                        | Result                  | Contract                                                                                    |
| ------------------------------------------ | ----------------------- | ------------------------------------------------------------------------------------------- |
| `io.read_exact(read, buffer)`              | `io.Read`               | Fill the initialized span or report a short read/EOF/error with the accumulated count       |
| `io.write_all(write, bytes)`               | `io.Write`              | Retry partial progress until all input is written or an error occurs                        |
| `io.copy(read, write, scratch)`            | `io.CopyResult`         | Stream through caller-owned scratch; report total read/written counts and an optional error |
| `io.limited(read, limit <uint64>)`         | Concrete reader adapter | Borrow a callable and expose at most limit bytes as a bounded stream                        |
| `io.bytes_reader(bytes)`                   | Concrete reader cursor  | Borrow bytes and advance through them without allocation                                    |
| `io.buffer_writer(&!buffer)`               | Concrete writer cursor  | Append to a bounded byte list; report a partial count and capacity failure when full        |
| `io.buffered(read, &!scratch)`             | Concrete reader adapter | Borrow initialized scratch for read-ahead                                                   |
| `io.stdin()`, `io.stdout()`, `io.stderr()` | Standard-stream handles | Borrow the runtime's standard channels without taking ownership of the host descriptors     |

These are statically checked callable contracts, not an implicit interface lookup
or automatic dictionary allocation. For example, pass `file.read` and
`destination.write` explicitly. Captured borrows live as long as the adapter;
callable environments keep their concrete types. Storing an adapter does not
start a thread or read ahead before an operation is requested.

Reader cursors and adapters expose `.read(buffer)` with the `io.Read` contract;
the buffer writer exposes `.write(bytes)` with the `io.Write` contract. Pass those
bound callables to other I/O helpers, retaining their owners and captured borrows
until the helper finishes.

Mutable readers/writers require exclusive access to their state. Standard-stream
handles synchronize each host operation; different writes may interleave, so one
write_all is not a promise that a complete multi-write message is atomic.
Standard streams remain open when these borrowed handles leave scope.

`CopyResult` has `read <uint64>`, `written <uint64>`, and `error <io.Error><null>`.
Counts reflect actual progress even if a later write fails. Scratch must be
nonempty. These loops acknowledge task cancellation between transfers. Native
blocking file operations may occupy a worker; the API does not pretend every
filesystem has asynchronous completion.

## Paths are data

`@"path"` operates lexically. A `path.Path` is a borrowed UTF-8 path validated for
the selected target's path syntax; `path.parse(text)` returns that view or
`path.InvalidPath`. Embedded NUL is invalid. A path does not imply that the target
exists, is accessible, or stays inside a directory after symlink resolution.

| API                                         | Result                                                   | Contract                                                                                                  |
| ------------------------------------------- | -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `path.is_absolute(value)`                   | `boolean`                                                | Inspect target path syntax                                                                                |
| `path.basename(value)`, `.extension(value)` | `string`                                                 | Borrow textual components; extension includes the final dot, with dotfiles alone treated as extensionless |
| `path.parent(value)`                        | `path.Path` or `null`                                    | Borrow the lexical parent where present                                                                   |
| `path.clean_into(value, &!buffer)`          | `path.Path` or `path.Error`                              | Normalize redundant separators and lexical dot components in caller storage                               |
| `path.join_into(parts, &!buffer)`           | `path.Path` or `path.Error`                              | Join a nonempty path list; reject absolute components after the first                                     |
| `fs.canonicalize(value, allocator)`         | `path.Owned` or `fs.Error` or `memory.AllocationFailure` | Resolve filesystem identity and symlinks explicitly                                                       |

Lexical normalization preserves root/volume identity and does not cross above an
absolute root. It does not change case, expand `~`, interpolate shell variables,
or substitute import aliases. A clean path is not a proof of directory confinement.
`path.Owned.view()` borrows its path; dropping the owner invalidates that view.
Host-native filenames that are not representable in UTF-8 use `fs.NativeName`
from directory iteration and the corresponding native open operations, never a
lossy text conversion.

Path `_into` operations take caller-owned bounded byte lists. They validate and
measure before writing, preserve the output on failure, and return a path view
borrowing the replaced contents. Input must not alias mutable output.

## Files and directories

| API                                                                          | Result                                                                  | Contract                                                                       |
| ---------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `fs.open(path, mode)`                                                        | `fs.File` or `fs.Error`                                                 | Open an owned file handle with explicit access/create policy                   |
| `file.read(buffer)`, `.write(bytes)`                                         | `io.Read`, `io.Write`                                                   | Transfer bytes with visible partial progress                                   |
| `file.seek(offset <int64>, origin)`                                          | `uint64` or `fs.Error`                                                  | Seek a seekable handle; origin is Start, Current, or End                       |
| `file.metadata()`                                                            | `fs.Metadata` or `fs.Error`                                             | Query kind, byte length, permissions, and available timestamps                 |
| `file.sync()`                                                                | `null` or `fs.Error`                                                    | Request durable data/metadata flush from the host                              |
| `file.close()`                                                               | `null` or `fs.Error`                                                    | Consume the owner and report the close result                                  |
| `fs.read_all(path, allocator, limit <usize>)`                                | `collections.Vector<uint8>` or `fs.Error` or `memory.AllocationFailure` | Read under an explicit byte limit; never trust metadata as the final length    |
| `fs.write_all(path, bytes, mode)`                                            | `null` or `fs.Error`                                                    | Open, write fully, and close; not an atomic replacement                        |
| `fs.replace(path, bytes, options, allocator)`                                | `null` or `fs.Error` or `memory.AllocationFailure`                      | Stage in the destination directory, then replace according to the options      |
| `fs.entries(path, allocator)`                                                | `fs.Directory` or `fs.Error` or `memory.AllocationFailure`              | Open an owned, unsorted directory cursor                                       |
| `fs.create_directory(path)`, `.remove_file(path)`, `.remove_directory(path)` | `null` or `fs.Error`                                                    | Perform exactly the named operation; directory removal requires it to be empty |
| `fs.rename(source, destination, replace <boolean>)`                          | `null` or `fs.Error`                                                    | Request a same-filesystem rename with explicit replacement permission          |

`fs.ReadOnly`, `ReadWrite`, `CreateNew`, and `ReplaceContents` are immutable mode
values. ReadOnly and ReadWrite require an existing file. CreateNew uses exclusive
creation and fails if the path exists; ReplaceContents creates or truncates.
Both creation modes open the resulting file for writing.
There is no append behavior hidden in an ordinary write mode.

File owners are move-only. Normal cleanup releases a handle without throwing;
call sync/close explicitly to observe durability or close failures. A failed
close still consumes the handle and must not be retried against a potentially
reused native descriptor. File positions are shared by that handle's operations,
so concurrent access requires an explicit ownership/synchronization choice.

`fs.Metadata` includes `kind`, `size <uint64>`, `permissions`, and nullable
`modified`, `accessed`, and `created` date.Timestamp fields. Missing timestamps
remain null rather than pretending to be the Unix epoch. A metadata snapshot is
not a lock on the underlying filesystem object.

`Directory.next()` returns an entry, `iter.End`, or `fs.Error`. Entry names borrow
the cursor's current buffer and expire before the next advance. NativeName can
be passed to `fs.open_native(directory, name, mode)`; text conversion is fallible.
Iteration does not recurse or follow child symlinks. Use an explicit stack and
policy for tree traversal.

Replace options contain `replace <boolean>` and `sync <boolean>`. The helper
writes and closes a temporary sibling before the rename. It reports whether an
error occurred before or after replacement became visible. A host unable to
provide the requested atomic replacement or durability reports Unsupported; it
never silently downgrades to delete-then-copy. Symlink behavior is reported by
the selected filesystem contract, not inferred from path cleanup.

## Environment and processes

`env.snapshot(allocator)` returns an owned `env.Environment`, `env.Error`, or
`memory.AllocationFailure`. `.get(name)` returns a borrowed `<string><null>`;
absence differs from an empty value. A snapshot is stable even if foreign code
later changes the process environment. Text conversion errors are explicit;
there is no lossy replacement of native data.

`process.arguments(allocator)` returns `process.Arguments`, `process.Error`, or
`memory.AllocationFailure`. `.program()` borrows the invocation name, and `.slice()`
borrows UTF-8 arguments excluding that name. It preserves argument boundaries;
it does not reconstruct a shell command and split it again. Non-UTF-8 native
arguments produce a text conversion error. `process.working_directory(allocator)`
returns `path.Owned` or a corresponding process/allocation error.

| API                                     | Result                                                                 | Contract                                                                                                   |
| --------------------------------------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `process.spawn(spec, allocator)`        | `process.Child` or `process.Error` or `memory.AllocationFailure`       | Start an executable with explicitly supplied arguments, directory, environment, and standard-channel modes |
| `child.wait()`                          | `process.Status` or `process.Error`                                    | Wait and reap once, acknowledging task cancellation during the wait                                        |
| `child.take_stdin()`                    | `process.PipeWriter` or `null`                                         | Transfer the parent end of a Pipe stdin, leaving that slot empty.                                          |
| `child.take_stdout()`, `.take_stderr()` | `process.PipeReader` or `null`                                         | Transfer the selected parent output-pipe end, leaving that slot empty.                                     |
| `child.terminate()`, `.kill()`          | `null` or `process.Error`                                              | Request graceful termination or host-supported forced termination                                          |
| `process.run(spec, allocator, limits)`  | `process.Output` or `process.RunFailure` or `memory.AllocationFailure` | Collect stdout/stderr concurrently with byte limits and a monotonic deadline                               |

A spawn spec has `executable <path.Path>`, `arguments <string[]>`,
`directory <path.Path>`, `environment <&env.Environment>`, and `stdin`, `stdout`,
`stderr` modes selected from `process.Inherit`, `Null`, or `Pipe`.
Executable paths are explicit; no shell or PATH search is implied. The host copies
argv/environment during spawn, so the child does not retain their borrowed views.
The explicit allocator covers argument marshalling and retained child state;
it must outlive the Child. A failed spawn releases partial state and handles.
Pipe modes initially retain owned parent handles inside Child. Use the `take_`
methods to extract them; direct field moves from the opaque Child are forbidden.
Each method exclusively borrows Child, allocates nothing, and returns `null` if
the selected mode was not Pipe or its handle was already extracted. Extraction
does not consume Child, reap the child, or change its other pipe slots. An
extracted handle has independent ownership and no borrow of Child; its retained
allocator must still outlive it. Child cleanup cannot close an extracted handle.

`PipeReader.read(buffer)` and `PipeWriter.write(bytes)` use `io.Read` and
`io.Write`; each requires exclusive access to its handle. Both expose a consuming
`close()` returning `<null><io.Error>`; a failed close still consumes the handle.
Dropping an extracted handle closes its parent end without throwing. Closing a
stdin PipeWriter delivers EOF after previously committed bytes have been read;
closing an output PipeReader stops collection and may make later child writes
fail. Extraction and closure never close the child's copies of its descriptors.
Child, PipeReader, and PipeWriter are move-only, transferable under the
[allocator/capability rules](../tasks-and-channels.md#capability-rules),
and are not `tasks.Sync`. Move extracted output readers into tasks to drain both
streams concurrently. Waiting before draining a full output pipe can block.

A Child owns the reaping obligation. Dropping an unreaped child requests
termination, closes remaining parent pipe ends, and waits; cleanup can block.
There is no implicit detached process. `Status` records a nullable `code <int32>`
and nullable `signal <int32>`, exactly one present for a completed process.
A nonzero application status is a normal Status, not automatically a spawn error.
`wait` exclusively borrows Child. After a successful wait it retains the completed
Status and subsequent waits return that same Status without reaping again.
`terminate` and `kill` on a reaped Child return `null` without signaling anything;
they never act on a reused process ID. An unsuccessful wait before host completion
retains the reaping obligation with Child. If the host instead confirms that the
child is no longer waitable (for example, foreign code reaped it), Child settles
with that process error: repeated waits return the same failure, and cleanup or
termination methods never signal the old process ID. Extracted pipe handles may
still be drained/closed after the child is reaped.

`process.run` limits contain `stdout_bytes <usize>`, `stderr_bytes <usize>`, and
`deadline <time.Instant><null>`. It requests termination and reaps the process if
capture exceeds a limit or deadline. Output owns byte vectors and a Status;
RunFailure retains available output, termination evidence, and its failure cause.
Captured data is never silently truncated and reported as a complete success.
The run helper requires stdout/stderr to use Pipe, supplies and drains those
pipes itself, and rejects other output modes. Stdin must be Inherit or Null;
an explicitly fed stdin pipe belongs in a spawn-based workflow.

## Network services

`net.parse_address(text)` returns an inline IPv4/IPv6 `net.Address` or
`net.InvalidAddress`. It parses numeric addresses only, without DNS or a default
port. `net.endpoint(address, port <uint16>)` combines values without opening a
socket. `net.scoped_endpoint(address, port, scope <uint32>)` additionally selects
an IPv6 interface index and returns Endpoint or InvalidAddress; zero means no
scope and a nonzero scope on IPv4 is invalid. Interface indices belong to the host.

| API                                                    | Result                                                       | Contract                                                           |
| ------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------------ |
| `net.resolve(name, port, allocator, deadline)`         | `net.Addresses` or `net.Error` or `memory.AllocationFailure` | Resolve a host with an explicit nullable monotonic deadline        |
| `net.connect(endpoint, deadline)`                      | `net.Stream` or `net.Error`                                  | Connect a TCP stream, without implicit retries to another endpoint |
| `net.listen(endpoint, backlog <uint32>)`               | `net.Listener` or `net.Error`                                | Bind and listen; zero port lets the host select one                |
| `listener.local_endpoint()`                            | `net.Endpoint`                                               | Read the actual bound address and port                             |
| `listener.accept(deadline)`                            | `net.Stream` or `net.Error`                                  | Wait for one accepted stream                                       |
| `stream.read(buffer)`, `.write(bytes)`                 | `io.Read`, `io.Write`                                        | Byte-stream I/O; message boundaries are not preserved              |
| `stream.deadline(instant <time.Instant><null>)`        | `null`                                                       | Set/clear the stream's read and write deadline                     |
| `stream.shutdown_write()`                              | `null` or `net.Error`                                        | Send an orderly write-side shutdown while retaining reads          |
| `net.datagram(endpoint)`                               | `net.Datagram` or `net.Error`                                | Bind a UDP endpoint                                                |
| `socket.receive(buffer, deadline)`                     | Datagram result or `net.Error`                               | Return sender, count, and explicit truncation status               |
| `socket.send(destination, bytes, deadline)`            | `null` or `net.Error`                                        | Submit one complete datagram or fail                               |
| `stream.close()`, `listener.close()`, `socket.close()` | `null` or `net.Error`                                        | Consume the owner and release its host resource                    |

Network owners are move-only. Blocking waits are task cancellation points;
network runtime support suspends the waiting task rather than occupying a worker
for the whole wait. Without an executor they block the calling host thread.
Explicit network deadlines return TimedOut errors; cancellation inherited from a
task follows the task's unwind contract. Successful stream data already read or
written remains progress even if the deadline then expires.

Addresses owns its resolver results and exposes a borrowed slice; DNS order is
not promised stable across calls. A truncated datagram reports the bytes retained
and `truncated : true`; the discarded suffix cannot be recovered by another read.
A zero-length datagram is a message, not end-of-stream. None of these APIs infers
TLS, an application protocol, or an encoding from a port number.

## Recorded effects

Filesystem, process, environment, network, clock, and entropy observations are
external inputs. Recording captures supported boundaries and labels uncaptured
ones explicitly. Replaying a saved executable does not automatically make a live
network peer or mutable filesystem deterministic. The
[replay contract](../diagnostics.md#replay-fidelity) governs which recorded results
replace live effects and when a replay must report missing evidence.

See the self-contained [file hash](../../programs/file-hash/README.md) and
[file copy](../../programs/file-copy/README.md) projects for initialized buffers,
partial progress, explicit acquisition, and checked sync/close results.
