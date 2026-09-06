# Runtime working rules

- Read [README.md](README.md), the root [STATUS.md](../STATUS.md),
  [COMPILER.md](../COMPILER.md) and the relevant language reference before edits.
- The current code prototypes cleanup and guarded stack allocation. Do not describe
  it as working task contexts, a scheduler, compiler integration or qualified DWARF
  unwinding.
- Keep storage bounded and caller-owned. Document ownership, failure, suspension
  and unwind behavior for every API. Never silently allocate to extend a borrow.
- Use explicit cleanup edges and initialized-state tracking. Host destructors or
  exceptions cannot supply Meowy ownership or cancellation semantics.
- Preserve fatal cleanup-panic probes as subprocesses with exact evidence. Do not
  count arbitrary crashes, missing tools or unsupported sanitizer runs as passes.
- Guard probes must verify the fault address, protection code and isolated process
  outcome. Keep direct guard writes distinct from task-overflow or stack-switch proof.
- Run `python3 -B runtime/check.py` and the runtime Python regressions after behavior
  changes. Keep static verification distinct from native and sanitizer evidence.
- Send each logical step's findings, validation, blockers and next steps to the
  sole root tracker writer while delegating; otherwise update root `STATUS.md`
  and record the checkpoint in root `STATUS_STEP_LOG.md`.
- Follow root naming, visibility and comment rules. Add no dependencies without
  a concrete need and an explicit version/revision plan.
