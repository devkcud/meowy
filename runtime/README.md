# Native cleanup prototype

This directory exercises an explicit cleanup protocol independently of the
compiler. It uses C++20 and Clang 22.1.8 with exceptions and RTTI disabled. The
compiler still links its existing scalar runtime; this prototype adds no Meowy
syntax or runtime symbols to generated programs.

```sh
python3 -B runtime/check.py
python3 -B -m unittest discover -s runtime/tests -p 'test_*.py'
```

The runner builds debug, optimized release, and ASan/UBSan executables in a
temporary directory. Each executes 14 cleanup cases and two fatal subprocesses.
Fatal cases require `SIGABRT` and exact P008 evidence, including the initial exit
cause and failing cleanup; an arbitrary crash cannot pass. Core dumps are disabled
in those children. Timeouts kill and reap the subprocess group.

Use `--build-dir runtime/build` to retain executables. `--clang PATH` selects a
compiler executable but still requires version 22.1.8. `--no-sanitizers` explicitly
omits sanitizer checks. Missing tools, sanitizer failures and unsupported execution
environments fail the selected checks. LeakSanitizer requires an environment
without ptrace-based sandbox supervision; run the normal command with the required
environment access when that restriction applies.

## Storage and cleanup contract

The experimental C++ interface is
[include/meowy/cleanup.hpp](include/meowy/cleanup.hpp), in
`meowy::prototype::v0`. Its name is a revision marker; it is **not** a stable C ABI,
compiler bridge, public FFI or qualified private Meowy runtime ABI.

- A noncopyable `Stack` borrows a caller-provided `Entry` array. Arrays must not
  overlap between live stacks. The array and stack must remain at stable addresses.
  Storage admission is bounded; this implementation allocates no backing storage.
  Marks and tokens cannot survive destruction or reconstruction of their stack.
- Call `reserve()` immediately before construction. A full stack returns `full`
  before construction starts. After successful initialization, `arm()` registers
  the payload, release callback and operation name. Only the newest reservation
  may be armed, once, so reservation order agrees with initialization order.
  A failed construction leaves its reservation unarmed; cleanup ignores it.
- A release callback is `Panic (*)(void *) noexcept`. A zero code means success;
  a nonzero code is a panic. Callbacks must finish synchronously. A callback cannot
  mutate the stack currently cleaning up; those operations return `invalid`.
  C++ exceptions are not the panic protocol.
- `mark()` records a boundary. `unwind()` releases armed entries after that mark
  in reverse order and removes all reservations after it. Scope exits are explicit
  calls; C++ destructors perform no Meowy cleanup. Tokens and nonempty marks carry
  generation identifiers, rejecting stale references after a slot is reused.
- `disarm()` consumes one armed cleanup obligation after an explicit successful
  release or transfer outside this protocol. It does not call the callback or
  move the payload. Disarmed reservations occupy capacity until their scope exits.
- `transfer()` moves an armed cleanup obligation to another stack. It preserves
  the callback and payload address, disarming the source only after destination
  admission succeeds. Failure leaves the source responsible. **Object storage
  does not move, and its lifetime is not extended.** A later owner must already
  have valid surviving storage; this operation provides no relocation algorithm.
- Payload storage must survive its callback; operation names and second-panic
  messages must survive fatal reporting after the callback returns. Original panic
  messages must survive the entire unwind and outcome inspection. These addresses
  remain the caller's responsibility, including after a transfer.
- Successful `unwind()` returns its exit reason and original panic unchanged.
  Cleanup panic is always fatal, including on normal completion. P008 writes the
  original cause, operation and second panic to stderr, then aborts. Fatal process
  termination does not promise further cleanup.

These functions are synchronous and do not suspend, schedule or synchronize.
Mutation requires exclusive caller access. Release callbacks may run resource
operations of their own; the stack does not grant allocation or failure recovery
guarantees for those operations.

## Lowering experiments

| Edge selected by generated code | Explicit protocol exercised |
| --- | --- |
| Normal completion | Release all remaining local obligations back to the scope mark. |
| Named leave with a completed result | Transfer the emitted result obligation into surviving caller-owned storage tracking, then release inner locals to the named mark. |
| Named restart | Keep outer reservations; release all current iteration locals and emitted results, then reserve fresh iteration slots. |
| Failed construction | Leave incomplete slots unarmed; release only successfully initialized owners. |
| Recoverable panic | Release locals and partial emitted results; return the original panic status to an explicit boundary. |
| Acknowledged cancellation | Release to an explicit boundary with a cancellation reason. No scheduler or checkpoint is implemented. |

The fixtures supply these edges directly. The compiler does not yet generate
them. In a full implementation, completed results cannot be published until
required child joins and cleanup succeed. This stack has no child handles and
cannot enforce that ordering. The result-transfer fixture only proves obligation
transfer and reverse cleanup for a scope without children.

Keep partial emissions on their iteration stack until normal completion commits
the result. Moving them to a separate stack while the body is still running loses
the combined reverse initialization order on panic when locals and emissions are
interleaved. A future lowering that transfers earlier must preserve that ordering
with additional metadata. The partial-transfer fixture models a committed owner
transfer, not an early publication of a block's incomplete result.

The language rules remain [memory cleanup](../docs/reference/memory.md#cleanup),
[named scope cleanup](../docs/reference/values-and-blocks.md#named-scopes-and-cleanup)
and [task scope exit](../docs/reference/tasks-and-channels.md#scope-exit).
[COMPILER.md](../COMPILER.md#prove-the-runtime-before-it-gets-comfortable) specifies
the actual stack/unwind implementation plan.

## Validation boundary and next steps

This is a bounded protocol experiment on the current Linux host. It does not
qualify task stacks, native stack unwinding or the documented v0.0.1 release.
There is no vendored Boost.Context, stack switching, guard-page allocation,
register-preservation test, worker pinning, scheduler, join, timer or channel.
There is no LLVM landing pad, Meowy personality function or pinned unwind library.
Panic codes/messages are borrowed test inputs; source spans, task identity and
diagnostic attachment are absent. Cancellation is an explicit cleanup edge only.
Nothing promotes owners or borrows into arbitrary heap storage.

1. Design the compiler's generated cleanup edges and initialized-slot metadata
   alongside moves and partial initialization. Decide whether the experimental
   ordered reservation restriction should survive that design.
2. Implement and independently qualify the planned context wrapper with bounded
   stacks, guard pages, register preservation and sanitizer switching hooks.
3. Add LLVM landing pads, a Meowy personality and task-root outcomes using a pinned
   unwind library; preserve P008 and cleanup ordering across nested calls.
4. Exercise a suspended child borrowing a parent local, cancellation while joining,
   and cleanup that waits for children before releasing their borrowed storage.
   Independent passing cleanup/context tests do not qualify their interaction.
