# Package identity and build graphs

[Modules](modules-and-ffi.md) · [Artifact formats](artifact-formats.md)

These rules define the initial distribution's dependency resolver. An alias names
one dependency edge; it is not a package's identity. No registry, version ranges,
implicit upgrades, dependency build scripts, or ambient linker search is involved.

## Sources, revisions, and identities

The initial remote transport is Git over HTTPS, using SHA-1 Git object IDs. A
`fetch` operand has the form `https://HOST/PATH.git`. HOST is an ASCII DNS name
in lowercase; an explicit port is permitted and port 443 is removed. PATH is
case-sensitive and contains nonempty ASCII components beginning with a letter,
digit, `-` or `_`, followed by those characters or `.`; `.` and `..` components
are forbidden. Ports must be integers in `1..65535`. User information,
percent escapes, queries, fragments, trailing slashes, and other transports are
rejected with `E505`. A `.git` suffix is required. The resulting URL is the
canonical source string. Different canonical URLs are different sources, even
when they currently serve the same Git objects. Redirects may retrieve content
but do not replace this declared identity. Credentials are transport inputs, never
part of a source ID, lock, build record, or capsule.

`hash` is exactly 40 lowercase hexadecimal digits naming a commit. `tag` is a
nonempty Git tag name without the `refs/tags/` prefix; annotated tags are peeled
to a commit. `ref` is either a branch name or its `refs/heads/NAME` spelling;
normalize both to `refs/heads/NAME`. Git refname validity rules apply; revision
expressions such as `main~2`, abbreviated hashes, and arbitrary ref namespaces
are not selectors. Resolution records the normalized selector and full commit.
An already locked tag/ref is not contacted again to test whether it has moved.
The declared selector must still equal the lock's selector; changing it requires
an explicit update even if it would select the same commit.

A remote package key is `(canonical source, commit, package directory)`. The
directory is `.` for a fetched repository's root; a `path` dependency within that
repository may select another directory with its own manifest. Its path must stay
within the same immutable repository tree. Such a subpackage keeps the same
source/commit and uses its normalized repository-relative directory. An escaping
remote `path` dependency is `E505`; it cannot select a builder's adjacent files.

For serialized IDs, hash the UTF-8 source, a NUL, commit, a NUL, and directory,
using SHA-256; prefix the lowercase hexadecimal result with `git:`. A module key
adds its normalized path within that package. For a local package, canonicalize
its directory, express it relative to the root project with `/` separators, and
prefix with `local:` (`local:.` is the root). Explicit siblings may therefore be
`local:../geometry`. Canonical filesystem identity, including symlink resolution,
determines whether two local spellings share a package. Lock IDs use these
root-relative names; capsules preserve a mapping when relocating their inputs.
Relative imports and aliases do not introduce new package identities.

Two revisions of one source may coexist. They initialize separately and their
nominal types are distinct. A diamond selecting the same source/commit/directory
shares one package and each module initializes once. For example, if `app -> a`
and `app -> b` both select `geometry` commit X, they share X; if b selects Y, both
X and Y remain and a nominal `X.Error` cannot match `Y.Error`. There is no
automatic version unification. Import cycles across any of these edges remain
`E502`, including local paths and facade re-exports.

An alias reaching a source outside its project's directory keeps the importing
package's configuration context. A `path` dependency instead creates/selects the
target package context. The same physical file used under different package
contexts is a different module; an alias and a relative import within the same
context share it. An alias may not enter another directory containing a `mod.mwy`
or its descendants: select that package with `path` instead (`E505`). This prevents
one local file from silently acquiring another project's import policy.

## One lock authority

The root project's `mod.lock` is the only resolution authority for its complete
build. A dependency's own lock is ignored; its manifest declares requirements.
Local packages need no source lock or frozen working-tree digest. Their remote
requirements still need entries in the root lock. A project with no remote
requirements may omit `mod.lock`; its distribution identity is then taken from
the selected compiler and recorded in its build/session inputs.

