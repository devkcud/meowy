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
  Header/call opacity in precise authority and all authority coverage and permission
  checks remain unchanged. Shared-only reset bodies retain their previous behavior.
  An iteration's static loan site cannot authorize a retained earlier-iteration view.
- Graph construction, header coverage, acquisition initialization, source expiry,
  move/availability checking, parent suspension and last-use conflicts remain
  separate requirements. Certificates do not replace any of those proofs.
- Scratch marks, dependency edges, propagation and liveness consume existing graph
  work/state budgets. Invalid IDs, unrooted demand or exhausted work fail closed
  with B001. No flags, allocations or analysis steps are added to generated programs.

## Certified shared headers

- `loans/restarts.rs::header_transfer` retains each definition's required active
  guards from the existing predecessor/activity proof. Metadata is generated only
  when carried obligations exist; it does not invent initial or backedge values.
- `loans/restart_headers.rs` checks every reachable reset predecessor. Definitions
  must match the metadata keys, and all predecessors of one reset target must agree
  on those keys. Each required guard must be covered by actual transfers. Missing
  certificates, hidden definitions and incomplete entry or backedge transfers fail
  with B001; one rooted predecessor cannot substitute for the others.
- An inactive nullable path can require FALSE and therefore no transfer on that
  predecessor. Its active predecessors still need coverage and rooted ancestry.
  Nested targets keep separate header identities and predecessor sets.
- The frontier proof excludes only the opacity marker attached to a certified
  header definition. Explicit opacity elsewhere, input/expired origins and exclusive
  ancestry still propagate through every dependency. Precise authority retains its
  original header opacity; certificates grant no new exclusive permission.
- The [mixed-headers example](examples/mixed-headers.mwy) prints 1, 2, 8 while a
  mutable shared reference changes independently of an iteration-local exclusive
  loan. Existing copies, physical conflicts, owner reset and Leave keep their rules.
  Shared-only restart authority is unchanged, and metadata/audit work is bounded
  by the existing graph budgets.

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

Known shared roots may now pass through certified headers in mixed bodies. Genuinely
unknown call/input ancestry, unrooted demand and exclusive descendants remain gated
at reset frontiers. Replacing a shared descendant before the edge can end its loan;
merely dropping the original exclusive holder does not end a retained descendant.
Exclusive header carriage and weaker explicit opacity are not enabled.
Nullable/reference-bearing carried initialization and wider value analysis remain
separate work.

## Evidence

Seven source groups and three graph groups cover local mutation, widths, moves,
children, conflicts, nested reset demand, shared/call descendants, owner resets,
Leave, Boolean invalidation, unrelated-root gates, opaque/unrooted demand and work
or malformed-transfer failures. Five native groups exercise accepted output and
primary rejections in debug/release. The example prints 8 in both profiles.

Six further source groups, four graph groups and five native groups cover mixed
headers, inactive nullable paths, nested targets, owner resets, Leave, missing
entry/backedge coverage, metadata/definition removal, explicit opacity and conflicts.

All ten compiler checks pass: 541 library and 525 native tests, 20 Python tests and
68 debug/release examples, formatting, Clippy, build and repository contracts.
Conformance remains 10 passed, 13 unsupported, 0 failed. Runtime/backend, reference
fixtures and dependencies are unchanged; runtime/editor checks were not rerun.
The full v0.0.1 release remains incomplete.
