# Meowy compiler bootstrap

This directory contains a working Rust compiler with a C++20 LLVM backend.
It checks standalone Meowy source and produces Linux x86-64 native executables.
It implements scalar programs, record composition, nullable unions, branch
narrowing and shared references to local and emitted storage, including guarded
block results, immutable records and unions carrying references, direct-function
borrow contracts, shared reborrows, last-use checks for mutable owners, and inline
bounded lists of copyable reference-free elements. It is not the complete v0.0.1 language.
Read [STATUS.md](STATUS.md) for gaps, validation evidence, and the next work,
and [AGENTS.md](AGENTS.md) before changing the implementation.

## Build and run

From the repository root:

```sh
cargo build --locked --manifest-path compiler/Cargo.toml
compiler/target/debug/meowy check compiler/examples/factorial.mwy
compiler/target/debug/meowy run compiler/examples/hello.mwy
compiler/target/debug/meowy run compiler/examples/factorial.mwy --profile release
compiler/target/debug/meowy run compiler/examples/nullable.mwy
compiler/target/debug/meowy run compiler/examples/references.mwy
compiler/target/debug/meowy run compiler/examples/borrow-results.mwy
compiler/target/debug/meowy run compiler/examples/borrow-liveness.mwy
compiler/target/debug/meowy run compiler/examples/borrowed-records.mwy
compiler/target/debug/meowy run compiler/examples/optional-borrows.mwy
compiler/target/debug/meowy run compiler/examples/borrow-functions.mwy
compiler/target/debug/meowy run compiler/examples/reborrows.mwy
compiler/target/debug/meowy run compiler/examples/scope-borrows.mwy
compiler/target/debug/meowy run compiler/examples/bounded-lists.mwy
compiler/target/debug/meowy run compiler/examples/list-unions.mwy
compiler/target/debug/meowy run compiler/examples/compound-lists.mwy
compiler/target/debug/meowy run compiler/examples/element-borrows.mwy
compiler/target/debug/meowy run compiler/examples/element-writes.mwy
compiler/target/debug/meowy run compiler/examples/nested-writes.mwy
compiler/target/debug/meowy run compiler/examples/effectful-lists.mwy
compiler/target/debug/meowy run compiler/examples/dynamic-lists.mwy
compiler/target/debug/meowy run compiler/examples/mutable-fields.mwy
compiler/target/debug/meowy run compiler/examples/mixed-writes.mwy
compiler/target/debug/meowy run compiler/examples/emitted-slots.mwy
compiler/target/debug/meowy run compiler/examples/emitted-borrows.mwy
compiler/target/debug/meowy run compiler/examples/immutable-slots.mwy
compiler/target/debug/meowy run compiler/examples/reference-slots.mwy
compiler/target/debug/meowy run compiler/examples/transitive-borrows.mwy
compiler/target/debug/meowy run compiler/examples/temporary-borrows.mwy
compiler/target/debug/meowy run compiler/examples/reference-temporaries.mwy
compiler/target/debug/meowy run compiler/examples/mutable-references.mwy
compiler/target/debug/meowy run compiler/examples/guarded-references.mwy
compiler/target/debug/meowy run compiler/examples/leave-references.mwy
compiler/target/debug/meowy run compiler/examples/restart-references.mwy
compiler/target/debug/meowy run compiler/examples/transitive-restarts.mwy
compiler/target/debug/meowy run compiler/examples/header-activity.mwy
compiler/target/debug/meowy run compiler/examples/expired-restarts.mwy
compiler/target/debug/meowy build compiler/examples/loop.mwy --output compiler/build/sum
compiler/build/sum
```

