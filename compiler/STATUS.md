# Compiler handoff and work tracker

Updated: 2026-09-06. Guarded reference results and native cleanup integration pass.
Full v0.0.1 remains incomplete. The implementation milestone is complete; its remaining limits are recorded below.
This file tracks compiler implementation; [../STATUS.md](../STATUS.md) tracks the wider project.
Historical checkpoints and completed checklists are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
Tracker split verified: historical entries and checklists are preserved exactly;
834 local links and diff whitespace checks pass. No implementation changed.
Baseline commits: `54080b1` (compiler), `c82354f` (tests/examples), `f2feea2` (docs).
Union commits: `8209163` (compiler/guard behavior), `177169e` (native coverage/example),
`f749708` (handoff). Reference commits: `71a7baf` (compiler), `064e305` (coverage).
Repository work: `122b022` (schema examples), `6f6a0c1` (verification tooling).
Guarded results: `5e7ad41`. Native cleanup prototype: `e0987be`. Runtime gate: `1ba3c65`.

## Current objective

Completed: guarded bare-reference block results preserve every origin and exclude
proven discarded emissions. Native and release examples pass. The independent
`../runtime/` cleanup prototype and combined repository gate also pass. Next:
CFG loan liveness, aggregate/function borrow contracts and native context/unwind
qualification before mutable/exclusive loans, owners or generated cleanup.
The existing reference remains authoritative. Do not change fixtures to make tests pass.

## Resume here

1. Read this file, `README.md`, `AGENTS.md`, and `../COMPILER.md`.
2. Inspect `git status --short` and recent commits; preserve existing work.
3. Run `cargo test --manifest-path compiler/Cargo.toml` from the repository root.
4. Run `python3 compiler/tests/conformance.py`; 14 unsupported cases are currently expected.
5. Consult the validation evidence below before claiming any gate passed.
6. After every compiler work step, update `Next steps` and add a checkpoint to
   `STATUS_STEP_LOG.md` before continuing. Include investigations, edits, checks, and decisions.

No dependency downloads or remote services are needed by the current Rust design.
The bootstrap uses installed LLVM/Clang/LLD 22.1.8 and Rust 1.98.1. It does not yet
provide a bundled sysroot or a qualified distribution. Never claim the documented
Linux 5.4/glibc 2.31 execution baseline from this workstation build.

## Implementation map

| Component | Files | Current state |
| --- | --- | --- |
| Workspace and interfaces | `Cargo.toml`, `rust-toolchain.toml`, `src/ast.rs`, `src/hir.rs`, `src/lib.rs` | Builds offline with no external Rust dependencies |
| Lexer and parser | `src/lexer.rs`, `src/parser.rs` | Implemented for the documented bootstrap subset; malformed-input and depth checks pass |
| Names, types, flow | `src/check.rs`, `src/flow.rs` | Scalar/record unions, branch narrowing and guarded emissions; 21 checker and 5 guard tests pass |
| Shared storage/borrow origins | `src/hir.rs`, `src/borrow.rs`, `OWNERSHIP.md` | Immutable local/record places, guarded block results and all-root escape checks; 8 source-test groups pass |
| Native backend | `src/backend.rs`, `build.rs`, `native/` | Verified LLVM → ELF pipeline including tagged unions; 14 focused backend tests pass |
| CLI and diagnostic rendering | `src/main.rs`, `src/driver.rs`, `src/diagnostic.rs` | Implemented and exercised through native tests |
| Native tests, examples, documentation | `tests/`, `examples/`, `README.md` | 29 native tests, 4 harness tests and 7 runnable examples |

Agents share this checkout. After a sudden stop, file existence is not proof that
a component compiles. The interfaces are `parser::parse`, `check::check`,
`backend::emit_ir`, and `backend::emit_object`. `src/lib.rs` composes the frontend.
Rust modules use explicit visibility where Rust allows it. Follow the user's
short-name, immutable-value, and no-comment conventions.

## Still outside this compiler

Items here must not be silently accepted with different semantics. `B001` is an
internal bootstrap capability diagnostic, not an assigned reference error code.
The supported portion of the language uses the reference's established `E...`
codes. The bootstrap's JSON diagnostic stream is not a release artifact schema.

