# Meowy compiler bootstrap

This directory contains a working Rust compiler with a C++20 LLVM backend.
It checks standalone Meowy source and produces Linux x86-64 native executables.
It implements scalar programs, record composition, nullable unions, branch
narrowing and shared references to ordinary local storage, including guarded
block results, immutable records and unions carrying references, direct-function
borrow contracts, shared reborrows, last-use checks for mutable owners, and inline
bounded lists of copyable reference-free elements. It is not the complete v0.0.1 language.
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
compiler/target/debug/meowy run compiler/examples/borrow-liveness.mwy
compiler/target/debug/meowy run compiler/examples/borrowed-records.mwy
compiler/target/debug/meowy run compiler/examples/optional-borrows.mwy
compiler/target/debug/meowy run compiler/examples/borrow-functions.mwy
compiler/target/debug/meowy run compiler/examples/reborrows.mwy
compiler/target/debug/meowy run compiler/examples/scope-borrows.mwy
compiler/target/debug/meowy run compiler/examples/bounded-lists.mwy
compiler/target/debug/meowy run compiler/examples/list-unions.mwy
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
The [borrow liveness example](examples/borrow-liveness.mwy) updates scalar and record
owners after the final use of their shared references, including loop iterations.
The [borrowed records example](examples/borrowed-records.mwy) copies nested reference
fields, projects selected fields and a primary reference, and releases each loan
after that component's last use.
The [optional borrows example](examples/optional-borrows.mwy) narrows nullable
reference fields and unions of different reference types before dereferencing.
The [function borrows example](examples/borrow-functions.mwy) returns borrowed
views through direct calls and releases their input loans after the final use.
The [reborrows example](examples/reborrows.mwy) takes references to original
record fields through shared references and returns them through functions.
The [scope borrows example](examples/scope-borrows.mwy) contrasts local parameter
and receiver copies with shared receivers that keep the original owner alive.
The [bounded lists example](examples/bounded-lists.mwy) preserves an original list
while appending to its copy, checks one-based positions and compares initialized
elements within an unchanged inline capacity.
The [list unions example](examples/list-unions.mwy) chooses list alternatives by
element type, literal range and capacity while preserving concrete element widths.

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
  Unary operators keep their operand type before the result enters an expected
  union, preserving checked widths and boolean operations.
- Blocks with primary and immutable named emissions, record composition,
  scalar-primary projection, dispatch, and duplicate/uninitialized slot checks.
- Normalized scalar/record unions, nullable field and primary defaults, and
  conversions between compatible union sets without numeric widening.
- Runtime type predicates and proven ascriptions, including immutable field paths,
  complementary conditions, short-circuit operands and early-exit narrowing.
  Assignments invalidate proofs about the changed value.
- Shared references to ordinary local bindings and their concrete record
  fields, address equality, reference copies and copyable dereference. Record field
  access through a reference copies the field. Borrow origins are checked before
  lowering. Bare-reference block results preserve every possible origin across
  branches and named exits; completed results cannot retain expired locals.
  Restart, panic and enclosing leave discard emissions when proven by flow guards.
  Mutable owners can be assigned after the last use of every overlapping shared
  reference. Live aliases, reference operands and retained block results protect
  their owners from writes; conflicting assignments report E302.
- Immutable records with shared-reference primary, named and nested components.
  Whole-record copies preserve every reference; field and scalar-primary access
  track only the selected components. All retained components must outlive their
  receiving block. Record equality compares the full shape, including addresses.
- Immutable reference-bearing unions and optional fields. Injection, widening and
  proven narrowing preserve the active member's borrow origins. Absent reference
  fields carry no loan; type predicates inspect the discriminant without copying
  reference payloads. Copies and equality still consume every active reference.
- Shared reborrows of reference-free referents: `&*view`, `&view.field` and nested
  parenthesized paths, including reference-valued prefixes such as
  `&holder.view.field`. Reference-valued calls/blocks evaluate once. Derived
  function results retain all active input lifetime bounds. Union payload addresses,
  reference-bearing pointees and exclusive reborrows remain unavailable.
- Scope-local references to by-value parameters and dispatch `self` bindings.
  Their addresses cannot escape their storage scopes. Shared-reference and
  reference-carrier dispatch retain original origins and all-input bounds.
- Inline bounded lists `T[N]` with a separate initialized length, typed/inferred
  literals, `.size()`, one-based copy indexing, value-returning `.add()`, whole-value
  replacement and equality of initialized elements. Elements can be scalars,
  reference-free records/unions or nested bounded lists. Inference preserves typed
  widths and never invents a union or projects a record primary to reconcile items.
  Whole-list references use the same lifetime and final-use checks as other owners.
- Direct functions, explicit-result recursion, strict mutual-forward groups,
  conditional matchers, and named-scope `leave`/`restart`, including scoped aliases.
- Shared-reference function inputs and results, including immutable record/union
  carriers. A returned reference retains every active borrow-carrying input under
  the conservative public contract, including ignored inputs of another type.
  Scalar results and scalar-only projections end those loans after the call.
