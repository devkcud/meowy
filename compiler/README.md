# Meowy compiler bootstrap

This directory contains a working Rust compiler with a C++20 LLVM backend.
It checks standalone Meowy source and produces Linux x86-64 native executables.
It implements scalar programs, record composition, nullable unions, branch
narrowing and shared references to immutable local storage, including guarded
block results. It is not the complete v0.0.1 language.
Read [STATUS.md](STATUS.md) for gaps, validation evidence, and the next work,
and [AGENTS.md](AGENTS.md) before changing the implementation.

## Build and run

From the repository root:

```sh
cargo build --locked --manifest-path compiler/Cargo.toml
compiler/target/debug/meowy check compiler/examples/factorial.mwy
compiler/target/debug/meowy run compiler/examples/hello.mwy
compiler/target/debug/meowy run compiler/examples/factorial.mwy --profile release
compiler/target/debug/meowy run compiler/examples/nullable.mwy
compiler/target/debug/meowy run compiler/examples/references.mwy
compiler/target/debug/meowy run compiler/examples/borrow-results.mwy
compiler/target/debug/meowy build compiler/examples/loop.mwy --output compiler/build/sum
compiler/build/sum
```

The examples print a greeting, `3628800`, and `5050`. The
[records example](examples/records.mwy) demonstrates primary values, named fields,
interpolation, and dispatch. The [nullable example](examples/nullable.mwy) exercises
absent fields and a fallback function that narrows a value after an early exit.
The [references example](examples/references.mwy) compares storage addresses and
copies values through shared references. The [borrowed results example](examples/borrow-results.mwy)
selects between surviving owners and discards an iteration-local borrow on restart.

The compiler requires Rust **1.98.1** and LLVM, Clang, LLD, and LLVM ar **22.1.8**.
The native tools are resolved at the explicit `/usr/bin/` paths in `build.rs`;
LLVM development headers/libraries and the host C/C++ development environment
must be installed. The Rust workspace has no external crate dependencies.
Cargo builds the C++ bridge and embeds the separate runtime archive in the compiler.

This is a host bootstrap: LLVM's shared library and the pinned Clang/LLD paths
must remain available. It uses the host libc development files at link time.
It is not yet a portable compiler distribution with a bundled sysroot, and has
not qualified the reference's Linux 5.4/glibc 2.31 baseline.

## Implemented language

- UTF-8 sources, original byte spans, retained lexer trivia, compact punctuation,
  comments, escaped strings, and nested interpolation.
- Lexical value and type namespaces, primitive type aliases, ordinary shadowing,
  and aliases of the resolved `core` and `debug` intrinsics.
- Null, booleans, signed/unsigned 8–64-bit integers, `isize`/`usize`, float32/64,
  and borrowed literal strings. Numeric operations preserve the operand types.
- Immutable and mutable local bindings; checked integer arithmetic, bitwise
  operations, comparisons, and short-circuit boolean operators.
- Blocks with primary and immutable named emissions, record composition,
  scalar-primary projection, dispatch, and duplicate/uninitialized slot checks.
- Normalized scalar/record unions, nullable field and primary defaults, and
  conversions between compatible union sets without numeric widening.
- Runtime type predicates and proven ascriptions, including immutable field paths,
  complementary conditions, short-circuit operands and early-exit narrowing.
  Assignments invalidate proofs about the changed value.
- Shared references to immutable ordinary local bindings and their concrete record
  fields, address equality, reference copies and copyable dereference. Record field
  access through a reference copies the field. Borrow origins are checked before
  lowering. Bare-reference block results preserve every possible origin across
  branches and named exits; completed results cannot retain expired locals.
  Restart, panic and enclosing leave discard emissions when proven by flow guards.
- Direct functions, explicit-result recursion, strict mutual-forward groups,
  conditional matchers, and named-scope `leave`/`restart`, including scoped aliases.
- `debug.print`, streamed interpolation at output calls, `debug.panic`, string
  byte length, and string comparison.

Emissions continue executing the block. `check` analyzes application effects
without running them. Debug and release both preserve dynamic arithmetic checks.
The initial panic runtime reports failure and exits; recoverable unwinding and
owned-resource cleanup remain unimplemented.

