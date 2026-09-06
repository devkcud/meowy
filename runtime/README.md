# Native runtime prototypes

This directory exercises explicit cleanup, owned payloads, guarded stack allocation
and pinned context switching with bounded scheduling independently of the compiler. It uses C++20 and
Clang 22.1.8 with exceptions and RTTI disabled. The
compiler still links its existing scalar runtime; this prototype adds no Meowy
syntax or runtime symbols to generated programs.

```sh
python3 -B runtime/check.py
python3 -B -m unittest discover -s runtime/tests -p 'test_*.py'
```

The runner builds debug, optimized release, and ASan/UBSan executables in a
temporary directory. Each executes 14 cleanup cases and two fatal subprocesses,
plus 10 stack allocation cases, a kernel admission-refusal subprocess and two
guard-fault subprocesses. Each profile also runs 10 context cases and a fatal
cleanup-after-resume subprocess. The sanitizer profile additionally requires an
ASan stack-use-after-return report for a deliberately expired fiber local.
Each profile also runs 22 scheduler cases, a real admission-refusal subprocess,
a fatal task-cleanup subprocess and four child/scope protocol probes.
Each profile also runs 13 owned-value cases and three fatal owned-cleanup probes,
including rejected admission and owned-result discard during scope closing.
Cleanup panic cases require `SIGABRT` and exact P008 evidence, including the initial exit
cause and failing cleanup; an arbitrary crash cannot pass. Core dumps are disabled
in those children. Timeouts kill and reap the subprocess group.

Stack probes require `SIGSEGV`, `SEGV_ACCERR`, the exact protected boundary
address and execution of the fault handler on an alternate signal stack. They
prove guard protection by direct writes, not recovery from an overflowing task.
The normal allocation cases run under ASan/UBSan/LSan. Guard probes disable ASan's
SIGSEGV handler and instrumented faulting access so the kernel signal is inspected
directly. Context tests separately exercise the fiber hooks with instrumented
callback bodies, live cross-switch stack borrows and the expired-local probe.

Use `--build-dir runtime/build` to retain executables. `--clang PATH` selects a
compiler executable but still requires version 22.1.8. `--no-sanitizers` explicitly
omits sanitizer checks. Missing tools, sanitizer failures and unsupported execution
environments fail the selected checks. LeakSanitizer requires an environment
without ptrace-based sandbox supervision; run the normal command with the required
environment access when that restriction applies.

## Explicit owned values and task payloads

[include/meowy/owned.hpp](include/meowy/owned.hpp) adds a noncopyable `Owned` token
over fixed caller-provided bytes. This owns an initialized resource in real
storage; it is separate from the cleanup stack's obligation-only transfer.

- Backing buffers must be valid, exclusively held, stable and nonoverlapping across
  live `Owned` values and scheduler slots. They must outlive their tokens and any
  borrowed views. The primitive checks destination capacity/alignment and rejects
  overlapping source/destination buffers; it cannot validate arbitrary C++ pointers.
- A `ValueOps` descriptor supplies size, alignment, move, drop and diagnostic name.
  It must have static lifetime. Move is infallible and non-suspending: it constructs
  the destination and ends the source object's lifetime without releasing the
  transferred resource. Drop is non-suspending and ends the object's lifetime;
  returning a panic takes the existing fatal P008 cleanup path. Neither callback
  may rely on automatic C++ destructors to implement Meowy ownership.
- `reserve(ops)` validates empty storage before construction. The caller explicitly
  constructs the object and then calls `commit()`. A failed or cancelled constructor
  leaves the token reserved, and `release()` skips destruction of that uninitialized
  slot. Partial construction must clean its own initialized pieces before release.
- `move_to(destination)` preflights both tokens, relocates the object and empties
  the source. Failure leaves source ownership intact. Both tokens reject reentry
  during relocation; a moved-from token cannot expose the old object through
  `data()`. `release()` explicitly destroys a live value once and is harmless on
  empty storage. Token destruction never silently releases an object.
