# Word count

[Worked programs](../README.md) · [Map contracts](../../reference/stdlib/collections.md#runtime-key-maps)

Count words across a small text buffer, preserve their first-seen order, and
report repeated words. Splitting uses literal ASCII spaces inside each line;
empty segments are skipped after ASCII trimming. Case and punctuation remain
significant. This intentionally simple token policy is separate from Unicode
word segmentation.

From the repository root:

```sh
cd docs/programs/word-count
meowy check
meowy run
```

Program output:

```text
meowy: 2
makes: 2
tiny: 1
tools: 2
```

[mod.mwy](mod.mwy) selects [main.mwy](main.mwy).
[count.mwy](count.mwy) exports the map type and one increment operation. Its
lookup scope copies the current count and ends the shared element borrow before
removing or inserting an entry. Overflow is checked before removal. Replacement
reuses the retained map capacity, and insertion does not silently overwrite an
equal key.

The map explicitly selects `memory.heap`, `collections.hash_string`, and
`collections.equal_string`. Its initial capacity of four can grow; a separate
inline list limits this report to sixteen distinct words and preserves output
order. Map iteration order is never used as a presentation guarantee.

Map keys and the order list contain borrowed UTF-8 views into the input literal.
That input has static storage, so every stored view remains valid. A version
reading an owned input buffer must keep that owner alive until both collections
are finished. The map owns its table allocation and releases it when the main
scope ends. Its keys do not individually allocate strings.

Try a seventeenth distinct word to exercise the report capacity. Failed list
append retains the unchanged list and rejected word. A failed map insert retains
its key and proposed count; the program inspects those concrete fields and then
releases the failure value during scope exit. It also handles count overflow,
allocation failure, and output failure. Every failure stops the report with
status 1; successful completion produces status 0. Input splitting and counting
finish before any result lines are written, although a later output failure may
still leave a partial report.
