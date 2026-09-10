# Compiler handoff and work tracker

Updated: 2026-09-10. Broader literal imports and function-scope module identities
are complete and passed the compiler gate. No failing checks remain. Three tested
commits precede this separate documentation handoff.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Commit series

Documentation-bearing graphs are in progress. The tree started clean. Investigation
confirmed graph spans are disjoint, while documentation source slicing and checking
still assume one file. Preserve attachment, signature, link and privacy validation.

1. Offset-aware source reads complete: all 11 documentation library tests, fmt and
   Clippy passed. Shifted attachment/link/signature and Unicode/CRLF checks passed.
2. File visibility and exported function parameter modeling complete. All 13
   documentation library and nine native documentation groups, fmt and Clippy passed.
3. Per-file initializer documentation checking complete. All 11 graph library and
   17 native file-module groups passed, alongside 16 library/nine native documentation
   groups, fmt and Clippy. Local E801/E802/E803 and file isolation are covered.
4. Imported value/type link checking complete. All 19 documentation library tests,
   fmt and Clippy passed, including facade aliases, private/inaccessible targets,
   distinct namespaces and private inferred record roots.
5. Add native diagnostics/execution boundaries and a runnable documented graph.
6. Document the supported slice and run the complete compiler gate.

Commit each tested slice before proceeding; split further if review thresholds require.
Multi-file site generation and public indexes remain separate.

## Current compiler slice

`modules/discover.rs` walks all supported AST statement/expression/type operands
iteratively, including function bodies, matcher branches, annotations and interpolation.
It collects relative Import nodes, ignores comment/plain-string text and sorts sites
by source position. Work is bounded to 262144 queued nodes per file; imports retain
the existing edge limit. Duplicate sites or exhausted work fail closed with B001.

`modules/load.rs` resolves this complete discovery result using the existing source
snapshots, canonical identities, manifest boundary checks and dependency ordering.
Inactive/unused imports still initialize before their importer/entry. Missing paths
and cycles remain E501/E502 at the real import site, including nested type operands.
No runtime loading, package resolution or general type evaluation is introduced.

`check/exports.rs::import_module` resolves only registered compile-time module
identities. Functions may bind local import aliases and use exported functions/types
without treating that identity as a captured local value. Runtime module-data reads,
borrowed module storage, private members and mutable/annotated identity bindings keep
their gates. Existing shared/exclusive call and returned-reference rules are unchanged.

Inline and nested initializer imports can select supported members. All discovered
inputs, including dependencies used only inside unused functions, remain protected
from build/IR output replacement. Initializer panic still prevents entry execution.
See [MODULES.md](MODULES.md) and [the example](examples/scoped-imports/main.mwy).

## Actual validation

- Discovery: three graph groups passed, including source order, nested expression/
  type traversal, shifted spans, large-tree budget failure and duplicate-site rejection.
  A native nested/inline initializer group passed in both profiles.
- Function-scope and boundary checks: seven native groups passed, covering local
  function/type aliases, repeated calls, capture/privacy gates, reference/exclusive
  calls, inactive/unused dependencies, cycles/missing paths, initializer panic,
  plain-string exclusion and hard-link protection.
- Broader file/module compatibility suites passed after integration. Focused fmt
  and Clippy passed for each slice.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 674 library and 637 native Rust tests (1311 total), 16 tooling plus
  4 compiler-harness Python tests, 79 standalone and four multi-file examples in
  debug/release. Gate log: `/tmp/meowy-scoped-imports-gate.log`.
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

| Responsibility | Existing owner |
| --- | --- |
| AST discovery and canonical graph ordering | `src/modules/discover.rs`, `src/modules/load.rs` |
| Source snapshots and graph compilation | `src/modules.rs` |
| Disjoint parser spans | `src/parser.rs::parse_documented_at` |
| Exported value/type namespaces, scopes and public signatures | `src/check/exports.rs`, `src/check/names.rs` |
| Data export gate and shared declaration checking | `src/check/statements.rs`, `src/check/functions.rs` |
| Complete graph checking and ownership | `src/check.rs::check_imports` |
| File-mapped diagnostics and input/output protection | `src/driver.rs` |
| Native file-site mapping and formatting | `src/backend/sites.rs`, `native/runtime.cpp` |
| Carried initialization and indexed access/write proof | `src/loans/emission_init.rs`, `src/loans/elements.rs`, `src/loans/control.rs` |

## Still outside this compiler

The full package/manifest graph, resource module values, generic specialization,
captures, public FFI, wider ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Plan support for documentation-bearing module graphs through `modules.rs::compile`,
   `documentation/model.rs` and `check/documentation.rs`. The current graph rejects
   any documentation blocks when multiple files are loaded. Preserve per-file spans,
   attachment/signature/link validation and module scope; do not simply ignore docs.
   Keep multi-file site generation/public indexes separate until their links and
   visibility are modeled. Add focused fixtures and commit each validated slice.
2. Preserve runtime data-capture and borrowed-export gates, package/manifest policy,
   generic type-evaluation boundaries and ownership/header proofs. Keep STATUS current,
   commit reviewable slices, and do not push or recreate STEP logs.