- `TaskSlot(capture_bytes, result_bytes)` admits fixed payload buffers independently
  of its guarded execution stack. Default slots still support borrowed callbacks
  but have no owned-payload capacity. No owned operation grows these buffers or
  allocates an auxiliary container. Owned result storage must remain valid even
  after the completed task's execution stack is unmapped.
- `submit_owned(body, capture)` and `Task::spawn_owned(body, capture)` accept a live
  capture after worker/capability/body checks. Invalid API preconditions preserve
  caller ownership. Valid submission consumes it **before executor admission**:
  full capacity or generation exhaustion drops it once even though no ticket can
  be admitted; payload-capacity or OS admission failure also releases it without
  running the body. An admitted failure ticket retains `spawn_failed` and any
  failed stack rollback for explicit join retry. Storage failures are reported
  separately from context failures.
- On accepted admission, the body receives a pointer to the relocated capture in
  its slot, not the original caller buffer. Owned submission has no separate user
  cleanup callback; its descriptor handles capture release after the body and all
  children finish. The borrowed submission API keeps its existing callback cleanup.
- During its body, an active task can `set_result(value)` from an initialized
  `Owned` token or `emit_capture()` to move its capture directly into the result.
  Result storage is initialized at most once. Failure preserves the source value.
  Both operations reject outstanding children and cleanup-phase calls; capture
  borrows must finish at explicit joins before capture storage can move. After
  `emit_capture()`, the body must stop using its old capture pointer.
- A normal result survives settlement while the remaining capture is released.
  A body panic releases an already emitted result before the capture, in reverse
  initialization order. Cleanup panic is fatal and does not promise further drops.
  Results must own their transferred resources or refer only to independently
  surviving storage: a result cannot borrow task-stack or capture storage that
  settlement releases. This prototype has no compiler-enforced payload type/lifetime
  contract; descriptors and callbacks must uphold those requirements.
- `Scheduler::join_owned(ticket, destination)` and `Task::join_owned(child, destination)`
  require an empty destination. They preflight the result move, release the completed
  context, then relocate the result and consume the ticket. A failed destination
  check or context release leaves the result in its original slot, destination
  untouched, and ticket/parent obligation valid for retry. Joining allocates no
  additional result storage. Plain `join()` rejects a live owned result rather
  than silently discarding it; use `join_owned()` and explicitly release the owner,
  or explicitly close the child's task scope to discard the result.
- Scheduler-owned move/drop callbacks reject scheduler reentry and task suspension.
  Generic `Owned` operations still require caller-exclusive access and non-suspending
  callbacks. Admission and join never manufacture ownership for a borrowed pointer.

Native tests cover relocation, initialized-only cleanup, alignment/capacity and
overlap errors, callback reentry, accepted full/storage/OS failures, panic ordering,
destination/release retry, owned child-to-parent transfer and parent capture borrows.
A real pipe descriptor remains open through transfers and closes only when its final
owner is explicitly released. Automatic compiler payload layout, full typed task
results, allocation policy and cancellation unwinding remain separate work.

## Explicit task scope closing

`Task::mark()` opens a scope for the currently running task; `Task::close(mark)`
finishes it explicitly while the parent's C++ locals still exist. It is a runtime
foundation for future generated scope-exit code, not automatic joining or unwinding.

- Each `TaskSlot` contains exactly 16 scope records (`Task::scope_limit`). This is
  bounded prototype metadata capacity in the caller's slot array, independent of
  the configured execution-stack budget. Opening a seventeenth scope returns
  `full` without changing existing records.
- Marks are opaque tokens tied to parent identity and a nonreused generation.
  Generation exhaustion rejects opening instead of wrapping. Only the innermost
  open mark can close; default, foreign, stale, already closed and out-of-order
  marks reject. The active-task and worker checks also apply to scope operations.
- A mark records the next admission sequence. Closing selects remaining direct
  children admitted since that mark, in submission order, and waits through the
  existing worker-yielding child-join path. Children admitted earlier remain the
  parent's responsibility. An inner close does not consume an outer scope's older
  children. This deterministic reporting order grants no language scheduling promise.
