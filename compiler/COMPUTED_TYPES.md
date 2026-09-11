# Computed type blocks

The bootstrap supports straight-line blocks in required type expressions, using
existing type construction and lexical scopes from the checker:

```meowy
<Counts>:{
    element:<int32>
    -><(element)[4]>
}
values<Counts>:[3,7]
```

Local immutable, unannotated bindings hold supported type values. Local type aliases
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

## Explicit limits

One outer type-value resolution shares these bootstrap bounds with nested resolutions:

| Bound | Maximum |
| --- | --- |
| Resolver expression and block-statement visits | 4096 |
| Active resolver depth | 64 |
| Concrete type nodes traversed across resolved values/local aliases | 16384 |

Exhaustion reports B001. Nested blocks share the counters; independent roots reset
them. Existing parser, type/layout and proof limits still apply. These limits qualify
bootstrap support only; they do not implement the language's logical E220 counters.

Scalar scratch bindings, mutable scratch, branches/restarts, labeled blocks,
annotated or named emissions, general expression statements and source/helper calls
remain unsupported in type blocks. `core.Type` parameter/result annotations, generic
specialization, type equality, full purity analysis, intrinsic descriptions and the
complete required evaluator remain separate. Runtime type-value storage remains gated.

See [the runnable example](examples/computed-types.mwy), [STATUS.md](STATUS.md),
the [language contract](../docs/reference/compile-time.md), and the
[implementation pipeline](../COMPILER.md#the-pipeline).
