# Private owned-string foundation

[strings.hpp](include/meowy/strings.hpp) and [strings.cpp](src/strings.cpp) implement
the byte-copying resource needed by the documented
[strings.copy](../docs/reference/stdlib/text-and-data.md#owned-strings-and-formatting).
This is a private native prerequisite. Source construction, dynamic string-view
origins and automatic owner cleanup remain gated in the compiler.

## Ownership and allocator

`Strings::copy(owner, text, allocator, failure)` requires an empty, caller-provided
Owned buffer. It validates the allocator and reserves the static payload descriptor
before allocating. Nonempty text requests exactly its byte length: no terminator,
capacity growth or hidden allocation. It copies bytes while the source is live,
then constructs and commits the payload. The source may end after return. Empty
text commits a live zero-length owner without allocating or releasing heap bytes.

The private byte allocator has a context and non-suspending, noexcept allocation
and release callbacks. A successful allocation returns exclusive writable storage
of the requested size, aligned for bytes and disjoint from all live source/control
storage. Null means exhaustion. Release receives the original pointer and size
and must release it once. This byte-only callback ABI is not the general language
allocator representation or a public extension mechanism.

`Strings::heap()` returns an immutable static handle using malloc/free. Native
tests supply a caller-owned allocator to force failure and count allocations and
releases; no source-visible failure switch exists. Custom callbacks are trusted
native code and must synchronize shared state if used across workers. The
allocator descriptor/context must outlive every owner that retains it; copying
its handle never extends that lifetime.

Allocator callbacks cannot reenter the active owner: its data, reserve, commit,
release, move and view operations reject while construction/destruction runs.
Callers must preserve their cleanup frames, reservations and source bytes across
the constructor; callbacks must not invalidate that caller-owned storage. No
callback can suspend or unwind through C++. The constructor does not arm a cleanup
frame; reserve that obligation beforehand and arm the committed owner afterward.
On a subsequent caller-side failure, release the still-owned resource exactly once.

## Results and transfer

StringStatus is a separate i32 family: ok=0, invalid=1, full=2, misaligned=3,
occupied=4, allocation_failed=5. Storage/allocator validation failures preserve the
previous owner and failure output and perform no allocation. Allocation exhaustion
clears the uncommitted reservation and writes `AllocationFailure` with cause
exhausted=1, requested bytes and alignment 1. The owner is empty and can be retried.
Success clears failure facts to zero. This private typed evidence is not yet a
source-level nominal error value or the library's public error representation.

The constructor does no size addition/multiplication, so it has no size-overflow
branch. Wider allocator-backed constructors must define their own checked-size
failure rather than reuse a null pointer or panic as a failure value.

The static ValueOps descriptor moves the allocation handle and ends the source
payload lifetime without copying text or allocating. Drop releases the allocation
once. The existing Owned and generated transfer bridge provide failed-transfer
preservation, destination-before-source arming and enclosing panic-cause handling.
Views require a committed owner with this exact descriptor. Invalid views leave
their output unchanged. A view borrows its owner; native pointer stability after a
move is not permission to extend its future source-language lifetime.

## Scalar generated ABI

All calls have the `meowy_string_` prefix and `_v0` suffix. Use `heap()` for the
static allocator, `ops()` for the static descriptor, and `bytes()`/`alignment()` for
payload storage. Open ordinary Owned metadata using the existing cleanup bridge.

`copy(owner, text, size, allocator, failure)` accepts a live empty owner, valid
input bytes and a writable failure output. The pinned 64-bit failure layout is
`{i32 cause, i64 bytes, i64 alignment}`, size 24, alignment 8, offsets 0/8/16.
Static assertions enforce it. `view(owner, text_output, size_output)` writes a
borrowed pointer and byte length only on success. An empty view may have a null
pointer and length zero. All metadata, outputs, source and live payload regions
must satisfy the normal nonoverlap/lifetime contract; null checks do not validate
arbitrary pointer provenance. Input is already valid UTF-8 from a language string;
the private byte-copy ABI does not perform UTF-8 validation or repair.

Seven native groups exercise source destruction, embedded NUL/UTF-8, empty values,
deterministic exhaustion/retry, preflight failures, live-value preservation,
relocation, callback reentry, descriptor/view validation and armed panic cleanup.
The runtime runner includes them in debug/release/ASan/UBSan/LSan profiles.
An LLVM-native compiler probe copies stack-local text to heap storage, overwrites
the source, transfers through the real Owned bridge, reads the view and unwinds.
