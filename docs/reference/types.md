# The type system

[Documentation index](../README.md)

meowy is statically typed. Every binding, field, parameter, emission, and task
result has a type before execution. Type expressions are compile-time values;
ordinary runtime control flow cannot change a type or specialize a function.

## Primitive and storage types

| Type                     | Meaning                                                    |
| ------------------------ | ---------------------------------------------------------- |
| `<null>`                 | The single value `null`                                    |
| `<never>`                | No normally produced value; used for divergence or panic   |
| `<boolean>`              | `true` or `false`                                          |
| `<int8>` … `<int128>`    | Signed integers of width 8, 16, 32, 64, or 128 bits        |
| `<uint8>` … `<uint128>`  | Unsigned integers of the same widths                       |
| `<isize>`, `<usize>`     | Signed and unsigned integers of the target pointer width   |
| `<float32>`, `<float64>` | Binary floating-point values of the stated width           |
| `<string>`               | Borrowed, immutable UTF-8 byte view                        |
| `<error>`                | Common descriptor for recoverable error values             |
| `<T[N]>`                 | Inline list with length at most constant `N`               |
| `<T[]>`                  | Borrowed immutable slice, with runtime length              |
| `<&T>`, `<&!T>`          | Shared and exclusive references                            |
| `<*T>`, `<*!T>`          | Read-only and writable raw pointers                        |
| `<(T, U) -> R>`          | Non-capturing function pointer                             |
| `<!(T, U) -> R>`         | Function pointer requiring a caller-proven safety boundary |

`N` is a non-negative compile-time integer. A type alias can name any composite
type. Extended numeric widths belong in libraries with explicit representation
and conversion rules; there is no target-dependent meaning of a built-in `int`.

Primitive type names denote well-known type values, not reserved words. A type
alias's right-hand side can be a type literal or a compile-time expression that
produces a type, such as `ffi.record("C", ["x" : <float32>])`. Name lookup and
aliasing do not change a type's identity or its representation.

An error's common representation is a tagged, fixed-size descriptor with a static
code and message plus an optional owned payload box. Errors without a payload do
not allocate. The descriptor keeps its concrete error identity for matching;
erasure to `<error>` does not erase that tag. Creating a boxed payload requires an
allocator and can itself fail.

An erased error payload does not imply `tasks.Send`. Preserve a concrete
transferable error type when sending failures across tasks; opaque task-runtime
errors provide their own transfer guarantees.

The [errors module](stdlib/errors.md) supplies custom definitions and metadata
inspection. `errors.define<P>(spec)` exposes a nominal `<definition.Error>` and
an allocation-free constructor retaining `P` inline. A generated error with
`P = null` fits the common descriptor; other generated payloads stay concrete
until explicitly boxed. Plain records do not become errors by naming a field
`code`, `message`, or `cause`.

## Inference and assignment

```meowy
age := 18                 # inferred int32, for every assignment #
count <uint8> := 18        # uint8 #
maybe <uint8><null> := null
maybe = 42
```

Initializers are checked against annotations. Without an annotation, an ordinary
scalar literal is widened to its primitive type. String-literal unions are kept
when an expected type requests them.

Assignment must fit the original type. There is no implicit string parsing,
signedness change, integer-width change, floating/integer conversion, or dynamic
boxing. Untyped numeric literals may be checked against the other operand's type.
Two already typed arithmetic operands must have the same numeric type.

Unions accept any of their members. A descriptor-compatible concrete error may
be passed as `<error>` without allocation. An error retaining arbitrary inline
owners must stay concrete or have its payload explicitly boxed before erasure.
In a matcher, the predicate `value<error>` recognizes either representation
without converting it. References may shorten their lifetimes, and an exclusive
reference may be reborrowed as shared. Other storage conversions are explicit.
In particular, making a slice of a bounded list uses `list.slice()`.

