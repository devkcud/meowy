# Pinned Boost.Context primitive

Upstream: [boostorg/context](https://github.com/boostorg/context), release tag
`boost-1.90.0`, revision
[`6ff80e0575133c6d0b704e03bbb956a0c3b9551a`](https://github.com/boostorg/context/tree/6ff80e0575133c6d0b704e03bbb956a0c3b9551a).
The official tag and source downloads were verified for this import.
[manifest.json](manifest.json) records every source URL and SHA-256 checksum.

Only `make_x86_64_sysv_elf_gas.S` and `jump_x86_64_sysv_elf_gas.S` are compiled.
`fcontext.hpp` is an unchanged ABI reference; the build does not include it or
require the remaining Boost headers. The wrapper mirrors the two-pointer transfer
layout and the make/jump signatures behind private names.

All four imported files are unchanged, including copyright and comments. The
[Boost Software License](LICENSE_1_0.txt) is retained in full. Runtime style rules
must not strip or rewrite these upstream notices.

Build-time preprocessor definitions rename `make_fcontext` and `jump_fcontext` to
`meowy_make_context_v0` and `meowy_jump_context_v0`; their ELF visibility remains
hidden. The assembly's terminal `_exit` path is unreachable during valid wrapper
use because the wrapper entry never returns to that trampoline.

The selected target is Linux x86-64 SysV ELF LP64. The build disables CET code
generation; the wrapper rejects active shadow stacks using the Linux status
interface. No TSX, segmented stacks, other architecture, high-level continuation,
forced unwind or `ontop_fcontext` implementation is imported.

`runtime/check.py` checks the complete file set, target, release/revision format,
source URL consistency and checksums before building. These offline checks detect
changes relative to the recorded import; they do not independently authenticate
GitHub or prove the release tag mapping without an upstream fetch.

For an update, verify the official tag revision again, review source/ABI/license
changes, copy the exact files, update the manifest, and run every native and
sanitizer profile. Keep the primitive replaceable behind the meowy wrapper.
