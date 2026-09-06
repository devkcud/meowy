# Repository working rules

- Keep explanations short and use short, clear names.
- Declare visibility wherever the language permits, including implicit public APIs.
- Use immutable bindings, `const` or `final` for values that do not change; use
  `static` for members belonging to a class.
- Do not prefix private members with `_` unless the surrounding project does.
- Do not add code comments unless the logic is extremely complex or unclear.
- Read `STATUS.md` before working. Compiler changes also follow
  `compiler/AGENTS.md` and `compiler/STATUS.md`.
- Treat `docs/reference/` as the language contract and `COMPILER.md` as the
  implementation plan. Preserve reference fixtures and existing user changes.
- After each logical work step, update the relevant STATUS with findings, actual
  validation, blockers and concrete next steps. Keep a single tracker writer when
  delegating. Leave enough context to resume without the conversation.
- Keep tools, editor integration, library and runtime progress visible in the root
  tracker. Distinguish static checks, compiler execution and release qualification.
- Reuse existing checks and architecture. Add dependencies only when necessary.
- Run checks appropriate to changed behavior. Never count unsupported features,
  missing tools or crashes as successful conformance rejections.
- Split requested commits by coherent behavior and dependencies. Do not push,
  publish or send messages to others without explicit authorization.
