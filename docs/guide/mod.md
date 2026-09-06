# Configure a project with mod.mwy

[Documentation index](../README.md) · [Language tour](README.md)

`mod.mwy` describes a project's imports, public exports, build settings, and
optional coding-style policy. Put it at the project root so source files share
one explicit configuration.
The file uses ordinary meowy blocks and emissions; its well-known field names
are configuration values, not language keywords.

## Start with the sample

The [sample manifest](mod.sample.mwy) configures the worked packet program and
exports its decoder. From this repository's root, copy the template into place:

```sh
cp docs/guide/mod.sample.mwy mod.mwy
meowy check
meowy run
```

The template lives beside this guide, but its paths are written for the installed
root `mod.mwy`. Its `programs` alias points to `./docs/programs`, and its build
entry is `./docs/programs/main.mwy`. The `.sample.mwy` filename is not discovered
as a project manifest automatically.

For another project, use that project's source paths. A small command-line
project might have this layout:

```text
project/
  mod.mwy
  main.mwy
  cmd/
    cli/
      help/
        main.mwy
```

Its manifest can be:

```meowy
-> import : {
    -> aliases : {
        -> help : "./cmd/cli/help"
    }
}

-> build : {
    -> entry : "./main.mwy"
    -> profile : "debug"
}
```

## Give local source directories import names

`import.aliases` maps names to local directories. Each directory is relative to
the manifest, so a file can use the same import from anywhere in that project:

```meowy
help : @"help/main.mwy"
help.show()
```

That is the root `main.mwy` for the layout above. In `cmd/cli/help/main.mwy`, export
the function it calls:

```meowy
debug : @"debug"

-> show <null> : () {
    debug.print("Usage: app --help")
}
```

`@"help/main.mwy"` selects the directory named `help`, then the exact file
`main.mwy` inside it. It resolves to `cmd/cli/help/main.mwy` regardless of the
importing file's depth or the shell's working directory. The `help` binding in
the source is an ordinary local name; it need not match the import prefix.

| Import                       | Resolution                                                       |
| ---------------------------- | ---------------------------------------------------------------- |
| `@"help/main.mwy"`           | The `main.mwy` file under the directory in `import.aliases.help` |
| `@"help/topics/options.mwy"` | An explicit nested file under that directory                     |
| `@"./main.mwy"`              | A file relative to the importing source file                     |
| `@"debug"`                   | The foundational `debug` module                                  |

Use a path alias when several files need a stable name for part of the same
source tree. Relative imports remain useful for nearby companion files. Both
forms identify the same module when they resolve to the same canonical file;
using an alias does not initialize it again.

Alias targets are compile-time strings naming directories. Include the filename
and `.mwy` extension in the import: `@"help"` does not guess `main.mwy` or load
every file in the directory. The prefix is the complete first path component,
so `helpful/main.mwy` does not match `help`. A suffix cannot escape its mapped
directory through `..` or a symlink.

