# Compiler handoff and work tracker

Updated: 2026-09-09. Carried reference-free list initialization is complete and
passed the compiler gate. No failing checks remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Current compiler slice

`borrow/carried.rs::shape` now classifies eligible storage as Plain or List while
retaining the existing 256-part/32-level bounded traversal. A list adds one container
and visits its element type once, independent of capacity. Nested scalar/unit/list/
record elements are supported; nullable/union/reference/foundation/owning shapes
remain gated, even under zero capacity. Existing `list.rs` checks still enforce
capacity at most 65,536, layout at most 1 MiB and one-based initialized-length bounds.
The frontend's `eligible` interface remains unchanged.

The existing `loans/emission_init.rs` state proof tracks the whole initialized
length/payload slot. It preserves ancestor slots across inner restarts and clears
owner slots on reset. Empty lists still need one completed emission; no per-element
initialization or synthetic reads were added. Copies retain independent values,
whole replacement and `.add()` results keep existing type/length semantics, and
record list fields can be replaced without indexed reservations. Named fields and
primary list slots use the same initialization proof.

`carried::storage` keeps original list-containing carried storage accesses gated
at three entry points: ordinary Acquire, exclusive indexed acquisition and SetPath
indexed writes. The latter has its own reservation path and cannot be guarded only
at Acquire. Gates also cover scalar-field borrows from a record containing a list.
Independent copies, completed-result locals and plain scalar/record sibling slots
retain existing rules. These restrictions must remain until their separate
acquisition/reservation proof is qualified.

See [the contract](OWNERSHIP.md#carried-reference-free-lists) and
[carried-lists.mwy](examples/carried-lists.mwy). No AST/HIR, backend, runtime, editor,
reference conformance fixture or dependency changes were needed.

## Actual validation

- The first copy/length/replacement regression reproduced B001 at the former
  enclosing-emission restart gate before implementation.
- Focused `cargo test --locked --manifest-path compiler/Cargo.toml --target
  x86_64-unknown-linux-gnu --target-dir compiler/target carried_lists` passed
  10 source/shape/proof groups and 7 native groups, with native cases in both profiles.
- Source/proof coverage includes nested/empty/primary lists, records with list
  fields, independent copies, whole replacement/addition, owner resets, Leave,
  incomplete/duplicate initialization, type/mutability errors, shape/depth/work
  limits and storage/reservation gates. Empty-list graph evidence rejects removed
  or repeated Emit events and confirms no synthetic loan-value uses or acquisitions.
- Native checks preserve length/payload, dynamic bounds after conditional replacement,
  once-only initializer effects, old copies, ordinary sibling loans, owner reset,
  Leave, partial panics and primary diagnostics. Only obsolete list-rejection
  fixtures were removed after focused source/native proof passed.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 604 library and 565 native Rust tests (1169 total), 16 tooling
  plus 4 compiler-harness Python tests, and 74 examples executed in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities remain outside the full language gate.
- Repository checks cover local links, 23 catalog records and 7 schemas/6 examples.
  Local-link and whitespace checks passed; external links were not fetched.
- Editor and separate runtime/sanitizer gates were not rerun; their code is unchanged.

## Prior capabilities and other areas

The [pointer syntax migration](BORROW_SYNTAX.md) is complete: tight prefix
`&`/`&!`/`*`, immediate selected-field `.&`/`.&!`/`.*`, grouping for complete targets.
The preceding slice passed Vim/Neovim and source migration preservation checks.

Plain carried records retain shared projections/reborrows while their owner lives.
Local exclusive Boolean/integer/float fields use exact mutable paths and Acquire;
every exclusive loan/descendant must end before restart. Shared-header certificates,
conservative call/input ancestry, old copies, mutability, source expiry and
E301/E302/E303 remain intact. Whole-record/non-scalar exclusive values, nullable/
union/reference-bearing/foundation/owning and top-level unit carried slots remain gated.

Standalone documentation supports attachment, checked links/signatures, E801-E805,
`doc check`/`doc build`, local API pages and checked/opt-in examples. Net/HTTP/TLS
remain specified library work; module/type/I/O/task foundations, capability types
and lifecycle implementation precede executable adapters.

## Architecture and proof boundaries

| Responsibility | Existing owner |
| --- | --- |
| Pointer syntax and precedence | `src/parser/expressions.rs` |
| Carried shape and collection access gate | `src/borrow/carried.rs` |
| Frontend carried obligations | `src/check/statements.rs` |
| Whole-slot initialized/active state and Acquire | `src/loans/emission_init.rs` |
| Shared direct/projected acquisition | `src/loans/transitive.rs` |
| Exclusive indexed acquisition and indexed writes | `src/loans/elements.rs`, `src/loans/control.rs` |
| Shared headers and exclusive restart frontier | `src/loans/restart_headers.rs`, `src/loans/exclusive_restarts.rs` |

## Still outside this compiler

The full module/package graph, generic specialization, captures, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Qualify shared carried-list storage/element borrows through
   `loans/transitive.rs::referenced`, `loans/emission_init.rs::emission_acquire`,
   `list.rs::element_borrow` and `borrow/carried.rs::storage`. Separate shared
   acquisition from exclusive indexed acquisition/SetPath gates; preserve those
   latter restrictions until their reservation/lifetime proof is qualified.
2. Require active whole-slot initialization, actual initialized-length checks,
   precise projected sources, parent authority and owner expiry. Add source/native
   cases for inner restarts, nested lists/record fields, old copies, last use,
   owner completion/reset, Leave and conflicting replacement. Run focused tests
   and `tools/verify.py --compiler` before relaxing the shared-acquisition gate.
3. Keep broader carried unions/references/owning cleanup and exclusive header
   carriage separate. Continue module graphs/library foundations; optional doc
   polish does not block compiler work. Write new fixtures with migrated pointer
   syntax and derive byte-span expectations from actual source text.
4. Keep root/compiler STATUS current after logical steps, commit cohesive validated
   work and do not push or recreate STEP logs. Full release qualification remains open.
