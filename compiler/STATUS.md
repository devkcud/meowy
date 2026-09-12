# Compiler handoff and work tracker

Updated: 2026-09-12. Local-record composition evidence is being implemented.
The prior module composition gate passed. Full v0.0.1 remains incomplete.
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

`Module.primary` pairs a direct integer emission ID with its checked initializer
`Input`. `inputs.rs::module_integer` accepts scalar and mixed-record module identities;
checked HIR primary projections retain that evidence in integer copies and re-exports.
Synthetic module records remain ineligible as whole-record inputs.

Required arithmetic and integer-annotated scratch can project a mixed module's integer
primary. `expressions.rs` materializes it only with an integer context; full module
hints preserve record identity for aliases, fields and type queries. Unannotated
mixed-module scratch and bare mixed-module extents remain unavailable. Function-local
required reads work; ordinary runtime captures retain B001. Runtime HIR, constant
folding, storage and initialization order are unchanged.

`Module.inputs` maps eligible immutable named exports to their original checked
local IDs and retained forwarding work. Direct top-level compositions of registered
file modules forward each eligible named input and integer primary independently.
Each hop retains two extra work visits for its copy/projection; runtime HIR stays
unchanged, and compile-time function/type exports are not implicitly forwarded. Existing integer/record evidence retains source failures, values and work;
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

Nonmodule/conditional compositions and inline required import roots remain unavailable
as computed inputs. Helper purity, non-integer/mutable scratch and full required
evaluation remain separate. See [COMPUTED_TYPES.md](docs/COMPUTED_TYPES.md#imported-immutable-inputs).

Nested records retain unit primaries, immutable integer/record fields, 256 total fields
and 32 record levels. Declared record aliases retain known errors on unreachable paths;
inline unreachable emissions whose HIR loses field identity remain unavailable.
Other bootstrap limits remain 4096 visits, 64 active resolver/validation levels and
16384 traversed type nodes. These are not language E220 counters. Direct extents outside
computed roots retain their existing profile.

## Actual validation

- `eb3840e`: export source IDs and forwarding work; all 731 library/692 native tests
  passed with unchanged direct-export behavior.
- `0a3dd81`: module composition evidence; all 732 library/696 native tests, fmt and
  Clippy passed. Metadata coverage verifies source IDs, complete type, unchanged
  bind/projection HIR and absence of synthetic whole-record input evidence.
- `36cf2c2`: all nine native composition groups and the library identity test passed.
  Native cases run debug/release and cover primary/named/subrecord forwarding,
  widths, privacy, capture/conditional/nonmodule gates, silent check/build, startup
  diamonds, repeated work, ancestor effects and failure before facade execution.
- Dependency E107 probes check ordinary dependency diagnostics. Existing local
  integer/record tests separately cover retained-error provenance.
- The facade guide example passed debug/release with output `7`/`ready`. Extracted
  files: `/tmp/meowy-composed-input-doc-zu94l07n`. Updated guides and root handoff.
- `python3 -B tools/verify.py --compiler`: all ten checks passed, including fmt,
  Clippy, build, 732 library/701 native Rust tests (1433 total), 20 Python tests,
  and existing examples. Log: `/tmp/meowy-composed-inputs-gate.log`.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug/release. Local links,
  catalog/schema and whitespace checks passed. Unsupported cases do not count as
  successful language rejections; full release qualification remains open.
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

Whole-record module inputs, nonmodule/conditional composition inputs, helper
initializer eligibility, module-data captures, borrowed module storage, package/manifest
resolution, full required evaluation and generic specialization, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain separate. Host execution does not qualify minimum
platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Active plan and next steps

Composition lowers to one temporary Bind followed by primary/field projections.
Export identities now retain bounded record paths (`f763ee8`). Local record
evidence accepts these projections, preserving complete ancestor evidence.

Dependency-ordered commits:

1. Complete: export input identity now includes a bounded record path. All nine
   export/path library tests and all nine native composition groups passed; fmt
   passed. Existing module behavior and runtime HIR remain unchanged.
2. Complete: eligible local compositions retain projected field and unit-primary
   evidence. All 733 library/704 native tests, fmt and Clippy passed. Log:
   `/tmp/meowy-local-compose-tests.log`. Retained sibling/tail errors keep E107
   source spans with declared shapes; unannotated unreachable shapes retain B001.
3. Complete: top-level local composition exports retain a shared record ID and
   field path. All 734 library/706 native tests, fmt and Clippy passed; log:
   `/tmp/meowy-record-export-tests.log`. Inline/projected/copied execution, unchanged
   HIR, widths, privacy, ancestor effects and conditional gates passed. Mutable
   module fields retain the earlier B001 file-export rejection.
4. Add independent staging/work integration coverage and update the supported guide
   and root handoff. Run `python3 -B tools/verify.py --compiler` across the series.

Keep unit primaries, immutable integer/record fields and existing shape bounds.
Synthetic module namespaces remain ineligible as whole records. Conditional exports,
helper purity, borrowed storage and packages stay separate. Do not push.
