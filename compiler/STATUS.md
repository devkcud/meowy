# Compiler handoff and work tracker

Updated: 2026-09-12. Record-composing module re-export work is in progress.
The previous mixed-primary compiler gate passed; no current failures are known. Full v0.0.1 remains incomplete.
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

Composed/conditional emissions and inline required import roots remain unavailable
as computed inputs. Helper purity, non-integer/mutable scratch and full required
evaluation remain separate. See [COMPUTED_TYPES.md](docs/COMPUTED_TYPES.md#imported-immutable-inputs).

Nested records retain unit primaries, immutable integer/record fields, 256 total fields
and 32 record levels. Declared record aliases retain known errors on unreachable paths;
inline unreachable emissions whose HIR loses field identity remain unavailable.
Other bootstrap limits remain 4096 visits, 64 active resolver/validation levels and
16384 traversed type nodes. These are not language E220 counters. Direct extents outside
computed roots retain their existing profile.

## Actual validation

- `533bf22`: copied module primary evidence; 12 focused native groups passed.
- `fdc2cd7`: required projections and complete module type hints; 16 focused groups,
  731 library/688 native tests, fmt and Clippy passed.
- `5ee7044`: startup/evidence integration; all 20 primary groups and fmt passed.
  Cases execute in debug/release and cover silent check/build, named effects,
  dependency diamonds, repeated cached work, original dependency E107 spans,
  transitive effect refusal and runtime failure before dependent initialization.
- Dependency-span probes exercise ordinary dependency diagnostics. Existing local
  integer/record tests separately cover retained-error provenance.
- The new two-file guide example passed debug/release with output `7`/`ready`;
  extracted files: `/tmp/meowy-mixed-primary-doc-il4r1eby`.
- `python3 -B tools/verify.py --compiler`: all ten checks passed, including fmt,
  Clippy, build, 731 library/692 native tests (1423 total), 20 Python tests, and
  existing examples. Log: `/tmp/meowy-mixed-primary-gate.log`.
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

Whole-record module inputs, composed/conditional export inputs, helper
initializer eligibility, module-data captures, borrowed module storage, package/manifest
resolution, full required evaluation and generic specialization, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain separate. Host execution does not qualify minimum
platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8.

## Active commit plan

Investigation: `statements.rs::emit` lowers record composition to one temporary bind
and primary/field emissions. It does not retain export eligibility. Only a checked
local registered in `Checker.exports` will qualify for this slice; ordinary records,
inline blocks and conditional compositions remain gated. Compile-time function/type
exports are not runtime record fields and must not be implicitly re-exported.

1. Represent named export evidence as its original local ID plus retained forwarding
   work in `exports.rs`. Update integer/subrecord path reads to charge that work.
   This prerequisite preserves existing acceptance, HIR and direct-export costs;
   run the complete Rust tests and inspect its staged diff before committing.
2. Preserve named and integer-primary evidence when a direct top-level composition
   reads a registered module local. Reuse the checked emitted fields and primary ID,
   charge copy/projection work per hop, and retain privacy and whole-record gates.
   Include focused native acceptance, width/effect/capture and metadata tests.
3. Add independent integration cases for startup ordering, silent checks/builds,
   transitive work, record ancestor evidence and dependency failures.
4. Update supported guides and root handoff; run the complete compiler gate and
   commit the documentation slice. Keep runtime implementation and fixtures unchanged.

## Next steps

1. Export metadata now pairs the original local ID with retained forwarding work.
   `inputs/records/paths.rs` and `inputs/records.rs` charge it for integer and
   subrecord reads; direct-export costs remain unchanged. Formatting passed;
   all 731 library/692 native Rust tests passed. Log:
   `/tmp/meowy-composed-input-paths.log`. Inspect/stage and commit this prerequisite
   before implementing composition metadata transfer.
2. Complete the composition and integration slices in order, updating this handoff
   after meaningful steps and committing each validated slice.
3. Keep conditional exports, helper purity, borrowed storage and packages separate.
   Keep STATUS concise and current; never recreate STEP logs or push.
