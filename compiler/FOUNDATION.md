# Foundational ownership identities

The bootstrap resolves `@"core"`, `@"debug"`, `@"memory"` and `@"strings"` to
explicit module identities. Ordinary lexical lookup determines what a module alias
means; a local named `memory`, `strings`, `heap` or `copy` gains no intrinsic behavior.

The new memory/string slice recognizes these identities:

| Module | Types | Values/operations |
| --- | --- | --- |
| memory | Allocator, AllocationFailure | heap |
| strings | Owned | copy |

Module aliases, item aliases, named type aliases and computed aliases of these
type identities can be declared. They produce no runtime owner or allocation.
Storage, parameters, references, predicates and calls involving the unavailable
types/operations report B001. Unmodeled members of these partial modules also
report B001, so documented but unimplemented APIs cannot become false E201/E202
language rejections. This is not complete module/member validation.

```meowy
memory : @"memory"
strings : @"strings"
heap : memory.heap
copy : strings.copy
<Owner> : <strings.Owned>
kind : strings.Owned
<Same> : <(kind)>
```

`FoundationType` retains nominal identity in checked type aliases; it is separate
from the set of lowerable runtime HIR types. No record shape can forge one of these
types. Runtime use stays gated at the checker rather than reaching a backend with
an invented resource layout. Existing core/debug behavior and unknown lexical
name diagnostics remain unchanged.

The [private string prototype](../runtime/STRINGS.md) now supplies static heap
allocation, typed native failure evidence and existing-bridge copy/move/drop/view
operations. Compiler archives include it and LLVM probes execute it, but source
`strings.copy` does not call it yet. Native AllocationFailure evidence is not a
completed source nominal error implementation. The eventual concrete AllocationFailure
is [not descriptor-compatible](../docs/reference/stdlib/errors.md#choose-inline-storage-or-explicit-erasure);
its small native layout must not permit implicit storage as the common error descriptor.

Next implement lowerable resource/error types, string-view/allocator origins and
bounded initialized-state/drop schedules from [OWNING_HIR.md](OWNING_HIR.md).
Keep constructor calls gated until normal, Leave, Restart and panic cleanup have
source-level proof. General packages, module graphs, generic library APIs and
source recovery remain outside this foundation slice.