The examples print a greeting, `3628800`, and `5050`. The
[records example](examples/records.mwy) demonstrates primary values, named fields,
interpolation, and dispatch. The [nullable example](examples/nullable.mwy) exercises
absent fields and a fallback function that narrows a value after an early exit.
The [references example](examples/references.mwy) compares storage addresses and
copies values through shared references. The [borrowed results example](examples/borrow-results.mwy)
selects between surviving owners and discards an iteration-local borrow on restart.
The [borrow liveness example](examples/borrow-liveness.mwy) updates scalar and record
owners after the final use of their shared references, including loop iterations.
The [borrowed records example](examples/borrowed-records.mwy) copies nested reference
fields, projects selected fields and a primary reference, and releases each loan
after that component's last use.
The [optional borrows example](examples/optional-borrows.mwy) narrows nullable
reference fields and unions of different reference types before dereferencing.
The [function borrows example](examples/borrow-functions.mwy) returns borrowed
views through direct calls and releases their input loans after the final use.
The [reborrows example](examples/reborrows.mwy) takes references to original
record fields through shared references and returns them through functions.
The [scope borrows example](examples/scope-borrows.mwy) contrasts local parameter
and receiver copies with shared receivers that keep the original owner alive.
The [bounded lists example](examples/bounded-lists.mwy) preserves an original list
while appending to its copy, checks one-based positions and compares initialized
elements within an unchanged inline capacity.
The [list unions example](examples/list-unions.mwy) chooses list alternatives by
element type, literal range and capacity while preserving concrete element widths.
The [compound lists example](examples/compound-lists.mwy) selects element widths
using checked intermediate values, grouped negation and short-circuit expressions.
The [element borrows example](examples/element-borrows.mwy) takes checked references
into original list storage, returns an element through a function and replaces the
owner after the references' final uses.
The [element writes example](examples/element-writes.mwy) replaces initialized
elements after the final use of a shared view, preserving the list's length and copies.
The [nested writes example](examples/nested-writes.mwy) updates a selected matrix
element while preserving other rows and retaining the indices chosen before the RHS.
The [effectful lists example](examples/effectful-lists.mwy) selects numeric and
record element widths after checking each block's effects once in source order.
The [dynamic lists example](examples/dynamic-lists.mwy) selects element types from
returned and mutable primitive locals while using their actual values at runtime.
The [mutable fields example](examples/mutable-fields.mwy) updates a declared mutable
field after its final shared read while preserving an earlier record copy.
The [mixed writes example](examples/mixed-writes.mwy) updates fields inside a list
held by a record, preserving copies and allowing changes to a separate holder field.
The [emitted slots example](examples/emitted-slots.mwy) mutates named fields during
construction; later reads and the returned record observe the same updated storage.
The [emitted borrows example](examples/emitted-borrows.mwy) reads those fields through
shared references and writes after their last use. An inner block can pass out a
reference when its emitted owner belongs to a still-active outer result.
The [immutable slots example](examples/immutable-slots.mwy) borrows immutable emitted
fields and list elements while preserving constant and initialized-length facts.
The [reference slots example](examples/reference-slots.mwy) distinguishes a borrowed
field inside a result from a stored reference copied out of it. Each follows its
own storage lifetime.
The [transitive borrows example](examples/transitive-borrows.mwy) borrows a whole
reference-carrying record and a reference-valued cell, then copies their contents
while preserving the original pointee lifetimes.
The [temporary borrows example](examples/temporary-borrows.mwy) borrows computed
values within one statement, preserving evaluation order and distinct owner cells.
The [reference temporaries example](examples/reference-temporaries.mwy) copies
contained references out of temporary cells while keeping their original owners.
The [mutable references example](examples/mutable-references.mwy) reassigns a shared
reference while preserving an earlier copy and reads a borrowed reference cell
for the last time before replacing its contents.
The [guarded references example](examples/guarded-references.mwy) selects a reference
in a matcher and updates only the owner that the selected view no longer borrows.
The [leave references example](examples/leave-references.mwy) keeps an earlier RHS
reassignment when leaving the target skips the unfinished outer assignment.
The [restart references example](examples/restart-references.mwy) carries a new
reference into the next iteration while a copy made before the loop keeps its target.
The [transitive restarts example](examples/transitive-restarts.mwy) carries pointers
to reference-bearing records through a loop while preserving an earlier record copy.
The [header activity example](examples/header-activity.mwy) carries null and active
reference variants through successive iterations without inventing a null-path loan.

The compiler requires Rust **1.98.1** and LLVM, Clang, LLD, and LLVM ar **22.1.8**.
The native tools are resolved at the explicit `/usr/bin/` paths in `build.rs`;
LLVM development headers/libraries and the host C/C++ development environment
must be installed. The Rust workspace has no external crate dependencies.
Cargo builds the C++ bridge and embeds the separate runtime archive in the compiler.

This is a host bootstrap: LLVM's shared library and the pinned Clang/LLD paths
must remain available. It uses the host libc development files at link time.
It is not yet a portable compiler distribution with a bundled sysroot, and has
not qualified the reference's Linux 5.4/glibc 2.31 baseline.

The [exclusive references example](examples/exclusive-references.mwy) moves a scalar
reference, reborrows it and captures an indirect store target before replacing the
holder. Shared/exclusive children preserve their parent authority through last use.

