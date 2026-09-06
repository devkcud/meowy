# Text and data without mystery storage

[Pawterns](README.md)

Text looks innocent until two identical-looking names disagree about their
bytes, or a tiny JSON document brings a surprisingly large family of allocations.
These recipes keep the inputs small enough to see who owns what. Put each
complete entry file in its own project as described in
[Your first project](first-project.md). Helper modules live beside that entry.
`debug.panic` below treats an unexpected failure of the stated fixture as a failed
demonstration; the [CLI recipes](cli-and-files.md) show recoverable application
boundaries with explicit exit statuses and checked output.

## Same café, different bytes

Someone enters “café.” Someone else enters what looks like “café.” Your byte
comparison disagrees. Neither person is necessarily wrong; decide which text
equivalence your application wants before asking the bytes to agree.

This complete `main.mwy` normalizes a decomposed accent into NFC, counts grapheme
clusters, and compares the result with a precomposed spelling. The first literal
contains an `e` followed by U+0301; the second contains U+00E9.

```meowy
bytes : @"bytes"
debug : @"debug"
iter : @"iter"
strings : @"strings"
unicode : @"unicode"

original : "café"
expected : "café"
buffer <uint8[32]> := []
normalized : unicode.normalize_into(original, unicode.NFC, &!buffer)
| normalized <error> | debug.panic(normalized)

clusters <usize> := 0
cursor := unicode.graphemes(normalized)
'count {
    item : cursor.next()
    | item <iter.End> | 'count.leave()
    clusters = clusters + 1
    'count.restart()
}

debug.print("Original bytes: {original.size()}")
debug.print("NFC bytes: {normalized.size()}")
debug.print("Graphemes: {clusters}")
debug.print(bytes.equal(normalized.bytes(), expected.bytes()))
-> 0
```

Expected output:

```text
Original bytes: 6
NFC bytes: 5
Graphemes: 4
true
```

The 32-byte list is caller-owned inline storage. `normalized` borrows it; neither
the string view nor the cursor owns a second copy. Keep the buffer unchanged until
all its views expire. Normalization validates and measures before writing, so
insufficient capacity returns `strings.BufferTooSmall` with the buffer unchanged.

NFC is a policy choice, not an automatic property of strings. Normalize both
external inputs when neither spelling is already canonical. Case folding is a
separate operation, and a grapheme count is not a terminal-column measurement.
Byte equality remains exact and case-sensitive after normalization. For substring
search instead, `strings.contains` applies the same exact matching rule.

For a numeric field, make trimming equally explicit. This fragment uses the
`strings` and `debug` imports above:

```meowy
quantity : strings.to_integer<uint16>(strings.trim_ascii(" 42 "))
| quantity <strings.ParseError> | debug.panic(quantity)
debug.print(quantity)   # 42 #
```

Without trimming, that input is a parse error; `"42cats"`, negative unsigned
values, and overflow remain errors after trimming. Parsing does not allocate.