Names must be unique across local path aliases and package dependency names.
They cannot replace foundational names such as `debug` or `core`. Aliases are
scoped to their declaring project; a dependency resolves its own aliases from
its own manifest. The [import reference](../reference/modules-and-ffi.md#local-path-aliases)
defines normalization, collisions, and failure behavior.

## Put package dependencies beside aliases

A directory prefix addresses source files. A package dependency selects another
project and imports its public facade. Keep both under `import`:

```meowy
-> import : {
    -> aliases : {
        -> help : "./cmd/cli/help"
    }

    -> geometry : {
        -> path : "../geometry"
    }
}
```

This example assumes a sibling package at `../geometry` with its own `mod.mwy`.
`@"geometry"` imports that package's `export` facade; `@"help/main.mwy"` imports a
specific local source file. The `aliases` field holds the path map, so it cannot
also hold a package dependency record at that same level.

For a remote package, replace the `path` record with `fetch` and exactly one
revision selector:

| Field               | What to provide                                        |
| ------------------- | ------------------------------------------------------ |
| `path`              | A local package directory relative to this manifest    |
| `fetch` with `hash` | A remote source and a full immutable commit identifier |
| `fetch` with `tag`  | A remote source and a release name to resolve          |
| `fetch` with `ref`  | A remote source and a branch/reference to resolve      |

Use `meowy deps resolve` after adding a remote dependency. Use
`meowy deps update geometry` when deliberately changing a locked selection.
Ordinary checks and builds do not advance revisions or rewrite `mod.lock`.
Local source aliases require no remote lock entry, and foundational modules need
no dependency declaration. Captures and reproducible builds still record the
exact local source files reached through aliases.

## Choose the public exports

The manifest's `export` block describes the facade seen by other packages. For
example, a library exposing the help module could add:

```meowy
-> export : {
    -> help : @"help/main.mwy"
}
```

The importing package can then access the facade's `help` field. The directory
alias stays local to the library; it is not copied into the caller's import map.
An alias makes a file addressable but does not export its private bindings.
Imports in this facade use the same alias table as the project's source files.

Use named emissions to export values and types. Keep construction helpers and
private state as ordinary bindings in source modules. The entry file must remain
separate from imported modules, so do not re-export the same file selected by
`build.entry`.

## Select the entry and runtime settings

`build.entry` is a real file path relative to the manifest. Import prefixes apply
to `@"..."` operands; they do not rewrite `build.entry`, dependency `path`, or
native artifact paths. For the small project above, use `"./main.mwy"` even though
its imports use `help/...`.

| Build field | When to set it                                                       |
| ----------- | -------------------------------------------------------------------- |
| `entry`     | Select the file executed by `meowy run` or compiled by `meowy build` |
| `profile`   | Choose `"debug"` while developing or `"release"` for optimized code  |
| `target`    | Select a different architecture, operating system, and ABI           |
| `native`    | Declare exact native link artifacts required by foreign calls        |
| `executor`  | Supply runtime storage and workers when the entry starts tasks       |

Both profiles preserve overflow checks, bounds checks, ownership, and cleanup.
The CLI can override the entry, profile, or target for one invocation without
rewriting the manifest. See [entry selection](../cli/README.md#projects-and-entry-selection).

For a task-using entry, add an executor inside `build`:

```meowy
-> executor : {
    -> workers : 2
    -> max_tasks : 64
    -> stack_bytes : 65_536
    -> allocator : "system"
}
```

Worker count, admitted task count, and per-task stack storage are separate
budgets. A channel's queue capacity or a group's result capacity does not choose
them. The [runtime configuration reference](../reference/modules-and-ffi.md#build-settings)
defines their bounds and failure behavior.

## Keep configuration predictable

The recognized top-level emissions are `import`, `export`, `build`, and `gatostyle`.
`aliases` is an optional table inside `import`; omitting it means there are no
local path prefixes. An empty import block is still valid when only foundational
and relative imports are needed.

Manifest expressions may combine literals, immutable bindings, and pure
compile-time helpers. They cannot run the application, inspect ambient shell
variables, access the network, or perform I/O. Dependency fetching is an explicit
CLI operation over declared inputs, not an effect of evaluating a configuration
block. Resolving imports in `export` describes the module graph without executing
module initialization.

An invalid alias table is a configuration error (`E505`); a missing source file
is an import-resolution error (`E501`); a name that shadows a foundational module
is `E508`. Diagnostics identify the alias declaration and the import that used
it. Use `meowy err explain E505` for the rule or `meowy err explain 1` for the
saved occurrence and its source context.

## Define the project's coding style

Use `gatostyle` for layout, preferred expression forms, and code-quality checks:

```meowy
-> gatostyle : {
    -> preset : "structured"
    -> rules : {
        -> call_form : {
            -> level : "warning"
            -> prefer : "dispatch"
            -> fix : "safe"
        }
    }
}
```

This policy prefers first-argument dispatch and permits rewrites when their
evaluation and ownership behavior is proven equivalent. `meowy style check`
reports findings; `meowy style fix --diff` previews eligible edits. `meowy fmt`
uses only the layout settings. The [gatostyle guide](gatostyle.md) defines every
option, custom project rules, scoped exceptions, and the repair contract.

Omitting the block selects the structured preset. Use `preset : "none"` to
start with all style checks disabled. The sample excludes
`./editor/nvim/tests/fixtures` because those fixtures intentionally contain
invalid source for highlighting tests; exclusions do not affect language checks
or the editor's regression script.
