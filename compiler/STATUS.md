# Compiler handoff and work tracker

Updated: 2026-09-10. Straight-line integer block initializer eligibility passed the
compiler gate. No failing checks or unfinished code remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Commit series

1. `9722d9e` — retain checked values beyond lexical scope with child-first evidence.
2. `23a81ce` — strict block eligibility and required-only proven value reads.
3. `d6bfc10` — native values/effects/budgets and a block-seed example.
4. This separate documentation handoff records the complete gate below.

Each implementation/test slice passed focused checks before its commit and remained
below the review threshold. Plan dependency-ordered commits before the next feature.

## Current compiler slice

`Input` now retains a checked integer value as well as original errors and transitive
work. `check/inputs.rs` folds child evidence through the existing constant arithmetic,
preserving widths and source-ordered failures after lexical scopes close. It no longer
needs the departed block's lexical constant map to prove its result.

`check/inputs/blocks.rs` recognizes checked integer blocks containing immutable eligible
bindings and exactly one primary emission targeting that block. Nested eligible blocks
work. Every statement contributes evidence/work, including unused bindings after the
primary; the emission is not an early return. Calls, mutation, branches/restarts,
named/outer emissions and other statements remain unproven.

Required expression reads use proven values only while both required checking and a
computed-type root are active. Ordinary runtime locals, reads and initialization remain
unchanged. Inputs still require immutable provenance; runtime parameters, mutable state,
effectful results and imported data do not become static merely through folding.

Unreachable blocks whose HIR type is `never` retain error-only evidence when possible.
They do not supply an invented integer result. Required reads surface hidden failures,
including unused arithmetic after emission, as E107 at the original source expression.
Source identities, exact integer widths and documentation signatures are preserved.
See [COMPUTED_TYPES.md](COMPUTED_TYPES.md) and the updated
[block-seed example](examples/computed-types.mwy).

Transitive/cached reads and unused block work charge the shared bootstrap bound.
Limits remain 4096 visits, 64 active resolver/validation levels and 16384 traversed
type nodes, with B001 exhaustion. They are not language E220 counters. Direct extents
outside computed-type roots retain their previous profile. Runtime captures, owning/
borrowed export rules and normal type/layout checks remain intact.

## Actual validation

- Value-retention prerequisite passed eight initializer library and four native groups,
  including scope exit, unsigned complement, large widths and child-error precedence.
- Block integration passed 13 initializer-related library and seven native matching
  groups. Earlier computed-type compatibility passed 14 library/13 native groups.
- Three focused block library and four new native groups passed: nested/module values,
  unused tail failures/work, excluded effects, mutable/control gates, cache costs and
  dependency-local error spans from check/build/run. Execution uses both profiles.
- Updated example execution, exact documentation signatures, fmt and Clippy passed.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 708 library and 659 native Rust tests (1367 total), 16 tooling plus
  four compiler-harness Python tests, 80 standalone and five multi-file examples in
  debug/release. Gate log: `/tmp/meowy-block-inputs-gate.log`.
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

Block evidence lives in `src/check/inputs/blocks.rs`; required-only constant materialization
is in `src/check/expressions.rs`. Runtime constant folding remains separate.

## Still outside this compiler

Field/imported-data and helper initializer eligibility, module-data captures, borrowed
module storage, package/manifest resolution, full required evaluation and generic
specialization, public FFI, wider ownership/cleanup, executable networking, public
artifacts/replay and LSP remain separate. Host execution does not qualify minimum
platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Next steps

1. Plan bounded eligibility for immutable integer record-field projections. Inspect
   `check/inputs.rs`, checked record/emission HIR and `type_values/scalars.rs` before
   choosing an evidence representation. Preserve whole-initializer effect checks,
   concrete field identity, immutable storage and original diagnostic/work provenance;
   a known field value alone must not hide effects in the containing initializer.
   Keep imported data and helper calls separate initially. Record reviewable slices
   and accepted/rejection/native no-initializer-execution tests before implementation.
2. Preserve capture/borrowed-export and ownership/header proofs. Run focused checks
   per slice and the complete compiler gate for new behavior; keep STATUS concise,
   commit reviewable slices, never push or recreate STEP logs.
