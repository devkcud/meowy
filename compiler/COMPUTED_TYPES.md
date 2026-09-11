# Computed type blocks

The bootstrap supports straight-line blocks in required type expressions, using
existing type construction and lexical scopes from the checker:

```meowy
settings:{->limits:{->base<uint8>:{offset<uint8>:1;->offset+1}}}
<Counts>:{
    element:<int32>
    base:settings.limits.base
    capacity:base*2
    -><(element)[capacity]>
}
values<Counts>:[3,7]
```

Local immutable, unannotated bindings hold supported type values. Immutable integer
bindings may have an explicit integer annotation, such as `base<uint8>:2`.
Local type aliases
such as `<Count>:<int32>` use the separate type namespace. Nested blocks may produce
types for these bindings or for computed annotations. Parenthesized expressions and
supported type queries retain their existing behavior. Symbolic type members such as
`core.int32`, foundation type aliases and imported types preserve their identities.

Exactly one unlabelled, unnamed, unannotated primary emission supplies the result.
An emission does not exit the block: subsequent declarations and errors are checked
in source order. Duplicate primary emissions report E205. A missing type result or
a runtime value used as a type reports E211; duplicate local declarations use E203.
The block's local names leave scope when it finishes. Constructed records, lists and
references retain ordinary storage, layout, mutability and ownership rules.

No HIR statements, runtime locals or functions are created for the type block itself.
Exported aliases can use computed blocks, and importers consume the same resulting
types. Application initializers still run only during program execution. Existing
runtime-parameter type queries inspect the parameter type, without treating its
value as a compile-time input. Documentation derives the resulting signatures and
checks local declaration links while the construction scope is active.

Known `debug.print` and `debug.panic` calls in evaluated positions report E219,
including calls through resolved aliases and calls after a primary emission.
The checker does not execute them. This is a narrow effect check; source helper
purity and transitive call analysis are not implemented by this slice.

## Integer calculations

Local integer bindings retain their checked width and signedness through aliases,
subsequent calculations and type queries. Supported expressions are integer literals,
eligible names, parentheses, unary `-`/`~` and binary `+`, `-`, `*`, `/`, `%`, `&`, `|`,
`^`. They reuse the scalar checker and constant evaluator. Incompatible widths use
E213, literal overflow E216 and invalid arithmetic E107. Negative or unrepresentable
list capacities retain E104. Required arithmetic is checked even inside an unreachable
runtime branch, and its temporary checking state is restored afterwards.

Inputs may be static integers in the construction scope or immutable integer bindings
with recorded initializer eligibility. The checker tracks separate evidence over
checked literals, aliases, supported unary/arithmetic expressions and straight-line
integer blocks. Nested immutable integer records also carry complete initializer evidence.
Every local value dependency must already have that evidence; a folded constant alone does not
establish eligibility. Exact widths and source declaration identities are preserved.

Required reads can use eligible lexical inputs across function scopes. This does not
enable runtime captures or expose private names from another file. Original runtime
bindings and application effects remain in the program; checking/building does not
execute them. Runtime parameters, mutable state and effectful results remain unavailable
(E211). Blocks with effects, mutation or control flow, imported-data reads and helper
calls remain unproven; unsupported folded inputs retain B001.

Evidence retains integer failures from unreachable runtime paths. A required read
reports E107 at the original failing expression, even through aliases. Dependency
work is charged transitively on each read, including cached values and repeated
references, against the shared bootstrap bound. Independent required roots reset it.
Extents within computed-type roots use the same eligibility checks as scratch bindings.
Direct extents outside those roots retain their existing supported profile.

The [example](examples/computed-types.mwy) calculates a capacity of four from an
eligible immutable `uint8` record field. Scalar scratch produces no runtime locals, and documentation preserves
its actual integer signature. Floating-point, boolean and text scratch, comparisons,
shifts, mutable scratch and helper calls remain separate capabilities.

## Block initializers

A supported integer block has immutable eligible integer bindings and exactly one
primary emission targeting that block. Nested eligible blocks are supported. Every
statement is inspected in source order, including unused bindings after the emission;
an emission does not return early. Assignments, named/outer emissions, calls, branches,
restarts and other runtime statements do not receive eligibility evidence.

Evidence retains checked integer values after block locals leave scope, along with
source failures and transitive work. Required reads use those proven values with their
original widths. Ordinary runtime reads and initialization remain unchanged. Unused
integer bindings still contribute work and failures, so they cannot hide an invalid
operation after the result emission.

The checked result must have a concrete integer type. Unreachable blocks whose HIR
result was erased to `never` can retain error-only evidence, but do not supply an
invented integer result. Hidden arithmetic failures remain E107 at their original spans.

## Record-field inputs

Named immutable record bindings and their aliases can supply integer leaves to copied
integer bindings, computed scratch and list extents. Every record has a unit primary
and nonempty immutable fields containing integers or records of the same kind. A shape
is bounded to 256 total fields across all descendants and 32 record levels; unused
local records must also have supported shapes. Checked field-index paths retain exact
widths independently of source declaration order.

Nested record construction shares scoped scalar/record evidence and work. Initializers
may use earlier emitted records, record aliases, integer fields and eligible scalar
blocks/locals. Every initializer statement remains checked. A selected leaf cannot hide
an effect, mutable descendant, invalid sibling computation or unused tail work.

Ordinary immutable subrecord aliases retain the complete original ancestor's errors
and transitive work. A subsequent leaf read cannot escape that evidence by shortening
the path. Hidden failures remain E107 at the originating expression, even when the
failing sibling lies outside the projected subrecord. No partial record is admitted.
Unreachable inline record emissions whose field identity was erased from HIR remain
unavailable; declared record aliases can retain known error evidence.

Direct required paths have a named local record root, optionally grouped. Nested paths
and copied subrecords preserve checked type/member identity. Eligible lexical leaves
may be used inside functions without enabling ordinary runtime captures. Qualified
type identities such as `core.int32` retain their separate behavior.

Reference projections, inline roots, non-integer leaves, empty records and non-unit
primaries remain outside this slice. Imported data remains gated, including indirect
copies through internal module bindings. Checking never runs initializers; ordinary
record reads and runtime initialization remain unchanged.

## Explicit limits

One outer type-value resolution shares these bootstrap bounds with nested resolutions:

| Bound | Maximum |
| --- | --- |
| Resolver/scalar-validation expressions and block-statement visits | 4096 |
| Active resolver depth | 64 |
| Concrete type nodes traversed across resolved values/local aliases | 16384 |

Exhaustion reports B001. Nested blocks share the counters; independent roots reset
them. Existing parser, type/layout and proof limits still apply. These limits qualify
bootstrap support only; they do not implement the language's logical E220 counters.

Non-integer scalar scratch, mutable scratch, branches/restarts, labeled blocks,
annotated or named emissions, general expression statements and source/helper calls
remain unsupported in type blocks. `core.Type` parameter/result annotations, generic
specialization, type equality, full purity analysis, intrinsic descriptions and the
complete required evaluator remain separate. Runtime type-value storage remains gated.

See [the runnable example](examples/computed-types.mwy), [STATUS.md](STATUS.md),
the [language contract](../docs/reference/compile-time.md), and the
[implementation pipeline](../COMPILER.md#the-pipeline).