Lock schema 1 stores remote packages and remote selection requests. A request is
keyed by `(requesting package ID, dependency alias)` and identifies the normalized
fetch/selector and selected remote package ID. Path edges are derived from the
manifests; a remote path subpackage appears in `packages` if it supplies or is
the target of a recorded request. Every listed remote package carries its whole
repository content digest. Identical source/commit pairs have identical digests.
The [lock schema and example](artifact-formats.md#lockfiles) give the exact format.

`deps resolve` preserves existing request selections, fills missing requests,
checks the entire resulting graph, and fails on a changed selector or inconsistent
existing package. It removes entries no longer reachable from any declared root
dependency. `deps update ALIAS` re-resolves that root alias's remote edge and all
remote requests reached exclusively through the newly selected subtree; a request
also reachable from an unselected root edge retains its existing selection. If
that retained selection conflicts, fail and name the other root alias that must
also be updated. `deps update` with no alias permits re-resolution of all remote
requests, including those reached through local path dependencies. An ALIAS that
names a local dependency is rejected with `E505`; use the all-dependencies form
to update its remote requirements. Updates never rewrite manifest selectors.
Both operations replace the lock only after the complete graph validates.

Ordinary commands may retrieve only already selected content. Missing/conflicting
request entries use `E503`, unavailable content `E501`, and content mismatch
`E504`. Unused lock entries do not add imports or initialization roots; resolve
removes them. Offline resolution requires the exact Git objects and selector
metadata in the local verified store; it never guesses a branch tip.

There is exactly one compiler/foundational-library/runtime distribution in a
build. A lock's `distribution` must equal the running distribution's release
string and distribution digest. There is no second standard library per package.
A mismatch is `E507`, before checking dependent source. A deliberate
`deps update` without an alias adopts the running distribution after checking the
resolved manifests/graph; `resolve` and a selected-alias update never change that
identity. `lsp.toolchain` remains an additional editor gate, not a CLI selector.

## Content verification

Remote tree digests include every tracked regular file in the selected commit,
including manifests, native artifacts, and dependency lockfiles even though those
locks do not control resolution. Exclude Git administrative storage. Gitlinks
(submodules), symlinks, invalid UTF-8 paths, and unsupported tree modes are rejected
as unavailable package content (`E501`); fetching does not execute filters, LFS
smudge commands, hooks, or checkout programs. An LFS pointer is ordinary tracked
bytes, not an instruction to fetch an undeclared payload.

Sort paths by unsigned UTF-8 byte order. The SHA-256 input is the ASCII bytes
`meowy-tree-v1`, one NUL, then each file's four-byte big-endian path-byte length,
path bytes (no leading `./`), one mode byte (`0x00` for 100644, `0x01` for 100755),
eight-byte big-endian content length, and exact file bytes. Paths have nonempty
components other than `.`/`..`, `/` separators, no NUL, and no duplicates. Empty
directories have no entry. Hashes are written as `sha256:` plus 64 lowercase hex
digits. Timestamps, ownership, archive order, and checkout line-ending conversion
are not inputs; verification uses Git blob bytes and executable mode directly.

Local source is live development input. A build snapshots and hashes every used
source/manifest and declared native/data artifact, including external alias files,
before checking. It fails/retries if those inputs change during snapshotting;
later phases use that snapshot. Reproducible export includes those exact bytes
and their original package/path mapping, without freezing local edits in the lock.

## Build settings across packages

| Field                                                                      | Application/test root                                                   | Imported package                                                                             |
| -------------------------------------------------------------------------- | ----------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| `import`, `export`                                                         | Resolve root imports/facade                                             | Resolve that package's graph with its own aliases                                            |
| `build.entry`                                                              | Entry chosen by the command; tests do not execute it                    | Ignored as an entry; it must still not be imported if declared as that package's entry       |
| `build.profile`, `target`, `cpu`, `optimize`, `jobs`, `debug_info`, `link` | Select one effective whole-build policy, after allowed CLI overrides    | Validated as configuration but do not override the consumer's policy                         |
| `build.executor`                                                           | Sole standalone executor configuration, also used inside each test case | Does not create/configure another executor; task-using code requires the consumer's executor |
| `build.native`                                                             | Add exact artifacts                                                     | Add exact artifacts from every package in the selected import graph                          |
| `test`, `gatostyle`, `lsp`                                                 | Configure the selected root/tool operation                              | Do not inherit into the consumer or add tests/style work                                     |

Declared native paths resolve relative to their own manifest. Canonical duplicate
artifact paths/content identities are linked once, in stable package-ID order
then declaration order. The driver resolves static archive dependencies to a fixed
point, so archive placement is not an accidental dependency API. Duplicate strong
native definitions are a link failure; no last-library-wins policy is permitted.
All declared artifacts in an imported package participate even if its foreign
call later proves unreachable. Unimported dependencies contribute no native inputs.
Every artifact must satisfy the root target/link policy (`E606`); a dependency's
own target value cannot certify its binary for another target. There are no
per-dependency compiler flags or generated bindings during a build.

An imported library without an executor can define functions that start tasks;
the graph checker propagates that requirement through calls and verifies it at
the selected standalone entry/test root. A reachable task-starting call without
an executor is `E404`; an unused function is still checked for ownership/types,
but does not create a runtime requirement merely by existing.