| Area | Work remaining | Required evidence |
| --- | --- | --- |
| Full frontend | Complete grammar, stable item IDs, recovery CST/editor integration, all type forms | Conformance, compact syntax properties, malformed UTF-8 and parser fuzzing |
| Type system | Literal types/unions, subtraction, callable environments, generics/capabilities, full type queries, nominal identity | All type and callable fixtures plus negative boundary cases |
| Required evaluation | Type-producing helpers, effect analysis, cycle checks, logical budgets, specialization | E211/E219/E220 cases and determinism/budget tests |
| Ownership | Mutable/exclusive borrows, returned/temporary views, moves, partial initialization, captures and cleanup | Use-after-move and borrow rejection; exact-once cleanup on every exit |
| Collections | Bounded lists, arrays, slices, maps, vectors, allocators | Extent/count/bounds cases and allocation-failure behavior |
| Runtime | Owned allocations, recoverable panics, unwinding, tasks, channels, timers, cancellation | Task/unwind prototype, one-worker progress, cleanup and sanitizer tests |
| Modules/projects | Relative imports, manifest policy, exports, aliases, root locks, dependency graph | Worked projects, offline locked builds, duplicate/revision identity tests |
| Native ABI | Clang ABI adapters, explicit native artifacts, record classification | Separate C fixtures, argument/return layout, safety-boundary checks |
| Standard library | Meowy library sources and documented APIs beyond foundational bootstrap output | API-to-test coverage, all 16 worked projects, pinned Unicode/time/calendar data |
| Tooling | Test command, LSP, gatostyle/fmt, inspection and repair workflows | Shared diagnostics, negotiated positions, safe rewrites and isolated test cases |
| Artifacts/replay | Canonical schemas/readers, sessions, capsules, integrity, runtime events/replay | Schema validation, corruption/budget tests, replay after moving original sources |
| Distribution | Maintained bootstrap recipes/xtask, pinned source inventory, vendoring, bundled tools/sysroot | Offline rebuild, dependency/license inventory, descriptor digests |
| Target qualification | Baseline host, static/shared closure, LTO modes, DWARF 5 and unwind information | ELF inspection, minimum-host execution, reproducible builds and size measurements |

## Known bootstrap limits worth preserving during handoff

- Shared borrowing is limited to immutable ordinary local roots and concrete
  record fields. Parameter/receiver/emission places, mutable roots, temporary
  owners, exclusive loans, reference-carrying aggregates/signatures and reference
  reassignment are B001. `OWNERSHIP.md` records the remaining stages.
- Bare-reference block results support guarded alternatives and named exits.
  Every retained origin must outlive its receiving block. Discarded emissions keep
  operand effects but do not escape. Reference-carrying aggregates are still B001.
- Assigning predicates invalidates prior facts. Audit found 15 safe mutated-predicate
  combinations conservatively rejected with E303; stronger relation/dataflow
  tracking is pending. No unsafe acceptance was found in 288 bounded combinations.
- `../runtime/` proves an explicit cleanup protocol only. Generated programs still
  use the scalar runtime. Task stacks, Meowy personality/landing pads, child joins
  and cross-stack partial-result ordering must be implemented before integration.

- Only direct, noncapturing functions are lowered. A function declaration can be
  aliased, but first-class function-pointer storage and anonymous/indirect calls remain unavailable.
- Record fields remain immutable. Nullable primary/field joins and scalar/record
  union predicates now work. Mutable bindings can replace records; this invalidates
  field proofs. Proof caches include both place and storage type across variants.
- Multiple matching numeric widths in an expected union report E207 rather than
  choosing one. Bind an explicitly typed member first; no width conversion is implicit.
- Interpolation streams only at print/panic boundaries. It does not construct a
  heap string or a general-purpose standalone string value.
- Runtime arithmetic and explicit panic report P002/P006 and exit. Their diagnostics
  still need source locations and operand evidence, and recoverable unwinding is absent.
- Integer remainder at signed minimum divided by -1 is zero; signed division at
  the same boundary panics. Preserve this distinction in both profiles.
- Float32 literals are parsed directly to float32, avoiding double rounding through float64.
- Do not infer a whole block's constant from its last emission: earlier paths may
  leave with another value. A regression covers this previous bug.
- Shared boolean guards replace the former 4096-path enumeration limit. Node,
  cache and work budgets in `src/flow.rs` produce B001 when exhausted. Named-block
  entry conservatively forgets mutable proofs; restart with writes into surviving
  outer result slots reports B001. Full loop fixed points and ownership MIR remain pending.
- The compiler embeds its runtime archive but depends on the installed LLVM shared
  library and exact native tool paths. It is not relocatable/offline distribution packaging.
- LLVM IR is verified before and after optimization. The bridge copies explicitly
  bounded source bytes into an owned LLVM buffer; preserve that FFI boundary.
- Broad fuzzing, full source/input resource limits, canonical artifact diagnostics,
  rich related spans, source-level debug information, and failure capsules are pending.
- A 65,536-parameter stress input timed out after 60 seconds during review; broad
  frontend scaling is not qualified. Compact correlated-guard exhaustion returns B001.
- The parser limits recursive descent and caps constructed AST depth at 256 with
  B001. A 20,000-term expression previously overflowed the checker stack; regression
  coverage now verifies graceful rejection. Keep the depth guard before recursive passes.
