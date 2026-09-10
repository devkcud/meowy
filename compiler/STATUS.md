# Compiler handoff and work tracker

Updated: 2026-09-10. Native panic file labels are complete and passed the compiler
gate. No failing checks remain. Implementation was split into four tested commits;
this handoff accompanies the separate documentation commit.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Commit series

1. `2c57546` — runtime file-site helpers and ABI/captured-cause tests.
2. `407688f` — validated backend source ranges and explicit P006 mapping/tests.
3. `6f38463` — P001/P002/P003 mapping and evaluation-order tests.
4. `eb6edf4` — multi-file driver integration and CLI regressions.
5. Documentation/handoff records the successful final gate below.

Each implementation commit includes focused tests and stays below the split-review
threshold. Continue planning commit slices before implementation; do not bundle
future module/export work into one feature-sized commit.

## Current compiler slice

Multi-file native P001/P002/P003/P006 failures now print canonical source paths and
local half-open byte ranges. The driver passes its source snapshots to
`backend::emit_ir_with_sources` only for graphs with multiple files. One-file CLI
programs and `emit_ir` retain their exact prior numeric-offset format.

`backend/sites.rs` validates ordered disjoint ranges, at most 64 labels and 1 MiB
of label bytes; unmapped or cross-file failure spans reject lowering. Used labels are
embedded once per source. Runtime helpers quote/escape paths without allocation or
mutable global source context, and preserve UTF-8 names. Legacy helper signatures
remain available. Panic copies retain file-site evidence and original cleanup causes.

Arithmetic operands, list bounds/capacity checks and explicit panic interpolation
keep their evaluation order and existing failure codes. An inner failure keeps its
own site; outer unfinished panic text cannot replace that cause. Initializer failure
still prevents entry effects. No runtime filesystem lookup, package/export support,
public artifact format or full native stack diagnostics are added.

Relative imports retain immutable reference-free exports, isolated private scopes,
canonical graph identities and ordered once-only initialization. Compiler diagnostics
and dependency output protection are unchanged. Function/type exports, borrowed
module storage, runtime module captures and package/manifest support remain gated.
See [MODULES.md](MODULES.md) for the format and current limits.

## Actual validation

- Runtime helper probes passed in debug/release: legacy and named formats,
  escaped UTF-8 labels, panic copying and retained original causes after cleanup failure.
- Three explicit-panic groups and two checked-operation groups passed in both
  profiles: local ranges, invalid-map rejection, deduplicated labels, nested faults,
  arithmetic causes, bounds prefixes, capacity and completed operand effects.
- All 82 backend tests passed after lowering integration. Five CLI groups passed
  in both profiles: all four codes, entry/dependency sites, canonical escaped aliases,
  diamond initializer failure and unchanged one-file output. Focused fmt/Clippy passed.
- `python3 -B tools/verify.py --compiler`: all 10 checks passed, including fmt,
  Clippy, build, 666 library and 612 native Rust tests (1278 total), 16 tooling plus
  4 compiler-harness Python tests, 79 standalone examples and one multi-file example
  in debug/release. Gate log: `/tmp/meowy-panic-file-sites-gate.log`.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  cases do not count as language rejections or full release qualification.
- Local links, 23 catalog records, 7 schemas/6 examples and whitespace checks passed.
  External links were not fetched. The compiler's native runtime helpers changed
  and were executed; separate prototype runtime/sanitizer and editor gates were
  not rerun.

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
| Native file-site mapping and formatting | `src/backend/sites.rs`, `native/runtime.cpp` |
| Carried initialization and indexed access/write proof | `src/loans/emission_init.rs`, `src/loans/elements.rs`, `src/loans/control.rs` |

## Still outside this compiler

The full package/manifest graph, richer module exports, generic specialization,
captures, public FFI, wider ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Plan small slices for annotated function/type exports through `check/names.rs`,
   `check/statements.rs`, `parser/statements.rs` and the module contract. Preserve
   canonical item identity, public annotations, private scopes and initialization
   order. Add focused multi-file fixtures with each implementation slice, then run
   the final compiler gate. Package policy remains separate.
2. Keep runtime module captures/reference exports gated until their storage/lifetime
   proof exists. Preserve ownership/header certificates, call/input opacity, old
   copies and owner expiry. Keep STATUS current, commit validated slices as they
   finish, and do not push or recreate STEP logs.