See [UTF-8 and Unicode](../reference/stdlib/text-and-data.md#unicode-operations),
[number parsing](../reference/stdlib/text-and-data.md#numbers-mathematics-and-randomness),
and the [text-lab project](../programs/text-lab/README.md).

## Count words without wrestling the borrow checker

You just want to add one. The map lookup hands you a borrow, and the update wants
exclusive access. Give those two operations their own turns instead of trying
to make the borrow checker blink first.

Create this complete helper module as `tally.mwy`. A successful result is the new
count. The helper copies the small numeric value out of a scoped shared lookup
before obtaining exclusive access to change the map.

```meowy
collections : @"collections"
numbers : @"numbers"

-> <Counts> : <collections.Map<string, uint32>>
-> <Failure> : <collections.Duplicate<string, uint32>><collections.InsertFailure<string, uint32>><numbers.RangeError>

-> increment <uint32><Failure> : (counts <&!Counts>, word <string>) 'result {
    current <uint32> := 0
    'lookup {
        stored : counts.get(&word)
        | stored <null> | 'lookup.leave()
        current = *stored
    }

    next : numbers.checked_add(current, 1)
    | next <error> | {
        'result -> next
        'result.leave()
    }

    counts.remove(&word)
    inserted : counts.insert(word, next)
    | inserted <error> | {
        'result -> inserted
        'result.leave()
    }
    -> next
}
```

Use it from this complete `main.mwy`:

```meowy
collections : @"collections"
debug : @"debug"
iter : @"iter"
memory : @"memory"
tally : @"./tally.mwy"

allocated : collections.map<string, uint32>(
    memory.heap, 2, collections.hash_string, collections.equal_string
)
| allocated <error> | debug.panic(allocated)
counts := allocated
words <string[3]> : ["meowy", "tiny", "meowy"]
cursor := iter.slice(words.slice())

'words {
    word : cursor.next()
    | word <iter.End> | 'words.leave()
    count : tally.increment(&!counts, *word)
    | count <error> | debug.panic(count)
    debug.print("{*word}: {count}")
    'words.restart()
}
-> 0
```

Expected output follows the input order:

```text
meowy: 1
tiny: 1
meowy: 2
```

The map owns its table allocation, while these string keys refer to static
literals. For keys borrowed from a file buffer or decoded document, that owner
must outlive the map's uses. Putting a view into a map does not extend the view's
lifetime. Copy text into an explicit owner when the source must be released.

Capacity `2` is initial capacity, not a permanent limit. New distinct keys can
grow the table and fail allocation. `insert` does not overwrite: duplicates and
allocation failures return errors retaining the offered key and value. Removing
an old entry preserves capacity for its replacement. Arithmetic is checked before
removal, so count overflow leaves the old entry intact. This helper does not
claim a general transactional update for arbitrary fallible transformations.

Do not print `map.entries()` and assume insertion order. This recipe prints each
update as it occurs. For one final line per word, keep an explicit ordered key
list, as the [word-count project](../programs/word-count/README.md) does.

See [runtime-key maps](../reference/stdlib/collections.md#runtime-key-maps) and
[the existing count helper](../programs/word-count/count.mwy).

## Decode a bounded schema, then finish encoding before writing

Your tool needs two fields from a JSON document, not an invitation to accept any
shape a caller dreams up. Decode a known schema, keep its owner nearby, and finish
the outgoing JSON before showing it to the outside world.

This complete `main.mwy` decodes a fixed fixture into a typed record. The output
borrows the decoded name, so the document remains alive through serialization.
Escaped opening braces belong to meowy string syntax; the actual JSON input has
ordinary braces.

```meowy
debug : @"debug"
io : @"io"
json : @"json"
memory : @"memory"

<Config> : <{
    name <string>
    retries <uint8>
    labels <string[4]>
}>

wire : "\{\"name\":\"night shift\",\"retries\":3,\"labels\":[\"batch\",\"local\"]\}"
limits : {
    -> bytes <usize> : 1024
    -> depth <usize> : 8
    -> nodes <usize> : 32
}
document : json.decode<Config>(wire, memory.heap, limits)
| document <error> | debug.panic(document)
config : document.root()
summary : {
    -> name : config.name
    -> retries : config.retries
}

encoded <uint8[128]> := []
'encode {
    writer := io.buffer_writer(&!encoded)
    result : json.write(writer.write, summary)
    | result <error> | debug.panic(result)
}

output := io.stdout()
sent : io.write_all(output.write, encoded.slice())
| sent.error <io.Error> | debug.panic(sent.error)
newline : io.write_all(output.write, "\n".bytes())
| newline.error <io.Error> | debug.panic(newline.error)
-> 0
```

Expected output uses the encoder's record-field order:

```json
{ "name": "night shift", "retries": 3 }
```

Try a fifth label, `retries: 256`, an unknown property, or a duplicate property.
Each is a decode failure; the decoder does not silently discard or narrow it.
The byte/depth/node limits bound accepted work. They are not a promise that the
document occupies at most 1,024 heap bytes: decoded nodes, strings, and allocator
bookkeeping add storage. Allocation failure releases partial decoded state.

The `json.Document<Config>` owns decoded storage. `root()`, `config.name`, and
the summary's name are borrows; returning only `summary` from a helper that owns
and drops `document` would be invalid. Return the document owner, or explicitly
copy the data into an owner whose lifetime serves the caller.

The writer's exclusive borrow ends at `'encode`, making the encoded list readable.
A full buffer may contain an encoded prefix, but that prefix has not reached
stdout. Once `write_all` begins, output errors can still leave an externally
visible prefix; buffering prevents encoding failures from doing that, not I/O
failures. Larger bounded messages need a larger buffer or an explicitly allocated
storage choice.

See [JSON storage and limits](../reference/stdlib/text-and-data.md#json-with-bounded-work-and-explicit-storage),
[writer progress](../reference/stdlib/io-and-system.md#readers-and-writers), and the
[JSON report project](../programs/json-report/README.md).