- `debug.print`, streamed interpolation at output calls, `debug.panic`, string
  byte length, and string comparison.

Emissions continue executing the block. `check` analyzes application effects
without running them. Debug and release both preserve dynamic arithmetic checks.
The initial panic runtime reports failure and exits; recoverable unwinding and
owned-resource cleanup remain unimplemented.

Dynamic integer failures report P002 with the source operator, original operands,
integer width/signedness, numeric range and half-open source byte span. Overflow
and a zero divisor are distinguished; signed minimum remainder by `-1` remains zero.
For example, an `int8` addition can report:

```text
panic[P002]: int8 + overflow (left 127, right 1; range -128..127) at bytes 14..17
```

Explicit `debug.panic` streams its supplied message once, then appends its P006
call-site byte span. If message evaluation itself panics or leaves the scope,
the outer panic does not append a misleading site or terminator. A completed panic
exits with status 1. These are bootstrap text diagnostics, not the
release panic artifact format or a recovery/unwind implementation.

Unavailable constructs report **B001**, including slices, named list positions,
reference/owned list elements, element mutation/borrowing, other collection APIs, exclusive borrows,
borrows of temporary storage, capturing closures, generic/type-producing
helpers, imports beyond the foundational bootstrap modules, and mutable record fields. String interpolation outside an
output call requires the future formatting/storage implementation and is rejected.
The [tracker](STATUS.md#still-outside-this-compiler) covers the full remaining scope.

List capacities accept non-negative integer constants and checked scalar
expressions. General required evaluation through blocks or calls remains
unavailable. Bootstrap limits are 65,536 slots and 1 MiB of inline layout per list;
exceeding those implementation budgets reports B001. Dynamic bounds/fullness
failures report P001/P003 with the position or capacity, initialized length and
source byte span. They exit through the initial panic runtime.
With several expected list alternatives, capacity and compatible element types
must select exactly one. Multiple viable choices report E207; all capacities being
too small reports E103. Literal range can select a width, but no preference is given
to a smaller capacity or a default numeric width. Pure contextual literals may wait
for typed elements; other expressions are checked once in source order.
Unresolved contextual effects or nested candidate constraints remain B001: provide
an explicit element/list annotation. Candidate selection allows up to 256 list
alternatives and charges the existing analysis work budget.

References can pass through local blocks and immutable aliases while their owners
remain alive. The checker reuses branch/completion proofs and checks all possible
borrow origins. Retained local escapes report E303; discarded emissions still
evaluate their operands and effects. References can also be stored in immutable
record and union components and direct-function signatures. Mutable carriers,
reassignment and direct reference formatting require future analysis. Named-emission
storage remains unavailable as a borrow root; parameter and receiver copies may
be borrowed only while their local storage survives. Missing
origin proofs or exhausted analysis budgets produce B001.
Borrow liveness follows branches and named loop edges. An assignment evaluates its
right-hand side before writing: `owner = *view + 1` is valid when that is the last
use of `view`. A later use of that view makes the write a conflict. Replacing a
record overlaps references to any of its fields.
Copying a record counts as a use of all its references, even if a later operation
selects only one field. Direct projection, scalar comparison and scalar-primary
formatting do not keep unrelated component loans alive.
See [the storage design](OWNERSHIP.md) for the remaining analysis stages.

Union literals receive a numeric width when the expected union has one matching
numeric member. Multiple candidate widths require an explicitly typed value;
the checker reports E207 instead of choosing a width. Record constructors must
select one compatible union member; ambiguous shapes also report E207. Union
equality requires the same normalized union type on both sides. Inspect a nullable
value with a type predicate, or compare it with a null value explicitly typed as
the same union. Named blocks conservatively
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
| `src/list.rs`, `src/list_context.rs` | Bounded lists, literal candidate constraints and inference work budgets |
| `src/flow.rs` | Shared boolean guards for reachability, disjoint emissions and narrowing |
| `src/borrow.rs` | Guarded component origins, block-result transfers and lexical lifetime checks |
| `src/borrow_contract.rs` | Symbolic function inputs, caller origin substitution and all-input lifetime bounds |
| `src/borrow_value.rs` | Active union variants, component paths, coercions and bounded value snapshots |
| `src/loans.rs` | Guarded CFG, per-component reference liveness and shared-loan/write conflicts |
| `src/diagnostic.rs`, `src/driver.rs`, `src/main.rs` | Diagnostics, commands, build publication and process launch |
| `src/backend.rs` | Typed LLVM IR lowering and bridge interface |
| `native/bridge.cpp` | LLVM verification, optimization and object emission |
| `native/runtime.cpp` | Versioned scalar output and panic ABI, separate from LLVM |
| `build.rs` | Exact native-tool version checks, bridge/runtime bootstrap |
| `tests/native.rs`, `tests/conformance.py` | Native regression and language catalog execution |

The full architecture and release gates remain in [COMPILER.md](../COMPILER.md).
