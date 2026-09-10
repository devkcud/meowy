# Relative file modules

The driver supports a bounded executable slice of
[the module contract](../docs/reference/modules-and-ffi.md). A top-level immutable,
unannotated binding may import an exact relative `.mwy` file:

```meowy
settings:@"./settings.mwy"
d:@"debug"
d.print(settings.port)
```

```meowy
private:7
->port:8080
->label:"local"
```

The second file exports only `port` and `label`. Ordinary bindings remain private.
A primary emission exports the module's primary value. Exported values may be
immutable reference-free scalars, records or bounded lists; no mutable descendants,
references, unions, owning values or foundational resource values are enabled.
A module alias is a compile-time identity, so aliasing or importing it again does
not rerun initialization or create another module-storage binding. Reading an
exported Copy value retains normal value-copy semantics.

## Resolution and initialization

Relative paths resolve from the importing file, with exact extensions and `/`
separators. Canonical filesystem paths, including symlinks, identify each module
within this standalone context. Repeated imports and diamonds share one snapshot
and initializer. Each source is parsed separately; source text is not concatenated.
The graph's dependency order assembles isolated AST blocks and internal bindings,
then passes the complete program through the existing checker, ownership passes
and backend. Private lexical scopes remain separate.

Imports are collected from top-level immutable unannotated bindings, with optional
grouping around the literal import. Dependencies initialize before their importer,
with siblings visited in source order, even when an import binding appears after
an ordinary statement. All dependencies initialize before the entry body. Cycles
report E502 with the ordered path chain and closing import site. Missing files,
directories and malformed relative paths report E501. No working-directory fallback,
extension guessing, network access or package resolution is added.

Initializer failure stops startup before dependent/entry effects. Export restrictions
avoid claiming module resource cleanup or borrowed static exports. The
[diamond example](examples/modules/main.mwy) prints shared, left, right, entry, 17;
the shared module initializes once in both profiles.

## Source identity and output protection

Every file retains its original bytes and a disjoint range of internal span offsets.
Lexer/parser errors and semantic errors are mapped to the originating canonical
file, local byte range, line and column. Interpolation and Unicode spans retain the
same mapping. Existing one-file library compilation remains available without a
filesystem context. The driver snapshots inputs before checking; build/IR outputs
cannot replace any loaded source, including a canonicalized alias.

Native panic messages still use the existing numeric byte-offset format. For a
multi-file graph those are internal graph offsets; file labels and local runtime
sites are not yet implemented. This slice qualifies compiler diagnostics with file
identity, not public replay/source-map artifacts or complete native diagnostics.

## Limits and remaining gates

Graphs are bounded to 64 files, 32 active dependency levels, 4096 import edges,
4 MiB per source and 16 MiB of snapshot span space. Export shapes reuse the bounded
256-part/32-level reference-free shape check. Existing parser, type and ownership
budgets still apply. Exhaustion reports B001; it never admits an incomplete graph.

The entry's existing manifest refusal remains. `--standalone` explicitly bypasses
that entry policy without loading configuration. Relative imports cannot enter a
different nearest manifest context or import `mod.mwy` as executable source. This
is not a package identity or manifest implementation. Bare package names, path
aliases and remote dependencies remain gated; foundational lookup is unchanged.

Function/type exports, nested/conditional imports, mutable or annotated import
bindings, references to module storage, module values in function bodies and
multi-file documentation checking remain B001. Type-export syntax now receives an
explicit capability diagnostic. Standalone documentation retains its existing checks.
Copying an exported value into a local uses ordinary local borrowing/mutation rules.

Eight graph/checker groups and nine native groups cover canonical diamonds/symlinks,
relative resolution, snapshots and limits, compiler error spans, privacy/export
boundaries, initialization order/failure, and source-output protection. Native
execution runs in debug and release. The full compiler-gate result is recorded in
[STATUS.md](STATUS.md); this is not complete language or distribution qualification.
