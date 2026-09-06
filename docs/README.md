# meowy documentation

This documentation defines the language's semantics and the foundational library
contracts used in its programs. The reference is authoritative; the guide teaches
those rules by building small programs, and the design notes explain their intent.

## Learn the language

1. Read the [guide](guide/README.md) for bindings, functions, blocks, and loops.
2. Read [values and blocks](reference/values-and-blocks.md) to understand what
   emissions do, when code executes, and how primary values compose with fields.
3. Read [types](reference/types.md) and [collections](reference/collections.md)
   before defining public data structures.
4. Read [memory](reference/memory.md) for ownership and predictable storage.
5. Read [tasks and channels](reference/tasks-and-channels.md) before introducing
   concurrent work.

## Look something up

| Question                                              | Reference                                                         |
| ----------------------------------------------------- | ----------------------------------------------------------------- |
| What does this punctuation mean?                      | [Syntax](reference/syntax.md)                                     |
| Does `->` return? Does `{ ... }` create a function?   | [Values and blocks](reference/values-and-blocks.md)               |
| Can a mutable binding change type?                    | [Types](reference/types.md)                                       |
| How do errors and nullable values narrow?             | [Types](reference/types.md#unions-and-narrowing)                  |
| Where does memory come from, and when is it released? | [Memory](reference/memory.md)                                     |
| Is `T[4]` an array or a capacity limit?               | [Collections](reference/collections.md)                           |
| Who joins a task? What does cancellation guarantee?   | [Tasks and channels](reference/tasks-and-channels.md)             |
| How are native data and functions declared?           | [Modules and FFI](reference/modules-and-ffi.md#native-interfaces) |
| What APIs do the snippets rely on?                    | [Library contracts](reference/standard-library.md)                |
| Which failures are values?                            | [Diagnostics](reference/diagnostics.md)                           |

## Read complete programs

The [worked programs](programs/README.md) cover validation, allocation-free packet
decoding, ordered task results, and a bounded producer/consumer channel. They are
part of the documentation, with explanations of their data and failure paths.

[Design notes](design.md) record the language's main decisions.
[The manifest](../mod.sample.mwy) describes the project configuration vocabulary.

## Conventions

Code fences marked `meowy` use language syntax. A snippet explicitly labeled
**invalid** demonstrates a diagnostic. API tables describe contracts; angle
brackets in prose denote types, not command placeholders.

Reference rules use **must** for requirements and **may** for permitted choices.
Target-dependent properties, such as pointer width and C layout, must be supplied
by the selected build target. No sample module version or performance measurement
is implied by a code listing.

The documentation uses four-space indentation and omits semicolons at line ends.
Formatting never rewrites evaluation order or merges bindings.