On an erased `<error>`, a concrete matcher tests the retained nominal tag. It
does not move the boxed owner into inline storage. `errors.view<E>` checks and
borrows the stored concrete error; `errors.take<E>` consumes it after a proven
match. An ascription cannot replace this explicit extraction. The erased
descriptor itself is neither `memory.Copy` nor `tasks.Send`, even if a particular
stored error has both capabilities.

Every normal path must initialize a binding before it is read. Untyped empty
lists require an expected element type. A function taking no arguments still
requires `()` at the call site.

## Unions and narrowing

Adjacent types form a union:

```meowy
<MaybeCount> : <uint32><null>
<Theme> : <"dark"><"light">
```

Unions are sets: order and duplicates do not matter. `<never>` adds no member.
The containing primitive absorbs its literal subtypes. A closed union carries a
discriminant and storage large enough for its largest variant; a type predicate
tests that discriminant, not a conversion or a guess based on field names.

```meowy
fallback <:T> : (value <T><null>, alternative <T>) 'result {
    | value <null> | {
        'result -> alternative
        'result.leave()
    }
    -> value
}

name : fallback<string>(null, "Dev")
```

The successful arm leaves the function, so `value` excludes `null` at the final
emission. An arm that merely logs an error and continues does not establish this.

`!<U>` subtracts covered alternatives from a type. It changes a compile-time set,
not the underlying runtime value. Removing a base such as `<error>` removes every
error alternative. Subtracting a literal from an unrestricted primitive is not
representable as a new primitive type and is rejected; use a predicate instead.

**Invalid — the initializer can still be null:**

```meowy
value <string><null> : null
present <(value<>!<null>)> : value
```

Within a proven branch, an ascription such as `value<string>` is permitted. It
does not insert a trap, parse, or unchecked cast. If the current flow type is not
assignable to the requested type, the ascription is rejected.