- Closing explicitly discards owned child results. It releases the completed
  child's context first, then destroys its result and consumes its ticket. Failed
  context release leaves that child/result and the mark intact; a later close can
  retry. Earlier successfully reclaimed children are never joined or dropped again.
  Plain joins retain their existing protection against discarding owned results.
- `ScopeClose` reports cumulative `joined`, `panicked` and `spawn_failed` counts
  plus the first consumed failure. Counts advance only after successful reclamation
  and remain in the slot across retries, including retries through a copied mark.
  A failure to reclaim exposes the `pending` child and its `pending_result`, with
  context/storage errors and outcome, separately from those cumulative counts.
- Counts summarize every failure consumed by close; only the first diagnostic is
  retained in this bounded report. Additional diagnostic attachment is not provided.
  The caller must handle child failures explicitly. `ok` means scope closing
  finished, not that all children succeeded; close does not automatically raise a
  Meowy panic or propagate cancellation. Panic text remains borrowed and must
  survive the entire close, any retries and subsequent report use. Children manually joined elsewhere are
  already observed and do not contribute to a later close's counts.
- A body or cleanup callback must close all its marks before returning. An open
  mark is a private fatal protocol violation, even when it contains no children.
  A cleanup callback may open/close scopes while its own locals are still alive.
  C++ destruction never calls close, and an open mark cannot extend local lifetimes.

Close waits for children to finish; it neither requests cancellation nor imposes
a time limit. No callback can be forcibly unwound by this prototype. Native checks
cover nested boundaries, older children, exact capacity, parent/worker generations,
failure summaries, body/cleanup locals and partial close retry with an owned result.
Fatal probes distinguish unclosed-scope misuse from P008 when discarded result
cleanup itself panics.

## Bounded worker scheduler

[include/meowy/scheduler.hpp](include/meowy/scheduler.hpp) adds one explicitly
driven worker over the pinned context wrapper. This is an experimental runtime
API; it does not implement the complete Meowy task or group contract.

- Construct `Scheduler` on its owning worker with a caller-owned `TaskSlot` array
  and fixed per-context usable stack bytes. The array is also the bounded runnable
  queue: selection scans runnable slot states without another allocation. The
  array must remain at a stable address and must not overlap another live
  scheduler's storage. It must outlive the scheduler and every pending ticket.
- `submit(body, data, cleanup)` reserves a vacant slot and creates a guarded
  context without running either callback. Full capacity returns `full` with no
  ticket. Both queued and settled-but-unjoined slots count against capacity.
  Context creation failure returns `context_failed` **with a valid settled ticket**;
  inspect and join that ticket to consume its `spawn_failed` result and reclaim
  any retained mapping from a failed rollback. No body or cleanup runs on failed
  admission; the caller remains responsible for its data and resources.
- Callback type is `Panic (Task &, void *) noexcept`; a zero code is normal
  completion. For plain `submit`/`spawn`, `data` is borrowed caller-owned storage,
  not an automatically copied or moved capture. Owned submission is described
  above. The temporary `Task` capability cannot be copied or outlive
  its callback activation. It exposes `yield()`, child `spawn()` and waiting
  child `join()` within body/cleanup calls. Child APIs require the exact active
  task identity; a parked parent's or sibling's capability cannot control work
  while another task is running.
  All borrowed data and result/panic text must outlive their use, including later
  outcome inspection or use after join.
- The body executes first; its optional cleanup executes on the same context
  before the outcome becomes settled. Cleanup may yield, retaining the occupied
  slot and unpublished outcome until it finishes. Cleanup receives the same
  caller-owned data, and must not reference body-local storage whose function has
  already returned. Owners local to a C++ body need explicit cleanup inside that
  body before return. Compiler-generated Meowy ownership edges are still pending.
- `Task::spawn(body, data, cleanup)` admits a direct child in the same fixed slot
  pool, recording the parent's complete ticket. Roots, descendants, waiting tasks
  and settled tasks all share that capacity. Failed context admission returns a
  settled child ticket that still belongs to its parent and must be joined;
  capacity refusal without a ticket creates no child obligation.
