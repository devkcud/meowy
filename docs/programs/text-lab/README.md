# Text lab

[Worked programs](../README.md) · [Text and Unicode APIs](../../reference/stdlib/text-and-data.md)

Measure the same visible word before and after NFC normalization. The source
literal ends with two code points, U+0065 followed by U+0301. Keep that literal
decomposed when editing it; the distinction is the point of the example.

From the repository root:

```sh
cd docs/programs/text-lab
meowy check
meowy run
```

Program output:

```text
Original: 6 bytes, 5 scalars, 4 graphemes
NFC: 5 bytes, 4 scalars, 4 graphemes
Normalized: café
```

[mod.mwy](mod.mwy) selects [main.mwy](main.mwy). The reusable
[measure.mwy](measure.mwy) helper consumes borrowed Unicode cursors and returns
three inline counters. A byte length is useful for storage and wire formats;
scalar and grapheme counts answer different text-processing questions. A grapheme
count does not promise a terminal-column count.

The input literal has static storage. The normalized text borrows a 32-byte
inline list owned by the main scope; the formatter finishes reading that view
before the list is released. The helper returns only counts, so it does not
retain the text or cursor. No operation selects a heap allocator.

To explore capacity failure, change the output buffer to `<uint8[4]>`. NFC needs
five bytes, so normalization reports `strings.BufferTooSmall` and leaves the
empty buffer unchanged. The program writes the error to stderr and exits with
status 1. Stream failures also produce status 1; streamed output can already
contain a prefix. Success produces status 0.
