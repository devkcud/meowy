# Scalar panic outcomes

Generated synchronous calls propagate P001, P002, P003 and P006 through explicit
failure exits. The diagnostic still streams to stderr at its original evaluation
point. A bounded, caller-owned runtime Panic also retains the code and message
after the failing function returns, so later cleanup can preserve the original
cause. This is private bootstrap lowering, not source-level recovery, task
unwinding or the release panic artifact format.

## Generated call boundary

A generated function has internal LLVM signature
`i1 (ptr panic, ptr result, ...typed arguments)`. True means the result was stored;
false means a panic is present and result storage was not written. Even a function
whose body emitted a result before panicking leaves its caller's result untouched.
The caller loads only on success. A Never function can propagate failure; its
impossible success continuation remains unreachable.

The panic pointer names a live, aligned runtime Panic owned by the caller, not
storage in the callee. Every caller forwards that pointer and propagates false to
its own `panic_exit`, skipping unfinished expressions, remaining arguments,
stores and caller effects. Recursion follows the same boundary. No exception,
thread-local error slot or hidden diagnostic allocation implements propagation.

`meowy_entry(ptr panic)` is the internal source entry. The native main creates an
empty snapshot and maps its outcome to process exit 0 or 1. Main does not reprint
the diagnostic. Success leaves the initially empty snapshot unchanged. There is
no source catch/resume path; automatic owning-HIR cleanup at `panic_exit` remains
the next integration in the [owning-HIR design](OWNING_HIR.md).

## Capture and output

The private `meowy_panic_*_v0` functions share formatting with ordinary scalar
output. Text and formatted primitive parts append to a snapshot while streaming
their full bytes. Arithmetic/index/capacity capture functions initialize the
appropriate code, retain original operands and source spans, finish the existing
diagnostic line, and return. Legacy terminating helpers remain available for the
older native boundary but are not called by new source lowering.

Each explicit panic expression has separate pending storage. Its P006 prefix
streams first; each message expression runs once before its returned value is
formatted. Only a fully evaluated message receives the outer site and newline,
then copies its snapshot to the caller's outcome. A nested failure supplies its
own snapshot; an outer Leave/Restart discards the pending message. Neither case
publishes a partial outer panic or erases already-streamed output.

The snapshot contains the message and byte-site suffix, excluding the printed
`panic[Pnnn]: ` prefix and final newline. It uses the runtime Panic's 256-byte
prefix, UTF-8 boundary handling and original-length/truncation metadata. Output
continues beyond that capacity. Each source chunk is copied while live; snapshot
copies remain independent after the source snapshot or buffer is overwritten.

The private capture ABI requires valid caller-provided storage. Query its size
with `meowy_cleanup_panic_bytes_v0`; use the bridge's supported alignment (16 for
this pinned target). `meowy_panic_begin_v0` explicitly initializes or resets an
idle snapshot. `meowy_panic_copy_v0` copies between initialized snapshots. Callers
must not reset storage still needed by cleanup. Output failure remains a fatal
process exit, as in the existing bootstrap writer; recoverable I/O is a separate
library contract.

## Cleanup evidence and limits

Generated native probes call actual source-lowered code, copy its returned panic,
reset the original snapshot, then unwind armed callbacks through the
[cleanup bridge](../../runtime/GENERATED_CLEANUP.md). Debug/release cases check reverse
drop order and exact P008 diagnostics with the original P001/P002/P003/P006 cause.
Additional probes check result publication only on success, complete-cause cleanup
after an abandoned message, primitive/union/primary formatting and UTF-8 truncation
with full stderr preservation. A recursive source regression verifies that failure
skips later arguments and caller effects.

These callbacks and cleanup frames are native fixtures. Ordinary meowy programs
still have no destructible resource type or automatically armed cleanup frame.
Task cancellation, child settlement, DWARF landing pads, rich source identities,
release replay and minimum-host qualification remain unimplemented.