The [exclusive functions example](examples/exclusive-functions.mwy) passes moved
and reborrowed scalar handles through nested direct calls. The bounded
[argument contract](EXCLUSIVE_FUNCTIONS.md) describes entry validation and remaining
wider-result restrictions.
The [reference returns example](examples/reference-returns.mwy) carries a selected
exclusive loan through a call and resumes its parent after the returned view ends.
The [return contract](REFERENCE_RETURNS.md) keeps guarded input authority separate
from conservative lifetime bounds.
The [reference blocks example](examples/reference-blocks.mwy) moves an emitted
child, completes through Leave and cancels another result without undoing its move.
The [block-result contract](REFERENCE_BLOCKS.md) describes retained demand and scope.
The [exclusive fields example](examples/exclusive-fields.mwy) mutates disjoint scalar
fields while reading a primary value. The [field contract](EXCLUSIVE_FIELDS.md)
describes mutable paths, owner lifetimes and remaining root restrictions.
The [exclusive slots example](examples/exclusive-slots.mwy) borrows initialized
emitted scalars beyond their alias scope while their target block remains alive.
The [slot contract](EXCLUSIVE_SLOTS.md) requires exact backing types.

## Implemented language

- UTF-8 sources, original byte spans, retained lexer trivia, compact punctuation,
  comments, escaped strings, and nested interpolation.
- Lexical value and type namespaces, primitive type aliases, ordinary shadowing,
  and aliases of the resolved `core` and `debug` intrinsics.
- Null, booleans, signed/unsigned 8–64-bit integers, `isize`/`usize`, float32/64,
  and borrowed literal strings. Numeric operations preserve the operand types.
- Immutable and mutable local bindings; checked integer arithmetic, bitwise
  operations, comparisons, and short-circuit boolean operators.
  Unary operators keep their operand type before the result enters an expected
  union, preserving checked widths and boolean operations. Non-returning operands
  propagate through scalar operators while preserving evaluation order and prefixes.
- Blocks with primary and named emissions, record composition,
  scalar-primary projection, dispatch, and duplicate/uninitialized slot checks.
- Normalized scalar/record unions, nullable field and primary defaults, and
  conversions between compatible union sets without numeric widening.
- Runtime type predicates and proven ascriptions, including immutable field paths,
  complementary conditions, short-circuit operands and early-exit narrowing.
  Assignments invalidate proofs about the changed value.
- Shared references to ordinary local bindings and their concrete record
  fields, address equality, reference copies and copyable dereference. Record field
  access through a reference copies the field. Borrow origins are checked before
  lowering. Bare-reference block results preserve every possible origin across
  branches and named exits; completed results cannot retain expired locals.
  Restart, panic and enclosing leave discard emissions when proven by flow guards.
  Mutable owners can be assigned after the last use of every overlapping shared
  reference. Live aliases, reference operands and retained block results protect
  their owners from writes; conflicting assignments report E302.
- Exclusive references to mutable ordinary boolean, integer and float locals,
  including scalar fields of mutable reference-free Copy records. Every crossed
  field must be mutable. Sibling fields and primary projections are disjoint;
  whole-owner/ancestor accesses conflict with live field loans.
  Immutable handles permit scalar stores; replacing a handle requires a mutable
  binding. Moves and reinitialization preserve authority; unavailable holders report
  E301 or E309. Shared/exclusive reborrows retain guarded parent relationships and
  reject overlapping external or suspended-parent access with E302. Expected shared
  scalar-reference types create a reborrow without moving the exclusive holder.
  Indirect stores capture the pointer before RHS effects and complete only on
  returning paths. Named Leave, short circuits and conditional moves are supported.
  Direct functions accept scalar exclusive parameters with primitive or bare scalar-
  reference results and primitive or scalar-reference arguments. Passing a holder moves it; `&!*p` keeps
  the parent available after the call. Shared parameters reborrow without consuming
  the parent. Entry checks reject suspended parents and conflicting arguments even
  for unused parameters or non-returning callees. Direct receiver syntax uses the
  same contract; nested and recursive calls retain symbolic input permissions.
  Returned references retain guarded captured-input parents. Their all-input lifetime
  bounds protect storage without authorizing access. Exclusive results move; shared
  returned children permit compatible parent reads and suspend parent writes.
  Direct mutable emitted scalar aliases also support exclusive borrowing after
  initialization when their completed backing type is identical. Canonical slot
  identity and target-block lifetime survive lexical alias scope and guarded views.
  Immutable aliases report E305; widened backing remains B001.
  Wider exclusive signatures/result shapes, carriers, cells, projected record aliases,
  dispatch blocks, indexed/reference paths, collections, carrier results, comparisons and restart bodies
  remain B001. Anonymous scalar-reference blocks preserve existing guarded loan
  identities and consume exclusive emissions. Retained results protect their owners
  through completion; proven cancellation preserves effects and moves without
  keeping a future result loan alive. Unsupported call/result/cell/dispatch-block crossings also reject
  shared values carrying exclusive ancestry.
