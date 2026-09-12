# Compiler handoff and work tracker

Updated: 2026-09-12. Named imported immutable inputs passed the complete compiler
gate. No failing checks or unfinished code remain. Full v0.0.1 remains incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Active commit plan

1. `fa3336d` records eligible direct integer primary emissions in `check/exports.rs` and
   `check/statements.rs`. Retain each checked emission ID and initializer
   evidence without changing runtime HIR or constant folding. Verify scalar metadata,
   source errors/work, named exports and conditional/effectful gates.
2. `c6e13c7` resolves primary inputs through `inputs.rs`, `type_values/scalars.rs` and required
   materialization in `expressions.rs`. Support named module aliases, arithmetic
   copies and primary/named re-exports while preserving widths, work charging and
   runtime capture refusal. Keep implementation and native regressions together.
3. Add independent native initialization, dependency span and work-budget scenarios.
4. Update supported-slice documentation and both handoffs; run the full compiler gate.

Primary metadata now pairs the existing HIR emission ID with its integer evidence.
Runtime HIR and constant folding are unchanged. The three focused metadata tests and
all 731 library/673 native tests passed; no failures remain. Only scalar integer
module values will consume primary metadata. Whole module records, composed/conditional
primaries, inline roots and helper purity remain separate.

Primary lookup now consumes evidence only for scalar integer module identities.
The four native primary-input groups and all 731 library/676 native tests passed,
along with fmt and Clippy. Aliases, re-exports, required function-local types and
exact-width errors work; runtime captures and mixed module record inputs remain
rejected. Existing mixed-record copy rejection remains E211. No failures remain.

All nine focused primary-input groups passed, including five new integration groups.
Check/build remain silent, shared imports initialize once, scoped required extents
work, transitive effects reject without execution, original dependency E107 spans
survive, and repeated reads charge full tail work. Runtime startup failures still
stop the entry in both profiles. No failing checks remain.

## Current compiler slice

`Module.inputs` maps eligible direct immutable named exports to their original checked
local IDs. Existing integer/record evidence retains source failures, values and work;
synthetic module bindings remain ineligible as whole-record inputs. Private initializer
dependencies stay private. Ordinary module shape and initialization checks still run.

`inputs/records/paths.rs` resolves a checked module field to the explicit export ID.
Required leaves, copied integers, projected records and named re-exports preserve exact
widths and complete ancestor evidence, including unselected siblings and unused tail
work. Each required read charges that retained work again. Path bounds allow one extra
module namespace segment; record and file-export shape bounds remain unchanged.

Unrelated module initialization effects do not disqualify a pure export. Every effect
inside its initializer or ancestor still prevents eligibility. Check/build never execute
initialization; ordinary storage, captures and dependency initialization order remain
unchanged. Required imported leaves can be used inside functions; ordinary runtime
module-data capture remains B001.

Primary/composed/conditional exports and inline import roots remain unavailable as
computed inputs. Helper purity, non-integer/mutable scratch and full required evaluation
remain separate. See [COMPUTED_TYPES.md](COMPUTED_TYPES.md#imported-immutable-inputs).

Nested records retain unit primaries, immutable integer/record fields, 256 total fields
and 32 record levels. Declared record aliases retain known errors on unreachable paths;
inline unreachable emissions whose HIR loses field identity remain unavailable.
Other bootstrap limits remain 4096 visits, 64 active resolver/validation levels and
16384 traversed type nodes. These are not language E220 counters. Direct extents outside
computed roots retain their existing profile.

## Actual validation

- Metadata prerequisite: four focused tests plus all 728 library/668 native tests.
- Lookup slice: ten native computed-field groups, all 728 library/669 native tests,
  fmt and Clippy passed. Accepted execution uses debug and release.
- Integration: all six imported-input groups passed. Check/build are silent; diamond
  dependencies initialize once in source order. Transitive ancestor effects reject
  without execution; one retained-work read passes while repeated reads exhaust the
  bootstrap budget. Dependency E107 spans and imported integer widths remain exact.
- The dependency-span probe checks ordinary dependency diagnostics; existing local
  record evidence tests separately cover retained-error provenance.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 728 library/673 native Rust tests (1401 total), 16 tooling/four
  harness Python tests, and existing standalone/multi-file examples in both profiles.
  Log: `/tmp/meowy-imported-inputs-gate.log`.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases do not count as language rejections. Local links, 23 catalog records,
  7 schemas/6 examples and whitespace checks passed; external links were not fetched.
- Backend/runtime code, reference fixtures and dependencies are unchanged. Editor
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

Primary/composed/conditional export inputs, helper initializer eligibility, module-data
captures, borrowed module storage, package/manifest resolution, full required evaluation
and generic
specialization, public FFI, wider ownership/cleanup, executable networking, public
artifacts/replay and LSP remain separate. Host execution does not qualify minimum
platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Next steps

1. Update `COMPUTED_TYPES.md`, `MODULES.md`, `README.md` and both handoffs for scalar
   primary imports. Keep module-record primaries, composed/conditional emissions,
   inline roots and runtime captures explicitly separate.
2. Run the complete compiler gate and local links, then commit the documentation
   handoff. Record the next bounded capability; never push or recreate STEP logs.
