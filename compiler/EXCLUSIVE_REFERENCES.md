# Exclusive-reference implementation design

This is the next implementation slice after terminal restart-source expiry
(`7906333`, native coverage `324e9ae`). It specifies an implementation of the existing
[memory rules](../docs/reference/memory.md),
[reference conversions](../docs/reference/types.md#inference-and-assignment), and
[ownership diagnostics](../docs/reference/diagnostic-codes.md#ownership-borrows-and-storage).
The design is not implemented: `&!value`, `<&!T>` and indirect assignment remain
bootstrap capabilities gated by B001. Planned rejections below are not current
conformance results.

## First supported slice

Enable exclusive references to initialized, mutable ordinary locals containing
booleans, integers or floats. Support local reference bindings, moves between
bindings, replacement of mutable reference bindings, scalar dereference reads and
writes, and shared/exclusive reborrows of those scalars. Include conditional paths,
short-circuit evaluation, nested blocks and named `leave` before opening the gate.
An immutable reference binding can mutate its referent through `&!T`; changing the
reference value itself still requires a mutable binding.

Keep exclusive-bearing records, unions, lists, reference cells, emitted aliases,
temporary owners, dispatch receivers, function arguments/results and captures B001
until their transfer and lifetime proofs exist. Reject inferred forms as well as
explicit annotations. Initial support excludes exclusive pointer equality and
effectful exclusive-valued block results rather than passing them through the
existing Copy paths. Ordinary scalar results computed through a reference remain
supported. Selected record fields are a follow-up using the existing mutability
and canonical-place rules; indexed exclusive access comes later.

These exclusions also apply to authority derived from an exclusive loan when the
type contains only shared references. For example, `id(&*p)` returning a shared
reference, `{->view:&*p}` and `s:&*p;cell:&s` need parent-authority propagation that
the current shared call/carrier/reference-cell paths do not supply. Keep those
boundary crossings B001 initially. Check guarded authority facts as well as types;
a `has_exclusive(Type)` test alone is insufficient. Flat shared-local copies of a
scalar reborrow are included and must preserve their child loan's identity.

For the first implementation, a body combining exclusive values with a resolved
restart remains B001. This deliberately includes iteration-local exclusive values:
a static borrow-site ID alone does not distinguish a fresh acquisition from an
earlier execution. Other bodies retain all existing shared-reference restart
support. Relax this gate only after forward availability and backward authority
demand both solve bounded loop fixed points.

## What exists and what is missing

| Owner | Existing behavior | Required change |
| --- | --- | --- |
| `parser/expressions.rs`, `parser/types.rs`, `parser/statements.rs` | Parse `&!`, `<&!T>` and dereference assignment | Keep the existing grammar |
| `check/expressions.rs`, `check/names.rs` | Reject exclusive expressions/types | Open gates only after all downstream proofs exist |
| `hir.rs` | Shared `Type::Reference`; reference reads are Copy | Explicit reference mode, Copy classification, consuming access and indirect-store representation |
| `check/references.rs`, `check/mutation.rs`, `check/statements.rs` | Resolve addresses, direct writes and mutability | Separate reference inspection from consumption; validate pointee access independently of pointer-cell mutability |
| `borrow_value.rs`, `borrow_value/pointee.rs` | Component origins, public bounds, activity and expiry | Carry authority separately without changing the meaning of Source or bounds |
| `borrow/value.rs`, `borrow/origins.rs`, `borrow/control.rs`, `borrow/mutable.rs`, `borrow/branches.rs`, `borrow/exits.rs` | Shared-only origin traversal, version restoration and Facts production | Propagate guarded authority without treating exclusive reads as copies; reject excluded authority crossings |
| `loans/state.rs`, `loans/values.rs`, `loans/control.rs` | Reference uses/defs/transfers and physical writes | Record reads, acquisitions, consumption and authorized indirect access in the existing CFG |
| `loans/solve.rs`, `loans/branches.rs` | Backward demand and guarded reach/joins | Forward initialized/moved state plus shared/exclusive conflicts |
| `loans/transitive.rs` | Summary transfers for shared reborrows | Explicit parent authority and bounded descendant demand |
| `backend.rs`, `backend/storage.rs`, `backend/aggregate.rs` | IR types, operation dispatch, cells, pointer loads and scalar stores | Preserve reference mode checks; capture an indirect target once and store only on its returning edge |

Use focused `loans/access.rs` and `loans/init.rs` modules if these responsibilities
outgrow their owners. Extend the existing CFG instead of constructing a second
one. `check/functions.rs`, `check/blocks.rs`, `check/temporaries.rs` and collection
construction must preserve the first-slice exclusions even when a type is inferred.

## Storage, values and authority

Keep three identities distinct:

1. `Source` identifies physical storage/provenance and lifetime. Reuse canonical
   Local and Slot roots, original slot views, projections, Temporary statement
   ownership and symbolic Input paths. Source::Expired remains terminal and
   nonphysical. It can never grant permission through a newly initialized site.
2. Immutable value IDs describe snapshots and demand transfers. Existing local
   reads create fresh versions; those IDs cannot identify exclusive permission.
3. A bounded loan identity names access authority, with shared/exclusive mode and
   guarded actual sources. A value can hold guarded alternative loan IDs; a derived
   loan retains guarded alternative parents. Root acquisitions have no parent.
   Moving a reference transfers those identities; reading a pointer for dereference
   does not create a new authority. Never merge loans merely because origins match.

For example, a branch assigning `p` from either `&!a` or `&!b` must retain that
guarded choice when `&*p` creates a child. One arbitrary parent or an unguarded union
cannot prove which owner is suspended on each path. Charge every alternative and
guarded edge in the existing storage/work ledgers.

Actual origins authorize access only through the corresponding loan. Public bounds
restrict lifetime and retain their conservative dependencies; they never authorize
writing an unrelated argument or make two references interchangeable. Keep source
and bound roles distinct until the conflict query that needs each role. Existing
shared bundles may still combine them for conservative liveness.

Exclusive function inputs are initially gated. When added, symbolic Input paths
need a real overlap/authority model; the current physical-write comparison that
returns false for Input is not sufficient. Do not attach LLVM `noalias` or other
alias promises merely because a reference has exclusive mode.

## Explicit accesses and last use

Record source-level accesses before folding can erase their evidence. Ordinary
scalar Local reads and tag inspections currently add no physical-read event, which
is safe for shared-only loans but would miss reads conflicting with exclusivity.
Each access records its kind, span, guarded canonical region and authorizing loan,
if any. Distinguish accessing a reference cell from accessing its referent.

| Active loan | External read/shared acquisition | External write/exclusive acquisition | Access through that loan |
| --- | --- | --- | --- |
| Shared | Allowed | E302 | Read only |
| Exclusive | E302 | E302 | Read/write, subject to live descendants |

An acquisition itself is an access even if its result is unused. Its own newly
created authority does not conflict with that acquisition. A live child may use
the authority delegated along its parent chain; an unrelated loan to the same
address may not. Shared sibling reborrows can coexist. An exclusive child excludes
overlapping siblings and parent referent access; a shared child permits compatible
parent reads but suspends parent writes/exclusive acquisition until its last use.

Use backward demand to end loans after their last required access, including
derived and transferred values. Do not synthesize reads at branch/header merges.
Preserve guarded source activity and canonical overlap: whole-owner access overlaps
its fields; proven siblings may be disjoint; all views within the first indexed
collection retain the existing conservative overlap boundary.

## Moves and initialization

Add forward guarded availability to the existing graph. Track initialized, moved
and otherwise unavailable alternatives independently of backward loan liveness.
An unused variable is not thereby moved, and a live reference does not prove its
holder initialized. Every read, borrow or move requires availability on its actual
entered guard.

- A normally returning initializer creates the binding's available value. A
  consuming use of an exclusive local transfers its authority and marks that
  source moved at the evaluation point. Initialization, assignment and discarded
  value expressions consume non-Copy values. Consuming a moved value is E301.
  Grouping and same-type ascriptions preserve that consuming context; `(p)` or
  `p<&!int32>` cannot bypass the move.
- `*p`, `&*p` and `&!*p` inspect `p` without moving it. A read of `*p` copies the
  scalar referent. Exclusive-to-shared conversion must create a shared reborrow;
  it is not a type relabel or pointer copy granting another exclusive owner.
- Reassigning a mutable moved binding is legal without reading its previous value.
  Evaluate the RHS first; record any moves/effects that finish there. Only a
  returning RHS installs the new destination. Leave or panic skips that store
  without undoing completed earlier moves.
- Join only actual continuing predecessor states, with their guards. A definitely
  moved use reports E301; an access with feasible initialized and unavailable
  paths reports E309. Preserve branch-local facts through short circuits and
  exact-target Leave queues. Do not restore pre-branch availability over an exit.
- Destroying or leaving a reference holder removes its value without destroying
  the scalar referent. No nontrivial destructor is introduced by this slice.
  Ending actual borrowed storage still invalidates all surviving views with E303.

Proposed authority-transfer policy: moving a parent handle while a child borrows
the referent transfers the same suspended parent authority to the destination.
The child retains its original owner lifetime and parent relationship; no lifetime
bound on the old handle cell is invented. The moved source is unavailable, and
the destination regains conflicting access only after child demand ends. This is
an implementation choice consistent with the existing owner-lifetime rules, not
an already-tested language feature. If transfer cannot be proved, retain B001 for
that combination. An actual `&p` reference-cell borrow is a distinct later case.

## Indirect stores and scoped exits

Represent an indirect store separately from assigning the pointer binding. Evaluate
and capture the target pointer exactly once before the RHS. On a returning path,
keep the authorizing loan demanded through the final scalar store. A read through
that same loan on the RHS is legal; an unrelated owner write is not.

If the RHS leaves or panics, there is no final store and no artificial future
pointer use. Completed RHS moves, writes and outputs still occur. Use the same
returning-edge discipline as existing indexed-write reservations. Never reevaluate
the target or restore a stale aggregate after the RHS.

Scope/statement exits must be explicit ownership events before later owned-value
support. For this scalar-reference slice they invalidate storage and retire holder
state without generating runtime destruction. Later owned payloads require flags
for initialized parts, reverse-order cleanup on normal/Leave/restart/panic paths,
and task joins while parent storage lives. Connect those events to runtime
mark/close only after payload layouts and failure-batch handling are defined;
this design does not qualify generated cleanup or unwinding.

## Acceptance matrix for implementation

These are future acceptance conditions. All rows containing an exclusive reference
currently stop with B001 before ownership semantics are checked.

| Expected after implementation | Source |
| --- | --- |
| Accept immutable handle, mutable referent | `x:=1;p:&!x;*p=2;v:*p;x=3` |
| Accept move to a new holder | `x:=1;p:&!x;q:p;v:*q` |
| E301 on moved holder | `x:=1;p:&!x;q:p;v:*p` |
| E301 after discarded consuming use | `x:=1;p:&!x;p;v:*p` |
| Accept reinitialization | `a:=1;b:=2;p:=&!a;q:p;v:*q;p=&!b;*p=3` |
| E302 on direct owner read | `x:=1;p:&!x;v:x;w:*p` |
| E302 on direct owner write | `x:=1;p:&!x;x=2;v:*p` |
| E302 on competing exclusive acquisition | `x:=1;p:&!x;q:&!x;v:*p;w:*q` |
| E302 even for unused acquisition | `x:=1;s:&x;p:&!x;v:*s` |
| Accept shared child then parent write | `x:=1;p:&!x;s:&*p;v:*s;*p=2` |
| Accept compatible parent read | `x:=1;p:&!x;s:&*p;v:*p;w:*s` |
| E302 while shared child is needed | `x:=1;p:&!x;s:&*p;*p=2;v:*s` |
| E302 while a shared child's copy is needed | `x:=1;p:&!x;s:&*p;t:s;*p=2;v:*t` |
| Accept exclusive child then parent write | `x:=1;p:&!x;q:&!*p;*q=2;v:*q;*p=3` |
| E302 while exclusive child is needed | `x:=1;p:&!x;q:&!*p;v:*p;w:*q` |
| Accept authorized RHS read | `x:=1;p:&!x;*p=*p+1` |
| E302 through the returning store | `x:=1;p:&!x;*p={x=2;->3}` |
| Accept skipped final store | `'out{x:=1;p:&!x;*p={x=2;'out.leave()}}` |
| E305 on immutable owner | `x:1;p:&!x` |
| E303 after owner scope | `a:=1;p:=&!a;{b:=2;p=&!b};v:*p` |
| Accept parent authority transfer when proved | `x:=1;p:&!x;s:&*p;q:p;v:*s;*q=2` |
| Accept captured target despite handle replacement | `a:=1;b:=2;p:=&!a;*p={p=&!b;->3};v:a;w:*p` |
| E301 through grouping | `x:=1;p:&!x;q:(p);v:*p` |
| E301 through ascription | `x:=1;p:&!x;q:p<&!int32>;v:*p` |

Use scalar function parameters for unknown branch conditions. The first program
must become E309; reinitializing `p` after the move inside that branch must accept:

```meowy
f<null>:(flag<boolean>) {
    a:=1;b:=2;p:=&!a
    |flag|{q:p;v:*q}
    v:*p
}
```

The following must also become E309: the early Leave carries moved state, while
normal completion initializes a new value. Moving and reinitializing before an
unconditional Leave must accept.

```meowy
f<null>:(flag<boolean>) {
    a:=1;b:=2;p:=&!a
    'out {
        |flag|{q:p;v:*q;'out.leave()}
        p=&!b
    }
    v:*p
}
```

Short-circuit criteria must include E309 after `flag&&{q:p;->*q>0}` when `p` is
then read, and acceptance after `false&&{q:p;->*q>0}` because that RHS never moves
it. Complementary guards must allow a move under `flag` and use of the original
holder only under `!flag`. A child created after conditional owner replacement
must suspend exactly the guarded parent alternatives, preserving safe writes to
the other owner.

Retain B001 for exclusive loops, signatures/results, inferred carriers and
reference cells. Once those forms are implemented, moving a non-Copy referent
through a borrow must use E304; do not claim that diagnostic is proved while the
containing shape is still unsupported. Shared indirect writes should become E305
when the new access checker can validate their otherwise supported scalar path.

## Validation and delivery order

Design checkpoint, 2026-09-07: ran 43 standalone `check --json` probes using the
existing pinned compiler at `target/x86_64-unknown-linux-gnu/debug/meowy`.
Thirty-eight exclusive cases and one shared indirect-write case returned B001;
shared controls returned acceptance, E302, E303 and E305 as expected. This verifies
current gates and source parsing only, not the planned exclusive behavior. The
disposable probe sources/results are in `/tmp/meowy-exclusive-probes/` for this
session. No production source or reference conformance fixture changed.

1. Add explicit access events to the current CFG, starting with physical scalar
   reads and existing writes. Preserve shared diagnostics, execution order and
   last-use acceptance. Charge retained events and source paths before allocation.
2. Add distinct bounded authority IDs and forward availability, preserving guards,
   value versions and source/bound roles. Prove local moves, replacement, child
   demand and exact Leave behavior before admitting source-level exclusivity.
3. Integrate reference mode, consuming contexts and scalar indirect stores across
   checker, origin/loan passes and backend. Reject excluded inferred forms before
   partial facts reach later phases. Keep parser grammar and runtime ABI unchanged.
4. Enable only the complete first slice; convert the matrix into native execution
   and exact-code regressions. Test each accepted program in debug/release, including
   old shared loops, first-collection reservations, canonical aliases, expired
   sources, once-only effects and skipped stores. Run the full compiler gate.
5. Extend fields, reference cells, signatures/returns, aggregate initialization and
   restarts as separate proof-bearing slices; connect owned cleanup afterward.

Reuse existing node, value, origin, liveness and shared-work limits. Authority
registries, parent edges, availability alternatives, access events and clone/merge
work all count toward those logical budgets. Charge parent walks and require
acyclic derivation; exhaustion remains B001 and cannot publish partial facts.
Do not add a dynamic epoch counter or silently erase availability on reset.