- Immutable records with shared-reference primary, named and nested components.
  Whole-record copies preserve every reference; field and scalar-primary access
  track only the selected components. All retained components must outlive their
  receiving block. Record equality compares the full shape, including addresses.
- Mutable fields in reference-free Copy records. Field mutability is part of the
  type's shape and survives construction, composition, unions and list contexts.
  Named field paths on mutable local owners support assignment when every crossed
  field is mutable. Writes preserve other fields and copies; disjoint shared views
  may stay live, and overlapping views must finish before the store.
- Mutable emitted names backed by result fields. After `->count:=1`, `count=2`
  updates the returned field; scalar reads and mixed field/index writes use that
  storage. Initializers still run once. Wider final field types, nullable fields,
  named enclosing targets and restarts preserve the alias's declared type.
- Shared borrows of mutable emitted names, their concrete fields and initialized
  list elements. References use the slot's target-block lifetime and retain their
  declared pointee type. Overlapping writes are allowed only after the final shared
  use; publication cannot carry a reference into its own construction storage.
- Immutable reference-free emitted names use the same result storage and shared
  borrow lifetimes. Their constant, variant and initialized-length facts remain
  available. The immutable name cannot be assigned or used to write a nested field
  or element, including mutable fields inside its value; those writes report E305.
- Immutable emitted references and reference-carrying records/unions also use actual
  result cells. Copying a stored reference keeps its original pointee origins and
  input bounds. Borrowing a concrete reference-free field instead follows the
  carrier's cell lifetime and does not read unrelated reference fields. Selected
  fields of ordinary locals and copied parameters/receivers follow the same rule.
- Immutable reference-bearing unions and optional fields. Injection, widening and
  proven narrowing preserve the active member's borrow origins. Absent reference
  fields carry no loan; type predicates inspect the discriminant without copying
  reference payloads. Copies and equality consume active references directly
  contained in the value.
- Shared reborrows: `&*view`, `&view.field` and nested
  parenthesized paths, including reference-valued prefixes such as
  `&holder.view.field`. Reference-valued calls/blocks evaluate once. Derived
  function results retain all active input lifetime bounds. Union payload addresses
  remain unavailable; scalar exclusive reborrows follow the rules above.
- Whole-carrier and reference-cell shared borrows, including nested dereference
  copies and reborrows through stored references. Bounded pointee summaries preserve
  contained origins, nullable activity and call bounds. A direct dereference copy
  may outlive the outer cell while its contained pointees survive. Public function
  results retain all-input bounds through later dereferences. Field and tag reads
  consume only the selected contents; pointer equality does not read pointees.
- Scope-local references to by-value parameters and dispatch `self` bindings.
  Their addresses cannot escape their storage scopes. Shared-reference and
  reference-carrier dispatch retain original origins and all-input bounds.
- Shared borrows of Copy temporaries, including reference values, computed records,
  list elements and same-statement calls/reborrows. Each owner is evaluated once
  and lasts through its complete statement. Matcher conditions share that lifetime
  with their controlled statement; nested block statements have separate owners.
  Storing a reference does not extend its lifetime, and later uses report E303.
- Reference-bearing temporary contents retain pointee origins, bounds and nullable
  activity beneath the borrowed cell. Direct dereference copies can outlive that
  cell when their pointees survive; public call bounds remain attached. Materializing
  a value reads its directly contained references, while deeper pointee summaries
  are followed only when later operations need them.
- Mutable ordinary locals with a fixed shared-reference type, such as `view:=&owner`.
  Reassignment changes subsequent reads while earlier copies retain
  their original pointees and call bounds. A live borrow of the reference cell
  blocks reassignment; its final read may occur in the assignment's RHS. Mutable
  nullable reference bindings/carriers remain B001.