Unavailable constructs report **B001**, including collections, exclusive borrows,
borrows of mutable or temporary storage, capturing closures, generic/type-producing
helpers, imports beyond the foundational bootstrap modules, and mutable record fields. String interpolation outside an
output call requires the future formatting/storage implementation and is rejected.
The [tracker](STATUS.md#still-outside-this-compiler) covers the full remaining scope.

References can pass through local blocks and immutable aliases while their owners
remain alive. The checker reuses branch/completion proofs and checks all possible
borrow origins. Retained local escapes report E303; discarded emissions still
evaluate their operands and effects. Reference-bearing signatures, aggregates,
reassignment, dispatch and direct formatting require future analysis. Parameter,
receiver and named-emission storage remain unavailable as borrow roots. Missing
origin proofs or exhausted analysis budgets produce B001.
See [the storage design](OWNERSHIP.md) for the remaining analysis stages.

Union literals receive a numeric width when the expected union has one matching
numeric member. Multiple candidate widths require an explicitly typed value;
the checker reports E207 instead of choosing a width. Named blocks conservatively
forget mutable-value proofs at entry. Restarts that may emit again into a surviving
outer result report B001 until full loop dataflow is implemented.

## CLI behavior

`meowy help` lists the implemented commands and options. An explicit `.mwy` entry
is required. Ancestor `mod.mwy` files are detected and rejected because project
configuration is not implemented. `--standalone` deliberately bypasses that policy
for isolated source checking, including the conformance harness.

```sh
compiler/target/debug/meowy build compiler/examples/records.mwy \
  --profile release --output compiler/build/records --emit-llvm compiler/build/records.ll
compiler/target/debug/meowy check docs/conformance/sources/compact_min.mwy --standalone --json
```

`--emit-llvm` and `--json` are bootstrap inspection options. JSON diagnostics go to
stderr as one object per line with schema `meowy.bootstrap.diagnostic`, version 1,
code, message, path, half-open byte start/end, and one-based line/column. This is an
internal format, distinct from the documented release artifact schemas. `B002`
identifies bootstrap tool or output failures; it cannot count as a source rejection.

Default outputs are `build/x86_64-unknown-linux-gnu/<profile>/<entry stem>` under
the source directory. `--output` is relative to the current working directory.
Successful links replace outputs atomically. Failed builds preserve earlier files,
and `run` stops on failure. Source, manifest, lockfile, replay, and symlink output
paths are protected. `run` inherits working directory and streams, forwards
arguments after `--`, and returns the application's exit or signal status.

## Tests

```sh
cargo test --locked --manifest-path compiler/Cargo.toml
cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path compiler/Cargo.toml --check
python3 compiler/tests/conformance.py
python3 -m unittest discover -s compiler/tests -p 'test_*.py'
```

The Rust suite includes native execution in debug and release, overflow behavior,
scope control, records, literal limits, diagnostics, and safe output replacement.
The conformance harness runs each existing reference fixture independently and
compares actual checking results and stdout. Unsupported cases are listed separately.
Use `--strict` to require every catalog case; that full-language gate is expected
to fail while bootstrap gaps remain. `../docs/conformance/check.py` only validates
the fixture catalog and does not execute the compiler.

## Implementation map

| File | Responsibility |
| --- | --- |
| `src/lexer.rs`, `src/parser.rs`, `src/ast.rs` | Lossless tokens and punctuation-aware syntax |
| `src/check.rs`, `src/hir.rs` | Resolution, scalar types, emission flow, checked lowering input |
| `src/flow.rs` | Shared boolean guards for reachability, disjoint emissions and narrowing |
| `src/borrow.rs` | Guarded immutable origins, block-result transfers and lexical lifetime checks |
| `src/diagnostic.rs`, `src/driver.rs`, `src/main.rs` | Diagnostics, commands, build publication and process launch |
| `src/backend.rs` | Typed LLVM IR lowering and bridge interface |
| `native/bridge.cpp` | LLVM verification, optimization and object emission |
| `native/runtime.cpp` | Versioned scalar output and panic ABI, separate from LLVM |
| `build.rs` | Exact native-tool version checks, bridge/runtime bootstrap |
| `tests/native.rs`, `tests/conformance.py` | Native regression and language catalog execution |

The full architecture and release gates remain in [COMPILER.md](../COMPILER.md).
