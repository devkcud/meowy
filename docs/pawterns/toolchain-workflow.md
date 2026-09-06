# Make the tools pull their weight

[Pawterns](README.md) · [First project](first-project.md) · [Lean binaries](lean-binaries.md)

The ecosystem gets useful when it can answer “why?” without making you recreate
Tuesday's terminal session from memory. Give the editor a project, give gatostyle
your preferences, and keep a failure around long enough to interrogate it.

## Let the editor read the same project you do

Start in a project with a `mod.mwy` and `main.mwy`, such as
[the greeting](first-project.md). Add this block to its manifest. If `lsp` already
exists, merge the fields into that block; duplicate emissions are not overrides.

```meowy
-> lsp : {
    -> version : 1
    -> check : {
        -> trigger : "save"
        -> scope : "project"
    }
    -> format_on_save : false
}
```

Ask the command line what it sees:

```sh
meowy --version
meowy lsp config --resolved main.mwy
meowy lsp doctor main.mwy
```

The config command prints effective JSON. The doctor describes readiness and
missing inputs without executing the application. Configure your editor's LSP
client to launch the selected `meowy` executable with arguments `["lsp"]`, use
stdio, and attach language ID `meowy` to `.mwy` files.

Don't launch a second copy in your terminal expecting a chat prompt. That process
is waiting for protocol messages; your editor is supposed to do the talking.
The [Vim/Neovim runtime](../../editor/nvim/README.md) supplies file detection and
highlighting, with the LSP client configured separately.

With this policy, saving triggers project checking. Hover and completion can
still compute facts for the current buffer before a save. The disk-based config
and doctor commands cannot see those unsaved edits, which explains many apparent
“the editor and terminal are fighting” moments.

Keep analysis settings in `mod.mwy`. A team can additionally set `lsp.toolchain`
to its exact distribution version string; schema `version : 1` is a different
number with a different job. A pin mismatch stops semantic answers and edits for
that project. It does not install or switch the compiler used by your shell.
See [LSP compatibility](../reference/lsp.md#versions-that-must-agree).

## Give gatostyle your taste, then inspect the edits

You like pipelines. Your colleague likes naming every intermediate value. Both
can write valid meowy; your project can pick a house style and explain it.

Add this manifest block, or merge it into an existing `gatostyle` block:

```meowy
-> gatostyle : {
    -> preset : "structured"
    -> rules : {
        -> call_form : {
            -> level : "warning"
            -> prefer : "dispatch"
            -> fix : "safe"
        }
        -> bindings : {
            -> level : "hint"
            -> prefer : "named"
            -> fix : "manual"
        }
    }
}
```

From the project directory:

```sh
meowy style config --resolved main.mwy
meowy style check
meowy style fix --diff
```

The resolved configuration shows which preferences apply. The diff previews
eligible edits; it does not write them. After reading it, apply with
`meowy style fix`, then run `meowy check`.

For the pure scalar helpers in [composition](composition.md), a nested call may
be eligible to become a dispatch chain. When rewriting would change borrowing,
cleanup, evaluation, or task boundaries, the tool needs proof. “But it looks nicer”
is an excellent style opinion and a terrible ownership proof.

Use `meowy fmt --diff` when the job is just layout. `meowy fmt` writes that layout;
it does not apply semantic call-form rewrites. A `style check` warning can make CI
fail without making the source an invalid program. The
[gatostyle guide](../guide/gatostyle.md) defines that distinction and all the knobs,
including the spacebar-on-holiday `minimal` preset.

## Inspect and replay a deliberate failure

Let's break something small enough to understand. In a disposable greeting
project, create this complete `failure.mwy` beside `mod.mwy`:

```meowy
debug : @"debug"

debug.panic("The greeting has filed a complaint")
```

**This program deliberately panics.** Run it as an explicit entry:

```sh
meowy run failure.mwy
meowy err summary --entry failure.mwy
```

The failure is `P006`, the code for an explicit `debug.panic`. The summary gives
the session, occurrence, and capture status. If it lists this as occurrence `1`
with a complete capsule, inspect and replay it:

```sh
meowy err explain 1 --entry failure.mwy
meowy err inspect 1 --entry failure.mwy --verbose
meowy err reproduce 1 --entry failure.mwy --verbose
```

`inspect` reads preserved evidence. `reproduce` re-enters the failing phase with
the captured inputs. Successful reproduction exits successfully because it matched
the saved failure, even though the reproduced program itself panicked. That is
one of the few times “it broke again” is the result you were hoping for.

To carry this particular failure elsewhere, choose a fresh output filename:

```sh
meowy err export 1 --entry failure.mwy --output greeting-failure.replay
./greeting-failure.replay --inspect
./greeting-failure.replay --verbose
```

The executable capsule carries its recorded sources and tools. Its host
requirements and capture limits still apply; export does not repair missing
inputs. The ordinary program and this diagnostic bundle have different storage
budgets.

Keep the summary's session ID if you will run another check or program before
returning to this failure. Every completed check/build/run advances that entry's
selection, including successful runs; `--session ID` selects the saved one
explicitly. For source errors with repair candidates, preview them with
`meowy err fix all --entry PATH --session ID --diff --recommended`, replacing
`PATH` and `ID` with that entry and its captured session, before applying an edit.
A deliberately requested panic need not have an automatic fix.

See [saved sessions](../cli/README.md#saved-sessions),
[replay fidelity](../reference/diagnostics.md#replay-fidelity), and
[concurrency debugging](deadlocks-and-shutdown.md). Printing an ordinary error
value does not create one of these diagnostic occurrences.

## Put the boring checks on repeat

Your future changes deserve the same questions every time: does the code obey
the language, does it follow the project's style, and do the chosen cases still
behave the same way?

For a project containing the standard suite from
[composition](composition.md#give-a-pure-helper-a-small-regression-driver), save
this as `ci.sh` at the project root:

```sh
#!/bin/sh
set -eu

meowy check --offline --color never --quiet
meowy style check --offline --color never --quiet
meowy test --offline --color never --quiet
```

Run `sh ci.sh` from that directory. `set -e` stops this sequence at its first
failed command. The first check covers the application entry; `test` checks the
discovered suite graphs, builds their harness, and runs the selected cases. It
does not execute the application entry. A file merely sitting in the directory
is not automatically part of every checking graph.

Prepare the selected toolchain, target inputs, and locked dependency content
before this sequence. Offline mode requires those inputs locally and never means
“quietly skip the missing dependency.” Resolve additions or update revisions in
an explicit dependency-management step, not in the middle of a source check.

The suite above has fixed inputs and no external data effects. Other suites may
create files, open sockets, or wait for tasks; configure their process count,
watchdog, output limit, and seed in `mod.mwy`. `--offline` constrains tool
dependency access, not network calls made by a case. A fresh case process does
not give that case a private filesystem or network.

Use `meowy test --no-run` for a separate build-only check, such as a target that
the CI host cannot execute. It does not replace a run on a compatible host.
Keep `test.allow_empty` false when the job is supposed to exercise cases: an
accidentally empty selection should fail visibly. Skipped cases remain listed as
skipped; a green status does not make them tested. See
[testing Pawterns](testing.md) for fixtures and failure inspection.