- Guarded shared-reference assignments in matcher arms and `&&`/`||` right operands.
  Returning paths merge their possible values; skipped paths retain their incoming
  value, and panicking paths contribute no continuation. Earlier copies stay fixed.
  Merging does not read a reference: its lifetime and loan are checked when demanded.
  Owner writes and cell borrows retain their guards through nested branches.
- Forward named-scope leaves with shared-reference reassignment. A leave captures
  surviving reference values for its exact target, then joins them with normal
  completion there. Nested exits preserve completed effects and skip unfinished
  stores/calls. Exiting a target does not extend its local, slot or temporary storage.
- Bounded restart analysis for mutable references with supported nested-reference
  and record summaries. Header values preserve typed component paths for initial
  and backedge origins/bounds, with iteration guards reset. Old copies and physical
  cell loans remain separate; header transfers add no read. Nested targets and
  skipped RHS/call work retain their normal execution order.
- Stored union activity in restart headers, including nullable reference fields and
  nested tagged records. Stable choices preserve parent/member relationships across
  analysis passes. A predecessor may omit a reference transfer only when its actual
  path is proved inactive; active missing paths still fail with B001.
- Expired restart-carried sources and public bounds. A reference can be overwritten
  before its next read; using an expired value reports E303 even after its original
  storage site runs again. Live ancestor statement temporaries survive inner restarts.
- Shared borrows of initialized bounded-list elements, such as `&values[index]`,
  including nested list/record paths and direct-function results. The parent
  reference stays live through returning index evaluation, so conflicting owner
  writes report E302. E101/P001 check one-based initialized bounds before producing
  an element address. Copied parameter/self elements cannot escape their scope.
  Lifetime analysis conservatively treats all indices in a list as overlapping;
  runtime reference equality still uses the actual element addresses.
- Checked element assignment to mutable local lists, including nested targets
  such as `matrix[row][column] = value`. It checks each one-based initialized
  position from root to leaf before evaluating the replacement, then updates only
  that element. Returning index/RHS evaluation keeps parent storage reserved
  against writes; shared reads may finish before the final store. Paths can mix
  mutable fields and indices, such as `holder.items[i].value` or `rows[i].items[j]`.
  The first indexed collection defines the reserved and final-write region;
  holder fields outside that collection can remain disjoint.
- Inline bounded lists `T[N]` with a separate initialized length, typed/inferred
  literals, `.size()`, one-based copy indexing, value-returning `.add()`, whole-value
  replacement and equality of initialized elements. Elements can be scalars,
  reference-free records/unions or nested bounded lists. Inference preserves typed
  widths and never invents a union or projects a record primary to reconcile items.
  Whole-list references use the same lifetime and final-use checks as other owners.
- Direct functions, explicit-result recursion, strict mutual-forward groups,
  conditional matchers, and named-scope `leave`/`restart`, including scoped aliases.
- Shared-reference function inputs and results, including immutable record/union
  carriers. A returned reference retains every active borrow-carrying input under
  the conservative public contract, including ignored inputs of another type.
  Scalar results and scalar-only projections end those loans after the call.
- `debug.print`, streamed interpolation at output calls, `debug.panic`, string
  byte length, and string comparison.

Emissions continue executing the block. `check` analyzes application effects
without running them. Debug and release both preserve dynamic arithmetic checks.
The initial panic runtime reports failure and exits; recoverable unwinding and
owned-resource cleanup remain unimplemented.

Dynamic integer failures report P002 with the source operator, original operands,
integer width/signedness, numeric range and half-open source byte span. Overflow
and a zero divisor are distinguished; signed minimum remainder by `-1` remains zero.
For example, an `int8` addition can report:

```text
panic[P002]: int8 + overflow (left 127, right 1; range -128..127) at bytes 14..17
```

Explicit `debug.panic` streams its supplied message once, then appends its P006
call-site byte span. If message evaluation itself panics or leaves the scope,
the outer panic does not append a misleading site or terminator. A completed panic
exits with status 1. These are bootstrap text diagnostics, not the
release panic artifact format or a recovery/unwind implementation.

