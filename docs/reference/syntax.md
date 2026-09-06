# Syntax

[Documentation index](../README.md)

Source files use UTF-8 and the `.mwy` extension. Names are case-sensitive. The
portable identifier set is ASCII letters, digits, and `_`, with a letter or `_`
first. Type aliases conventionally start with an uppercase letter.

## Literals and comments

| Form                            | Meaning                           |
| ------------------------------- | --------------------------------- |
| `null`, `true`, `false`         | Null and boolean literals         |
| `42`, `1_024`, `0xff`, `0b1010` | Integer literals                  |
| `3.5`, `1.0e-3`                 | Floating-point literals           |
| `"hello"`                       | UTF-8 string literal              |
| `"value: {expression}"`         | Interpolated string               |
| `[1, 2, 3]`                     | Bounded list literal              |
| `{ -> x : 1 }`                  | Block with a named emission       |
| `# comment #`                   | Delimited comment; may span lines |

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

## Forms at a glance

| Form                        | Meaning                                       |
| --------------------------- | --------------------------------------------- |
| `name : value`              | Immutable binding                             |
| `name := value`             | Mutable binding                               |
| `name <T> : value`          | Explicit binding type                         |
| `name = value`              | Reassign a mutable binding                    |
| `<Name> : <T>`              | Type alias                                    |
| `-> value`                  | Primary emission                              |
| `-> name : value`           | Immutable named emission                      |
| `-> name := value`          | Mutable named emission                        |
| `(x <T>) { ... }`           | Function value                                |
| `f <R> : (x <T>) { ... }`   | Function declaration with result type `R`     |
| `f(value)`                  | Function call                                 |
| `value.name`                | Field selection                               |
| `value.(f)`                 | Call `f` with `value` as its first argument   |
| `value.{ ... }`             | Evaluate block with `self` bound to `value`   |
| `\| condition \| statement` | Conditional matcher arm                       |
| `'scope { ... }`            | Named, immediately evaluated block            |
| `'scope -> value`           | Primary emission into a named enclosing block |
| `'scope.leave()`            | Finish that named block                       |
| `'scope.restart()`          | Clean up and restart that named block         |
| `value <T>`                 | Type predicate in a matcher condition         |
| `value<>`                   | Compile-time type query                       |
| `value<T>`                  | Proven type ascription; no conversion         |
| `@"name"`                   | Module import                                 |
| `&value`, `&mut value`      | Shared or exclusive borrow                    |
| `*reference`                | Access a safe reference's referent            |
| `>> expression`, `<< task`  | Start or join a task                          |
| `&group<T[N]>`              | Declare a bounded task group                  |
| `&group >> expression`      | Submit a task to a group                      |

The escaped pipes in the table stand for literal `|` characters. Type ascription
and generic specialization attach directly to their subject (`value<T>`,
`choose<string>(...)`). A type predicate is separated from its subject
(`value <T>`) in a matcher. A comparison such as `age < 18` has an expression on
the right, not a closed type form.

`<T><U>` is a union in a type position, and `!<U>` subtracts members from a type.
Generic arguments name types without an extra pair of angle brackets:
`<task<int32>>`. Use a type alias for a union inside a generic argument.

## Operators and evaluation order

From highest to lowest precedence:

| Level | Operators/forms                                                                      |
| ----- | ------------------------------------------------------------------------------------ |
| 1     | Calls, field selection, indexing, dispatch, type query/ascription                    |
| 2     | Unary `!`, `-`, `~`, dereference `*`, borrow `&`, `&mut`, task start `>>`, join `<<` |
| 3     | `*`, `/`, `%`                                                                        |
| 4     | `+`, `-`                                                                             |
| 5     | Integer bitwise `&`, then `^`, then `\|`                                             |
| 6     | `<`, `<=`, `>`, `>=`, type predicates                                                |
| 7     | `==`, `!=`                                                                           |
| 8     | `&&`                                                                                 |
| 9     | `\|\|`                                                                               |

Binary arithmetic operators associate left-to-right; comparisons cannot be
chained. Assignment, emissions, and matchers are statement forms. Parentheses
override precedence. `>>` and `<<` are never bit shifts; use `bits.shl` and
`bits.shr`. Put a bitwise `|` expression in parentheses inside a matcher so it
cannot be confused with an arm delimiter.

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

`null`, `true`, `false`, and the type operators are reserved. `leave` and `restart`
are control operations only on labels. `unsafe { ... }` marks a checked boundary
for operations requiring a safety proof; it is not a way to disable type checking.
