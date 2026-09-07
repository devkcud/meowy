# Private generated cleanup bridge

[generated.hpp](include/meowy/generated.hpp) exposes scalar C-linkage calls over the
existing cleanup Stack and owning Panic snapshot. [generated.cpp](src/generated.cpp)
is linked into the compiler's native archive. The interface is private to the pinned
Linux x86-64 bootstrap; it is not a release ABI or a new Meowy language feature.

Normal Meowy code generation still has no owning resource type or cleanup calls.
Compiler backend tests generate LLVM callbacks and explicit cleanup edges, emit native
objects and link the actual archive. They prove this boundary can be called correctly;
they do not prove automatic scope lowering, task cancellation or DWARF unwinding.

## Storage and scalar ABI

All functions have the `meowy_cleanup_` prefix and `_v0` suffix. Status is an i32:
0 means ok, 1 means full, 2 means invalid. Unwind reasons are i32: complete=0,
leave=1, restart=2, panic=3, cancel=4. A reason names an already-selected exit;
passing cancel does not request task cancellation or join children.

| Storage | Layout or sizing | Lifetime |
| --- | --- | --- |
| Frame | `bytes(capacity)` and `alignment()` queries; opaque bytes | From successful open through successful finish |
| Token | `{ptr owner, i64 index, i64 id}`, size 24, alignment 8 | Only within its original live frame; stale or foreign tokens reject |
| Mark | `{ptr owner, i64 depth, i64 anchor}`, size 24, alignment 8 | Existing Stack prefix rules; reuse follows that Stack's mark contract |
| Panic | `panic_bytes()`; opaque owning bytes, aligned to `alignment()` | Explicitly initialized before use; source text may then end |
| Payload | Caller-defined generated layout passed as `ptr` | Stable until its armed callback finishes or ownership is explicitly transferred |

Static assertions check the token/mark layout and the opaque alignment assumptions.
Alignment 16 is sufficient for the current target, as checked in the bridge. Size
queries return zero on overflow. Open rejects null/misaligned/undersized storage.
The frame contains its Stack, entries and callback records in one caller-owned region;
no heap allocation, hidden growth, process registry or host-destructor cleanup occurs.

Callers supply valid, writable, nonoverlapping storage for live frames, output handles,
payloads and diagnostics. Passing arbitrary dangling pointers or pretending an unopened
buffer is a live frame violates the ABI. Frame APIs cannot validate pointer provenance.
Do not open an already-live frame or mix tokens across frame lifetimes, even if the
same address is reused. All access is confined to the current worker and is synchronous.

## Initialization and unwind

Open starts an empty frame with the requested entry capacity. Mark records its current
Stack prefix. Reserve consumes one entry but does not initialize a payload; capacity
failure leaves the output token untouched. Construct the payload, then arm its token
with a non-suspending drop callback and a nonempty static operation name. Failed arm
preserves the previous entry and callback record. Partially constructed payloads must
clean their own initialized pieces before leaving the reserved entry unarmed.

A callback has the LLVM signature `void(ptr payload, ptr panic)`. The second pointer
is a constructed, initially empty owning Panic. To report a cleanup failure, call
`panic_init` while the message bytes are still valid and check its status. It captures
at most 256 bytes using the existing UTF-8 truncation rule and retains original size.
The callback can then destroy its message source. Omit capture for successful cleanup.
Callbacks cannot suspend, unwind through C++, mutate their active frame or end its
lifetime. Bridge reentry returns invalid, including finish during the last callback.

Disarm cancels one live cleanup obligation; it does not move or release its payload.
A compiler must complete actual relocation and arm the new owner before disarming the
old obligation. This bridge does not yet expose ValueOps or perform payload moves.

Unwind validates the mark and cause before releasing entries in reverse order.
Reserved-but-unarmed and disarmed entries are skipped. Panic exits require a nonzero
owning snapshot; other causes require no active panic. The initial snapshot is copied
before callbacks run. A callback panic uses the existing fatal P008 path with the
original cause and static operation name. No C++ exception crosses the ABI.

Finish succeeds only with an empty, idle frame. It ends metadata lifetime and never
silently unwinds live entries. The caller still owns and releases the backing bytes.

## Required compiler exit mapping

| Exit | Future generated action |
| --- | --- |
| Normal block completion | Relocate/retain emitted owners, then unwind that block's locals while result and parent storage remain valid |
| Leave | Unwind exited scopes from innermost through the target, retaining the completed target result |
| Restart | Unwind the target iteration and inner scopes, invalidate old initialized slots, then establish fresh iteration marks |
| Panic | Capture the owning diagnostic before releasing source storage; follow explicit cleanup edges or qualified landing pads to the task root |
| Cancellation | Request cooperative child cancellation, drain/join children while parent borrows remain valid, then release parent owners |

Backend `block` and Leave/Restart branches currently emit direct control flow; scalar
panic helpers print and terminate. Replacing these paths needs initialization/transfer
metadata and a failure outcome, not merely inserting calls to this bridge. Existing
loan lifecycle events prove availability and borrowing; they are not drop schedules.

Task::mark/close belongs to the scheduler contract, separate from Stack::mark/unwind.
Close can suspend, return report_full or retain a release failure. Generated code must
preserve the mark, report storage, consumed-progress state and parent locals until all
children settle and every failure report is drained. Close alone does not implement
cancellation. No scheduler, stack switching or DWARF dependency is added to this bridge.

## Evidence and next work

The runtime runner checks six bridge groups and two exact fatal subprocesses in each
selected profile. They cover LIFO/partial construction, all causes, foreign/stale/full
handles, nested marks, rejected initialization, callback reentry and snapshot lifetime.
Compiler LLVM probes exercise the real scalar ABI in debug/release, including an
owning diagnostic captured from callback-local bytes before those bytes are overwritten.

Next define generated payload move/drop descriptors and initialized-state metadata for
a bounded owning value. Then lower its normal/Leave/Restart cleanup with actual payload
relocation and exactly-once release. Panic/task support additionally needs retained
outcomes, child-close progress, cancellation and the pinned unwind mechanism.
