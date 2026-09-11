# Compiler handoff and work tracker

Updated: 2026-09-10. Immutable integer record-field eligibility passed the compiler
gate. No failing checks or unfinished code remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Commit series

1. `27f0c49` — scoped scalar evidence for nested/record-local initialization.
2. `2e4fa5c` — whole-record evidence with checked field indices and complete body checks.
3. `87fde75` — preserve that evidence in ordinary copied integer fields.
4. `449c3fa` — direct field reads in computed scratch and extents.
5. `a415b5f` — native boundaries, source diagnostics, budgets and updated example.
6. This separate documentation handoff records the complete gate below.

Each implementation/test slice passed focused checks before its commit and remained
below the review threshold. Plan dependency-ordered commits before the next feature.

## Current compiler slice

`check/inputs/records.rs` records complete initializer evidence for named immutable
records with a unit primary and 1..256 immutable integer fields. Checked field indices
retain values independently of declaration order. Record aliases preserve the shape
and evidence. Named field initializers may read earlier emitted fields through a
scoped scalar evidence map; nested scalar blocks and ordinary eligible locals work.

Every initializer statement is checked: selected fields cannot hide effects, mutable
siblings, invalid sibling arithmetic or unused tail work. The complete record's error
and transitive work accompany every field read. Error-only evidence can survive an
unreachable record with a declared shape; failures retain original source spans.
Synthetic module bindings receive no initializer evidence, keeping imported data gated.

`check/inputs.rs` propagates record evidence into copied integer bindings.
`check/type_values/fields.rs` resolves direct required paths from named immutable local
records (including grouped roots/aliases), preserving checked type/member identity.
`scalars.rs` charges the proof and reports failures before required materialization in
`expressions.rs`. Type identities such as `core.int32` keep their separate lookup.

Computed scratch and extents may read eligible lexical record fields across function
scope. Ordinary runtime captures remain gated. Runtime record reads and initialization
are unchanged; no initializer executes during checking/building. Direct extents outside
computed roots retain their existing profile. Nested records, references, inline roots,
non-integer fields/non-unit primaries, imported data and helper calls remain separate.
See [COMPUTED_TYPES.md](COMPUTED_TYPES.md) and the
[field-capacity example](examples/computed-types.mwy).

Existing scalar/block provenance, exact integer widths, source-ordered errors and
scope restoration remain intact. Bootstrap bounds remain 4096 visits, 64 active levels
and 16384 traversed type nodes; the record shape adds its 256-field bound. This is not
full required evaluation or language E220 accounting. Ownership/layout/export privacy
checks remain in the existing shared pipeline.

## Actual validation

- Scoped evidence passed 15 initializer-related library and 11 native matching groups.
- Seven record evidence/projection groups passed, including order, aliases, local
  field dependencies, exact values, excluded shapes, sibling errors/effects and bounds.
- Record compatibility included 38 native groups and the existing initializer exclusion
  test. Direct-read compatibility passed 14 computed-type library/17 native groups.
- Five new native groups passed: local module types/initialization, errors from an
  unselected sibling at the original dependency span, no effect execution, imported
  data/capture refusal and whole-record work. Execution uses both profiles.
- Updated example execution, exact documentation signatures, scratch visibility,
  fmt and Clippy passed. The new example distinguishes its field from a same-named local.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 717 library and 664 native Rust tests (1381 total), 16 tooling plus
  four compiler-harness Python tests, 80 standalone and five multi-file examples in
  debug/release. Gate log: `/tmp/meowy-record-fields-gate.log`.
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

## Still outside this compiler

Nested-record/imported-data and helper initializer eligibility, module-data captures,
borrowed module storage, package/manifest resolution, full required evaluation and
generic specialization, public FFI, wider ownership/cleanup, executable networking,
public artifacts/replay and LSP remain separate. Host execution does not qualify
minimum platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/
LLD/LLVM ar 22.1.8.

## Next steps

1. Plan bounded nested immutable integer-record eligibility through
   `check/inputs/records.rs` and `type_values/fields.rs`. Preserve checked field paths
   and the complete ancestor initializer's errors/effects/work, not just a leaf value.
   Reuse existing shape/depth limits, retain mutation/reference restrictions, and keep
   imported data/helper calls separate. Record dependency-ordered evidence/path/
   integration slices and accepted/rejection/native initialization tests before editing.
2. Preserve capture/borrowed-export and ownership/header proofs. Run focused checks
   per slice and the complete compiler gate for new behavior; keep STATUS concise,
   commit reviewable slices, never push or recreate STEP logs.
