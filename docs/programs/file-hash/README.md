# Stream a file through SHA-256

[Program collection](../README.md) · [Source](main.mwy) · [Project settings](mod.mwy)

This command hashes a file without loading it into memory. It combines a typed
CLI, an owned file, a reusable initialized byte buffer, incremental SHA-256,
and lowercase hex formatting into caller-owned storage.

From the repository root:

```sh
cd docs/programs/file-hash
meowy check
meowy run -- message.txt
```

The application prints:

```text
b062f36e550f8a1c726f4f00355b5e836654874c8a9809965b4b948262087812
```

[message.txt](message.txt) contains `meowy says hello` followed by LF: 17 bytes
total. Changing its newline changes its digest. For another input,
pass a path relative to the working directory, or an absolute path:

```sh
meowy run -- --help
meowy run -- /path/to/archive.bin
```

`process.arguments(memory.heap)` owns the argument storage; the parse result
and `path.Path` borrow from it. The file handle remains owned by the entry scope.
Each iteration takes an exclusive scratch slice for reading, then a shared view
of only the returned prefix. The read borrow ends before that shared view begins;
the view ends before the next read overwrites scratch.

A read may return bytes together with EOF or an error. The hash receives those
bytes before either condition is examined. An error still prevents printing a
digest, so a partial input is never presented as a complete-file hash. The read
scope emits its result and explicitly leaves on failure; emission alone would
continue running the loop.

The file is explicitly closed after the read scope, even when hashing failed.
Read/hash and close failures are both reported. Only a complete read and
successful close proceed to digest formatting. The hex encoder replaces a
64-byte bounded list; the resulting UTF-8 string borrows that list until output
finishes. No intermediate heap string is required.

Working storage is a 4,096-byte inline read buffer, inline hash state, a 32-byte
digest, and a 64-byte inline hex buffer, plus argument storage and the host file
handle. File size does not increase these buffer capacities.

Help and version return status `0`; invalid arguments or path syntax return `2`.
Open, read, hash-limit, close, encoding, and output failures return `1`. No digest
is printed on an input or close failure. A failed stdout write may expose a
prefix of the digest before reporting failure.

The relevant contracts are [I/O and files](../../reference/stdlib/io-and-system.md),
[hashing and encoding](../../reference/stdlib/text-and-data.md#binary-encodings-and-digests),
and [typed CLI descriptions](../../reference/stdlib/cli.md).
