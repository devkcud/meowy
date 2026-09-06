# A first tour

[Documentation index](../README.md) · [Configure mod.mwy](mod.md) · [gatostyle](gatostyle.md)

meowy programs describe values and their transformations. Start with a file named
`main.mwy`:

```meowy
print : @"debug".print

name : "meowy"
print("Hello, {name}!")
```

`@"debug"` imports a foundational module, `.print` selects an exported function,
and `(...)` calls it. Strings interpolate expressions inside braces.

The grammar uses punctuation, with no reserved keywords. `true`, `false`, `null`,
type names, and the contextual `self` are well-known bindings. Names such as
`print`, `leave`, and `restart` refer to values; their spelling is not syntax.

## Bindings have stable types

`:` creates an immutable binding. `:=` creates a mutable binding, and `=` assigns
to an existing mutable binding.

```meowy
limit <uint8> : 10
count <uint8> := 0
count = count + 1
```

Annotations can be inferred: `count := 0` infers `<int32>`. Inference chooses a
type at the binding's declaration; reassignment does not change it. A string
cannot later become an integer merely because its binding is mutable.

## Blocks build values

```meowy
user : {
    -> "Dev"
    -> id : 42
}

user     # primary value: "Dev" #
user.id  # field: 42 #
```

`-> expression` emits the primary value. `-> name : expression` emits a named
field. The block runs immediately, once, and finishes before `user` is available.
Emitting does not stop it:

```meowy
greeting : {
    -> "Hello"
    print("The block is still running")
    -> language : "en"
}
```

A block with fields but no primary emission is a record whose primary value is
`null`. It still has its own record type; it is not interchangeable with plain
`null`. An empty block has type `<null>`.

## Functions run when called

A parameter list before a block creates a function. The annotation before `:`
describes the result for this form of declaration.

```meowy
twice <int32> : (number <int32>) {
    -> number * 2
}

answer : twice(21)
also_answer : 21.(twice)
```

`value.(function)` passes `value` as the function's first argument. It is useful
for pipelines. An inline dispatch binds `self`:

```meowy
answer : 21.{ -> self * 2 }
```

Use `()` even when a function takes no arguments. Selecting a function field
does not execute it, and selecting a value field never recomputes it.

## Matchers are independent

```meowy
age <uint8> : 24

| age >= 18 | print("Adult")
| age >= 21 | print("At least twenty-one")
```

Both arms execute. There is no implicit `else` or first-match behavior. When you
need to select one result and finish, use a named scope and leave it explicitly:

```meowy
classify <string> : (age <uint8>) 'result {
    | age < 18 | {
        'result -> "minor"
        'result.leave()
    }

    -> "adult"
}
```

`'result ->` targets that scope's primary emission. `'result.leave()` finishes
the scope; it does not exit the whole program.

## Handle failures as values

`<uint8><error>` is a union: either an unsigned byte or an error. Matching an
error and leaving the branch lets the type checker prove the remaining value is
a byte.

```meowy
strings : @"strings"

describe <string> : (text <string>) 'result {
    age : strings.to_uint8(text)

    | age <error> | {
        'result -> "Enter a whole number from 0 to 255"
        'result.leave()
    }

    | age < 18 | {
        'result -> "Access denied"
        'result.leave()
    }

    -> "Welcome"
}

print(describe("24"))
```

Conversion creates a new typed value. Removing an error from a type expression
does not remove a possible error from a running program.

## Loops reuse named scopes

Bounded lists keep their elements inline. Their first position is **1**.

```meowy
values <int32[4]> : [10, 20, 30]
index <usize> := 1
total <int32> := 0

'loop {
    | index > values.size() | 'loop.leave()

    total = total + values[index]
    index = index + 1
    'loop.restart()
}

print(total)
```

The list has capacity four and length three. The guard handles an empty list too.
`restart()` cleans up the current iteration's locals before starting another
iteration. The surrounding `index` and `total` remain alive.

## Make costs visible

Records and bounded lists use value storage. Borrow them with `&value` to avoid
copying; use `&!value` for exclusive mutation. Dynamic storage uses an explicit
allocator. Resources are released when their owner leaves scope.

`>> computation()` starts an owned task, and `<< task` joins it. Named groups
collect a bounded number of tasks. Channels move messages between tasks and have
an explicit capacity. See the [concurrency reference](../reference/tasks-and-channels.md)
before using these operations: joining, cancellation, and closing an endpoint are
different actions.

Continue with [values and blocks](../reference/values-and-blocks.md), or inspect
the [complete programs](../programs/README.md).
