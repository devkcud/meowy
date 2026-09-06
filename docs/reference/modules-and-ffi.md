# Modules, configuration, and native interfaces

[Documentation index](../README.md)

## Files and exports

A file is a module body. Ordinary bindings are private; named emissions export
values or types. A file's primary emission is its primary exported value.

```meowy
-> <Point> : <{
    x <int32>
    y <int32>
}>

-> origin <Point> : {
    -> x : 0
    -> y : 0
}

-> translate_x <Point> : (point <Point>, distance <int32>) {
    -> x : point.x + distance
    -> y : point.y
}
```

Public function parameters and results are annotated. Types use `-> <Name> :`
to export into the type namespace. Imported modules cannot mutate another
module's private state through field selection.

```meowy
geometry : @"./geometry.mwy"
point <geometry.Point> : geometry.translate_x(geometry.origin, 10)
```

Relative imports resolve from the importing file, not the shell's working
directory. `@"debug"` resolves a foundational module, and `@"geometry"` can resolve
a package dependency declared in the containing project's manifest.
`@"help/main.mwy"` resolves a source file through a local path alias. Foundational
names cannot be shadowed by either kind of import alias. Import operands must be
string literals; runtime strings cannot load executable modules.

Canonical module identity includes its resolved package revision and file path.
Each module initializes once per program, after its imported modules, with sibling
imports initialized in source order. Repeated imports share the resulting module
value. Import cycles are rejected, including cycles through re-exports, so a
partially initialized module is never observable.

Exports live for the program lifetime and cannot borrow temporary initialization
locals. Copyable exports can be read by value; owned resource exports are borrowed
from module storage and cannot be moved out through an import. A module should
export constructors instead of a globally mutable resource. If initialization
panics, already initialized modules are released in reverse order and startup
fails before the entry file executes.

## Local path aliases

`import.aliases` is an optional record of names mapped to local source directories:

```meowy
-> import : {
    -> aliases : {
        -> help : "./cmd/cli/help"
    }
}
```

An alias name is a case-sensitive identifier. Its value must be a compile-time
string containing a relative directory path, resolved from the declaring
manifest's directory. Absolute paths, environment expansion, and remote URLs are
not alias targets. `../shared` may explicitly select a sibling source directory;
the files used from it become declared local build inputs.

Resolution follows these rules:

1. An import beginning with `./` or `../` remains relative to the importing file.
2. A bare name selects a foundational module or the public facade of a declared
   package dependency.
3. For `prefix/path.mwy`, the complete first component must name an entry in
   `import.aliases`. Append the suffix to that entry's directory and resolve the
   exact `.mwy` file. There is no fallback to the working directory or to a
   package's internal files.

For example, `@"help/main.mwy"` resolves to `./cmd/cli/help/main.mwy` relative to
the manifest. `@"help/topics/options.mwy"` selects a nested source file.
`@"help"` is an error when `help` is only a directory alias: there is no implicit
entry filename or directory import. Alias roots and suffixes are file paths,
not recursive alias substitutions; an alias target cannot refer to another alias.

Paths use `/` separators. Normalize `.` and `..` components, then canonicalize
filesystem paths. The suffix must remain inside the mapped root, including after
resolving symlinks; an escaping suffix is an import-resolution error. A missing
root, missing file, or directory where a file is required is also a resolution
error, with the alias declaration and import site attached as diagnostic spans.
The resolver does not guess extensions or search for a `main.mwy` file.

Alias names must be unique across `import.aliases` and the package dependency
entries beside it. A collision is a configuration error; no declaration order
or longest-prefix rule chooses a winner. Foundational names are unavailable in
either table. The outer `import.aliases` field is the path map, so it cannot also
be a dependency entry. These are configuration-schema rules, not reserved source
keywords.

Aliases belong to their declaring project. Imported packages resolve their own
imports with their own manifests and do not inherit the caller's alias map.
The manifest's `export` facade may use its project's path aliases too. The import
configuration is established before resolving facade imports; changing the order
of top-level manifest fields does not change that configuration.

Aliases change the spelling used to reach a module, not its canonical identity,
initialization count, or visibility. Two aliases, or an alias and a relative
import, reaching the same canonical module share its value. Private bindings stay
private. Build records and replay capsules include the exact referenced files
and their digests even when an alias root lies outside the project directory.

## The project manifest

`mod.mwy` is a restricted compile-time block. Its well-known emissions are:

| Name     | Contract                                                       |
| -------- | -------------------------------------------------------------- |
| `import` | Package dependency records and the optional `aliases` path map |
| `export` | Public package facade, including re-exports                    |
| `build`  | Entry file, profile, optional target, and native link inputs   |

Unknown top-level configuration names are diagnostics. Author/version metadata
can be ordinary exported fields if a package wants to expose it; it does not
participate in dependency resolution.

The manifest can combine literals, immutable bindings, and pure compile-time
helper functions. It cannot read the network, run shell commands, inspect ambient
environment variables, spawn tasks, or perform application I/O during evaluation.
Imports in an `export` facade describe the module graph; they do not execute
application initialization during manifest evaluation.

There is no flag that makes a manifest arbitrarily executable. Build inputs must
be declared so dependency resolution and code generation do not depend on hidden
machine state.

