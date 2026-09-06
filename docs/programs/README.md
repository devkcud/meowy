# Worked projects

[Documentation index](../README.md) · [Standard library](../reference/stdlib/README.md)

Each directory is a self-contained meowy project with its own `mod.mwy`, entry,
source files, and README. There is no shared manifest to copy or edit before
switching examples. Foundational imports need no package installation, and local
helper modules stay inside their project.

## Pick a project

| Project | What it combines | Inputs and effects |
| --- | --- | --- |
| [Age validation](age/README.md) | Typed policy module, decimal parsing, union narrowing, named exits | Four fixed inputs; diagnostic output |
| [Block composition](composition/README.md) | Primary values, exact record shapes, dispatch, matcher/ascription contexts | Fixed sensor reading; no allocation |
| [Packet decoder](packet/README.md) | Import aliases, package exports, borrowed wire bytes, explicit widening/shifts | Valid and truncated inline headers |
| [Unicode text lab](text-lab/README.md) | Bytes, scalars, graphemes, normalization, borrowed output buffers | Fixed Unicode text; no allocator |
| [Word count](word-count/README.md) | Text cursors, explicit map hashing/equality, mutation and borrow lifetimes | Fixed text; allocator-backed map |
| [JSON report](json-report/README.md) | Typed JSON, validation, owned decoded storage, buffered encoding | Fixed JSON input; explicit decoding allocator |
| [Seeded rolls](seeded-rolls/README.md) | Reproducible randomness, numeric conversion, bounded histogram | Fixed seed; inline counters |
| [Ordered tasks](tasks/README.md) | Bounded groups, module callables, ordered outcomes, deadlines | Explicit executor; three child tasks |
| [Bounded channel](channel/README.md) | Endpoint moves, backpressure, closure, partial-result handling | Explicit queue allocation and executor |
| [Calendar CLI](calendar-cli/README.md) | Typed options, generated help, calendar conversion, writer errors | Argv; six calendar profiles |

Start with age, composition, and packet for the language model. Text lab, word
count, JSON report, and seeded rolls explore text, data, ownership, and storage.
Tasks and channel make execution boundaries visible; calendar CLI combines
several modules into an application with explicit process statuses.

## Run one project

From the repository root:

```sh
cd docs/programs/calendar-cli
meowy check
meowy run -- --calendar chinese 2026-02-17
meowy run -- --help
```

Or select an entry from the repository root:

```sh
meowy run docs/programs/calendar-cli/main.mwy -- -c chinese 2026-02-17
```

The CLI finds the nearest `mod.mwy` from that entry. It loads that project's
aliases, profile, and executor settings. The application's working directory
remains the shell's working directory: a file argument such as `sample.txt` is
resolved there.

Each README provides commands, expected behavior, failure cases, and storage
costs. Task outcomes can depend on admission, deadlines, and scheduling.

## Build, inspect, and experiment

Inside any project:

```sh
meowy build --profile release
meowy style check
```

Build output goes under that project's `build/` directory. Task projects include
explicit executor budgets in their own manifests; changing a group's capacity
does not change the worker count or task admission limit. Packet also exposes its
decoder through its manifest's `export` facade, with the entry kept separate.

To capture scheduling and supported external inputs while exploring a failure,
use `meowy run --record-replay`, then the [diagnostic workflow](../cli/README.md#reproduce-a-failure).
A successfully handled application error and a captured runtime panic have
different meanings; the project README states which result to expect.

Keep experiments in the chosen project. Copying one directory carries its manifest,
helpers, and fixtures with it; another worked project is never a hidden dependency.
The [manifest guide](../guide/mod.md) explains how to change entry paths, local
aliases, public exports, and coding-style policy for a project of your own.