The surrounding grammar chooses the operation, not a space: `value<T>` and
`value <T>` are both predicates in matcher conditions and both ascriptions in
ordinary value expressions. Grouping a condition does not turn a predicate into
an ascription. Generic calls and type queries retain their own forms; see
[angle brackets in context](syntax.md#angle-brackets-in-context).

## Type queries

`expression<>` yields the static type of an expression without evaluating it.
Thus `get()<>` queries the result type and does not call `get`.

```meowy
name : "Dev"
other <(name<>)> : "Ada"
```

The `<(expression)>` form evaluates a type-producing compile-time expression
inside an explicit annotation delimiter. The example can therefore also be
written `other<(name<>)>:"Ada"`. It does not evaluate `name` at runtime. Type
subtraction can appear inside the same form, as in `<(value<>!<null>)>`.

A query at a refined program point observes the refined type. Queries cannot be
used to make a type depend on a runtime value. `<:T>` introduces a generic type
parameter in a declaration; it is not a cast and cannot annotate an ordinary
string binding as an arbitrary caller-chosen type.

## Records and composition

Record types are structural and exact: field names, field types, mutability, and
the primary type determine compatibility. Declaration order does not determine
type identity or a native ABI. Width subtyping that silently discards fields is
not implicit; build the desired smaller record explicitly.

```meowy
<Identity> : <{
    id <uint64>
    name <string>
}>

<Preferences> : <{
    theme <"dark"><"light">
}>

<User> : <{
    <Identity>
    <Preferences>
    nickname <string><null>
}>

user <User> : {
    -> id : 7
    -> name : "Dev"
    -> theme : "dark"
}
```

The nullable `nickname` is filled with `null`. Nullability permits omission at
construction, not disappearance from the record's layout. A nested record differs
from an expanded one: `identity <Identity>` creates one field, while `<Identity>`
expands its fields. Every collision, even one with identical types, is a static
error. Composing more than one non-null primary type is also an error.

Aliases do not create nominal distinctions: two aliases of `<uint64>` are the
same type. Use a wrapper record with a distinct named field when an identifier
must not be confused with another integer. Library resource and error types are
opaque and nominal; clients cannot forge them by copying their apparent fields.

Recursive records must pass through a reference, pointer, or owning indirection.
An inline record cannot contain itself at infinite size.

## Functions and generics

The result annotation on a function declaration is shorthand:

```meowy
increment <int32> : (value <int32>) { -> value + 1 }
callback <(int32) -> int32> : increment
```

For a function-valued alias such as `callback`, the annotation is the full
function type. Function parameters are required, including nullable parameters;
`<null>` permits a value, not an omitted argument. Parameter and result types of
public functions must be explicit. Local function results may be inferred.

`(parameters) !{ ... }` creates a function with a caller-proven safety contract.
Its type is `<!(Parameters) -> Result>` and its calls require a `!{ ... }` block.
Assigning it to an ordinary function pointer cannot erase that requirement.
An ordinary function may use an inner `!{ ... }` block after checking the necessary
preconditions itself. Neither form introduces a keyword.

### Generic type declarations

A leading `:` declares a type parameter. Generic type aliases put these binders
inside the alias name's parameter list:

```meowy
<Pair<:T>> : <{
    first <T>
    second <T>
}>
```

`<Pair<uint32>>` substitutes `<uint32>` for both fields. The `:` belongs to the
declaration; supplying an argument uses the type's name without that binder
marker or an extra pair of angle brackets. The alias does not create a runtime
constructor named `Pair`; construct a record with the desired annotation or
write an ordinary function that returns it.

### Multiple type parameters

Separate binders with commas and introduce **each** one with `:`. There is no
single-parameter restriction. This complete example declares and uses four
independent type parameters:

```meowy
-> <D<:K, :V, :Y, :Z>> : <{
    key <K>
    value <V>
    context <Y>
    status <Z>
}>

record <D<string, uint32, boolean, string>> : {
    -> key : "attempts"
    -> value : 3
    -> context : true
    -> status : "ready"
}
```

`K`, `V`, `Y`, and `Z` are ordinary names in the type namespace. They are scoped
to this declaration, must be unique within the binder list, and need not denote
different concrete types: the example supplies `<string>` twice. The leading
`->` exports the generic alias; omit it for a private alias.

Arguments are positional: the example binds `K = string`, `V = uint32`,
`Y = boolean`, and `Z = string`. A type use must supply exactly the declared
number of type arguments. There are no default type arguments, named arguments,
partial lists, or placeholder arguments. `<D<string, uint32>>` therefore does
not leave `Y` and `Z` for later. An ordinary binding may still omit its entire
annotation and infer its type from a value.

Each argument can itself be a concrete reference, list, function, or instantiated
generic type. Use a type alias when supplying a union:

```meowy
<MaybeStatus> : <string><null>
<OptionalStatusRecord> : <D<string, uint32, boolean, MaybeStatus>>
```

`D<K,V,Y,Z>` describes four parameter positions; `<K><V><Y><Z>` instead describes
one union. A union argument occupies one position. `<D<K,V,Y,Z>>` remains a
structural alias after substitution: its parameter list does not add a hidden
nominal tag, erase fields, introduce variance, or perform conversions.

Whitespace does not select any of these forms. For example, this is the same
declaration with a private alias and no spaces:

```meowy
<D<:K,:V,:Y,:Z>>:<{key<K>;value<V>;context<Y>;status<Z>}>
```

These are **type** binders. Library APIs such as `collections.Array<T,N>` also
accept a compile-time capacity in their documented `N` position; a declaration
`:N` would introduce a type parameter, not an integer capacity parameter.

### Generic functions and result types

A function's binder list comes after its name, before any explicit result
annotation. Using `D` from the preceding example, this exported constructor moves
each input into its corresponding result field:

```meowy
-> make_d<:K, :V, :Y, :Z><D<K, V, Y, Z>> : (
    key <K>, value <V>, context <Y>, status <Z>
) {
    -> {
        -> key : key
        -> value : value
        -> context : context
        -> status : status
    }
}

explicit : make_d<string, uint32, boolean, string>("attempts", 3, true, "ready")

count <uint32> : 3
inferred : make_d("attempts", count, true, "ready")
```

Both results have type `<D<string,uint32,boolean,string>>`. In the first call,
the explicit `<uint32>` argument gives `3` its expected type. In the second,
the already typed `count` supplies that information. Each concrete specialization
keeps normal inline storage, moves, borrows, and cleanup; constructing a generic
record does not implicitly allocate or box its fields.

The leading binder list declares parameters; it is **not** a result union member.
Subsequent type annotations form the complete result type. A single binder with
no separate result retains the shorthand used by `fallback`: `<:T>` declares `T`
and uses `<T>` as the result. Multiple binders have no implied result type; a
local function may infer its result from emissions, while an exported function
must provide an explicit result annotation.

| Function declaration prefix         | Type parameters    | Result contract                                            |
| ----------------------------------- | ------------------ | ---------------------------------------------------------- |
| `identity<:T> :`                    | `T`                | Shorthand result `<T>`.                                    |
| `consume<:T><null> :`               | `T`                | Explicit result `<null>`.                                  |
| `maybe<:T><T><null> :`              | `T`                | Explicit union result `<T><null>`.                         |
| `second<:K,:V><V> :`                | `K`, `V`           | Explicit result `<V>`.                                     |
| `make_d<:K,:V,:Y,:Z><D<K,V,Y,Z>> :` | `K`, `V`, `Y`, `Z` | Explicit constructed result.                               |
| `local_pair<:K,:V> :`               | `K`, `V`           | Inferred local result; incomplete at an exported boundary. |

For example, a nullable result needs its success type after the binder list too:

```meowy
-> maybe<:T><T><null> : (value <T>, present <boolean>) 'result {
    | present | {
        'result -> value
        'result.leave()
    }
    -> null
}
```

`value` moves into the result on the first path; otherwise normal scope cleanup
releases it. No copying constraint is needed. Function parameters and result
annotations may refer to any binder in the list, even when the result does not
mention every parameter. Bind all parameters in one list, rather than writing
several separate binder lists.

### Inference and specialization

Generic calls specialize for concrete compile-time arguments. Supply the entire
type-argument list or omit it and infer the entire list from the static types of
value arguments, including typed callback signatures and nested record fields.
Result annotations at the call site, capability requirements, and runtime values
do not choose otherwise unknown arguments. A parameter used only in the result,
or erased from every parameter type by alias expansion, needs an explicit list.

Inference first gathers information from already typed arguments. Literals are
checked against a uniquely established expected type; when none is available,
they follow the [ordinary literal defaults](syntax.md#literals-values-and-comments).
For example, `make_d("attempts", 3, true, "ready")` infers `V = int32`. Assigning that result
to a binding requiring `V = uint32` does not silently change the specialization.
Use an explicit list or an already typed argument, as above.

Repeated occurrences of one binder must resolve consistently. Inference does not
invent a union, numeric widening, or erased type to reconcile conflicting typed
arguments. Once the arguments are known, ordinary assignment compatibility
applies; explicitly supplying a named union permits its members as normal.

Structural matching uses only uniquely determined information. A parameter
`<T><null>` receiving `<string><null>` does not alone distinguish `T = string`
from `T = string|null`; both substitutions fit. Defer that ambiguity until other
arguments determine `T`, or require an explicit list. For example,
`fallback(null, "Dev")` determines `T = string` from its plain `<T>` alternative,
then checks that the first argument fits `<string><null>`.

Aliases are expanded structurally during inference. There is no hidden record
tag from which to recover a type argument that leaves no evidence in the fields.
Ambiguous or conflicting inference, a wrong generic arity, or a wrong argument
kind uses `E212`, identifying the binder and contributing arguments. Duplicate
binders use `E203`; unmet capabilities use `E210`; an incomplete exported result
uses `E214`. A type cannot depend on runtime input (`E211`).

The body must be valid for every type admitted by its declared constraints. An
unconstrained parameter cannot be added, indexed, or copied just because one call
happens to supply an integer. Supply a typed operation as a function parameter
when generic code needs it; there is no implicit operator-overloading dictionary.

### Per-parameter constraints

Generic values can be moved or borrowed without requiring them to be copyable.
A body using the same owned generic value twice is rejected unless its declared
constraint proves copying legal. `memory.Copy` and `tasks.Send` are compiler-known
capability values. A second `:` in a generic binder declares its requirement:
`<:T : memory.Copy>` accepts only copyable types. `&` joins requirements in this
position: `<:T : memory.Copy & tasks.Send>` requires both capabilities.

In a list, each requirement belongs only to the binder before it. A comma ends
that binder; the next `:` introduces the next one:

```meowy
memory : @"memory"
tasks : @"tasks"

<Envelope<:K : memory.Copy & tasks.Send, :V : tasks.Send>> : <{
    key <K>
    value <V>
}>
```

`K` must be both copyable and transferable; `V` only needs transferability.
The constraints apply at every specialization, including when a parameter is
unused in the alias body. An unconstrained neighboring parameter gains no
capabilities from them. In the constructor above, all four inputs may be owners
because each is moved once. A helper copying only the key constrains only `K`:

```meowy
-> copy_key<:K : memory.Copy, :V, :Y, :Z><K> : (source <&D<K,V,Y,Z>>) {
    -> source.key
}
```

This helper uses the preceding `D` declaration and `memory` import. It borrows
the record and copies the key without requiring `V`, `Y`, or `Z` to be copyable.
The original single-parameter form still works:

```meowy
memory : @"memory"

copy <:T : memory.Copy> : (value <&T>) {
    -> *value
}
```

Constraint names follow ordinary lookup in the type namespace. An alias of a
capability has the same meaning; an unrelated value with the same name cannot
grant copying or transfer privileges. There is no trailing constraint clause or
word-based modifier on a declaration.

### Callable environments

Capturing functions have an inferred, concrete environment type. They are not
function pointers and do not implicitly allocate. A closure borrows captures
when possible; an escaping closure must move the required owners into its
environment. Consuming a capture makes the closure callable only once. A closure
that mutates captures needs exclusive access to its environment on every call.
Generic function parameters can preserve a closure's concrete type; converting
it into a dynamically dispatched callable requires explicit allocated storage.

## Conversion and type erasure

`numbers.convert<T>(value)` performs a checked numeric conversion and returns
`<T><numbers.RangeError>`. Floating-to-integer conversion also rejects fractions,
NaN, and infinities. `numbers.truncate<T>` and `numbers.wrapping<T>` are separately
named operations for deliberately different behavior.

`strings.to_uint8(text)` parses text and returns a union. `value<uint8>` does
neither parsing nor numeric conversion. Pointer reinterpretation is a separate
unsafe operation with alignment and lifetime obligations.

`<any>` is explicitly erased owned storage for any non-null concrete type. It is
not a hand-maintained union of primitive types, and it includes records, functions,
and resource owners. It carries type identity and destruction information.
`dynamic.box(value, allocator)` creates it and can fail to allocate. Moving an
`any` moves its owner; copying or assigning an arbitrary value into it implicitly
is forbidden. A plain `null` requires `<any><null>`.

A matcher on `any` tests its stored concrete type. After a successful test,
`dynamic.take<T>(value)` consumes the box and recovers the owner; the test alone
does not copy or extract that owner. A reference into an erased value cannot
outlive the box. `any` is not `Send`, because erasure does not promise that the
hidden type can cross task boundaries.

## Numeric behavior

Integer overflow, division by zero, signed minimum divided by `-1`, and an invalid
shift count panic in all build profiles. Compile-time-known failures are static
errors. `numbers.checked_add` returns a union, and `numbers.wrapping_add` wraps
at the stated width. Saturating operations must also be named explicitly.

Signed division truncates toward zero; the remainder has the dividend's sign.
Floating-point operations use their declared precision and round to nearest,
ties to even. They support infinities, NaN, and signed zero; NaN is not equal to
itself. Optimizations may not reassociate arithmetic or replace checked integer
operations with wrapping ones unless the source explicitly chooses that behavior.
