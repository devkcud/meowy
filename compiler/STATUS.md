# Compiler handoff and work tracker

Updated: 2026-09-12. Scalar primary imports passed the complete compiler gate.
No failing checks or unfinished code remain. Full v0.0.1 remains incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Documentation relocation plan

Move every root compiler Markdown page except `AGENTS.md` and `STATUS.md` into
`docs/`, preserving content and correcting links in the same slice. Compiler behavior,
reference fixtures, commands and dependencies stay unchanged. Baseline repository
verification passed all four checks, including 1143 links and 16 tooling tests.

1. Moved foundation, allocator bounds, owning HIR and panic outcome pages;
   all 1143 links pass and text/headings are preserved.
2. Moved ownership and pointer syntax pages; all links and preserved content pass.
3. Moved exclusive reference, field and slot pages; links and preservation pass.
4. Moved exclusive element and restart pages; links and preservation pass.
5. Moved reference block/return and exclusive function pages; checks pass.
6. Move computed types, module guide and README; update the working instructions.

Each group includes its inbound/outbound link fixes and this handoff, stays within
the file review limit, and must pass local-link and preservation checks before commit.
Run repository verification and the compiler gate after the complete move. Preserve
the compiler implementation next steps below for resumption after this layout task.

## Current compiler slice

`Module.primary` pairs a direct integer emission ID with its checked initializer
`Input`. Only a module whose complete runtime type is integer can consume this
metadata. `inputs.rs::module_integer` supplies both ordinary integer-copy evidence
and required reads. Module aliases and primary/named re-exports keep exact widths,
errors and work without making a synthetic module record eligible.

`type_values/scalars.rs::required_primary` checks availability; `Work::input` shares
work/error validation with local and named-field inputs. `expressions.rs` materializes
a proven primary only inside required roots. Function-local required imports work;
ordinary runtime module-data captures remain gated. Runtime HIR, constant folding,
module storage and initialization order are unchanged.

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

Integer primaries of record-valued modules, composed/conditional emissions and inline
required import roots remain unavailable as computed inputs. Helper purity,
non-integer/mutable scratch and full required evaluation remain separate. See [COMPUTED_TYPES.md](COMPUTED_TYPES.md#imported-immutable-inputs).

Nested records retain unit primaries, immutable integer/record fields, 256 total fields
and 32 record levels. Declared record aliases retain known errors on unreachable paths;
inline unreachable emissions whose HIR loses field identity remain unavailable.
Other bootstrap limits remain 4096 visits, 64 active resolver/validation levels and
16384 traversed type nodes. These are not language E220 counters. Direct extents outside
computed roots retain their existing profile.

## Actual validation

- Metadata: three focused tests and all 731 library/673 native tests passed. Existing
  HIR emission identity, integer width, tail work and constant folding are preserved.
- Lookup: four native primary groups and all 731 library/676 native tests passed,
  alongside fmt and Clippy. Aliases/copies/re-exports/function types work; exact-width
  arithmetic and ineligible initializer/capture boundaries remain checked.
- Integration: all nine primary groups passed, including silent check/build, direct
  required extents and scoped imports, shared dependency initialization once, transitive
  effect refusal, dependency E107 spans, repeated cached-work charging, and preserved
  runtime initializer failure before entry. Runtime execution uses both profiles.
- The dependency-span probe checks ordinary dependency diagnostics; existing local
  record/integer tests separately cover retained-error provenance.
- `python3 -B tools/verify.py --compiler`: all ten checks passed, including fmt,
  Clippy, build, 731 library/681 native Rust tests (1412 total), 16 tooling/four
  harness Python tests, and existing standalone/multi-file examples in both profiles.
  Log: `/tmp/meowy-primary-inputs-gate.log`.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug/release. Unsupported
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

Record-valued module primary inputs, composed/conditional export inputs, helper
initializer eligibility, module-data captures, borrowed module storage, package/manifest
resolution, full required evaluation and generic specialization, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain separate. Host execution does not qualify minimum
platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Next steps

1. Plan integer primary projections of record-valued file modules separately. Start
   with `inputs.rs::module_integer`, `type_values/scalars.rs`, `type_values.rs` and
   `expressions.rs` primary projection handling. Reuse explicit primary emission
   evidence while preserving the complete record type for ordinary values/queries;
   never treat all module fields as a pure record. Keep named-field effects separate
   from the primary initializer and preserve exact widths, work and runtime captures.
   Record reviewable lookup/materialization/native slices before editing. Verify
   arithmetic/annotated required reads, mixed-field privacy, failures and startup order.
2. Keep composed/conditional emissions, helper purity, borrowed storage and packages
   separate. Run focused checks per slice and the full compiler gate for behavior;
   commit reviewable slices, never push or recreate STEP logs.