See the [sample manifest](../../mod.sample.mwy), which points to the documented
packet module and its companion entry file. The
[CLI guide](../cli/README.md#projects-and-entry-selection) defines manifest
discovery, entry overrides, and command-line build options.

## Dependency resolution

A remote dependency record contains a `fetch` source string and exactly one of:

- `hash`: a full immutable commit identifier;
- `tag`: a named release, resolved to an immutable revision;
- `ref`: a branch or reference, also resolved to an immutable revision.

A local dependency instead has `path`, naming a package directory relative to the
manifest, and cannot also have a remote selector. That package supplies its own
`mod.mwy` and public `export` facade. Package records live directly under `import`,
beside the optional `aliases` table. Dependencies are imported through their
declared names; source URLs are not runtime module expressions.

`import.aliases` maps local source directories rather than selecting package
revisions. Its entries require no remote lock entries and are not processed as
dependencies by `deps resolve` or `deps update`. An import with a path suffix uses
that local path map; declaring a package name alone does not expose arbitrary
subpaths of its package source.

`mod.lock` records each remote source's canonical identity, exact revision,
content digest, transitive dependencies, and selected foundational-library
version. The target and compiler identity are separate recorded build inputs.
Moving a tag or branch never silently changes an already locked build. A locked
build fails when source content differs from its digest, a required lock entry is
missing, or a manifest selector conflicts with the lock. Resolution is an explicit
dependency-update operation; ordinary compilation does not rewrite the lock.
Use [dependency commands](../cli/README.md#profiles-targets-and-dependencies) to
resolve missing entries or deliberately update selected revisions.

Native libraries and generated bindings are also declared inputs. Link search
paths cannot silently select an unrelated system library with the same filename.
Local path dependencies are suitable for development; reproducible distribution
must include their exact contents and digest.

## Build settings

`build.entry` is a relative source path. `build.profile` is `"debug"` or
`"release"`; both profiles preserve overflow checks, bounds checks, ownership,
and task cleanup. `build.target` identifies the architecture, operating system,
and ABI when cross-compiling. Omitting it selects the host target, recorded as a
build input. `build.native` lists explicitly selected native link artifacts.

The entry file executes after module initialization. It must not also be imported
as a module. Its primary result is `<null>` for success or `<int32>` for the
process status; a program's top-level recoverable error must be handled explicitly.
An uncaught panic terminates with a nonzero status.

Standalone programs that start tasks declare `build.executor`. For example,
inside the `build` block:

```meowy
-> executor : {
    -> workers : 2
    -> max_tasks : 64
    -> stack_bytes : 65_536
    -> allocator : "system"
}
```

`workers` is a positive worker count, `max_tasks` is a positive limit on admitted
but unjoined children, and `stack_bytes` is the per-task stack budget. Queued and
settled-but-unjoined children count against admission. `allocator` explicitly
selects `"system"` storage for standalone task stacks and scheduler state.
Invalid sizes are configuration errors. Failure to initialize the executor fails
startup; later task admission failure produces `tasks.SpawnFailed`.

An embedded host can instead supply an executor and allocator through its runtime
boundary. No standalone executor is selected implicitly by a task operator.
Code that cannot prove an executor is present must not start tasks. Runtime waits
suspend the waiting task so other runnable tasks can use the workers; a blocking
foreign call can occupy a worker until it returns.

## Native interfaces

An ordinary meowy record has no stable C layout. The compile-time function
`ffi.record` constructs a native record type from a convention and an ordered
list of named field types:

```meowy
ffi : @"ffi"

<Position> : ffi.record("C", [
    "x" : <float32>,
    "y" : <float32>
])
```

`ffi.record` is an ordinary named intrinsic, not a declaration modifier. It can
be aliased like other compile-time functions; the alias retains the constructor's
rules. Its arguments and result must be known at compile time. Each list element
has one unique literal name. List position determines native field order, so
layout does not depend on an ordinary structural record type's declaration order.

C-layout records preserve that explicit field order and use the target ABI's
padding and alignment. They cannot contain a meowy primary emission, expanded fields,
references, tasks, channels, erased values, or ordinary meowy unions. Use fixed
arrays for C arrays and raw pointers for C pointer fields. A C-layout wrapper is
nominal; a structurally similar ordinary record is not ABI-compatible.

`ffi.c_int`, `ffi.c_uint`, and related aliases follow the selected C target, rather
than assuming a C integer always has a particular width. `usize` is used only
where the foreign contract really specifies a pointer-sized unsigned value.

Foreign symbols are declared with `ffi.extern<Signature>(convention, symbol)`.
The result is a function pointer with a `!` safety requirement; calling it requires
a `!{ ... }` block.
For example, a native library can expose a pointer-and-length entry point:

```meowy
ffi : @"ffi"

consume_bytes : ffi.extern<(*uint8, usize) -> usize>("C", "consume_bytes")
```

This declaration alone does not prove the function's contract. A binding must
document which pointers are retained, whether null is permitted, who frees
returned storage, whether a call blocks, and whether callbacks or arguments are
thread-affine. Wrappers should convert those requirements into safe owners,
borrows, slices, and error unions wherever possible.

Do not pass a borrowed `<string>` as a C string. Construct a checked NUL-terminated
buffer with explicit storage and keep it alive for the required duration. Do not
pass an ordinary bounded list as a C array: its length metadata is not an element.

Unwinding across a foreign frame is forbidden. Callbacks catch task panics at a
meowy boundary and translate them into the foreign protocol. Variadic functions
require a fixed-signature wrapper. Reinterpreting native bytes as a record is
unsafe; decoding fields from a byte slice is the portable alternative shown in
the [packet program](../programs/packet.mwy).

## Larger integrations

A graphics binding can accept a meowy options record, fill nullable fields with
documented defaults, and explicitly lower it to native parameters. Window and
graphics-context owners carry thread-affinity constraints; their presence in a
record does not make them transferable to workers.

A database binding can expose connection and row owners, typed query errors, and
borrowed field views whose lifetime is bounded by the result set. Queries bind
parameters separately from SQL text. Nullable columns remain nullable until
checked; converting every row into an untyped record would lose these guarantees.
Long-running queries integrate with task cancellation at the driver's documented
I/O boundary. These libraries build on the language's ownership and error rules
without adding a new form of block or task.
