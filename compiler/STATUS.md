# Compiler handoff and work tracker

Updated: 2026-09-10. Exported type aliases are complete and passed the compiler
gate. No failing checks remain. Four tested commits precede this separate
documentation handoff. Broader literal-import discovery is now in progress
under the commit plan below.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Planned commits

1. Bounded AST discovery and loader wiring pass two graph groups and one native
   nested/inline initializer group. Broader file regressions (11 library/18 native),
   fmt and Clippy pass. Ready to commit; function-scope resolution is next.
2. Resolve function-scope imports as compile-time module identities while retaining
   runtime data-capture restrictions; test local aliases, calls, types and privacy.
3. Qualify inactive/unused imports, nested cycles/errors, budgets and a runnable
   example, without weakening source/output protection or package policy.
4. Document the supported locations and run the compiler gate across the series.

Keep each commit buildable with focused tests and within the 400-line/8-file
split-review threshold. Do not add dynamic loading or general type evaluation.

## Current compiler slice

`StmtKind::TypeAlias` explicitly records whether `-><Name>:` exports the alias.
Parser extraction retains private-alias behavior and original spans. Exported
names must be unqualified; labeled exports are gated. `check/exports.rs` handles
alias declarations and enforces unconditional file-level scope for exported types.

Each module retains separate `types` and `values` namespaces. `<module.Type>` resolves
only exported type specifications; private/missing names use E202 and duplicate
bindings use E203. Data/function exports may share a name with a type. No runtime
field, storage or wrapper is created for a type declaration.

Resolved specifications retain structural alias identity, normalized record fields,
mutability and existing nominal foundation tags. Type/signature copying charges
existing proof work. Callable signature aliases work in typed function re-exports;
stored function pointers remain gated. Existing computed type values can define
aliases, without enabling general type-expression evaluation.

Standalone documentation preserves type roles, spans, checked signatures and E803
public-doc policy. The typed geometry facade combines type, data and function exports.
Private type names, runtime module-data captures, borrowed module storage, unsupported
owning storage and package/manifest features retain their boundaries. See
[MODULES.md](MODULES.md#exported-type-aliases) and
[the example](examples/type-modules/main.mwy).

## Actual validation

- Parser slice: two focused groups, all 23 parser tests, documentation/module
  regressions, formatting and Clippy passed before its commit.
- Namespace slice: privacy, duplicates, separate type/value names, function-local
  type use and scope gates pass; parser/file suites and fmt/Clippy passed.
- Identity/integration: four library and six native exported-type groups pass,
  including transparent facades, HIR nominal/primitive identity, computed aliases,
  callable signatures, record permissions, reference bounds and the geometry example.
  Native execution runs in debug/release where applicable.
- Standalone documentation checks/builds, checked type signatures and missing-doc
  E803 policy pass.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 671 library and 629 native Rust tests (1300 total), 16 tooling plus
  4 compiler-harness Python tests, 79 standalone and three multi-file examples in
  debug/release. Gate log: `/tmp/meowy-type-exports-gate.log`.
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
| Canonical graph, snapshots, bounds and dependency order | `src/modules.rs`, `src/modules/load.rs` |
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

1. Plan broader literal import discovery through `modules/load.rs`, the AST and
   `check/names.rs::symbol`. The loader currently scans only top-level immutable
   bindings. Traverse supported expression/type operands with bounded work and
   preserve source order, canonical identity, cycles and once-only initialization.
   Qualify additional import locations in small tested slices; preserve runtime
   module-data capture and package/manifest gates.
2. Keep general type evaluation, generic specialization, callable storage and borrowed
   module values separate until their rules are proved. Preserve ownership/header
   certificates, call/input opacity and owner expiry. Keep STATUS current, commit
   validated slices as they finish, and do not push or recreate STEP logs.
