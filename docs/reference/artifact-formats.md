# Artifact formats and reader compatibility

[Dependencies](packages-and-builds.md) · [Diagnostics](diagnostics.md) · [Recording](replay-recording.md)

Schema 1 is defined by the checked-in JSON Schemas below, using JSON Schema
2020-12. Their `urn:meowy:schema:NAME:1` identifiers are stable local identifiers,
not download endpoints. Resolve them from the distribution's bundled schema set.

| Artifact                | Schema                                                | Example                                                      |
| ----------------------- | ----------------------------------------------------- | ------------------------------------------------------------ |
| Root `mod.lock`         | [lock](../schemas/lock.schema.json)                   | [mod.lock](../schemas/examples/mod.lock)                     |
| Saved occurrence        | [diagnostic](../schemas/diagnostic.schema.json)       | [diagnostic.json](../schemas/examples/diagnostic.json)       |
| Capsule manifest        | [capsule](../schemas/capsule.schema.json)             | [capsule.json](../schemas/examples/capsule.json)             |
| Build report            | [build-report](../schemas/build-report.schema.json)   | [build-report.json](../schemas/examples/build-report.json)   |
| Distribution descriptor | [distribution](../schemas/distribution.schema.json)   | [distribution.json](../schemas/examples/distribution.json)   |
| Runtime event           | [runtime-event](../schemas/runtime-event.schema.json) | [runtime-event.json](../schemas/examples/runtime-event.json) |

[Common definitions](../schemas/common.schema.json) are shared by these schemas.
The examples are schema fixtures, not binaries or an installed distribution.
Zero digests denote intentionally absent external example payloads; executable
readers do not treat zero as a wildcard. The capsule example is explicitly
incomplete and its included [source bytes](../schemas/examples/source.txt) and
diagnostic have real file digests. The build example is an incomplete report and
does not claim that a native binary was produced. The example distribution
descriptor is illustrative and its empty inventory cannot be installed or run.

## Common encoding and validation

Files are UTF-8 JSON without a BOM, duplicate keys, NaN, Infinity or invalid Unicode
surrogates. Writers terminate a JSON file with LF. Object member order has no
semantic significance except when its exact bytes are hashed as a file. Integers
in JSON numbers fit `-(2^53-1)..2^53-1`; larger language integers are decimal strings
without leading zeros or `+`, and signed zero is `0`. Test seed strings must fit
`uint64`. Byte payloads use separate inventory members; edit replacement bytes
use strict padded RFC 4648 base64, decoded and checked as UTF-8 before source edits.

Schema records reject unknown fields. Explicit named maps, such as pinned data
versions, have the key/value domain stated by their schema. Event argument/result
records have [operation-specific schemas](runtime-events.md); they are not open
maps. All required keys are present; `null` means
the explicitly documented unavailable/inapplicable value, not a missing default.
Digest values are `sha256:` plus 64 lowercase hexadecimal digits. File digests
cover exact bytes, without newline, path, or encoding normalization. Member paths
are nonempty, relative `/` paths, without NUL, backslash, empty components, `.` or
`..` components. They cannot escape the extraction directory through symlinks;
capsules contain regular files only and do not create links or special files.
Original source paths are descriptive strings, never extraction destinations.

Validate the schema before acting on an artifact. Then validate referential and
semantic constraints: unique IDs/paths/requests, sorted inventories, all required
member references present, byte lengths and digests matching, and source spans
within the identified source file with `start <= end`. Source ranges are zero-based
UTF-8 byte offsets, end exclusive, and both boundaries must lie between scalars.
Diagnostic display lines/columns are derived from those preserved bytes. A
process outcome has exactly one non-null `code`/`signal`; signal is positive.
Absent process outcomes use `null` for the entire record.

Schema files define the structural shape; these semantic checks are mandatory
even when a generic JSON Schema validator cannot compare two fields or read a
referenced file. An integrity check is not a signature or a trust decision about
executing another person's native payload.

### Canonical JSON

Argument digests need a single byte representation. Profile 1 canonical JSON
recursively sorts object keys by unsigned UTF-8 byte order, preserves array order,
uses no insignificant whitespace, decimal integers without leading zeros, and
literal `true`, `false`, `null`. Strings contain UTF-8 scalars unchanged except
`"` and `\` use `\"` and `\\`, and U+0000..U+001F always use lowercase
`\u00xx` escapes. Do not use short `\n`/`\t` escapes, escape `/`, normalize
Unicode, or escape other Unicode scalars. No final newline participates. This
encoding is used only where a digest explicitly calls for canonical JSON; whole
file and distribution descriptor hashes still cover their exact bytes.

## Lockfiles

`schema : "meowy.lock"`, `version : 1` contains `distribution`, `packages`, and
`requests`. The package and request identities, selector normalization, whole-tree
digest algorithm, coexistence rules, and root-lock authority are specified in
[package resolution](packages-and-builds.md). Sort packages by ID and requests
by `(from, alias)` using UTF-8 byte order. No duplicate package or request keys
are permitted. A request's selected package must exist; its source must equal
`fetch`, directory must be `.`, and a `hash` selector must equal its revision.
Non-root/local request origins must identify a recorded package. Local origins
must resolve to a package in the actual manifest graph. All package IDs must
equal the SHA-256 key derived from their source/revision/directory.

Loaded source manifests establish the actual path and remote edges. A lock alone
does not authorize an undeclared dependency. Missing/conflicting selections are
`E503`; source content digest failures remain `E504`. An invalid lock syntax/schema
is `E503` with its file/field; a newer unsupported lock version requires a matching
distribution, never a best-effort interpretation. Locks are rewritten only by
the explicit dependency commands, with a complete validated replacement.

