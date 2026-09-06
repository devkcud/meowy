# Syntax

[Documentation index](../README.md)

Source files use UTF-8 and the `.mwy` extension. Names are case-sensitive. The
portable identifier set is ASCII letters, digits, and `_`, with a letter or `_`
first. Type aliases conventionally start with an uppercase letter.

## No keywords

No identifier spelling is reserved by the grammar. Punctuation introduces
bindings, functions, blocks, types, matchers, and control targets. Words resolve
to values in a scope, including the well-known names supplied by the language.

`true`, `false`, and `null` are predefined constant values. Primitive types such
as `<boolean>` and `<uint8>` are predefined values in the type namespace. `self`
is a binding introduced by dispatch. All follow ordinary name resolution and
can be shadowed in an inner scope; shadowing cannot change their intrinsic
identity, representation, or implicit language behavior.

The foundational module `@"core"` exposes the predefined constants and types when
a local name shadows one of them. A missing primary emission, for example, still
produces the intrinsic null value regardless of a local binding named `null`.

```meowy
core : @"core"

{
    true : "an ordinary local name"
    enabled <core.boolean> : core.true
}
```

`leave`, `restart`, `Copy`, `Send`, and library operation names are not keywords.
Their behavior belongs to the value found through lookup. Aliasing an intrinsic
preserves its rules; giving an unrelated value the same name grants no special
behavior. For example, a scoped control operation can be called through an alias:

```meowy
'work {
    finish : 'work.leave
    finish()
}
```

The alias must stay inside its target's function, task, and lexical lifetime.
An ordinary record field named `leave` has no scope-control behavior unless it
actually contains that intrinsic value.

## Literals, values, and comments

| Form                            | Meaning                            |
| ------------------------------- | ---------------------------------- |
| `null`, `true`, `false`         | Predefined null and boolean values |
| `42`, `1_024`, `0xff`, `0b1010` | Integer literals                   |
| `3.5`, `1.0e-3`                 | Floating-point literals            |
| `"hello"`                       | UTF-8 string literal               |
| `"value: {expression}"`         | Interpolated string                |
| `[1, 2, 3]`                     | Bounded list literal               |
| `{ -> x : 1 }`                  | Block with a named emission        |
| `# comment #`                   | Delimited comment; may span lines  |

Comments do not nest. `#` inside a string is ordinary text. Strings accept `\n`,
`\r`, `\t`, `\0`, `\"`, `\\`, `\{`, and `\}` escapes. A raw newline is allowed
inside a string and is retained. There is no implicit indentation stripping.
String literals contain UTF-8 bytes and need not be NUL-terminated.

Integer literals are checked against their expected type; unconstrained integers
default to `<int32>` and unconstrained decimals to `<float64>`. A default that
cannot represent a literal is a diagnostic, not an automatic promotion. Negative
numbers use unary `-`.

## Statements and delimiters

A newline terminates a complete statement. `;` explicitly separates statements,
including multiple statements on one line. Newlines inside `(...)`, `[...]`, and
type expressions do not terminate an incomplete expression. A line beginning
with a dispatch `.` continues the preceding expression. Place a binary operator
at the end of a line to continue that expression on the next line.

Braces contain a sequence of statements. An expression statement evaluates and
discards its result; it does not implicitly emit that result.

No grammar production requires a space or tab. Indentation is presentation, and
spaces never select between a type test and an ascription. Use `;` when removing
a statement-ending newline. Record-type fields and type expansions can likewise
be separated with `;`, so a record type can be written `<{x<int32>;y<int32>}>`.
An optional final separator is permitted before the closing brace.

This is a complete program with no whitespace outside its string:

```meowy
debug:@"debug";value<int32><null>:7;|value<int32>|debug.print("{value}");
```

Compact source still has tokens. Identifiers and numbers cannot be joined into
different tokens, and multi-character operators such as `&&`, `->`, and `>>`
must stay intact. Use punctuation or a newline to separate tokens when needed;
removing whitespace with a text substitution is not a minifier. Comments and
literal contents retain their own bytes regardless of the surrounding layout.

## Forms at a glance

