# Compiler handoff and work tracker

Updated: 2026-09-10. Immutable integer initializer eligibility passed the compiler
gate. No failing checks or unfinished code remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Commit series

1. `fe74fd8` — bounded immutable initializer eligibility evidence over checked HIR.
2. `86d391e` — evidence-backed required reads, error provenance and transitive work.
3. `6044d20` — native initialization boundaries and eligible-seed example.
4. This separate documentation handoff records the complete gate below.

Each implementation/test slice passed focused checks before its commit and remained
below the review threshold. Plan dependency-ordered commits before the next feature.

## Current compiler slice

`check/inputs.rs::integer_input` inspects checked immutable integer binding initializers.
It accepts literal, eligible local alias, minus/complement and arithmetic/bitwise HIR
forms. Every local dependency needs prior evidence in `Checker.inputs`, keyed by
local ID. Blocks, fields, calls, parameters, mutable dependencies and non-integer
forms receive no evidence. Inspection is bounded and charges the existing proof work.
Constant folding alone does not grant initializer eligibility.

Evidence retains transitive dependency work and any invalid integer operation hidden
by unreachable runtime paths. Ordinary compilation does not fail merely because
such dead arithmetic has evidence; required use reports E107 at its original source.
Repeated/cached dependency reads still charge their full transitive work against the
computed-type root's bootstrap budget. Independent roots reset the counters.

`check/type_values/scalars.rs` consumes evidence before accepting runtime-local input
names. Its scoped required lookup permits eligible lexical reads across function scope
while leaving runtime capture checks unchanged. Checking/building never executes
initializers. Native application effects and initialization order remain intact.
Runtime parameters/mutable inputs and effectful results remain unavailable (E211);
unproven folded inputs retain B001. Imported data/helper purity remain separate.

Typed integer scratch retains exact widths, signedness and documentation signatures.
Computed extents share its input validation; direct extents outside computed roots
retain their existing profile. Type blocks emit one primary type without creating
runtime scratch. Existing layout, export privacy and ownership checks remain intact.
See [COMPUTED_TYPES.md](COMPUTED_TYPES.md) and the updated
[eligible-seed example](examples/computed-types.mwy).

Bootstrap bounds remain 4096 visits, 64 active resolver/validation levels and 16384
traversed concrete type nodes, with B001 exhaustion. These are not language E220
counters. Mutable/non-integer scratch, comparisons/shifts, control flow, helper calls,
symbolic `core.Type` signatures and specialization remain separate.

## Actual validation

- Six initializer library groups passed: immutable/transitive provenance, excluded
  runtime forms, hidden failures, scope/shadowing, cached work and independent roots.
- Compatibility included 14 computed-type library/parser and nine matching native
  groups before the four new native groups. Fmt and Clippy passed for every slice.
- Four new native groups passed: cross-function required reads with one initialization,
  check/build without application effects, normal startup panic in both run profiles,
  original dependency overflow spans, and unchanged capture/mutable/parameter gates.
- Updated example execution and exact constructed documentation signatures passed.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 703 library and 655 native Rust tests (1358 total), 16 tooling plus
  four compiler-harness Python tests, 80 standalone and five multi-file examples in
  debug/release. Gate log: `/tmp/meowy-initializer-inputs-gate.log`.
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

Immutable initializer evidence is in `src/check/inputs.rs`, recorded by ordinary
binding checking in `src/check/statements.rs` and consumed by scalar required reads.

## Still outside this compiler

Block/field/imported-data and helper initializer eligibility, module-data captures,
borrowed module storage, package/manifest resolution, full required evaluation and
generic specialization, public FFI, wider ownership/cleanup, executable networking,
public artifacts/replay and LSP remain separate. Host execution does not qualify
minimum platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/
LLD/LLVM ar 22.1.8.

## Next steps

1. Plan bounded straight-line block initializer eligibility through `check/inputs.rs`,
   checked block HIR and existing constant/purity analysis. The next intended source
   is an immutable integer initialized by a block with local eligible bindings and
   one primary scalar emission. Preserve source order, reject effects/mutable state,
   and retain original error spans/transitive work without executing initializers.
   Keep helper calls, control flow and imported data separate; record reviewable
   slices and accepted/rejection/native initialization tests before editing.
2. Preserve capture/borrowed-export and ownership/header proofs. Run focused checks
   per slice and the complete compiler gate for new behavior; keep STATUS concise,
   commit reviewable slices, never push or recreate STEP logs.