- Conformance supervision kills the whole compiler/application process group on
  timeout. A B001 result cannot hide a later compiler fault or count as a strict pass,
  even when its message is empty. Harness regressions protect these distinctions.

## Validation evidence

- 2026-09-06: repository was clean before implementation; no prior compiler existed.
- 2026-09-06: detected Rust 1.98.1, LLVM/Clang/LLD 22.1.8, CMake 4.4.3,
  Ninja 1.13.2, Python 3.14.7 on the current host.
- Previous implementation gate: `python3 -B tools/verify.py --all` passed all 14 checks outside
  ptrace supervision. It pins the Cargo target/output path and freshly built compiler.
- Cargo tests: 64 library and 29 native tests passed, including both output profiles.
- Repository checks: 16 tooling and 5 runtime Python tests, 829 local links,
  schema/catalog integrity, Vim and Neovim passed. Native cleanup passed 14 normal
  cases plus 2 fatal probes in each debug/release/ASan/UBSan/LSan profile.
- Sandbox LeakSanitizer failed due to ptrace; the unchanged full gate passed with
  the required outside-sandbox execution. No sanitizer check was disabled.
- Final `python3 -m unittest discover -s compiler/tests -p 'test_*.py'`: 4 harness tests passed.
- Final `cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings`
  and `cargo fmt --manifest-path compiler/Cargo.toml --check` both passed.
- Compiler conformance: 9 passed, 14 unsupported, 0 failed, both profiles.
  Passed: `compact_min`, `minimum_parenthesized`, `invalid_separator`, `forward_group`,
  `forward_interrupted`, `conditional_field`, `scalar_projection`, `function_equality`,
  `reference_identity`.
  This is NOT full conformance.
- Parser tests retain 10,000 deterministic malformed/Unicode inputs and long-chain
  stress regressions. These are smoke coverage, not comprehensive fuzz qualification.
- Backend focused validation: bounded FFI inputs, invalid LLVM syntax/SSA, ELF emission,
  integer boundaries, string/numeric output, short circuiting and `/dev/full` all passed.
- Checker focused validation: 21 tests passed, including nullable slot inference,
  complementary predicates, field narrowing, assignment invalidation, record composition
  and restart boundaries. Eight borrow source-test groups cover all candidate
  roots, lexical aliases, retained escapes, function isolation and discarded paths. Five guard tests
  include 16,384 independent variables.
- The reference catalog validator passed all 23 records; this was metadata validation only.
- Release compiler build with the explicit host target succeeded. Its borrowed
  results example prints exactly `11\n22\n42\ntrue\n` in release. The earlier
  references example also passed with `true\nfalse\n42\nmeowy\n`.
  The prior release compiler also ran `examples/nullable.mwy` with exact stdout
  `meowy\nnull\nmeowy\nguest\n`. The earlier records example also passed.
- Actual `readelf -h/-d/--version-info` inspection of `build/records` found ELF64
  x86-64 PIE, interpreter `/lib64/ld-linux-x86-64.so.2`, and only `libc.so.6` in
  DT_NEEDED. It requires GLIBC_2.34; the documented glibc 2.31 target is NOT qualified.
  No LLVM, Rust, or C++ runtime library appears in that executable's dependency list.

## Next steps

1. Add an explicit storage/control-flow graph for `src/borrow.rs` with backwards
   last-use liveness and loop fixed points. Preserve current guards and improve
   predicate assignment relations. Test valid post-use mutation and exact E302
   conflicts before accepting mutable/exclusive roots or reference reassignment.
2. Extend origin sets to reference-carrying record/union components and function
   input/result contracts. Verify all-input lifetime bounds and caller substitution
   with accepted nested returns and E303 rejections; keep unknown origins B001.
3. Add partial initialization, capture summaries and cleanup lowering; then enable
   bounded collections/slices and the owner-borrow fixtures. Test exact-once cleanup
   and temporary-owner rejection. Do not mark unsupported tests as passing rejections.
4. Add required evaluation, effects, logical budgets and constrained specialization.
   Then enable the type-helper, compile-effect/budget, and callable fixtures.
5. Extend `../runtime/` beyond explicit cleanup: qualify the pinned context wrapper
   with bounded stacks/guard pages and sanitizer hooks, then Meowy DWARF personality
   and landing pads. Test a suspended borrowing child that is cancelled and joined
   before parent cleanup. Preserve interleaved partial-result/local cleanup order.
6. Implement project/module graphs, native adapters and Meowy standard-library
   layers, then tools/artifacts and distribution qualification from the table above.
7. Keep the bootstrap gate green; expand the harness's REQUIRED set when a case
   becomes supported. Only `--strict` with zero unsupported cases passes the catalog's
   full-language gate, and even that catalog is only part of v0.0.1 qualification.