| Form                                    | Meaning                                                          |
| --------------------------------------- | ---------------------------------------------------------------- |
| `name : value`                          | Immutable binding                                                |
| `name := value`                         | Mutable binding                                                  |
| `name <T> : value`                      | Explicit binding type                                            |
| `name = value`                          | Reassign a mutable binding                                       |
| `<Name> : <T>`                          | Type alias                                                       |
| `-> value`                              | Primary emission                                                 |
| `-> name : value`                       | Immutable named emission                                         |
| `-> name := value`                      | Mutable named emission                                           |
| `(x <T>) { ... }`                       | Function value                                                   |
| `f <R> : (x <T>) { ... }`               | Function declaration with result type `R`                        |
| `f(value)`                              | Function call                                                    |
| `value.name`                            | Field selection                                                  |
| `value.(f)`                             | Call `f` with `value` as its first argument                      |
| `value.{ ... }`                         | Evaluate block with `self` bound to `value`                      |
| `\| condition \| statement`             | Conditional matcher arm                                          |
| `'scope { ... }`                        | Named, immediately evaluated block                               |
| `'scope -> value`                       | Primary emission into a named enclosing block                    |
| `'scope.leave()`                        | Finish that named block                                          |
| `'scope.restart()`                      | Clean up and restart that named block                            |
| `\| value<T> \| statement`              | Type predicate in a matcher condition                            |
| `value<>`                               | Compile-time type query                                          |
| `value<T>`                              | Proven type ascription in a value expression; no conversion      |
| `name<(expression)> : value`            | Binding annotated by a computed type                             |
| `@"name"`                               | Module import                                                    |
| `&value`, `&!value`                     | Shared or exclusive borrow                                       |
| `*reference`                            | Access a safe reference's referent                               |
| `>> expression`, `<< task`              | Start or join a task                                             |
| `&group<T[N]>`                          | Declare a bounded task group                                     |
| `&group >> expression`                  | Submit a task to a group                                         |
| `!{ ... }`                              | Block permitting operations with caller-proven safety conditions |
| `(x <T>) !{ ... }`                      | Function whose callers must establish those conditions           |
| `<:T : memory.Copy>`                    | Generic type binder constrained by a capability value            |
| `<D<:K,:V,:Y,:Z>> : <{...}>`            | Generic type alias with four independent type parameters         |
| `f<:K,:V><V> : (key<K>,value<V>) {...}` | Generic function with an explicit result type                    |

The escaped pipes in the table stand for literal `|` characters. Spaces around
angle brackets do not change their role: `value<T>` and `value <T>` have the same
meaning in the same grammatical position. See the contextual rules below.

