# Copy with explicit progress and cleanup

[Program collection](../README.md) · [Source](main.mwy) · [Project settings](mod.mwy)

This command copies an existing file into a newly created destination. It gives
`io.copy` explicit read/write callables and 4,096 bytes of initialized scratch,
then observes the destination's sync result and both close results.

From the repository root:

```sh
cd docs/programs/file-copy
meowy check
meowy run -- message.txt copied.txt
```

With no existing `copied.txt`, the application prints:

```text
Copied: 17 bytes
```

The new file contains exactly the bytes of [message.txt](message.txt), including
its final LF. Running the command again fails because `fs.CreateNew` refuses an
existing destination. The source is opened read-only. Selecting the same path
for both arguments therefore also fails without truncating the source.

```sh
meowy run -- --help
meowy run -- message.txt message.txt
```

After inspecting the copy, remove only the generated file if desired:

```sh
rm -- copied.txt
```

The copier retains counts of bytes read and bytes written separately. A write
failure can leave those counts different; stderr reports both counts alongside
the transfer error. It does not retry the entire file or claim that a partial
destination is complete. A read returning a valid prefix together with an error
still contributes its actual progress under the I/O contract.

Once the destination opens, sync and both closes are attempted before fallible
reporting starts. Sync also runs after a transfer failure to request flushing
the prefix that was written; it does not turn that transfer into success. A
failed close consumes its owner and is never retried. If destination creation
fails, the source is still explicitly closed and its close error is observed.

On transfer, sync, or close failure, the new destination remains available for
inspection and may be incomplete. This command does not promise atomic replacement
or delete partial output automatically. A successful sync requests the host's
file durability contract; it does not independently sync the containing directory.

Storage consists of one inline 4,096-byte scratch buffer, two owned host file
handles, scalar progress counters, and explicitly allocated argument storage.
The borrowed read and write callables expire when `io.copy` completes, allowing
the file owners to be synced and consumed afterward.

Help/version return status `0`; CLI and path-syntax errors return `2`. File,
transfer, sync, close, and output failures return `1`. A failed success-message
write can leave a complete copied file while the command reports status `1`.

See [I/O and file ownership](../../reference/stdlib/io-and-system.md#files-and-directories)
and [the CLI contract](../../reference/stdlib/cli.md).
