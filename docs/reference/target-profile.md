# Initial distribution target

[Build policy](optimization.md) · [Native ABI](native-abi.md)

The initial meowy distribution supports one native host and output target:
`x86_64-unknown-linux-gnu`. Other triples are rejected with `E507`; examples using
them explain configuration shape and do not advertise support. This profile is
a distribution commitment, not a restriction on possible future implementations.

| Property               | Initial profile                                                                                                                                                      |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Host and output        | Linux x86-64, ELF64, little endian, System V AMD64 ABI                                                                                                               |
| Minimum execution host | Linux kernel 5.4, glibc 2.31, x86-64 baseline including SSE2                                                                                                         |
| CPU setting            | `baseline` only; expanded requirement is the original x86-64/SSE2 feature set, without AVX or later extensions                                                       |
| C model                | LP64: char 8 bits, short 16, int 32, long/long long/pointer 64; plain char signed; IEEE binary32/binary64                                                            |
| Language pointer model | `usize`/`isize` are 64-bit; pointer size/alignment are 8 bytes                                                                                                       |
| Executable form        | Dynamic PIE with ELF interpreter `/lib64/ld-linux-x86-64.so.2`, or static ELF executable without an interpreter                                                      |
| Runtime linkage        | `platform`/`shared` use the dynamic runtime; `static` uses supplied CRT, glibc and runtime archives subject to the closure checks below                              |
| Link optimizations     | Section dead stripping and safe ICF supported; LTO `off`, `thin`, and `full` supported                                                                               |
| Debug information      | DWARF 5; embedded, `.debug` companion, or none; ELF build ID and required unwind metadata remain                                                                     |
| Stack cleanup          | DWARF unwinding within meowy frames; no unwind across a C boundary                                                                                                   |
| Executor               | Explicit worker count and task stack budget, system allocator, cooperative scheduling/cancellation; a child stays on its first worker through resumption and cleanup |

The distribution supplies its compiler, linker, runtime, foundational library,
target descriptor, headers/sysroot, static libc/thread/unwind archives and CRT
link objects as immutable inputs. It
uses LLVM object/IR conventions internally, but the selected backend version is
the version in that distribution, never an executable discovered from `PATH`.
Thin/full LTO accept that distribution's IR only; native ELF objects stay opaque.
There is no ambient `CC`, `CFLAGS`, `LDFLAGS`, sysroot search, or host-header input.

Each installed distribution has a `distribution.json` descriptor with schema
`meowy.distribution`, version `1`, release string, target triple, baseline feature
list, minimum kernel/libc versions, supported link/LTO/debug modes, and a sorted
inventory of relative member paths, byte sizes, and SHA-256 digests. The
distribution digest is SHA-256 of this descriptor's exact UTF-8 bytes; the
descriptor does not contain its own digest. File digests cover the compiler,
linker, runtime, standard library/rule data, sysroot, and target configuration.
`meowy --version`, locks, build reports, sessions, and capsules record the release
string and this digest. A changed payload requires a changed descriptor/digest.
The [artifact schemas](artifact-formats.md) include this descriptor and example.

Static output has no `PT_INTERP` or `DT_NEEDED` native loader dependency. The
selected archive closure must satisfy every native reference; the driver rejects
shared inputs and reachable dynamic-loader/NSS requirements rather than claiming
self-contained linkage. In the initial GNU profile, reachable `net.resolve` or
native resolver/NSS entry points (`getaddrinfo`, `gethostbyname`, `getnameinfo` and
their reentrant variants) are unavailable with static linkage (`E507`); numeric
socket addresses and ordinary TCP/UDP do not require DNS. Native `dlopen`/`dlmopen`
requirements are likewise unsupported in a static closure. These checks do not
prohibit static greetings, duration/calendar arithmetic, files or task programs.
Native bindings that perform runtime library loading are outside this static
profile. An opaque input whose required native closure cannot be established
from its declared artifacts and symbol/relocation graph is `E606`; absence of an
ELF interpreter alone is not evidence of an arbitrary foreign call's effects. No missing closure
silently selects shared output. A static executable needs the minimum Linux/CPU
profile but no host glibc installation; external application data still belongs
in its deployment. The distribution's bundled tools retain their own recorded
host requirements.

The build report lists actual required ELF libraries and versioned symbols, not
just the minimum profile baseline. A native input can require a later glibc or
additional shared library: the report records that stronger deployment closure.
`run` checks CPU, interpreter and loader dependencies on its host before launch;
missing/incompatible requirements are `E507`. `build` records requirements without
claiming deployment-host verification. Declared shared native inputs and their
resolved transitive library closure must be supplied as exact distribution or
package artifacts; no ambient same-name library is substituted at link/capture.
The output uses an origin-relative runtime directory for non-platform shared
artifacts, copied beside it. `libc.so.6`, the ELF loader, and their profile system
dependencies remain declared host requirements. Captures carry other required
shared libraries and preserve their exact digests.

The initial profile does not expose a native library output kind, C callbacks,
native exported entry points, or a host-provided executor. Standalone task code
uses `build.executor`. See [native ABI](native-abi.md) for the supported C-call
bridge, C scalar mappings, record layout, and restrictions.
