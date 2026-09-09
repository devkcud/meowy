# Compiler handoff and work tracker

Updated: 2026-09-09. The borrow/dereference syntax migration is complete and passed
compiler/native/editor checks. No failing checks remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the implementation plan. Keep this handoff current; Git holds history.
Do not recreate STEP logs.

## Current compiler slice

The approved [migration plan](BORROW_SYNTAX.md) is implemented. Prefix `&`, `&!`
and `*` bind before subsequent postfix operations. The boundary propagates through
prefix chains; parentheses reset it. `&items[i]` is `(&items)[i]`, while `&(items[i])`
borrows the element. `*record.field` is `(*record).field`, while `record.*field`
dereferences the selected field. `.&`, `.&!` and `.*` consume one field name and
then resume the postfix chain. Calls and type suffixes follow the same boundary.

`parser/expressions.rs` lowers dotted operations into existing `Unary(Field)` ASTs.
The enclosing minimum binding strength now reaches prefix parsing, which prevents
nested prefixes from consuming the following postfix chain. Groups use normal
expression parsing. Names, comments/newline rules, depth/tree budgets, type syntax,
logical/bitwise operators and non-pointer unary behavior retain their contracts.
AST/HIR representations, ownership passes, backend, runtime and dependencies are
unchanged. No legacy parser mode or public migration command was added.

Repository source strings and dynamic fragments preserve their previous meanings
through explicit grouping. Standalone examples and documentation also use the new
dotted forms. Type references/task-group syntax remain distinct. Source-byte
expectations were updated from actual new source text. The new
[pointer-syntax example](examples/pointer-syntax.mwy) demonstrates reference/value
selection, exclusive field mutation, indexed borrowing and field dereferencing.

## Actual validation

- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed: fmt,
  Clippy, build, 594 library and 558 native Rust tests (1152 total), 16 tooling and
  4 compiler-harness Python tests, and 73 examples executed in debug and release.
- All 21 parser tests pass, including 7 new groups for pointer/postfix binding,
  mixed prefix chains, groups, calls/type suffixes, malformed dotted syntax,
  interpolation/Unicode spans, newline continuation and bounded nesting/chains.
- Six new native groups pass in both profiles: copy/reference distinctions,
  selected-field operations, nested/sibling loans, mutability/conflicts, once-only
  call/index effects, reference expiry and retained capability gates.
- Vim and Neovim suites pass with dotted/grouped forms, logical `&&` and nested
  reference-type fixtures. No syntax-engine rewrite was needed.
- Old/new AST comparison, ignoring Group nodes and spans, matched 962 migrated
  Rust snippets and 37 changed standalone sources. Nineteen generated Rust
  fragments needed lexical/manual review. Five missed dynamic path substitutions
  and one fixed byte-span length were repaired; existing expected behavior and
  primary diagnostic codes remain unchanged. A residual operand scan found none.
- Documentation audit: 33 aligned Markdown fences matched; 20 were unparseable
  before and after. Two changed standalone sources likewise retain their prior
  unsupported parse boundaries. All seven isolated migrated expressions from
  those unsupported examples matched. This is preservation evidence, not execution.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities remain outside the full language gate.
- Final local-link and whitespace checks passed: 1083 links in 103 Markdown files
  and `git diff --check`. Repository contracts also cover 23 catalog records and
  7 schemas/6 examples. External links were not fetched. The separate runtime/sanitizer gate
  was not rerun; no runtime/backend or reference conformance fixture files changed.

## Prior capabilities and other areas

Carried reference-free records retain whole-slot initialization across inner
restarts within 256 type parts and 32 levels. Shared projections/reborrows survive
while their result owner lives. Local exclusive Boolean/integer/float fields use
exact mutable paths and containing-slot Acquire; every exclusive loan/descendant
must end before restart. Shared-header certificates, conservative call/input
ancestry, old copies, mutability, source expiry and E301/E302/E303 remain intact.
Nullable, union, list, reference-bearing, foundation and top-level unit carried
slots remain gated. Whole-record/non-scalar exclusive values remain unsupported.

Standalone documentation supports attachment, checked links/signatures, E801-E805,
`doc check`/`doc build`, local API pages and checked/opt-in examples. Package graphs,
assets, public indexes and LSP/rename remain separate. Net/HTTP/TLS are specified
library work; capability types, module/type/I/O/task foundations and lifecycle
implementation must precede executable adapters.

## Architecture and proof boundaries

| Responsibility | Existing owner |
| --- | --- |
| Pointer syntax and precedence | `src/parser/expressions.rs` |
| Carried slot shape | `src/borrow/carried.rs`, `src/check/statements.rs` |
| Whole-slot initialized/active state | `src/loans/emission_init.rs` |
| Value/source lifetime and alias backing | `src/borrow/`, `src/borrow_value/` |
| Physical storage and loan authority | `src/loans/` |
| Shared restart-header coverage | `src/loans/restart_headers.rs` |
| Exclusive scalar paths and restart frontier | `src/loans/exclusive_restarts.rs` |

## Still outside this compiler

The complete module/package graph, generic specialization, captures, public FFI,
wider ownership/cleanup, executable net peers/HTTP/TLS, public artifacts/replay and
LSP remain separate. Host execution does not qualify minimum platforms or bundled
distributions. Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Write all new fixtures with the migrated pointer syntax. Group a full indexed,
   call-result or narrowed-reference target when it must be selected/evaluated before
   borrowing/dereferencing. Keep diagnostic spans tied to actual source text.
2. Resume fixed-capacity reference-free carried list investigation in
   `borrow/carried.rs`, `check/statements.rs` and `loans/emission_init.rs`.
   Establish whole-slot initialization, copies, replacement, owner resets and
   incomplete/duplicate rejection before expanding eligibility. Keep indexed
   acquisition/reservations separate until `loans/elements.rs` and existing proofs
   qualify them with focused source/native coverage and the compiler gate.
3. Preserve exact sources, active initialization, shared-header certificates,
   genuine call/input opacity, exclusive backedge rejection, owner expiry and old
   copies. Broader carried shapes, exclusive header carriage and owning cleanup
   need separate bounded work. Continue module graphs/library foundations in parallel
   with optional documentation polish; the standalone doc slice is complete.
4. Keep root/compiler STATUS current after logical steps, commit cohesive validated
   work and do not push or recreate STEP logs. Full public artifacts/replay, LSP,
   minimum-platform/distribution qualification and the full release remain open.
