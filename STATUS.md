# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Added a private generated-code cleanup bridge over the runtime Stack and owning
  Panic snapshot (`2288ed5`; compiler proof `2da831f`; contract `53f8e6b`).
  Caller-owned storage, explicit initialized slots, checked token/mark
  layouts and callback reentry protection preserve reverse-order cleanup.
- The compiler native archive includes the bridge. LLVM callback probes execute it
  in debug/release and prove owning panic text survives destruction of its source.
  Ordinary Meowy programs still have no automatic owner cleanup or task lowering.
- All fourteen combined checks pass: 787 Rust tests (359 library, 428 native),
  35 Python, 48 debug/release examples, Vim/Neovim, 940 links, formatting, Clippy,
  build and schema/catalog checks. Runtime debug/release/ASan/UBSan/LSan pass all
  85 case groups per profile and required fatal/guard/admission/fiber probes.
- Sandbox LeakSanitizer failed under ptrace; the approved unsandboxed combined run
  passed. A bridge-using ELF imports only libc.so.6. Conformance still has 13 unsupported
  cases. Private ABI proof does not qualify automatic cancellation, DWARF or release.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalar ownership plus a native archive cleanup bridge | Generated payload descriptors and automatic owning-value cleanup |
| Runtime | Owning snapshots, cleanup bridge and bounded task prototypes | Payload relocation bridge, task-close progress, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Define generated payload move/drop descriptors using existing ValueOps/Owned and
   prove actual relocation across cleanup frames with LLVM-native fixtures. Arm the
   initialized destination before disarming the source; preserve ownership on failure.
   Then design owning-HIR drop schedules for normal/Leave/Restart and retained emissions.
   Keep task-close progress, cancellation and pinned unwinding separate until proved.
2. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
3. Extend aggregate/emitted-name/cross-element constraints in the list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
4. Preserve current compiler evidence; do not rerun green checks without a new change
   or concern. Run `python3 -B tools/verify.py --all` after wider integrations. LSan
   needs process inspection. Strict conformance still has 13 unsupported cases;
   this bootstrap gate and host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
