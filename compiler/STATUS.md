# Compiler handoff and work tracker

Updated: 2026-09-10. Annotated function exports and typed re-exports are complete
and passed the compiler gate. No failing checks remain. Five implementation/test
commits precede this documentation handoff. Exported-type work is now in progress
under the commit plan below; aliases must retain their existing structural identity.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Planned commits

1. Exported aliases have an explicit AST flag and share private-alias parsing.
   Two focused groups, all 23 parser tests, documentation/module regressions, fmt
   and Clippy pass. Ready to commit; semantic use stays gated until integration.
2. Add a separate exported type namespace and qualified type lookup. Support
   standalone/imported aliases with privacy, duplicate and scope regressions.
3. Qualify transparent alias/re-export identity, including computed type context,
   callable signatures, existing nominal types and value/type name separation.
4. Exercise documentation and a runnable typed facade with focused integration tests.
5. Update docs/handoff and run the compiler gate across the complete series.

Keep syntax, namespace behavior, integration and documentation reviewable. Include
focused tests in each commit and stay within the 400-line/8-file split threshold.
No package policy, generic specialization or runtime module-capture expansion.

## Current compiler slice

`check/exports.rs` tracks each file's compile-time function exports separately from
its runtime data fields. `declare_function` is shared with ordinary private
bindings. Named top-level `->f<Result>:(arg<Type>){...}` definitions require an
explicit public signature (E214), preserve recursion/private helper access and reuse
existing global function IDs. Member lookup exposes only exported names.

`->alias<(Args)->Result>:module.function` re-exports an existing identity with an
exact full signature (E207 on mismatch). Ordinary names must remain fresh (E203),
while duplicate exports and function/data collisions use E205. Result-slot checking
also catches unnamed record spreads; unreachable data emissions keep their rules.
Matcher-arm/nested exports remain gated by file-frame and scope-depth checks.
Function equality remains E222; HIR regressions verify shared/distinct call IDs.

Every graph file, including the entry, uses an isolated initializer binding;
standalone file roots also support function definitions. Dependency data remains
immutable/reference-free, while entry data retains its previous permissions.
Functions are compile-time namespace entries, not runtime callable fields. No HIR,
backend, runtime or dependency changes were required for calls.

Existing argument checks, shared/exclusive authority, public all-input bounds,
private-storage escape rejection, initialization order and callee panic sites remain
in force. Runtime module-data captures, borrowed module storage, exported types and
package/manifest features stay gated. See [MODULES.md](MODULES.md#annotated-function-exports)
and [the facade example](examples/function-modules/main.mwy).

## Actual validation

- Refactor: 14 library/35 native function groups and 47 checker tests passed.
- Definition/re-export work: file regressions, E214/E207/E203/E205/E222 boundaries,
  HIR call-ID identity, privacy, capture and cycle checks passed. Scope-depth
  validation rejects conditional function exports.
- Record-spread collision reproduced an incorrectly accepted duplicate; the shared
  result-slot check now rejects it and preserves unreachable emissions.
- Four cross-module call groups pass in debug/release: mixed data/function facade,
  recursion, shared returns, exclusive arguments, all-input bounds, private-storage
  escape and callee panic sites. Focused fmt/Clippy checks passed for every slice.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 667 library and 621 native Rust tests (1288 total), 16 tooling plus
  4 compiler-harness Python tests, 79 standalone and two multi-file examples in
  debug/release. Gate log: `/tmp/meowy-function-exports-gate.log`.
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
| Module function identities, scopes and public signatures | `src/check/exports.rs`, `src/check/names.rs` |
| Data export gate and shared declaration checking | `src/check/statements.rs`, `src/check/functions.rs` |
| Complete graph checking and ownership | `src/check.rs::check_imports` |
| File-mapped diagnostics and input/output protection | `src/driver.rs` |
| Native file-site mapping and formatting | `src/backend/sites.rs`, `native/runtime.cpp` |
| Carried initialization and indexed access/write proof | `src/loans/emission_init.rs`, `src/loans/elements.rs`, `src/loans/control.rs` |

## Still outside this compiler

The full package/manifest graph, type/resource module exports, generic specialization,
captures, public FFI, wider ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Plan small exported-type slices from `parser/statements.rs`, `ast.rs`,
   `check/names.rs` and `check/exports.rs`. Represent the type namespace explicitly,
   preserve private aliases/canonical identity and support the documented `-><Name>:`
   boundary without inventing syntax. Include parser and multi-file tests with each
   implementation step, then run the compiler gate. Package policy stays separate.
2. Keep runtime module-data captures, borrowed exports and callable storage gated
   until their storage/lifetime proofs exist. Preserve ownership/header certificates,
   call/input opacity, old copies and owner expiry. Keep STATUS current, commit
   validated slices as they finish, and do not push or recreate STEP logs.
