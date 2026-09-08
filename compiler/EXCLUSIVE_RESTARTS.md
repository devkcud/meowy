# Local exclusive borrows across result-storage restarts

Declared mutable carried Boolean, integer and float slots support exclusive
borrowing when the exclusive loan and all descendants end before every reachable
restart edge. The result cell may survive; the loan must not. This extends the
[scalar-slot contract](EXCLUSIVE_SLOTS.md) without changing the
[memory reference](../docs/reference/memory.md) or runtime representation.

## Supported slice

All exclusive acquisitions in a reset graph must address carried scalar slots.
Canonical result owner, root, lexical view and scalar shape are checked. Existing
backing-type, mutability, acquisition initialization and storage-lifetime checks
remain independent. Ordinary local/parameter exclusive roots, wider pointees and
exclusive handles in restart headers remain unsupported in this slice.

The [exclusive-carried example](examples/exclusive-carried.mwy) initializes a
result once, mutates its original cell and ends the handle before Restart:

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

Both profiles print `8` followed by a newline. Setting `first=false` after the
indirect store is significant: indirect stores and exclusive calls conservatively
forget Boolean knowledge. The initialization proof does not infer that a scalar
store cannot affect another flag. Restoring the flag afterward permits proof;
losing its value before the backedge can still produce B001.

## Frontier proof

- `borrow/mutable.rs::check` admits candidate exclusive restart bodies only when
  carried obligations exist. This does not authorize a source program by itself.
  `loans/exclusive_restarts.rs` must qualify every exclusive acquisition and prove
  all reset frontiers before the body can pass loan checking.
- `LoanId` still names a static acquisition site. The certificate follows separate
  rooted, exclusive and opaque marks through existing `Node.copies`, demand-only
  `Node.transfers` and reborrow-parent edges. Header transfer metadata is used for
  conservative ancestry only; it is not converted into a runtime read or precise
  permission grant.
- Root acquisitions establish rootedness. Unrooted values become opaque, and that
  uncertainty propagates to their descendants. Explicit opaque values and input
  or expired origins remain conservative. Multiple incoming paths merge marks by
  union; an exclusive or opaque path cannot be erased by a known shared path.
- Ancestry ignores predicate correlations except unreachable CFG nodes and literal
  false transfer guards. This prevents predicates from different iterations from
  cancelling an ancestor. Existing guarded liveness determines frontier demand;
  any non-false demand for exclusive or opaque ancestry rejects the reset edge.
- Only a successful certificate bypasses `solve_authority`'s blanket reset opacity.
  Explicit header/call opacity and all precise authority coverage and permission
  checks remain unchanged. Shared-only reset bodies retain their previous behavior.
  An iteration's static loan site cannot authorize a retained earlier-iteration view.
- Graph construction, header coverage, acquisition initialization, source expiry,
  move/availability checking, parent suspension and last-use conflicts remain
  separate requirements. Certificates do not replace any of those proofs.
- Scratch marks, dependency edges, propagation and liveness consume existing graph
  work/state budgets. Invalid IDs, unrooted demand or exhausted work fail closed
  with B001. No flags, allocations or analysis steps are added to generated programs.

## Lifetimes and limitations

Moves and shared/exclusive children can remain local to an iteration. E301 still
rejects definite moved-handle reads; E302 still rejects owner/child conflicts.
A shared child, carrier or returned view can retain an exclusive ancestor even
when the original holder is no longer read. Its demand across a backedge remains
B001, not a successful language-conformance rejection.

Owner reset can create a fresh local loan after fresh initialization. Leave keeps
completed effects and skips unfinished captured stores. Reinitializing a static
storage site does not revive expired references. Public call bounds and existing
call/result restrictions continue to apply.

Live opaque shared headers conservatively reject mixed bodies containing exclusive
loans, even when those headers may be unrelated. The next extension must distinguish
that header marker from genuinely unknown call/input ancestry and prove complete
predecessor coverage. A rooted path alone is not proof of every incoming path.
Do not enable exclusive header carriage or weaken explicit opacity as a shortcut.
Nullable/reference-bearing carried initialization and wider value analysis remain
separate work.

## Evidence

Seven source groups and three graph groups cover local mutation, widths, moves,
children, conflicts, nested reset demand, shared/call descendants, owner resets,
Leave, Boolean invalidation, unrelated-root gates, opaque/unrooted demand and work
or malformed-transfer failures. Five native groups exercise accepted output and
primary rejections in debug/release. The example prints 8 in both profiles.

All ten compiler checks pass: 531 library and 520 native tests, 20 Python tests and
67 debug/release examples, formatting, Clippy, build and repository contracts.
Conformance remains 10 passed, 13 unsupported, 0 failed. Runtime/backend, reference
fixtures and dependencies are unchanged; runtime/editor checks were not rerun.
The full v0.0.1 release remains incomplete.