- `Task::join(child)` accepts only the active task's direct child. It suspends the
  parent in `waiting` state while the host pump runs other work on the same worker.
  The child wakes that parent only after its body and cleanup have completed.
  A different child's completion cannot satisfy the wait. Successful join releases
  the child mapping and consumes its ticket; release failure keeps the parent's
  child count and settled ticket for explicit retry. Self, sibling, ancestor,
  unrelated root and foreign-scheduler joins are rejected.
- **Join children before their borrowed locals leave scope.** Returning from a
  body or cleanup with any unjoined child is a private fatal protocol violation,
  checked before more work is scheduled or the task settles. It is not P008 and
  is not recoverable Meowy cancellation. Automatically joining after a C++ body
  returns would access expired local storage; this prototype requires explicit
  joins while that storage is alive. Compiler-inserted scope-exit joins, including
  exceptional exits, remain future work.
  `Task::close()` provides an explicit bulk join/discard operation for a marked
  scope, with the same requirement to finish before its locals leave scope.
- Parent cleanup starts only after all body children have been joined. A cleanup
  callback may create and join its own children before returning. Nested parents
  use more slots from the same fixed pool, without a heap-allocated child list or
  recursive host scheduling. Parent/waiting identities and outstanding child counts
  appear in `inspect()` results, alongside open scope depth; a parent cannot settle
  or release with children or unclosed marks.
- A body panic result becomes `panicked` only after cleanup. A panic returned from
  cleanup takes the existing fatal P008 path with the body cause and cleanup
  operation. The scheduler does not infer panic from host exceptions or crashes.
- `pump(limit)` resumes at most `limit` contexts across yield/completion boundaries.
  The result reports actual resumes plus runnable and waiting slots. This bounds
  transitions, not CPU time: a body or cleanup that never yields can block the
  worker. There is no preemption, deadline or background worker thread.
- The current [selection policy](src/task_policy.cpp) scans from a rotating cursor,
  choosing round-robin among runnable slots. Completed slots are skipped. This
  provides the tested prototype behavior; language programs gain no scheduling
  order or fairness guarantee from it.
- `inspect(ticket)` reports an unpublished `pending` outcome until settlement.
  Host `Scheduler::join(ticket)` accepts only settled root tasks and remains
  nonblocking. A child can only be consumed by its parent's `Task::join`; the
  host cannot steal it. Joining releases storage, returns the outcome and consumes
  the ticket exactly once. A release failure preserves the settled slot and
  ticket for retry; capacity is not returned before successful release.
- Tickets contain scheduler identity, slot index and a monotonic generation.
  Stale, zero-generation and foreign-scheduler tickets reject. Generations never
  wrap; admission returns `full` when their range is exhausted. Tickets must not
  survive scheduler destruction/reconstruction, even at the same address.
- Every scheduler operation stays on its constructor's worker. That worker must
  remain alive until every ticket is joined. Callback attempts to reenter host
  submit, inspect, pump or join are rejected; child operations use the checked
  `Task` capability instead. No locks are held while executing callbacks;
  caller synchronization and exclusive storage ownership remain prerequisites.
  Destruction does not automatically drain, cancel or join pending work.

Tests cover bounded round-robin progress, empty/wrapped selection, pending cleanup,
capacity through settlement, stale/foreign tickets, worker and reentry rejection,
admission failures and release retry. Fault-injected rollback failure keeps its
mapping through a failed join; a kernel `RLIMIT_AS` refusal verifies a joined
`spawn_failed` outcome without running callbacks. Child tests keep parent and
cleanup locals alive across waits, exercise nested chains and unrelated worker
progress, and reject active-capability misuse. Unjoined-child probes require exact
private failure text and `SIGABRT`; no abandoned child is resumed to fake cleanup.
Scoped task groups, automatically lowered captures/results, implicit scope-exit
joins, cancellation unwinding, timers, channels and a multi-worker executor remain
unimplemented. An observed child panic is returned to the explicit join caller;
automatic propagation of unobserved child failures is not provided.

## Pinned context contract

[include/meowy/context.hpp](include/meowy/context.hpp) wraps the minimal
[pinned Boost.Context import](vendor/boost-context/README.md). The selected target
is Linux x86-64 SysV ELF LP64. The wrapper is an experimental private C++ interface,
not a stable runtime ABI or a scheduler.

