# Exclusive restart authority plan

This is an implementation plan, not supported language behavior. Exclusive references
in bodies containing Restart remain B001. The existing [scalar-slot contract](EXCLUSIVE_SLOTS.md),
[ownership implementation](OWNERSHIP.md) and [memory reference](../docs/reference/memory.md)
remain authoritative. Shared carried-scalar borrowing is already supported.

## Why acquisition alone is insufficient

- `borrow/carried.rs::validate` rejects exclusive borrowing of a carried slot.
  Removing that restriction still reaches the body-wide rejection in
  `borrow/mutable.rs::check`: any exclusive expression/input/result combined with
  a Restart target is unsupported.
- `loans/authority.rs::solve_authority` sets every value's authority opaque when
  the graph has a reachable reset edge. `loans/permissions.rs::permission` refuses
  exclusive access with opaque authority. This is an intentional safety boundary,
  not a redundant initialization check.
- `LoanId` identifies a static acquisition site, not one dynamic loop iteration.
  `Node.copies` propagates authority, while restart headers currently use
  demand-only `Node.transfers` and explicit opaque values. A frontier audit based
  only on the current authority map could miss ancestry passing through a header.
- `Source::Slot` and the canonical storage cell identify the result owner.
  `loans/emission_init.rs` proves active, initialized storage at acquisition;
  it does not prove which exclusive loan authorizes a later access.

## First bounded outcome

Permit exclusive borrows of declared mutable carried Boolean, integer and float
slots only when the exclusive loan and all its descendants end before every
reachable restart edge. The result cell may survive an inner restart; the loan
must not. Exclusive handles in restart headers remain unsupported. This does not
enable wider exclusive pointees, reference-bearing carried initialization or
general exclusive borrowing in restarted bodies.

An intended accepted case initializes one result cell, mutates through a local
exclusive handle, ends that handle's demand, then restarts the inner scope:

```meowy
d:@"debug"
<R>:<{n<int32>:=}>
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->n:=7
            p:&!n
            *p=8
            first=false
            'loop.restart()
        }
    }
}
d.print(r.n)
```

Expected output after implementation is `8` followed by a newline. Today this is
B001. Setting `first=false` after the indirect store is significant: indirect
stores and exclusive calls conservatively forget Boolean knowledge. Do not remove
that invalidation merely to admit the example.

## Implementation order

1. Establish charged, conservative exclusive ancestry at reset frontiers using
   the existing loan graph, liveness and parent lineage. Account explicitly for
   demand-only header transfers, shared descendants, returned views and bounds.
   Missing or opaque ancestry cannot certify a frontier as free of exclusive loans.
2. Reject any live exclusive ancestry on a reset edge. Audit actual predecessor
   demand with reset semantics, not just the syntactic holder's scope or type.
   A shared child can keep an exclusive parent live after the holder's last use.
   Preserve demand-only transfers; the proof must not introduce runtime reads.
3. Narrow blanket restart opacity only after that proof succeeds. Keep explicit
   header/call opacity and the existing permission coverage checks. A static
   acquisition site must never authorize another iteration's retained pointer.
4. Replace the body-wide and carried-slot gates only for the certified slice.
   Keep exact backing, acquisition initialization, owner lifecycle, move checking,
   parent suspension, public bounds and Boolean invalidation independent.
5. Add source and native regressions before claiming support, run the compiler
   gate, then update contracts and the root/compiler handoff with actual evidence.

## Required regressions

- Accept local Bool/Int/Float mutation, declared widths, disjoint sibling access
  and direct owner writes after the exclusive loan's final use.
- Accept moves and shared/exclusive reborrows that finish within the iteration;
  retain E301 moved-use and E302 parent/child or sibling-loan conflicts.
- Reject an exclusive holder crossing an inner backedge with B001, even when its
  result owner survives. Do not claim this unsupported case as a conformance pass.
- Reject a shared descendant carried through a mutable header, carrier or call
  result when it retains exclusive ancestry. Include an unused parent holder and
  a descendant used only on a later iteration.
- Exercise initial entry, multiple backedges, nested targets, owner reset, Leave
  and skipped RHS stores. Reinitializing the same static site must not revive a view.
- Keep missing/duplicate carried initialization and conservative Boolean-state
  failures. Contrast restoring the flag after an indirect write with losing the
  flag's proof through an indirect write or exclusive call.
- Direct graph cases must reject missing frontier coverage, opaque lineage,
  malformed transfers and exhausted work without publishing a certificate.
- Run accepted output and primary rejection checks in debug/release. Preserve
  shared restart regressions, reference fixtures and the explicit conformance gap.

## Evidence boundary

The investigation inspected the body gate, authority propagation, permissions,
restart transfers and existing authority regressions. No gate was lifted and no
compiler execution was rerun for this documentation-only step. The last compiler
gate remains 1036 Rust tests, 20 Python tests and 66 debug/release examples, with
10 conformance cases passed and 13 unsupported. Runtime/editor evidence is unchanged;
the full v0.0.1 release remains incomplete.
