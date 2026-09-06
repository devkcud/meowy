# Your first project

[Pawterns](README.md) · Next: [Compose values](composition.md)

Let's get a greeting on screen before it develops a dependency graph. Then we'll
give it a helper module and a package, because apparently saying hello is a
growth industry. Each step has its own complete file set.

## Print a greeting from a project

Give the greeting a home. A tiny manifest tells the command line and editor where
the entry lives, saving you from typing its filename for the rest of the afternoon.

Create a directory and work inside it:

```sh
mkdir hello-paw
cd hello-paw
```

Create `mod.mwy`:

```meowy
-> build : {
    -> entry : "./main.mwy"
    -> profile : "debug"
}
```

Create `main.mwy`:

```meowy
debug : @"debug"

debug.print("Hello, meowy!")
```

Then run:

```sh
meowy --version
meowy check
meowy run
```

The program prints `Hello, meowy!`. `check` validates the source graph; `run`
builds it and executes only after a successful build. The entry has no primary
emission, so it finishes with `<null>` and a successful process status.

`@"debug"` names a foundational module. It needs no dependency installation or
`import` entry in the manifest. `debug` is your local binding to that module;
`print` is an ordinary function selected from it. Even the manifest uses blocks
and emissions rather than a separate configuration language.

The greeting borrows static literal bytes. It needs no task executor and no
application-owned text buffer. `debug.print` is convenient diagnostic output;
an output failure panics. Use [a supplied writer](cli-and-files.md) when output
failure should be a recoverable application result.

**Try it:** bind `name : "friend"` and print `"Hello, {name}!"`. Interpolation
at `debug.print` streams into output. It does not require constructing an owned
intermediate string.

See [the language tour](../guide/README.md),
[entry selection](../cli/README.md#projects-and-entry-selection), and
[diagnostic output](../reference/stdlib/core.md#output-and-text).

## Move reusable work into a module

Your greeting has earned a second file. Keep the entry in charge of what happens
and put the reusable operation in a helper. Extend the first project to this layout:

```text
hello-paw/
  mod.mwy
  main.mwy
  greetings/
    hello.mwy
```

Replace `mod.mwy` with:

```meowy
-> import : {
    -> aliases : {
        -> greetings : "./greetings"
    }
}

-> build : {
    -> entry : "./main.mwy"
    -> profile : "debug"
}
```

Create `greetings/hello.mwy`:

```meowy
debug : @"debug"

-> show <null> : (name <string>) {
    debug.print("Hello, {name}!")
}
```

Replace `main.mwy` with:

```meowy
greeting : @"greetings/hello.mwy"

greeting.show("module")
```

From `hello-paw`, `meowy check` validates the graph and `meowy run` prints
`Hello, module!`.
The helper exports its function through `->`; its private `debug` binding stays
inside the helper. Exported functions annotate both their parameters and result.

First little trap: an alias selects a directory; the import still supplies a filename.
`@"greetings"` would ask for a package or foundational module, not guess a file
in this directory. `@"./greetings/hello.mwy"` is also valid from this entry and
identifies the same module. An alias does not create a second initialized copy.

Keep the greeting inside `show`; importing a helper should not make it start
talking. Module initialization happens when the program initializes its imports; selecting
`greeting.show` alone does not invoke it. The borrowed `name` remains valid
through the call and is not retained by the helper.

See [local path aliases](../reference/modules-and-ffi.md#local-path-aliases)
and the [packet project](../programs/packet/README.md) for a larger example.

## Share a helper through a package facade

Another application wants your greeting. Fame comes quickly. Give both callers
a package facade so they can share the helper through one public API. This recipe
is a separate two-project layout, with each manifest shown in full:

```text
workspace/
  greetings/
    mod.mwy
    lib.mwy
  hello-paw/
    mod.mwy
    main.mwy
```

In `greetings/lib.mwy`:

```meowy
debug : @"debug"

-> show <null> : (name <string>) {
    debug.print("Hello, {name}!")
}
```

In `greetings/mod.mwy`:

```meowy
-> export : {
    -> show : @"./lib.mwy".show
}
```

In `hello-paw/mod.mwy`:

```meowy
-> import : {
    -> greetings : {
        -> path : "../greetings"
    }
}

-> build : {
    -> entry : "./main.mwy"
}
```

In `hello-paw/main.mwy`:

```meowy
greetings : @"greetings"

greetings.show("neighbor")
```

Run `meowy check` and `meowy run` from `workspace/hello-paw`. The greeting is
`Hello, neighbor!`. The library needs no program entry: its manifest exposes a
facade, and the consuming application supplies execution.

The relative `path` names the sibling package, while `export` controls its public
surface. This is different from a directory alias into the application's own
sources. Changing local package files changes the consumer's build inputs;
recording their digests is not a substitute for retaining the source revision.
For remote packages, declare a real source and revision selector and resolve it
deliberately as described in [the manifest guide](../guide/mod.md#put-package-dependencies-beside-aliases).

Neither a facade nor an import asks the linker to retain every public helper.
Reachable operations and observable initialization determine the executable's
contents. Continue with [lean binaries](lean-binaries.md) when that becomes a
deployment concern.
