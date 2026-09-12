# Compiler handoff and work tracker

Updated: 2026-09-12. Imported immutable input work is planned below. The previous
export metadata passed four focused tests and all 1396 Rust tests. The previous
nested-record gate passed. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Active commit plan

1. `edcce7e` retains explicit named-export input identities in `check/exports.rs` and
   `check/statements.rs`, reusing integer/record evidence and checked HIR bindings.
   Only direct immutable module emissions qualify; private bindings and synthetic
   module records remain distinct. Verify metadata, effects and conditional gates.
2. `85c3b17` implements imported field evidence in `inputs` and `type_values/fields.rs`, including
   copied integers, projected records and re-exports. Keep ancestor errors/work,
   checked field identities, original file spans and runtime capture gates. Verify
   accepted native execution and relevant rejected inputs together.
3. Implemented four independent native initialization/provenance/budget groups.
   All six imported-input groups passed, including both runtime profiles.
4. Update supported-slice documentation and both handoffs; run the complete compiler
   gate and local links. Splitting tests from the documentation handoff keeps each
   review focused and below the size threshold.

Investigation: the existing checker intentionally omits synthetic module evidence.
Named emissions already have unique checked local IDs; retaining those identities
allows explicit export lookup without treating a file body as a pure record.
Eligibility belongs to each initializer, so unrelated module initialization effects
must still run exactly once at runtime but do not disqualify independent exports.
Primary/composed/conditional exports and helper purity remain separate.

The metadata prerequisite passed 728 library and 668 native tests. Imported field
lookup is now implemented through explicit exported IDs, never synthetic module
record evidence. Copied integers, subrecords, re-exports and function-local required
reads passed all ten native computed-field groups in both execution profiles.
Effects, conditional exports, private names and ordinary captures remain rejected.
The lookup slice passed all 728 library and 669 native tests, plus Clippy.
Four added integration groups passed: silent check/build, diamond initialization
once in source order, transitive ancestor-effect refusal, original dependency E107
spans/exact widths, and charging retained ancestor work on repeated projected reads.
The diagnostic-span probe validates ordinary dependency checking; retained-error
provenance also remains covered by the existing local record evidence tests.

## Current compiler slice

Record evidence stores integer leaves by checked field-index paths. Shared `Sources`
carry scoped integer and record evidence, so nested constructors can use earlier fields,
record aliases and eligible local calculations after lexical scopes close. Every record
has a unit primary and nonempty immutable integer/record fields. Shape checks count
256 total fields across descendants and at most 32 record levels; unused local record
shapes are checked too. Construction shares the existing work/depth guards.

`check/inputs/records/paths.rs` resolves checked HIR paths and projects subrecord maps.
A projected alias retains the complete original ancestor input's errors and work,
including siblings outside the selected subtree. A copied integer or later leaf read
cannot shed that evidence. `type_values/fields.rs` resolves grouped/nested named paths
through checked types and requires an integer leaf. Original widths and field identities
survive declaration order, aliasing and function-scope required reads.

Every initializer statement contributes evidence, including unused calculations after
emissions. Effects/mutation anywhere in a containing initializer prevent eligibility.
Known ancestor failures remain E107 at the original source expression; cached work is
charged on every required read. Required reads materialize only proven values, leaving
ordinary runtime storage, captures and initialization order unchanged.

Declared record aliases can retain error-only evidence on unreachable paths. Inline
unreachable record emissions that lose their field identity in HIR remain unavailable;
no guessed layout or concrete result is introduced. Imported data, reference/inline
roots, non-integer leaves, empty/non-unit records and helper calls remain separate.
See [COMPUTED_TYPES.md](COMPUTED_TYPES.md) and the
[nested field example](examples/computed-types.mwy).

Other bootstrap bounds remain 4096 visits, 64 active resolver/validation levels and
16384 traversed type nodes. These are not language E220 counters. Direct extents outside
computed roots retain their existing profile. Ownership/layout/export privacy checks
remain in the existing shared pipeline.

## Actual validation

- Path/scope prerequisite passed all 17 existing initializer/record library groups.
- Nested construction passed 21 initializer/record groups, including declared alias
  errors, forbidden descendant effects/mutation, total fields/depth and unused shapes.
- Seven nested evidence/path groups passed, plus the existing carried-record native
  group and five native field groups. Captures/references/missing fields stay gated.
- Four new native groups passed: module initialization/subrecord aliases, errors outside
  projected subtrees at original dependency spans, preserved ancestor effects/work and
  imported-data/runtime-capture refusal. Execution uses both profiles.
- Updated example execution, exact documentation signatures, scratch visibility,
  fmt and Clippy passed.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 724 library and 668 native Rust tests (1392 total), 16 tooling plus
  four compiler-harness Python tests, 80 standalone and five multi-file examples in
  debug/release. Gate log: `/tmp/meowy-nested-records-gate.log`.
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

Whole-record evidence is in `src/check/inputs/records.rs`; direct required field lookup
is in `src/check/type_values/fields.rs`. Native coverage is `tests/native/computed_fields.rs`.

Nested paths/subrecord evidence are in `src/check/inputs/records/paths.rs`.
`Sources` in `src/check/inputs.rs` carries scoped integer and record inputs.

## Still outside this compiler

Imported-data and helper initializer eligibility, module-data captures, borrowed
module storage, package/manifest resolution, full required evaluation and generic
specialization, public FFI, wider ownership/cleanup, executable networking, public
artifacts/replay and LSP remain separate. Host execution does not qualify minimum
platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Next steps

1. Update `COMPUTED_TYPES.md`, `MODULES.md`, `README.md` and both handoffs with the
   bounded named-export capability. Run the full compiler gate and local links.
   Keep unsupported cases distinct, commit reviewable slices, never push or recreate
   STEP logs.
