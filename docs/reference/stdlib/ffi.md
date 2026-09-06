# Native access

[Library index](README.md)

## Native access

`ffi.extern<Signature>("C", symbol)` resolves a declared link-time symbol as an
unchecked function pointer with type `<!(Parameters) -> Result>`. The signature
must contain only ABI-compatible types, and calls require a `!{ ... }` block.
`ffi.record("C", fields)` is a compile-time type constructor taking an ordered
list of named field types, such as `["x" : <float32>, "y" : <float32>]`. It
applies the target C ABI in list order. Both are ordinary named function values;
they are not special words in the grammar.
`ffi.c_int` and related aliases reflect the build target. There is no implicit
marshalling of strings, records, callbacks, or unions. See
[native interfaces](../modules-and-ffi.md#native-interfaces).
