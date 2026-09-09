# Meowy project status

Updated: 2026-09-09. This is the current project handoff; Git retains prior work.
[COMPILER.md](COMPILER.md) holds the implementation plan and
[compiler/STATUS.md](compiler/STATUS.md) the detailed compiler handoff.
Do not recreate STEP logs. The full documented v0.0.1 release remains incomplete.

## Current milestone

Local exclusive borrows of Boolean, integer and float fields in carried records
are implemented and passed the compiler gate. Exact mutable field paths use the
existing containing-slot initialization and storage-lifetime proofs. Exclusive
loans and their descendants must end before every reachable restart edge.
Nested fields, disjoint siblings, old copies, moves, children, calls, shared headers,
owner resets and Leave have source/proof and debug/release native coverage.
Whole-record and non-scalar exclusive paths remain gated.

Declared reference-free records retain whole-slot initialization across inner
restarts; shared record references and projected reborrows can survive those
restarts while their result owner lives. Record shape eligibility remains bounded
to 256 type parts and 32 levels. No backend, runtime or dependency changes were
needed for the exclusive-field extension.

The bounded [documentation slice](COMPILER.md#documentation-completion-slice) is
complete for standalone bootstrap sources: structural attachment, derived
signatures, checked links, diagnostics, local API pages and checked/opt-in examples.
The net/HTTP specification merge is complete: `@"net"` owns transports and
capability-typed peers; `net.http` supplies sender/receiver adapters. TCP/UDP keep
their native semantics. Networking, HTTP and TLS are specified library work,
not executable implementations.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  formatting, Clippy, build, repository contracts and compiler regression coverage.
- 1139 Rust tests passed: 587 library and 552 native. The 72 compiler examples run
  in debug and release. Python coverage is 16 tooling and 4 compiler harness tests.
- Focused exclusive carried-record coverage passed all 10 source/proof groups and
  6 native groups, with native output/diagnostics checked in both profiles.
- Conformance: 10 passed, 13 unsupported, 0 failed in debug and release. Unsupported
  cases are not successful language rejections or full release qualification.
- Final documentation check passed 1077 local links in 102 Markdown files;
  `git diff --check` passed. Repository checks also cover 23 conformance catalog
  records and 7 schemas/6 examples. External links were not fetched.
- Editor integration and the separate runtime/sanitizer gate were not rerun;
  no editor, runtime, backend or dependency files changed.

## Area handoff

| Area | Current boundary |
| --- | --- |
| Compiler | Carried plain-record initialization, shared projections and local exclusive scalar fields are supported. Wider carried shapes remain separate. |
| Documentation tooling | Standalone slice complete; package graphs, assets, public indexes and LSP remain separate. |
| Editor integration | Documentation fences supported in Vim/Neovim; no changes in this slice. |
| Standard library | One net package specifies peers and HTTP adapters; module/type/I/O/task foundations precede implementation. |
| Runtime and release | Bootstrap evidence is not minimum-platform, bundled-distribution or full v0.0.1 qualification. |

## Next steps

1. Investigate declared fixed-capacity, reference-free carried list slots through
   `compiler/src/borrow/carried.rs`, `compiler/src/check/statements.rs` and
   `compiler/src/loans/emission_init.rs`. Establish whole-slot initialization,
   copies, replacement and owner-reset behavior before changing the shape gate.
   Qualify indexed borrowing and reservations separately through existing element
   proofs; nullable, union, reference-bearing and owning slots remain separate.
2. Preserve exact field sources, active initialization, shared-header certificates,
   conservative call/input ancestry, exclusive reset frontiers, old-copy loans and
   no synthetic reads. Add focused source/native proof for any new capability and
   run the compiler gate.
3. Continue module graphs and library foundations before executable net peers/TLS;
   implement capability-typed configuration, bounded lifecycle and raw adapters
   before HTTP sender/receiver adapters, following COMPILER.md.
4. Keep root/compiler STATUS concise with actual evidence and concrete next steps;
   commit cohesive validated changes, never recreate STEP logs and do not push.
