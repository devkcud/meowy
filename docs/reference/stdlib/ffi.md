# Native access

[Library index](README.md)

## Native access

`ffi.extern<Signature>("C", symbol)` resolves a declared link-time symbol through
a compiler-generated ABI adapter. Its result is an unsafe **meowy-callable**
function pointer of type `<!(Parameters) -> Result>`; it is not the address of
the C function. Calls require a `!{ ... }` block. Convention and symbol are
compile-time string literals; this profile accepts only `"C"`.
`ffi.record("C", fields)` is a compile-time type constructor taking an ordered
list of named field types, such as `["x" : <float32>, "y" : <float32>]`. It
applies the target C ABI in list order. Both are ordinary named function values;
they are not special words in the grammar.

The [native ABI profile](../native-abi.md) enumerates every supported C alias,
parameter/result/field type, the `null` result mapping to C `void`, and adapter
assignment rules. An unlisted type is rejected; a native symbol's actual contract
is never inferred from its spelling. There is no implicit marshalling of strings,
ordinary records, callbacks, or unions. Native function addresses, callbacks,
native exports, dynamic loading, and host embedding are outside this profile.

Raw pointer arguments still require caller-proven lifetime, alignment, bounds,
initialization, and access rights. The adapter changes the calling convention,
not those obligations. [Native interfaces](../modules-and-ffi.md#native-interfaces)
describes exact link inputs and safe-wrapper responsibilities.
