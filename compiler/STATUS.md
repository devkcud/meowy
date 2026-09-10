# Compiler handoff and work tracker

Updated: 2026-09-10. Bounded relative value imports are complete and passed the
compiler gate. No failing checks remain.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Current compiler slice

The driver loads a bounded canonical file graph through `modules.rs` and
`modules/load.rs`. Top-level immutable unannotated bindings can import exact
relative `.mwy` paths, with grouping allowed. Paths resolve from each importer;
canonical paths/symlinks deduplicate diamonds. E501 covers invalid/missing files;
E502 identifies a cycle and its closing import. Files are snapshotted before checking.

Each file is parsed separately into disjoint source-span space. The graph assembles
isolated AST blocks with inaccessible internal bindings in dependency/source order;
source text is not concatenated. `check::check_imports` runs the complete existing
checker/ownership pipeline. `Value::FileModule` preserves compile-time alias identity
and one runtime initializer while exposing immutable reference-free value exports.
Private scopes remain separate. Dependencies initialize before importer/entry effects;
initializer panic prevents entry execution.

`driver::report_at` maps compiler errors to the actual file and local UTF-8 byte
range, line and column. All graph source paths participate in output protection.
The one-file library API and standalone documentation retain their prior path.
Native panic text still prints internal graph offsets; runtime file labels and
local runtime-site mapping are not implemented and must not be claimed.

Limits: 64 files, 32 active import levels, 4096 edges, 4 MiB per file and 16 MiB
snapshot span space. Export shapes reuse 256-part/32-level reference-free validation.
Package manifests remain refused unless the entry policy is explicitly bypassed
with `--standalone`; imported files cannot cross a different manifest context or
execute `mod.mwy`. Foundational imports are unchanged. Function/type exports,
mutable/reference-bearing exports, nested/conditional imports, module references,
module values in function bodies and documentation graphs stay gated. Type-export
syntax now reports B001 explicitly. See [MODULES.md](MODULES.md) and
[the runnable diamond](examples/modules/main.mwy).

## Actual validation

- `cargo test --locked --manifest-path compiler/Cargo.toml --target
  x86_64-unknown-linux-gnu --target-dir compiler/target file_modules` passed eight
  graph/checker groups and nine native groups, including the multi-file example.
  Native success/failure cases execute in debug and release where applicable.
- Evidence covers canonical diamonds/symlinks, importer-relative resolution,
  once-only initialization, private scopes, immutable records/lists/primaries,
  initializer panic, Unicode/interpolation compiler-error mapping, source snapshots,
  graph/source limits, export/context gates and protection of dependency inputs.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 659 library and 607 native Rust tests (1266 total), 16 tooling
  plus 4 compiler-harness Python tests, 79 standalone examples and one multi-file
  example in debug/release. The added dependency-hard-link protection case and
  Clippy passed afterward.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  cases do not count as language rejections or full release qualification.
- Local links, 23 catalog records and 7 schemas/6 examples passed. Whitespace checks
  passed; external links were not fetched. Gate log: `/tmp/meowy-file-modules-gate.log`.
- No backend, runtime, dependency or reference-fixture changes were needed. Editor
  and separate runtime/sanitizer gates were not rerun. Runtime panic file labels
  remain open.

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
| Module identities and immutable export gate | `src/check/names.rs`, `src/check/statements.rs` |
| Complete graph checking and ownership | `src/check.rs::check_imports` |
| File-mapped diagnostics and input/output protection | `src/driver.rs` |
| Carried initialization and indexed access/write proof | `src/loans/emission_init.rs`, `src/loans/elements.rs`, `src/loans/control.rs` |

## Still outside this compiler

The full package/manifest graph, richer module exports, generic specialization,
captures, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Carry file identity into native panic sites through `src/modules.rs` and
   `src/backend/{panic,arithmetic,lists}.rs` and `native/runtime.cpp`. Preserve
   existing one-file diagnostics;
   add multi-file bounds/arithmetic/panic tests with local spans and file names.
   Do not describe the current graph byte offsets as complete runtime diagnostics.
2. Extend module exports toward annotated functions/types through `check/names.rs`,
   `check/statements.rs`, `parser/statements.rs` and the module contract. Preserve
   canonical item identity, public annotations, private scopes and initialization
   order; do not enable runtime module captures or reference exports without their
   storage/lifetime proof. Package manifests/aliases remain a separate slice.
3. Preserve ownership/header certificates, call/input opacity, old copies and owner
   expiry. Keep root/compiler STATUS current, commit cohesive validated work, and
   do not push or recreate STEP logs.