## Saved diagnostics and capsules

A diagnostic contains its session/occurrence/code/phase, message, source span,
related spans, available facts, candidate edits, failure match identity and capture
state. The diagnostic catalog schema version is the artifact's `version`; code
meanings are the bundled catalog's stable entries. Human wording is preserved
in the saved message/facts and need not match another distribution's wording.
Candidate `N.M` must use this occurrence number N; candidate IDs are unique.
Edit hashes refer to the entire original source file. Closed captures have no
capture reasons; other states explain what is external, incomplete or unavailable.

A capsule's `inventory` lists its regular-file payload members in path order.
`diagnostic`, `runner`, `toolchain_descriptor`, event log, harness and every input
member must resolve into that inventory unless listed in `missing`. A closed
capsule has a runner, full toolchain closure, no missing members and no uncaptured
required boundary. The compiler/distribution and source digests must agree across
the manifest, diagnostic, descriptor and inventories. `namespace : "tests"`
requires preserved test policy/selectors/seeds; `entry` is null and its harness
is the executable graph. An entry session instead has an entry path and no test
record. Test seed derivation and policy values obey the testing reference.
The saved command is an argument array, never shell text: the runner invokes
its bundled tool directly, rewriting input/output paths into the scratch mapping
before phase execution. Original absolute paths are never output destinations.

The initial executable container is an ELF64 runner followed by uncompressed
member bytes, then the UTF-8 capsule manifest, then a 64-byte footer. Footer bytes
0..15 are exactly the 16 ASCII bytes `MEOWY-CAPSULE-V1`, without a NUL. Bytes 16..23 are the manifest
offset and 24..31 its byte length, unsigned little-endian uint64. Bytes 32..63 are
the raw SHA-256 of the manifest bytes. Payload files occupy the interval between
the runner image and manifest, in inventory order excluding the runner member.
Each such entry is an eight-byte
little-endian length followed by exactly those file bytes; its length equals the
inventory's `bytes`. The leading ELF image is itself the inventory member named
by `runner`; it is not duplicated among the length-prefixed entries. The leading
image must match that member's byte count and digest. The first payload offset is
therefore
the runner member's recorded length. Reject overlapping/out-of-bounds extents,
extra unaccounted bytes, missing records, size overflow, footer mismatch, or
unverifiable runner bytes before executing a payload. The manifest excludes itself
from the inventory to avoid self-hashing; the footer authenticates its bytes for
integrity purposes. No compression, nested archive, or executable script stub is
part of container version 1.

An incomplete exported capsule still needs its runner to be an executable; if the
runner itself cannot be recovered, `err export` returns `E704` and does not create
an executable-looking file. Stored manifest/diagnostic inspection remains possible.
Otherwise export preserves `incomplete` and missing-member descriptions. The
inventory contains only present members; a missing member is named in `missing`,
never given fabricated bytes or a success-looking digest entry.

### Runtime events

The event log is UTF-8 newline-delimited JSON, one runtime-event schema 1 record
per nonempty line, ending with LF. `seq` is strictly increasing from one;
predecessors must refer only to earlier events, without duplicates. A task/resource
must be created before its use, except root task 0 and standard-stream resource
IDs `stdin`, `stdout`, `stderr`. Event payload references resolve through the
capsule inventory. Large results split into ordered payload members within one
event, with concatenated byte count equal to the recorded result count.

`arguments_digest` hashes canonical JSON of `arguments`. Pointer-containing API
arguments are lowered to logical IDs and digests of initialized bytes; no padding
or uninitialized storage is read. The [recording profile](replay-recording.md)
and [operation registry](runtime-events.md) define exact argument/result fields,
tagged integer/float/record values, boundary results, scheduling, budgets and match outcomes.
An unknown operation/schema is unavailable replay, never an ignored event.

## Build reports

A report's four phase names remain the coarse checking/lowering/linking/reporting
phases. Its `inputs` gives source and tool identities, effective policy and pinned
data; `unavailable` explains any target/runtime/sysroot/policy value that is null.
Files are the observed snapshot; an unavailable input is not silently omitted
without a reason. `artifacts.report_path` has no self-size/digest. Executable,
debug and map entries identify actual produced artifacts. Section sizes distinguish
file bytes and mapped memory; retention edges explain shared code/data without
charging its size repeatedly. Transform and stack evidence mark uncertainty.

A complete report has no failed phase and identifies an executable and link map;
incomplete reports require their failed phase and may describe already produced
artifacts. A checking failure normally has null executable/map and unavailable
sections. Artifact file paths may be absolute or output-relative; they are
descriptive paths, not capsule extraction member paths. Section totals are not
RSS. `runtime.verified_on_host` distinguishes a declared deployment requirement
from one actually verified. A read of a report never executes the artifact.

## Reader compatibility

Distribution versions and schema versions are different identities. Readers that
advertise schema 1 accept schema 1 from older distributions, validate its exact
fields and preserve the old messages/code meanings. They do not require the host
CLI compiler to equal the saved compiler: actual replay uses the bundled compiler.
An unknown schema version is rejected before payload execution with `E704` and
names the required reader. Malformed capsule/diagnostic structure, member digest
failure or inconsistent references use `E705`. Existing lock errors use `E503`/
`E504` as above. Build-report consumers likewise report unsupported or invalid
input and do not turn it into a zero-sized successful report.

Changing an existing field meaning, required field set, or enum requires a new
schema version. Schema 1 is closed to unknown additive fields; publish schema 2
and an explicit reader/migration if more fields are needed. No command silently
migrates an immutable saved session or capsule. Newer tools may inspect old schema
1 without rerunning it; unavailable host/tool requirements stay explicit.
