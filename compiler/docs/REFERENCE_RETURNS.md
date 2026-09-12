# Guarded scalar reference returns

This implements bare scalar-reference results for the direct flat signatures in
[exclusive function arguments](EXCLUSIVE_FUNCTIONS.md), following the existing
[memory contract](../../docs/reference/memory.md). A result may be shared or exclusive
and point to a boolean, integer or float. Arguments remain primitive values or
shared/exclusive references to those scalars. Wider signatures keep their existing
shared behavior and opaque result ancestry; exclusive ancestry cannot cross them.

## Explicit evidence

`borrow_contract/returns.rs` records a bounded list of argument indexes and choice
guards. Each compatible argument is a possible result source. Shared results may
select shared or exclusive inputs; exclusive results may select only exclusive
inputs with the same pointee type. Callee checking independently proves every
returned origin belongs to an input and every access obeys its reference mode.
Local returned storage still fails with E303.

Choices partition the possible returned argument. The same choice guard selects
both the actual origins copied from that argument and its captured loan authority.
Source guards inside an argument remain intact. Call checking does not inspect the
callee body to specialize a caller's choice, so an ignored compatible argument can
still be a conservative candidate. No candidate means no proved normal reference
return; existing non-returning function behavior is preserved.

`Facts.returns` stores this evidence beside the call's origin state.
`loans/returns.rs` consumes it only after the normal-return edge, creating a parent
value whose guarded copy links name the captured argument values. A fresh result
loan has that parent and the result's mode. The solver follows existing guarded
parent identities, including earlier reborrows and nested returned references.
It never recovers permission by matching physical addresses. Equal-address arguments
retain distinct loan identities and distinct choice guards.

Missing evidence, invalid argument indexes or incomplete normal-return coverage
report B001. Candidate lists, facts, copy links, values and parent walks use the
existing origin, value, node, liveness and work limits. Exhaustion cannot publish a
usable partial proof. Shared call results outside this contract remain opaque.

## Lifetimes and permissions

All borrow-carrying arguments still contribute conservative lifetime bounds, including
ignored inputs and inputs whose pointee types cannot supply the result. Those bounds
never become result addresses or parent authority. They retain scope and conflicting
write/acquisition protection. An exclusive result does not turn a lifetime-only
bound into an exclusive read restriction on an unrelated input.

An exclusive result is non-Copy. Moving an exclusive argument invalidates that holder;
passing `&!*p` preserves `p` while the returned descendant suspends incompatible
parent access through its last use. Shared returned children allow compatible parent
reads but suspend parent writes. Copies and further calls preserve these constraints.

Call entry still validates maximum parameter access against all captured arguments
and live descendants. Returning a pointer does not weaken that entry contract.
Arguments execute once, left to right. Later Leave/panic preserves completed moves
and effects while skipping call entry and result creation. The result's loan is
created only on the normal-return edge.

## Function-root emissions

Bare scalar-reference emissions to a compatible function's root result are now an
allowed authority boundary. Retained emissions remain demanded until function exit;
writing through a suspended parent afterward reports E302. Emitting a moved holder
and reading it later reports E301. Callee-local owners cannot escape the root.

Named Leave and guarded emissions may select returning inputs. Nested and recursive
calls use the same checked signature contract. Anonymous scalar-reference nested blocks now preserve the same identities under the
[block-result contract](REFERENCE_BLOCKS.md). Named results and carriers,
reference cells, dispatch blocks and exclusive restart bodies remain B001. No
cleanup ABI, allocation, runtime destruction or LLVM alias promise was added.

## Evidence and next work

Sixteen native groups execute accepted cases in debug/release and check exact build
rejections: identity/shared returns, recursive forwarding, guarded selection,
scalar widths, all-input bounds, parent/sibling suspension, moves, retained emissions,
call entry, captured targets, argument replacement, scoped exits, panic and existing
shared-return restart behavior. Sixteen
forwarded return loans execute; a 512-stage chain reports B001 within the proof budget.
Three graph groups validate shared guards, distinct equal-address parents and
missing/incomplete evidence. The existing shared, scalar-local and argument matrices
remain enabled, and reference fixtures are unchanged.

The [reference returns example](../examples/reference-returns.mwy) demonstrates guarded
exclusive selection, parent resumption and shared returned children. The compiler
gate passes; full language conformance still has 13 unsupported cases. The next
bounded extension is projected/emitted scalar list-element borrowing with canonical
owner places, exact backing and captured-list reservation proofs. Wider carriers and generated cleanup need
separate initialization, transfer and destruction contracts.
