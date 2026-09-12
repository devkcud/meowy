# Runtime working rules

- Read [README.md](README.md), the root [STATUS.md](../STATUS.md),
  [COMPILER.md](../COMPILER.md) and the relevant language reference before edits.
- The current code prototypes cleanup, guarded allocation, pinned contexts and a
  bounded single-worker scheduler with explicit waiting child joins. Do not describe
  it as a complete task runtime, automatic scope-exit joining/cancellation, compiler
  integration or qualified DWARF unwinding.
- Keep storage bounded and caller-owned. Document ownership, failure, suspension
  and unwind behavior for every API. Never silently allocate to extend a borrow.
- Preserve owned payload initialization and actual relocation. Once valid owned
  submission accepts a capture, failed admission must release it exactly once.
  Failed joins must retain result ownership until context release and transfer
  succeed. Keep descriptor lifetime static and move/drop callbacks non-suspending.
- Use explicit cleanup edges and initialized-state tracking. Host destructors or
  exceptions cannot supply meowy ownership or cancellation semantics.
- Preserve fatal cleanup-panic probes as subprocesses with exact evidence. Do not
  count arbitrary crashes, missing tools or unsupported sanitizer runs as passes.
- Guard probes must verify the fault address, protection code and isolated process
  outcome. Keep direct guard writes distinct from task-overflow or stack-switch proof.
- Preserve the exact vendored Boost.Context sources, comments and license. Update
  revision/source/checksum metadata together after official upstream verification.
- Context transitions must preserve worker identity, live stack ownership and ASan
  fiber-hook ordering. Never release suspended storage or silently unwind through
  host continuation destructors. Keep transition instrumentation exclusions narrow.
- Scheduler slots remain occupied through cleanup and settlement until successful
  join/release. Failed admission with a retained mapping must remain reclaimable;
  stale tickets, callback reentry and foreign-worker access must reject.
- Only the active parent's Task capability may admit or consume its children.
  Never resume or reclaim a child after the scope of its borrowed locals has ended.
  Keep missing explicit joins a private fatal protocol violation until generated
  scope-exit joins and unwind support can preserve those lifetimes.
- Task scopes have fixed metadata capacity and parent-bound, nonreused marks.
  Close only the innermost mark, retain progress on release failure and discard an
  owned child result only after its context releases successfully. Close is explicit
  and never substitutes for cancellation or joining after C++ locals expire.
- Detailed scope reports use caller-provided bounded batches. Check capacity after
  settlement but before reclaiming a failed child; report_full retains its ticket
  and mark. Publish/count each detail only after successful consumption, and keep
  per-call reported counts distinct from cumulative scope progress.
- Construct owning Panic snapshots while their source bytes are valid, including
  before a drop callback destroys its text. Keep fixed storage, visible truncation
  metadata and copy-independent message views; never store a self-pointer or copy
  a dangling view after callback return. Operation-name lifetimes remain separate.
- Run `python3 -B runtime/check.py` and the runtime Python regressions after behavior
  changes. Keep static verification distinct from native and sanitizer evidence.
- Send each logical step's findings, validation, blockers and next steps to the
  sole root tracker writer while delegating; otherwise update root `STATUS.md`.
  Keep only the current handoff there; use Git history instead of STEP logs.
- Follow root naming, visibility and comment rules. Add no dependencies without
  a concrete need and an explicit version/revision plan.
