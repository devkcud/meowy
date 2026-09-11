# Compiler handoff and work tracker

Updated: 2026-09-10. Immutable integer scratch in computed-type blocks passed the
compiler gate. No failing checks or unfinished code remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Commit series

Immutable integer initializer eligibility is in progress; the tree started clean.
The constant folder also handles runtime blocks and does not establish purity.
Track separate evidence over checked integer HIR, restricted to literal/alias/unary/
arithmetic forms with already eligible immutable dependencies. Retain hidden arithmetic
failures from unreachable code and bounded transitive work for required reads.

1. Bounded HIR eligibility evidence implemented. Three provenance groups, 14
   computed-type library/parser and nine native matching groups, fmt and Clippy
   passed. No required reads consume the new evidence yet.
2. Consume that evidence in computed scalar bindings/extents. Permit static reads
   across function scope without enabling runtime captures; test budgets and diagnostics.
3. Add native/module/no-initializer-execution coverage and update the capacity example.
4. Document eligibility boundaries and run the complete compiler gate.

Blocks, fields/imported data, helper calls, mutable inputs and full purity/E220 remain
separate. Commit each validated slice and preserve unrelated work.

## Current compiler slice

Computed-type blocks support immutable integer scratch alongside local type values
and aliases. `Value::Static` retains a constant and its exact type; expression lookup,
type hints and documentation preserve width/signedness through aliases. Optional
integer annotations use ordinary literal and type checks. Scratch creates no runtime
locals/functions/statements, and block scopes still close on completion/error.

`check/type_values/scalars.rs` validates integer literals, eligible names, groups,
unary minus/complement and binary arithmetic/bitwise operands before using the existing
scalar checker and constant evaluator. Required arithmetic checks run with live reach
and restore prior reach/required state. E216/E107/E213 retain literal overflow,
invalid arithmetic and incompatible-width meanings. Known debug calls use E219.

Runtime parameters/mutable inputs and effectful initializer results are unavailable
(E211). Even folded immutable runtime bindings remain gated (B001): their transitive
initializer eligibility is not tracked. No runtime initializer is evaluated here.
`list.rs::list_extent` uses the same input validation inside active computed roots;
its pre-existing direct-extent profile outside those roots is unchanged.

Scalar validation shares the outer type root's bootstrap work/depth limits. Nested
blocks do not reset them. The bounds remain 4096 visits, 64 resolver/validation levels
and 16384 traversed concrete type nodes, with B001 exhaustion. They are not language
E220 counters. See [COMPUTED_TYPES.md](COMPUTED_TYPES.md) and the updated
[capacity example](examples/computed-types.mwy).

Type blocks still require one unnamed/unlabelled/unannotated primary type emission;
checking continues after it. Missing results and runtime names used as types use
E211; duplicates use E203/E205.
Mutable/non-integer scratch, comparisons/shifts, control flow, helper calls, symbolic
`core.Type` signatures and specialization remain separate. Existing type identity,
layout, export privacy, reference/ownership and documentation checks remain intact.

## Actual validation

- Typed values and calculations: 14 focused computed-type library/parser groups and
  nine matching native groups passed. Existing dead-path extent regression passed.
- Coverage includes narrow and large integer widths, aliases, signed/complement
  operations, division/overflow, mixed-width rejection, dead runtime branches,
  runtime/effectful inputs, unsupported scalar forms and shared expression budgets.
- Native coverage verifies exported capacities, exact dependency spans from
  check/build/run, no initializer execution on failures, and the folded-runtime-input
  gate. The example executes in both profiles; documentation retains uint8 signatures
  and scalar scratch leaves no runtime storage.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 697 library and 651 native Rust tests (1348 total), 16 tooling plus
  four compiler-harness Python tests, 80 standalone and five multi-file examples in
  debug/release. Gate log: `/tmp/meowy-integer-scratch-gate.log`.
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

Static integer validation/folding is in `src/check/type_values/scalars.rs`; `Value::Static`
in `src/check.rs` carries exact types into expression hints and documentation.

## Still outside this compiler

Runtime initializer eligibility, module-data captures, borrowed module storage,
package/manifest resolution, full required evaluation and generic specialization,
public FFI, wider ownership/cleanup, executable networking, public artifacts/replay
and LSP remain separate. Host execution does not qualify minimum platforms or bundled
distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Next steps

1. Plan explicit eligibility evidence for immutable integer runtime bindings used
   by required type evaluation. Inspect `check/statements.rs` binding construction,
   `check/scalars.rs::constant`, `list_context/pure.rs` and the new scalar validator.
   A folded value alone is insufficient: literals and transitively eligible arithmetic
   should be distinguishable from effects, runtime parameters and mutable state.
   Keep imports/helper purity separate initially. Record dependency-ordered slices and
   accepted/rejection/native no-initializer-execution tests before implementation.
2. Preserve capture/borrowed-export and ownership/header proofs. Run focused checks
   per slice and the complete compiler gate for new behavior; keep STATUS concise,
   commit reviewable slices, never push or recreate STEP logs.