- A caller-owned, noncopyable and nonmovable `Context` contains its `StackMemory`
  owner. `initialize(bytes, body, data)` explicitly allocates the requested guarded
  stack and prepares an entry without running the body. The current minimum is
  65,536 usable bytes, also subject to the allocator's page-alignment restriction.
  The context record and borrowed callback data must remain valid until release.
- States are `empty`, `ready`, `running`, `suspended` and `completed`. `resume()`
  starts or continues a ready/suspended context; `yield()` returns to that call.
  Yield works within ordinary nested body calls. Body return marks completion and
  explicitly switches back; it never returns into Boost's process-exit trampoline.
- Only a host execution frame may resume a context. Resuming another context from
  within a context is rejected. The host may alternate multiple contexts. The
  first resume selects the pthread, and every later resume/yield must use that
  worker. Creating a context on another thread before its first resume is allowed.
- The pinned worker must stay alive until the context is completed and released.
  All access requires caller synchronization; thread identity checks do not make
  concurrent calls safe. Thread-local state and signal masks belong to the worker,
  not an isolated task. Contexts must not change native shadow-stack configuration.
- `release()` accepts empty, unstarted or completed contexts. Running and suspended
  contexts are rejected, so their storage cannot be unmapped through this API.
  Once pinned, release also requires that worker. OS release failures retain the
  owner for retry. Destruction does not cancel, unwind, resume or release a context.
- Callback type is `void (Context &, void *) noexcept`. Callback data/results are
  caller-owned; there is no automatic capture allocation or typed task outcome.
  Callbacks explicitly run their cleanup before returning. C++ exceptions and
  destructor-driven continuation unwinding are not Meowy panic or cancellation.

The preserved machine state is the x86-64 calling-convention state: stack pointer,
continuation, callee-saved integer registers and floating-point control state.
Caller-saved registers retain ordinary call semantics. Tests check all six integer
callee-saved registers, FP rounding state, nested locals through 64 yields, host
alternation and execution within the guarded mapping.

ASan's [fiber interface](https://github.com/llvm/llvm-project/blob/llvmorg-22.1.8/compiler-rt/include/sanitizer/common_interface_defs.h)
is called immediately around each switch. The wrapper preserves each fake-stack
handle, discovers the host bounds on initial entry and destroys the fiber's fake
stack on terminal departure. Only the three transition shims exclude address
instrumentation so the hook handshake precedes instrumented body execution;
callback code remains instrumented. The gate enables stack-use-after-return
detection and requires the deliberate expired-local read to be diagnosed. ASan's
own fake-stack allocations are instrumentation overhead outside the configured
native usable-stack budget. TSan and sanitizer signal-handler interactions are
not qualified.

