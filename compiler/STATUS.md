# Compiler handoff and work tracker

Updated: 2026-09-12. Local-record composition inputs are implemented.
All ten compiler gate checks passed. Full v0.0.1 remains incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Documentation conventions and layout

[README.md](README.md), `AGENTS.md` and this handoff stay at the compiler root;
the 16 detailed guides live in `docs/`. Cargo and links use this layout.
[Documentation conventions](../docs/guide/documentation-style.md) require lowercase
`meowy` and readable example spacing. Tests, tooling and generated source keep their
own layouts. Intentional compact demonstrations and reference fixtures are preserved.

All 82 reformatted standalone examples retain executable tokens, literal contents,
ordinary comments and statement newlines. The nested documentation example also
retains its code tokens/attributes/output and passed `doc check --run-examples`.
Generated API-page branding is lowercase and covered by the renderer regression.
Git preserves that documentation series; the root STATUS links its preservation audit.

## Current compiler slice

Eligible unit-primary local records now preserve computed-input evidence through
composition, including inline sources, aliases, projected subrecords and extensions
with named fields. `inputs/records.rs` recognizes the checked temporary Bind plus
unit-primary/field projection HIR and retains complete ancestor eligibility, errors
and work. The runtime HIR and evaluation order are unchanged.

Direct top-level local-record compositions export fields through a shared checked
record ID and a bounded field path. `exports.rs::composed_record_inputs` records the
source evidence; `inputs/records/paths.rs::input_path` combines the retained path with
an importer projection. Subsequent module compositions preserve that identity/path
and add forwarding work. Every required read charges all retained work again.

File-module namespace eligibility stays distinct: direct module composition forwards
individually eligible exports and an eligible integer primary. It never grants
whole-record eligibility to a synthetic namespace. A local record constructed from
individual eligible exports can qualify. Mutable, effectful or unsupported siblings
and tails inside an ordinary record prevent eligibility for all composed fields;
unrelated file initialization does not. Privacy, exact widths, ordinary collision
checks, startup order, runtime captures and borrowed-export gates remain intact.

Required integer primary projections of scalar/mixed modules retain their prior
context rules: explicit integer scratch/arithmetic works; unannotated mixed-module
scratch and bare mixed-module extents remain unavailable. Required reads inside
functions work without granting ordinary runtime captures.

Nested records retain unit primaries, immutable integer/record fields, 256 total fields
and 32 record levels. Declared record shapes preserve known E107 source spans on
unreachable paths; unannotated unreachable shapes can lose field identity and remain
B001. Other bootstrap limits remain 4096 visits, 64 resolver/validation levels and
16384 type nodes; these are not language E220 counters. Native ownership analysis
may exhaust its own budget before a shape reaches its input-field limit.

Conditional composition/export evidence, helper purity, non-integer/mutable scratch
and full required evaluation remain separate. See
[COMPUTED_TYPES.md](docs/COMPUTED_TYPES.md#local-record-composition).

## Actual validation

- `f763ee8`: retained export paths; nine export/path library tests and nine existing
  native composition groups passed with unchanged runtime HIR.
- `cc8337f`: local composition evidence; 733 library/704 native tests, fmt and Clippy
  passed. Log: `/tmp/meowy-local-compose-tests.log`.
- `3e97948`: top-level local-record exports; 734 library/706 native tests, fmt and
  Clippy passed. Log: `/tmp/meowy-record-export-tests.log`.
- Final integration probes passed: repeated ancestor work versus separate roots in
  debug/release; silent check/build; startup diamonds initialized once. Checker-only
  coverage verifies the 256/257 field evidence boundary. The 256-field native probe
  hit B001 borrow-origin budget exhaustion, so it is not claimed as native support.
- The new guide example passed debug/release, printing `7`. Extracted files:
  `/tmp/meowy-local-compose-doc-cnm3qnoe`.
- `python3 -B tools/verify.py --compiler`: all ten checks passed, including 735
  library/708 native Rust tests (1443 total), 20 Python tests, fmt, Clippy and build.
  Log: `/tmp/meowy-local-record-inputs-gate.log`.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug/release. Local links,
  catalog/schema and whitespace checks passed; full release qualification is open.
- Runtime implementation, reference fixtures and dependencies are unchanged. Editor
  and separate runtime/sanitizer gates were not rerun; release qualification is open.

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

Whole-record module inputs, conditional composition inputs, helper
initializer eligibility, module-data captures, borrowed module storage, package/manifest
resolution, full required evaluation and generic specialization, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain separate. Host execution does not qualify minimum
platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Next steps

1. Plan conditional local-record evidence separately in `inputs/records.rs` and
   `inputs/blocks.rs`, following `../docs/reference/compile-time.md`. Establish which
   checked branch/flow facts can prove evaluation order and complete initializer
   eligibility before changing the gate. Cover selected/unselected effects, retained
   errors and repeated work; preserve the distinction from conditional module exports.
2. Keep helper purity, borrowed storage, packages and whole-module record inputs
   separate. Record reviewable commit slices before edits; never push or create STEP logs.
