# Compiler handoff and work tracker

Updated: 2026-09-10. Bounded computed-type blocks passed the compiler gate.
No failing checks or unfinished code remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Commit series

Immutable integer scratch for computed types is in progress; the tree started clean.
The scalar checker already handles literal widths, integer arithmetic and constant
folding. `Value::Constant` loses numeric width, so preserve typed static values before
reusing those operations. Restrict this slice to integers and eligible static inputs.

1. Typed static integers and literal/alias bindings complete. All ten computed-type
   library/parser and six native matching groups, fmt and Clippy passed. Literal
   overflow retains E216; width, scope and storage erasure are covered.
2. Reuse checked integer expressions with bounded form/input validation. Reject
   runtime dependencies and known effects; preserve width/overflow and shared budgets.
3. Add native/module/documentation coverage and a local-capacity example.
4. Document the integer-only boundary and run the complete compiler gate.

Runtime initializer eligibility is not tracked yet: even folded runtime bindings
remain unavailable. Mutable scratch, helper calls, floating/text/boolean scratch,
control flow and full E220 accounting stay separate. Commit each validated slice.

## Current compiler slice

`check/type_values.rs` resolves computed types and interprets straight-line type blocks.
Local immutable unannotated bindings hold type values; local aliases use the type
namespace. Nested blocks and symbolic type members preserve concrete identity.
Exactly one unlabelled/unnamed/unannotated primary emission provides the type result;
checking continues after it. Missing/non-type results use E211, duplicates E203/E205.
Scopes are restored on completion/error, and no HIR statements/storage/functions are
created for the construction block. Existing layout and ownership checks still apply.

`check/names.rs` routes computed annotations directly through this resolver, avoiding
its previous duplicate symbolic/type evaluation. Exported aliases and documentation
reuse the resulting types and checker scopes. Runtime-parameter type queries retain
existing type-only behavior without making runtime values eligible static inputs.

`Checker.type_work` shares 4096 expression/statement visits, 64 active resolver levels
and 16384 traversed concrete type nodes across one outer type-value resolution.
Nested blocks share counters; independent roots reset them, including after failure.
These are B001 bootstrap bounds, not the language's E220 logical evaluation counters.
Known resolved debug print/panic calls in evaluated positions use E219; source helper
purity and transitive effect analysis remain unimplemented. Calls never execute here.

See [COMPUTED_TYPES.md](COMPUTED_TYPES.md) and [the example](examples/computed-types.mwy).
Scalar scratch, mutable scratch, branches/restarts, labeled blocks, named/annotated
emissions, general statements/helper calls, `core.Type` signatures and specialization
remain separate. No runtime type-value storage or new syntax is introduced.

## Actual validation

- Resolver extraction: four exported-type library/six native groups and 14 graph
  library groups passed. Computed block compatibility also passed 19 documentation
  library and nine native documentation groups.
- Nine focused library/parser groups passed: scope/identity, missing/duplicate/wrong
  results, effects after emission, unsupported control, nested/wide work failures,
  independent-root reset, absent runtime storage and exact documentation signatures.
- Five native groups passed: computed exports through a facade, runtime queries and
  references, dependency E219 spans from check/build/run, documentation/example
  execution and dependency-local B001 budget failures. Execution uses both profiles.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 692 library and 648 native Rust tests (1340 total), 16 tooling plus
  four compiler-harness Python tests, 80 standalone and five multi-file examples in
  debug/release. Gate log: `/tmp/meowy-computed-types-gate.log`.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  cases do not count as language rejections or full release qualification.
- Local links, 23 catalog records, 7 schemas/6 examples and whitespace checks passed.
  External links were not fetched. Backend/runtime code, reference fixtures and
  dependencies are unchanged; editor and separate runtime/sanitizer gates were not rerun.

## Prior capabilities and other areas

Carried reference-free lists retain whole initialized length/payload through inner
restarts. Shape limits remain 256 parts/32 levels; list construction retains capacity
65,536, layout 1 MiB and one-based initialized-length checks. Nullable/union/reference-
bearing/foundation/owning and top-level unit carried slots retain their gates.

Shared whole-list views, nested element/record-field projections and reborrows
retain canonical Slot/Field/Element sources and parent reference identity. Index
expressions execute once and check current initialized length. Views can survive
inner restarts and alias scope exit while their result owner lives; owner expiry
cannot be undone by reinitializing the same physical site.

Selected-slot mutability is independent of whole-binding replacement. `Proofs.mutable`
tracks replaceable roots and `Proofs.fields` mutable owned descendants; `variable`
drives snapshots/refinements without granting writes through shared references.
Pointer syntax uses tight `&`/`&!`/`*`, immediate-field `.&`/`.&!`/`.*` and grouping
for complete targets.

Standalone documentation supports attachment, checked links/signatures, E801-E805,
`doc check`/`doc build`, local API pages and checked/opt-in examples. Net/HTTP/TLS
remain specified library work; module/type/I/O/task foundations and capability-typed
lifecycle implementation precede executable adapters.

## Architecture and proof boundaries

Graph loading/discovery remains in `src/modules/{load,discover}.rs`; snapshots and
compilation are in `src/modules.rs`. Literal imports cover the supported AST,
retain source order and initialize dependencies once before the entry. Canonical
paths, manifest boundaries, depth/file/edge/source/discovery budgets remain enforced.
`src/check/exports.rs` owns file export identity, privacy and signatures.
`src/check.rs::check_imports` retains complete graph ownership checking.
`src/driver.rs` protects graph inputs from output replacement and maps diagnostics.
Native file sites remain in `src/backend/sites.rs` and `native/runtime.cpp`.

Type-value resolution/work bounds live in `src/check/type_values.rs`; ordinary type
construction and symbol lookup remain in `src/check/names.rs`. Required list extents
still use `src/list.rs::list_extent` and scalar checks in `src/check/scalars.rs`.

## Still outside this compiler

Runtime module-data captures, borrowed module storage, package/manifest resolution,
full required evaluation and generic specialization, public FFI, wider ownership/
cleanup, executable networking, public artifacts/replay and LSP remain separate.
Host execution does not qualify minimum platforms or bundled distributions.
Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Next steps

1. Plan immutable scalar scratch inside required type blocks, starting from
   `check/type_values.rs::type_statements`, `list.rs::list_extent` and existing scalar
   constant/arithmetic checking. The intended next example computes a local capacity
   before constructing a list type. Preserve width/overflow checks and reject runtime
   inputs/effects without executing initializers. Keep this separate from mutable
   scratch, helper calls, symbolic `core.Type` parameters and full E220 accounting.
   Record dependency-ordered slices and accepted/rejection/budget tests before editing.
2. Preserve runtime data-capture/borrowed-export and ownership/header proofs. Run
   focused checks per slice and the complete compiler gate for new behavior; keep
   STATUS concise, commit reviewable slices, never push or recreate STEP logs.