The prototype rejects active CET shadow stacks through
[Linux's status interface](https://docs.kernel.org/arch/x86/shstk.html) and builds
without CET code generation. It does not silently disable an enabled feature.
Unknown status failures reject rather than switching with an unverified mode.

An explicit cancellation fixture keeps a parent local alive while a borrowing
context suspends, observes a caller-set cancellation flag, and runs cleanup that
itself yields before finishing. Release remains rejected until completion. This
proves that the current cleanup protocol and context switches compose in that
bounded scenario. The scheduler above adds fixed-slot child admission and explicit
waiting joins; automatic cancellation requests and task-root panic unwinding
remain pending.

## Guarded stack allocation contract

[include/meowy/stack_memory.hpp](include/meowy/stack_memory.hpp) adds the
experimental `StackMemory` owner in the same `meowy::prototype::v0` namespace.
This is the Linux `mmap` system-allocation foundation used privately by `Context`.
Standalone allocation tests do not execute a context on the mapping.

- `plan(bytes, page)` checks positive page-aligned usable bytes, two guard pages
  and a total within `PTRDIFF_MAX`, without allocating. The explicit usable budget
  is never rounded up. Page alignment is an experimental allocator restriction,
  not a newly imposed language manifest rule.
- `allocate(bytes)` obtains the host page size, reserves the complete mapping as
  `PROT_NONE`, and enables read/write access only for the usable middle pages.
  Guard pages are neither readable, writable nor executable; the payload has no
  execute permission. `size()` reports usable bytes, one guard's size and the
  complete mapped extent, so guard overhead stays visible outside the usable budget.
- Allocation is fallible and synchronous. Invalid sizes, overflow, page-query,
  mapping and protection failures have distinct statuses and preserve OS `errno`
  where applicable. A failed protection change tries to unmap immediately. If
  rollback also fails, both errors are returned and the owner retains the mapping
  for release retry; `data()` remains null because usable storage was not prepared.
- The owner is noncopyable and nonmovable. `allocate()` rejects an already owned
  mapping. Successful `release()` clears ownership and storage; release of an empty
  owner is harmless. Failed release retains the mapping, layout and prior data
  access for retry. C++ destruction does not silently release a mapping.
- The caller must explicitly release every admitted mapping, including retained
  rollback failures. All payload addresses are borrowed from that owner. Before
  release, every borrower and future executing or suspended context must have
  finished using the mapping. The allocator cannot establish those lifetimes.
- Allocation uses only the explicitly requested system mapping. It never promotes
  arbitrary owners or extends a borrow. `StackMemory` supplies no admission counter,
  allocator selection, stack pool or thread synchronization. The scheduler bounds
  admission separately through its caller-owned slot array.

Mappings reserve virtual address space; success does not guarantee future physical
memory availability. Guard pages cannot guarantee detection of an access that
jumps over them. The prototype supplies no recoverable stack-overflow path.

Native coverage includes zeroed/writable payload boundaries, complete accounting,
overflow, exclusive ownership, release/reuse and cleanup-stack integration. Test
linker wrappers inject individual `mmap`, `mprotect` and `munmap` failures, including
failed protection rollback with retained ownership. A separate child constrains
`RLIMIT_AS` to prove real kernel `ENOMEM` admission refusal without an owner, then
restores the limit before sanitizer teardown. No syscall injection enters the
runtime implementation or a future production library.

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
  a nonzero code is a panic. A callback may explicitly yield its active context;
  cleanup continues only when the callback returns. The stack stays busy across
  that suspension and cannot be mutated; such operations return `invalid`.
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

The cleanup stack does not itself schedule or synchronize. A callback controls any
explicit context suspension. Mutation requires exclusive caller access. Callbacks may run resource
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
| Acknowledged cancellation | Release to an explicit boundary with a cancellation reason. No cancellation checkpoint is implemented. |

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

This is a bounded runtime experiment on the current Linux x86-64 host. The pinned
context wrapper, guarded allocation and explicit cleanup have native and sanitizer
coverage. It does not qualify a full task runtime, native stack unwinding or the
documented v0.0.1 release. The bounded scheduler supplies one worker, explicit
parent/child ownership and waiting child joins. There is no automatic scope-exit
join, cancellation unwinding, timer, channel or multi-worker executor.
There is no LLVM landing pad, Meowy personality function or pinned unwind library.
Panic codes/messages are borrowed test inputs; source spans and diagnostic
attachment are absent. Tickets provide prototype task identity, not a recorded
runtime-event identity. Cancellation is an explicit cleanup edge only.
Nothing promotes owners or borrows into arbitrary heap storage.

1. Design the compiler's generated cleanup edges and initialized-slot metadata
   alongside moves and partial initialization. Decide whether the experimental
   ordered reservation restriction should survive that design.
2. Connect typed compiler moves/results and generated scope-exit calls to the owned
   payload and explicit task-scope primitives while locals are still alive. Keep
   context release behind terminal cleanup and child completion; preserve worker,
   admission and sanitizer invariants.
3. Add LLVM landing pads, a Meowy personality and task-root outcomes using a pinned
   unwind library; preserve P008 and cleanup ordering across nested calls.
4. Exercise a suspended child borrowing a parent local, cancellation while joining,
   and cleanup that waits for children before releasing their borrowed storage.
   Independent passing cleanup/context tests do not qualify their interaction.
