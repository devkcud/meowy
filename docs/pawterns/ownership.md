# Borrow the bytes, keep the owner

[Pawterns](README.md)

Storage bugs are remarkably good at turning tiny utilities into detective work.
Keep two questions handy: “who owns these bytes?” and “who still needs them?”
We'll encode an identifier, build a greeting, and negotiate with a very full list
without leaving either answer to guesswork.

Each recipe supplies a complete `main.mwy`. Use the [first-project
recipe](first-project.md) for the manifest and `meowy check` / `meowy run` workflow.
The snippets use `debug.print` for short transcripts; it panics if writing fails.
Use an explicit writer when output failure needs recovery.

## Eight hex characters, zero surprise allocations

**Problem:** you have a four-byte identifier and want its hex spelling. Eight
bytes of output should not require a small construction project on the heap.

Complete `main.mwy`:

```meowy
debug : @"debug"
encoding : @"encoding"
strings : @"strings"

status <int32> := 0

'main {
    identifier <uint8[4]> : [222, 173, 190, 239]
    output <uint8[8]> := []

    encoded : encoding.hex_encode_into(identifier.slice(), &!output)
    | encoded <error> | {
        debug.print(encoded)
        status = 1
        'main.leave()
    }

    text : strings.from_utf8(encoded)
    | text <error> | {
        debug.print(text)
        status = 1
        'main.leave()
    }

    debug.print(text)
    debug.print("Initialized output bytes: {output.size()}")
}

-> status
```

Expected output and status:

```text
deadbeef
Initialized output bytes: 8
```

Status is 0. Hex needs two output bytes per input byte, so eight slots suffice.
Capacity is the number of seats; length is how many are occupied. The output
list initially has capacity eight and length zero; the encoder
initializes its contents. `encoded` borrows those contents, and `text` is another
view of the same bytes after UTF-8 validation. Neither owns a second buffer.

The mutable `output` binding permits the exclusive borrow used by the encoder.
Do not mutate or move it while a returned view is still needed. Once the last
view use ends, the caller can reuse the storage. Its scope owns the bytes
throughout; placing the list in a record would put the inline storage in that
record instead of making it a heap allocation automatically.

**Exercise the failure:** change the output annotation to `<uint8[7]>`. Encoding
returns `encoding.BufferTooSmall`, leaves the original output unchanged, and
this program takes its status-1 path. Increasing capacity is a policy choice;
changing a bounded list to a borrowed `<uint8[]>` cannot make it grow.