Unavailable constructs report **B001**, including slices, named list positions,
reference/owned list elements, other collection APIs, non-scalar exclusive borrows,
borrows of owned temporary storage, capturing closures, generic/type-producing
helpers, imports beyond the foundational bootstrap modules, mutable nullable
reference bindings, mutable reference-bearing fields, mutable primary slots and alias
views requiring union retagging. String interpolation
outside an output call requires the future formatting/storage implementation.
The [tracker](STATUS.md#still-outside-this-compiler) covers the full remaining scope.

List capacities accept non-negative integer constants and checked scalar
expressions. General required evaluation through blocks or calls remains
unavailable. Bootstrap limits are 65,536 slots and 1 MiB of inline layout per list;
exceeding those implementation budgets reports B001. Dynamic bounds/fullness
failures report P001/P003 with the position or capacity, initialized length and
source byte span. They exit through the initial panic runtime.
With several expected list alternatives, capacity and compatible element types
must select exactly one. Multiple viable choices report E207; all capacities being
too small reports E103. Literal range can select a width, but no preference is given
to a smaller capacity or a default numeric width. Pure contextual literals may wait
for typed elements; other expressions are checked once in source order.
Pure scalar unary/binary expressions can also constrain candidates, including
grouped negation, arithmetic, bitwise operations and Boolean comparisons. Their
intermediate values use each candidate's exact width: `(127+1)-1` cannot select
`int8` merely because its final mathematical result is 127. Immutable scalar
constants retain their declared types. Short circuits and the expression's original
reach determine whether arithmetic executes; later effects are never replayed.
Unannotated lists keep their existing common-type rules for compound expressions.
When list candidates remain unresolved, an unlabeled element block may check its
context-independent prefix once, then select a type from a terminal sequence of
unconditional emissions. Candidate probes use the prefix's resulting reach and
bindings. The same block continues with the selected type; effects are neither
replayed nor deferred across later list elements. Same-owner primitive locals keep
their declared types and local shadowing resolves before result probing. Immutable
constants retain their values; mutable and nonconstant locals contribute unknown
values, without reusing their initializers. Ordinary scalar deferral remains
constant-only, so runtime reads stay before later list effects.
Ambiguous proved shapes report E207. Result constraints that depend on earlier
emitted names, reference/aggregate locals, unresolved primary composition or more
complex control flow remain B001 until their inference is implemented. A scratch
failure inside a symbolic short-circuit branch may retain an uncertain candidate;
ordinary checking with the actual flow facts decides a sole remaining choice.
Multiple uncertain choices remain B001. An explicit
element/list annotation supplies context for the existing ordinary block checker.
Candidate selection allows up to 256 list alternatives and 4,096 nodes per pure
scalar tree, charging constant bytes and
repeated checking work to the existing shared analysis budget.

References can pass through local blocks and immutable aliases while their owners
remain alive. The checker reuses branch/completion proofs and checks all possible
borrow origins. Retained local escapes report E303; discarded emissions still
evaluate their operands and effects. References can also be stored in immutable
record and union components and direct-function signatures. Fixed shared-reference
locals support reassignment with separate versions of their contents, including
matcher arms and short-circuit right operands. Branches start from their incoming
versions and merge only returning states under their actual guards. A skipped
assignment preserves the old value; copying it before a branch preserves its old
pointees and call bounds. Conditional lifetime checks do not revive expired values
when the predicate later changes.
Forward `leave` paths capture the reference values that survive the named target,
before inner scopes are discarded. Target completion merges these exits with
fallthrough without adding reference reads. A leave from an RHS keeps earlier
completed assignments and skips the unfinished store, call or index operation.
Initialized result emissions retain their own loans and path-specific proofs.
Restart headers are solved by bounded origin analysis before loan checking. In
bodies using reference reassignment, every mutable reference already present at a
restarted target's entry must have a supported summary shape. Reference, record and
stored union paths retain their origins, bounds and active members. Canonical member
choices are stable across passes and children are conditioned on their parent member.
Traversal still stops at reference-free referents. Exact initial/restart predecessor
proofs distinguish a null path from lost reference evidence; inactive paths add no
loan, while active paths retain their actual source and public bounds.
Sources and bounds in surviving ancestor storage or function inputs stay live.
Ended sources and storage owned by the restarted target or its descendants become
terminal expired identities. Overwriting such a reference before reading it is
accepted; actual expired use is E303. Reinitializing a Local, emitted slot or
Temporary site never revives an old reference. An ancestor statement's temporary
survives an inner restart while that statement remains active. The
[expired restarts example](examples/expired-restarts.mwy) exercises fresh iteration
storage, overwrite after loop exit and a surviving outer temporary.
The analysis forgets header-entry and iteration guards, and may widen correlations
between independent union fields or owners. Programs needing finer correlations may
be rejected. Canonical source/bound sets and member activity must converge within
64 passes and the existing work/storage budgets; incomplete proof reports B001.
Mutable reference carriers, mutable nullable reference bindings and direct reference
formatting require future analysis. A panic during
the RHS skips the store; nested blocks and call arguments retain evaluation order.
Named emissions use actual slot aliases. Reads and copies of stored references keep
their pointee origins; selected reference-free field addresses borrow the carrier's
storage. Whole-carrier and reference-cell borrows retain bounded summaries of their
contents. Direct dereference copies preserve contained origins independently of the
outer cell; existing function-call bounds continue to constrain copied references.
Field addresses of copied parameters and receivers cannot escape those local copies.
Missing origin proofs or exhausted analysis budgets produce B001.
Borrow liveness follows branches and named loop edges. An assignment evaluates its
right-hand side before writing: `owner = *view + 1` is valid when that is the last
use of `view`. A later use of that view makes the write a conflict. Replacing a
record overlaps references to any of its fields.
An assignment with no indices updates only the selected field. Its static offsets
remain valid when the RHS replaces the same Copy owner; RHS changes to other fields
are preserved. The final store still conflicts with any overlapping live shared
view. Immutable roots or crossed fields report E305, and incompatible field
mutability in declared construction or completing branches reports E206. Assignment
through shared-reference targets or temporary roots remains B001.
Named emissions register an alias after initialization. Later reads
and permitted mutable assignments resolve the actual result cell, with conversion
between the declared local type and a wider final slot type. Matching field mutability
is required for that backing; mixed paths address the compatible payload.
When an emission is proved discarded, an initialized local cell preserves its
remaining effects without projecting into an absent or incompatible result field.
Borrows of slot storage use the target block as their owner even when the alias name
was declared in an inner scope. Discarded cells are retained as target-owned partial
result storage. A shared slot borrow needs backing that matches the alias type or one
concrete union member, including a complete summary for reference-bearing referents.
Proper subunion views remain B001 because borrowing cannot retag a copied value.
References may pass through inner results, but escaping their target's publication
reports E303. Live overlapping writes report E302, including uses across an inner
restart. Restarting the target ends its iteration's storage. Mutable aliases publish
unknown variant activity, preventing stale initializer facts from hiding a conflict.
Immutable aliases retain their constant, variant and initialized-length facts.
References can nest through at most 64 layers in this bootstrap; exceeding that
construction limit reports B001 before deeper type/state cloning. Summary parts,
paths, transfers and candidate expansion also consume the existing shared budgets.
Nested reference types use separate `&` tokens, as in `<& &int32>`, or type aliases;
`&&` remains the logical operator token.
Temporary owners use ordinary operand types: `&1` borrows an `int32`, and borrowing
does not convert it to `&uint8`. Use an explicitly typed binding or function result
when another width is needed. Copying a temporary's value within its statement is
allowed; returning a reference from an inner statement does not extend its owner
to the surrounding expression. Reference-bearing Copy temporary owners preserve
their contents beneath the outer cell's lifetime; this does not introduce owned
cleanup, mutable reference carriers or reference-bearing list elements.
At actual function entry, all active argument origins and bounds are validated,
including nested summaries. Validation follows all returning argument evaluations;
a later argument that leaves or panics skips the call. Type predicates inspect tags
without reading reference payloads, while pointers used to access those tags must
still be alive.
Element assignment captures the local list's initialized length, evaluates its
index once and checks bounds, then evaluates the RHS once before storing. A bounds
failure skips the RHS. An index or RHS that leaves, restarts or panics skips the
remaining assignment. `values[2] = *view + 1` is valid when the RHS is the final use
of `view`; any later shared use conflicts, even when it selects a different index.
Replacing or modifying the first indexed collection during a returning index/RHS
reports E302, because the pending write depends on that storage. Static holder
fields outside that collection remain independent. Copy reads keep their existing
aggregate snapshot semantics.
For a nested target, each selected child has its own initialized length. The
compiler checks each prefix before evaluating the next index; a bounds failure
reports that prefix's byte span and skips every remaining index and the RHS.
Captured indices keep the destination fixed when later operands change index
variables. A borrow of any row or element conservatively conflicts with a nested
write if the borrow is used afterward.
The same ordered path handles both fields and indices. Every crossed field must
be mutable, and each index uses the selected list's own initialized length before
later indices or RHS effects. Views anywhere within the first indexed collection
conservatively overlap its writes, including views of other fields in its elements.
Copying a record counts as a use of all its references, even if a later operation
selects only one field. Direct projection, scalar comparison and scalar-primary
formatting do not keep unrelated component loans alive.
See [the storage design](OWNERSHIP.md) for the remaining analysis stages.

Union literals receive a numeric width when the expected union has one matching
numeric member. Multiple candidate widths require an explicitly typed value;
the checker reports E207 instead of choosing a width. Record constructors must
select one compatible union member; ambiguous shapes also report E207. Union
equality requires the same normalized union type on both sides. Inspect a nullable
value with a type predicate, or compare it with a null value explicitly typed as
the same union. Named blocks conservatively
forget mutable-value proofs at entry. Restarts that may emit again into a surviving
outer result report B001 until full loop dataflow is implemented.

## CLI behavior

`meowy help` lists the implemented commands and options. An explicit `.mwy` entry
is required. Ancestor `mod.mwy` files are detected and rejected because project
configuration is not implemented. `--standalone` deliberately bypasses that policy
for isolated source checking, including the conformance harness.

```sh
compiler/target/debug/meowy build compiler/examples/records.mwy \
  --profile release --output compiler/build/records --emit-llvm compiler/build/records.ll
compiler/target/debug/meowy check docs/conformance/sources/compact_min.mwy --standalone --json
```

`--emit-llvm` and `--json` are bootstrap inspection options. JSON diagnostics go to
stderr as one object per line with schema `meowy.bootstrap.diagnostic`, version 1,
code, message, path, half-open byte start/end, and one-based line/column. This is an
internal format, distinct from the documented release artifact schemas. `B002`
identifies bootstrap tool or output failures; it cannot count as a source rejection.

Default outputs are `build/x86_64-unknown-linux-gnu/<profile>/<entry stem>` under
the source directory. `--output` is relative to the current working directory.
Successful links replace outputs atomically. Failed builds preserve earlier files,
and `run` stops on failure. Source, manifest, lockfile, replay, and symlink output
paths are protected. `run` inherits working directory and streams, forwards
arguments after `--`, and returns the application's exit or signal status.

## Tests

```sh
cargo test --locked --manifest-path compiler/Cargo.toml
cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path compiler/Cargo.toml --check
python3 compiler/tests/conformance.py
python3 -m unittest discover -s compiler/tests -p 'test_*.py'
```

The Rust suite includes native execution in debug and release, overflow behavior,
scope control, records, literal limits, diagnostics, and safe output replacement.
The conformance harness runs each existing reference fixture independently and
compares actual checking results and stdout. Unsupported cases are listed separately.
Use `--strict` to require every catalog case; that full-language gate is expected
to fail while bootstrap gaps remain. `../docs/conformance/check.py` only validates
the fixture catalog and does not execute the compiler.

## Implementation map

| File | Responsibility |
| --- | --- |
| `src/lexer.rs`, `src/parser.rs`, `src/parser/`, `src/ast.rs` | Lossless tokens, parser state and focused expression/statement/type grammar |
| `src/check.rs`, `src/check/`, `src/hir.rs` | Shared checker state and focused resolution, type, expression and block checking modules |
| `src/list.rs`, `src/list_context.rs`, `src/list_context/` | Bounded lists, candidate selection, effectful blocks, isolated probes and inference budgets |
| `src/flow.rs` | Shared boolean guards for reachability, disjoint emissions and narrowing |
| `src/borrow.rs`, `src/borrow/` | Shared borrow facts, origin validation, value/control traversal and result transfers |
| `src/borrow_contract.rs` | Symbolic function inputs, caller origin substitution and all-input lifetime bounds |
| `src/borrow_value.rs` | Active union variants, component paths, coercions and bounded value snapshots |
| `src/loans.rs`, `src/loans/` | Guarded storage availability, value/control construction, authority and access conflicts |
| `src/diagnostic.rs`, `src/driver.rs`, `src/main.rs` | Diagnostics, commands, build publication and process launch |
| `src/backend.rs`, `src/backend/` | Bridge and generator core, aggregate/list/arithmetic/output lowering and focused native tests |
| `native/bridge.cpp` | LLVM verification, optimization and object emission |
| `native/runtime.cpp` | Versioned scalar output and panic ABI, separate from LLVM |
| `build.rs` | Exact native-tool version checks, bridge/runtime bootstrap |
| `tests/native.rs`, `tests/native/`, `tests/conformance.py` | Single native test target, shared harness, behavior-focused cases and catalog execution |

The full architecture and release gates remain in [COMPILER.md](../COMPILER.md).
