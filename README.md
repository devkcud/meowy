# meowy

**meowy** is a small, composable systems programming language built around
values, blocks, emissions, dispatch, and matching.

Its guiding principle is **what you write is what you get**: data has a concrete
representation, allocation has a visible cause, and concurrent work has an owner.
The same block vocabulary describes a calculation, a record, a module, or a
function body.

The grammar is punctuation-based and has no reserved keywords. `true`, `false`,
`null`, types, and contextual names such as `self` are well-known values and
bindings. See [names and syntax](docs/reference/syntax.md#no-keywords).

```meowy
<Reading> : <{
    sensor <uint16>
    value <int32>
}>

reading <Reading> : {
    -> sensor : 7
    -> value : 24
}

double <int32> : (value <int32>) {
    -> value * 2
}

sample : {
    -> reading.value.(double)
    -> sensor : reading.sensor
}

sample         # primary value: 48 #
sample.sensor  # named value: 7 #
```

Emitting does not end execution. A block produces at most one primary value and
one value for each emitted name on any execution path. Its fields have a static
shape; composing blocks does not require a dynamic object table.

## Documentation

Start with the [guide](docs/guide/README.md), or use the
[documentation index](docs/README.md) to browse the full reference.

| Topic                                                      | What it covers                                                         |
| ---------------------------------------------------------- | ---------------------------------------------------------------------- |
| [Syntax](docs/reference/syntax.md)                         | Bindings, literals, operators, functions, and scopes                   |
| [Values and blocks](docs/reference/values-and-blocks.md)   | Evaluation, emissions, dispatch, matching, and control flow            |
| [Types](docs/reference/types.md)                           | Inference, unions, narrowing, records, generics, and conversions       |
| [Memory](docs/reference/memory.md)                         | Ownership, borrowing, allocation, raw pointers, and cleanup            |
| [Collections](docs/reference/collections.md)               | Bounded lists, arrays, slices, vectors, and maps                       |
| [Tasks and channels](docs/reference/tasks-and-channels.md) | Structured concurrency, deadlines, cancellation, and communication     |
| [Modules and FFI](docs/reference/modules-and-ffi.md)       | Imports, exports, reproducible dependencies, and native boundaries     |
| [Library contracts](docs/reference/standard-library.md)    | The foundational APIs used throughout the documentation                |
| [Diagnostics](docs/reference/diagnostics.md)               | Labeled errors, ranked repairs, replay capsules, and failure behavior  |
| [Diagnostic codes](docs/reference/diagnostic-codes.md)     | Rule catalog, required evidence, and repair guidance                   |
| [Command line](docs/cli/README.md)                         | Check, build, run, inspect internals, apply fixes, and replay failures |

The [worked programs](docs/programs/README.md) put these rules together. The
[design notes](docs/design.md) explain the choices and the boundaries of the core.

## Project files

- `.mwy` files contain meowy source.
- `mod.mwy` describes imports, local path aliases, exports, and build settings.
  See the [manifest guide](docs/guide/mod.md) and its
  [sample](docs/guide/mod.sample.mwy).
- [Vim and Neovim support](editor/nvim/README.md) provides syntax highlighting,
  file detection, and buffer settings.

Inspired by Smalltalk, OCaml, Rust, Go, and Lisp.