The [encoding contract](../reference/stdlib/text-and-data.md#binary-encodings-and-digests)
defines preflight validation and unchanged output on failure. The
[file-hash project](../programs/file-hash/README.md) applies the same pattern to a
32-byte digest and a 64-byte hex buffer after streaming a file.

## Build a greeting that survives its own function

**Problem:** trim an input without copying it, then construct a greeting that can
survive independently of that input. “Hello” is a modest ambition; dangling
storage would be an unnecessarily dramatic way to deliver it.

Complete `main.mwy`, including both helpers:

```meowy
debug : @"debug"
memory : @"memory"
strings : @"strings"

trimmed <string> : (text <string>) {
    -> strings.trim(text)
}

greeting <strings.Owned><memory.AllocationFailure> : (name <string>) 'result {
    created : strings.builder(memory.heap, 32)
    | created <memory.AllocationFailure> | {
        'result -> created
        'result.leave()
    }
    builder := created

    prefix : builder.append("hello, ")
    | prefix <memory.AllocationFailure> | {
        'result -> prefix
        'result.leave()
    }
    suffix : builder.append(name)
    | suffix <memory.AllocationFailure> | {
        'result -> suffix
        'result.leave()
    }

    -> builder.finish()
}

status <int32> := 0

'main {
    source : strings.copy("  meowy  ", memory.heap)
    | source <memory.AllocationFailure> | {
        debug.print(source)
        status = 1
        'main.leave()
    }

    name : trimmed(source.view())
    debug.print(name)

    message : greeting(name)
    | message <memory.AllocationFailure> | {
        debug.print(message)
        status = 1
        'main.leave()
    }
    debug.print(message.view())
}

-> status
```

When allocations and output succeed, this prints:

```text
meowy
hello, meowy
```

`source` owns allocated text. `name` is a trimmed view and cannot outlive that
owner. Returning `name` from a helper that first creates and then drops `source`
would be invalid; copying the view does not preserve the bytes it points to.

`greeting` copies the name's bytes into its explicitly allocated builder.
`finish()` consumes the builder and transfers its allocation to the returned
`strings.Owned`. That result does not depend on the name's storage. No allocator
call is needed merely to move the owner to the caller. Its selected allocator,
`memory.heap`, must remain valid for that owner.

Every append is checked, even with an initial capacity of 32: an initial capacity
is neither a permanent bound nor a promise that every possible name fits.
Allocation failure leaves the builder's previous contents intact; these branches
discard that partial greeting through ordinary cleanup and propagate the failure.
The emission `->` alone would not finish a failure path, hence each named `leave()`.

**Try a lifetime change:** replace `-> builder.finish()` with
`-> builder.view()` and change the helper result to
`<string><memory.AllocationFailure>`, keeping the existing failure branches.
The view would borrow a local builder that is about to be destroyed, so the
checker rejects it.
Keep the owning result, or supply caller-owned output as in the first recipe.

See [returned lifetimes](../reference/memory.md#lifetimes) and
[owned text](../reference/stdlib/text-and-data.md#owned-strings-and-formatting).
The [JSON report](../programs/json-report/README.md) shows the same relationship
across modules: its summary borrows strings owned by the decoded document, which
stays alive until output finishes.

## When the list says “two”, believe it

**Problem:** Ada and Dev already occupy a two-slot list. Lin arrives. Decide
whether “two” is a real application limit or just your first guess at the size.
The former needs a recoverable rejection; the latter needs growable storage.

Complete `main.mwy`, showing both policies:

```meowy
collections : @"collections"
debug : @"debug"
memory : @"memory"

status <int32> := 0

'bounded {
    names <string[2]> : ["Ada", "Dev"]
    appended : names.try_add("Lin")
    | appended <collections.Full<string, 2>> | {
        debug.print("Rejected: {appended.value}")
        debug.print("Kept entries: {appended.list.size()}")
        'bounded.leave()
    }
    debug.print("Accepted entries: {appended.size()}")
}

'growing {
    created : collections.vector<string>(memory.heap, 2)
    | created <memory.AllocationFailure> | {
        debug.print(created)
        status = 1
        'growing.leave()
    }
    names := created

    reserved : names.reserve(3)
    | reserved <memory.AllocationFailure> | {
        debug.print(reserved)
        status = 1
        'growing.leave()
    }

    incoming <string[3]> : ["Ada", "Dev", "Lin"]
    position <usize> := 1
    'append {
        | position > incoming.size() | 'append.leave()
        pushed : names.push(incoming[position])
        | pushed <error> | {
            debug.print(pushed)
            status = 1
            'growing.leave()
        }
        position = position + 1
        'append.restart()
    }

    'inspect {
        view : names.slice()
        debug.print("First: {view[1]}")
        debug.print("Accepted entries: {view.size()}")
    }

    names.clear()
    debug.print("After clear: {names.size()}")
}

-> status
```

With successful allocation and output:

```text
Rejected: Lin
Kept entries: 2
First: Ada
Accepted entries: 3
After clear: 0
```

`try_add` sends Lin back with the original guest list intact: its `Full` result
retains ownership of both the unchanged bounded list and the rejected
element in `Full`. The operation copies copyable inputs and moves resource owners;
code handling a move-only list must recover it from that result. A full `.add()`
would instead panic, or be rejected when the overflow is statically known.

The vector has one owner and retains its allocator. `reserve(3)` requests at least
three **total** slots, not three additional slots. Capacity need not equal the
requested number. Reserving fails without altering the vector; failed `push`
retains its rejected element in `collections.PushFailure<T>` and leaves the vector
unchanged. This example reports failure and drops the retained value; an application
can instead arrange a retry policy.

The inspection scope makes it clear that its element views are finished before
`clear()` exclusively borrows the vector. This rule applies even when an operation
would happen to reuse the same allocation. `clear()` releases the elements but
keeps the allocated capacity for reuse; dropping the vector releases its storage
to its allocator, which need not immediately reduce process RSS.

Compare [bounded list ownership](../reference/collections.md#bounded-lists),
[vector growth](../reference/stdlib/collections.md#growable-vectors), and
[overlapping memory budgets](../reference/optimization.md). The
[word-count project](../programs/word-count/README.md) uses short lookup borrows
before mutating an allocated map, and keeps a separate bounded list for output
order.
