# Compiler handoff and work tracker

Updated: 2026-09-10. Documentation-bearing relative file graphs passed the compiler
gate. No failing checks or unfinished code remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Commit series

A bounded computed-type block slice is in progress; the tree started clean.
The audit found `check/names.rs::type_value` accepts literal/query/name/group forms,
but no block evaluation. The shared checker already owns type/value scopes and
concrete type construction; reuse them without emitting runtime code.

1. Existing resolver moved unchanged to `check/type_values.rs`. Four exported-type
   library and six native groups, 14 graph library groups, fmt and Clippy passed.
2. Root-shared visit/depth/materialization limits complete. All three focused
   computed-type/parser groups, fmt and Clippy passed. Nested/wide construction
   failures and successful/failed-root resets are covered.
3. Straight-line blocks complete. Seven focused computed-type/parser groups, four
   exported-type library/six native groups, 19 documentation library/nine native
   groups, fmt and Clippy passed. Computed annotations use one resolver path.
4. All nine focused library/parser and five native groups passed, including exact
   documentation signatures, absent runtime storage, example execution in both
   profiles, module exports, query/reference behavior and file-local failures.
   Fmt and Clippy passed.
5. Document the exact bootstrap boundary and run the complete compiler gate.

Keep helper calls, branches, mutable scratch, generic specialization and the full
language evaluation counters separate. This slice uses explicit bootstrap work/depth
limits (B001), not approximate E220 language-budget accounting. Commit each validated
slice and apply the review threshold before proceeding.

## Current compiler slice

Ordinary graph `check`, `build` and `run` check documentation in all discovered files,
including unused/inactive imports. `documentation/model.rs::at` reads each source
using its graph base, preserving attachment/markup spans and normalized CRLF/Unicode
maps. File models mark explicit exports public and ordinary bindings/types private;
single-file documentation keeps its existing public-coverage policy.

`check/exports.rs::module_value` activates and finishes each file's model.
`check/blocks.rs` checks module docs before that initializer scope closes. Function
and parameter docs use the checked declaration/signature scope, excluding body locals.
Models are not merged across files, and private lexical names remain isolated.

`check/documentation.rs` resolves imported function/data/type links through existing
export lookup. Facades and aliases preserve namespaces; missing/private targets are
E802. Inferred record-member links retain private root anchors. Public docs cannot
expose private local targets. Graph error mapping retains the owning file and local
byte range for E801/E802/E803 and documentation budget failures.

Ordinary compilation validates embedded example metadata without compiling or
executing example programs. Check/build do not execute initializers. Standalone doc
commands retain checked/opt-in examples; relative file imports in `doc check/build`
remain B001. Multi-file site generation, public indexes/coverage and example graph
resolution are separate. See [MODULES.md](MODULES.md) and
[the documented facade](examples/documented-modules/main.mwy).

## Actual validation

- Offset and export-model slices: shifted attachment/link/signature checks,
  Unicode/CRLF, public/private links and function parameters passed.
- Graph integration and links: 19 documentation library groups passed; compatibility
  included 11 graph library, 17 native file-module and nine native documentation groups.
- Six new native groups passed: documented facade execution, local diagnostics from
  check/build/run, unused dependencies, budgets, example/initializer boundaries,
  signature/capture/storage gates and explicit doc-command refusal.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 684 library and 643 native Rust tests (1327 total), 16 tooling plus
  four compiler-harness Python tests, 79 standalone and five multi-file examples in
  debug/release. Gate log: `/tmp/meowy-documented-modules-gate.log`.
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

## Still outside this compiler

Runtime module-data captures, borrowed module storage, package/manifest resolution,
generic specialization, public FFI, wider ownership/cleanup, executable networking,
public artifacts/replay and LSP remain separate. Host execution does not qualify
minimum platforms or bundled distributions. Toolchain: Rust 1.98.1 and LLVM/Clang/
LLD/LLVM ar 22.1.8.

## Next steps

1. Return to compiler type-construction foundations. Read the required-evaluation
   contract in `../docs/reference/compile-time.md` and the pipeline in `../COMPILER.md`;
   audit `src/check/names.rs::type_value` and its callers to identify the smallest
   bounded computed-type capability beyond literal/type-query/name/group handling.
   Record a concrete commit plan and accepted/effect/budget rejection fixtures before
   implementation. Preserve unsupported generic/package behavior and source spans.
2. Keep runtime data-capture/borrowed-export and ownership/header proofs intact.
   Run focused checks per slice and the complete compiler gate for new behavior;
   keep STATUS concise, commit reviewable slices, never push or recreate STEP logs.