`<T><U>` is a union in a type position, and `!<U>` subtracts members from a type.
Generic arguments name types without an extra pair of angle brackets:
`<task<int32>>` or `<D<string,uint32,boolean,string>>`. Use a type alias for a
union inside a generic argument. Declaration lists introduce each type binder
with `:`, as in `<:K,:V>`; argument lists omit those markers, as in `<K,V>`.
Parameters are positional, and an explicit list supplies every argument.
See [multiple type parameters](types.md#multiple-type-parameters) for complete
type declarations, functions, inference, and constraints.

A function's leading binder list is separate from its result annotations.
`f<:K,:V><V>` declares two parameters and result `<V>`; the binders are not union
alternatives. The existing `f<:T>` shorthand declares `T` and result `<T>` when
no separate result is written. `f<:T><T><null>` explicitly returns their union.
These forms retain the same meaning without spaces.

## Angle brackets in context

Declarations establish an annotation position: `name<T>:value` annotates the
binding, including inside a matcher body. Elsewhere, parse a complete type form
after an expression by these rules, independently of spacing:

1. Empty `<>` is always a type query.
2. Type arguments followed by call parentheses specialize that call:
   `accepts<T>(value)`. This rule also applies in a matcher condition.
3. In a matcher condition, a nonempty type suffix is a type predicate, taking
   precedence over the ascription interpretation. It has comparison precedence.
   `|value<T><U>|use(value)` tests membership in the union `<T><U>`.
4. In an ordinary value expression, that suffix is a proven ascription, with
   postfix precedence. `copy:value<T>` requests no conversion or runtime check.

Grouping parentheses in a condition retain its condition context. Boolean
operands of `&&`, `||`, and `!` do too. Call arguments, index expressions, and
the bodies of blocks and functions are ordinary value contexts; a nested matcher
starts its own condition context. Thus a predicate inside an argument must be
expressed through a matcher, not inferred from its distance to an outer `|`.

| Form                              | Interpretation                                 |
| --------------------------------- | ---------------------------------------------- |
| `\|value<T>\|use(value)`          | Test `value` and refine it in the arm          |
| `\|!(value<T>)\|reject()`         | Negate the type test                           |
| `\|accepts<T>(value)\|use(value)` | Call a specialized boolean function            |
| `\|accepts(value<T>)\|use(value)` | Pass a proven ascription to a boolean function |
| `\|flag\|copy:value<T>`           | Test `flag`; the body contains an ascription   |
| `copy:value <T>`                  | Ascription, even with a space                  |

The contexts compose through dispatch too:

```meowy
|t.{->self<MyCoolType>}<MyCoolType>|matched()
```

The dispatched block is an ordinary value context: `self<MyCoolType>` is an
ascription, and `->` emits that value. After `}`, the surrounding matcher context
resumes, so the outer `<MyCoolType>` is a predicate. This requires the flow type
of `self` to satisfy the ascription before the block emits; an outer test cannot
prove an earlier operation. With this exact block, the outer test succeeds if
evaluation completes normally. Dispatch still follows its usual move/borrow
rules. Neither the spaces nor the spelling of `self` introduces a special case.

For an ascription directly used as a condition, bind it first and match the
boolean binding. Parenthesizing `value<T>` alone still gives a type test there.
Because unary operators bind more tightly than predicates, write `!(value<T>)`
to negate a test; `!value<T>` tests the result of `!value`.

A complete type form wins over a relational interpretation, without consulting
whether a name resolves to a type. `age<18` is a comparison: it has no closing
type delimiter. `left<limit&&other>0` is two comparisons joined by `&&`; the
intervening operator cannot belong to the putative type form. Chained relational
comparisons are invalid; parentheses must express the intended grouping.

`<(expression)>` evaluates a compile-time expression that produces a type. It
allows computed annotations such as `other<(name<>)>:value` without using a space
to separate two identifiers. A function type instead contains an arrow after its
parameter list: `<(T)->R>`. Bare computed annotations such as `other name<>:value`
are not part of the grammar. See [type queries](types.md#type-queries).

## Operators and evaluation order

From highest to lowest precedence:

| Level | Operators/forms                                                                    |
| ----- | ---------------------------------------------------------------------------------- |
| 1     | Calls, field selection, indexing, dispatch, type query/ascription                  |
| 2     | Unary `!`, `-`, `~`, dereference `*`, borrow `&`, `&!`, task start `>>`, join `<<` |
| 3     | `*`, `/`, `%`                                                                      |
| 4     | `+`, `-`                                                                           |
| 5     | Integer bitwise `&`, then `^`, then `\|`                                           |
| 6     | `<`, `<=`, `>`, `>=`, type predicates                                              |
| 7     | `==`, `!=`                                                                         |
| 8     | `&&`                                                                               |
| 9     | `\|\|`                                                                             |

Binary arithmetic operators associate left-to-right; comparisons cannot be
chained. Assignment, emissions, and matchers are statement forms. Parentheses
override precedence. `>>` and `<<` are never bit shifts; use `bits.shl` and
`bits.shr`. Put a bitwise `|` expression in parentheses inside a matcher so it
cannot be confused with an arm delimiter.

`&!` is the exclusive-borrow operator; `&(!value)` instead borrows a negated
boolean. In type expressions, `<&!T>` is an exclusive reference and `<*!T>` is a
writable raw pointer. A `!` followed by a block opener marks an unchecked block,
with or without intervening whitespace. To negate a block's boolean primary,
write `!({ ... })`. These forms have distinct punctuation, not contextual words.

Operands and call arguments evaluate left-to-right. `&&` and `||` short-circuit.
Dispatch evaluates its receiver once. Task start captures its inputs at submission
and evaluates its operand in the child task; the concurrency reference specifies
that boundary.

## Names and scopes

Bindings are lexically scoped and visible after their declaration. Function
declarations can refer to themselves; mutual recursion requires explicit function
type declarations. Shadowing is allowed in a nested block, not by redeclaring a
name in the same block. `self` is a contextual binding and can be shadowed.

Types, labels, and values use distinct namespaces. A task group's `&name` shares
the spelling of a borrow expression, so a group and a value must not have the same
name in overlapping scopes. Label operations can only target a lexically enclosing
scope in the current function and task.

The names of intrinsic values never introduce extra parser productions. Function
constraints use punctuation in their type binders; returned borrows follow the
[lifetime rules](memory.md#lifetimes); native layout is selected by a normal
compile-time call to `ffi.record`. `!{ ... }` marks a boundary for operations
requiring a safety proof; it does not disable type checking.
