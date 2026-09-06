# JSON

[Library index](README.md) · [Text and data](text-and-data.md) · [I/O](io-and-system.md)

`@"json"` decodes a closed schema or a dynamic tree under one explicit Document
owner. It preserves numeric meaning and makes storage and accepted-work limits
visible. Its wire grammar is [RFC 8259 JSON](https://www.rfc-editor.org/rfc/rfc8259)
with the restrictions below; it does not use the language's numeric-literal grammar.

## Decode and retain the document

`json.decode<T>(text <string>, allocator, limits)` returns `<json.Document<T>>`,
`json.ParseError`, or `memory.AllocationFailure`. It accepts one complete JSON
value, with optional surrounding JSON whitespace (ASCII space, tab, CR, and LF).
It rejects a leading BOM, trailing non-whitespace, duplicate object member names,
and invalid escaped Unicode scalar pairs. Member-name equality compares decoded
UTF-8 exactly: `"a"` and `"\u0061"` are duplicates; canonically equivalent but
byte-distinct Unicode names remain distinct. It never normalizes or repairs text.

The document owns every decoded value, string, member name, and retained number
lexeme. It does not retain a borrow of the input text: unescaped strings are
copied too. The explicit allocator must outlive the document. Construction releases
all partial allocations on failure and leaves input unchanged. `document.root()`
returns `<&T>` borrowing the document. Root fields and nested views cannot outlive
that borrow; there is no `take_root` or move from its borrowed fields. Copy scalar
facts or construct separate owners when data must survive the document.

Document is move-only and has the conditional transfer/sharing capabilities in
the [capability rules](../tasks-and-channels.md#capability-rules).
Its destructor releases all decoded storage exactly once. Moving a document is
forbidden while any root or nested borrow is live. The decoder's no-input-borrow
result is an intrinsic storage contract; an ordinary wrapper still follows the
language's conservative public lifetime rule.

## Typed schemas and numbers

Supported `T` values are `boolean`, `null`, fixed-width integer types, `usize`,
`isize`, `float32`, `float64`, `string`, bounded lists, `collections.Array`,
`collections.Vector`, and null-primary records recursively containing supported
types. A supported non-null type may be unioned with `null`; no other union
decoding is provided. `json.Value` is also supported, including as a record field
or element. Other types are rejected statically, rather than producing a runtime
ParseError. Record aliases/expansions use their final field names; there are no
implicit wire-name annotations or case conversion.

JSON booleans, strings, objects, and arrays must match the target shape; numbers
are not coerced from strings. Missing nullable record fields become null. Other
missing fields and unknown fields are errors. Explicit null is accepted only by
a nullable target or `json.Value`. Fixed arrays require their exact length;
bounded lists cannot exceed capacity. Vector growth uses the document allocator.
Named-list alias metadata is not a wire schema: typed decoding rejects list
types carrying aliases, and ordinary bounded-list schemas carry none.

Integer decoding interprets the JSON decimal number exactly, including its
fraction and exponent. It succeeds only when the mathematical value is integral
and within the destination's range. Thus `1`, `1.0`, and `1e0` all decode to
integer one; `1.5` does not. Leading `+`, leading zeroes such as `01`, and bare
decimal points are invalid JSON. Every spelling of mathematical zero, including
`-0`, becomes integer zero, also for unsigned targets.

Float decoding rounds the exact decimal value to the destination IEEE binary
format using nearest, ties to even. Overflow that would round to infinity is an
error. Gradual underflow is supported: a result may be subnormal or signed zero,
with the JSON number's sign preserved on zero. No intermediate float64 rounding
is permitted when the target is float32. NaN and infinity tokens are invalid JSON.
These conversion rules also govern dynamic Number conversions below.

`json.Number` retains the exact valid source number lexeme, including exponent,
trailing fractional zeroes, and negative zero. Dynamic decoding imposes no
machine-integer or float exponent range: `18446744073709551616` and `1e1000000`
are valid dynamic numbers under the input limits. It does not allocate an expanded
decimal coefficient proportional to the exponent's numeric value. Explicit
machine conversions may reject them. A JSON number's finiteness is mathematical;
dynamic decoding never tests whether it fits a host float.

## Exact accepted-input budgets

Limits are an ordinary record with exactly `bytes <usize>`, `depth <usize>`, and
`nodes <usize>`, each positive. `json.DefaultLimits` is `{bytes: 1_048_576,
depth: 64, nodes: 100_000}` in descriptive notation. Invalid zero limits are a
ParseError with kind `"invalid_limits"` and position one; they do not panic.

- `bytes` counts every UTF-8 input byte, including whitespace and source escape
  spellings. Check the full input length before parsing or allocating a document.
  Excess input reports `"bytes_limit"` at one beyond the permitted prefix.
- `nodes` counts JSON values: the root counts one, every array element adds its
  value/tree nodes, and every object member adds its value/tree nodes. Containers
  count one each. Member names, commas, and whitespace add no nodes.
- `depth` counts enclosing containers, including a value itself when it is an
  array/object. A root scalar has depth zero; a root array/object has depth one;
  a nested container increases that depth by one. Scalars do not add a level.

For example, `{"x":0}` has two nodes and depth one; `[[]]` has two nodes and
depth two; `{"x":[0,null]}` has four nodes and depth two. Limits are inclusive.
During left-to-right parsing, charge a value's node and container depth at its
first token byte before descending or storing that value. Exceeding nodes or
depth reports that position, checking nodes first if both fail there. Earlier
syntax/type failures win; the initial whole-input byte check wins over parsing.
Limits apply equally to typed and dynamic decoding and are checked before a
corresponding schema conversion. A value's numeric conversion error points to
its first token byte. Missing required fields point to the object's closing brace;
unexpected EOF points to `text.size() + 1`.

These are input-acceptance limits, not an exact heap-byte quota. Syntax, decoding,
and numeric conversion must not expand work/storage according to an exponent's
numeric value. Allocator metadata, decoded UTF-8, and tree/storage bookkeeping
consume additional storage through the supplied allocator.

## Inspect dynamic values

`json.Value`, `json.Array`, `json.Object`, and `json.Number` are opaque immutable
borrowed views, not independently owned trees. Their storage remains in Document;
copying a view preserves that lifetime obligation. They are `memory.Copy`,
`tasks.Send`, and `tasks.Sync`, subject to the source lifetime and the prohibition
on sending non-static external borrows through channels. No dynamic JSON
mutation or independently owned dynamic-tree builder is part of this API.

`json.Kind` is the closed string-literal union `"null"`, `"boolean"`, `"number"`,
`"string"`, `"array"`, and `"object"`. The following methods borrow their receiver
and allocate nothing. Selecting an accessor on the wrong kind returns TypeError;
a prior `kind()` comparison is optional and does not change the accessor's type.

| API                             | Result                                | Contract                                                            |
| ------------------------------- | ------------------------------------- | ------------------------------------------------------------------- |
| `value.kind()`                  | `json.Kind`                           | Identify the represented JSON alternative.                          |
| `value.boolean()`               | `boolean` or `json.TypeError`         | Read a boolean.                                                     |
| `value.string()`                | `string` or `json.TypeError`          | Borrow decoded UTF-8 text.                                          |
| `value.number()`                | `json.Number` or `json.TypeError`     | Borrow the exact decimal number.                                    |
| `value.array()`                 | `json.Array` or `json.TypeError`      | Borrow an array view.                                               |
| `value.object()`                | `json.Object` or `json.TypeError`     | Borrow an object view.                                              |
| `array.size()`, `object.size()` | `usize`                               | Element or member count.                                            |
| `array.get(position <usize>)`   | `&json.Value` or `collections.Bounds` | One-based checked element access.                                   |
| `object.get(name <string>)`     | `&json.Value` or `null`               | Exact decoded-name lookup; absent differs from a present JSON null. |
| `object.members()`              | `json.MemberCursor`                   | Construct an inline cursor borrowing the object.                    |
| `cursor.next()`                 | `json.Member` or `iter.End`           | Yield the next member in parsed order.                              |
| `number.text()`                 | `string`                              | Borrow the complete original number lexeme.                         |
| `number.to_integer<T>()`        | `T` or `json.NumberError`             | Exact checked conversion to any supported integer type.             |
| `number.to_float<T>()`          | `T` or `json.NumberError`             | Checked rounded conversion to float32 or float64.                   |

`json.Member` is an ordinary immutable record with `name <string>` and
`value <&json.Value>`. `json.MemberCursor` is an opaque move-only inline cursor,
`Send` subject to its borrow lifetime, and not `Sync`; it allocates nothing and
has no ordinary equality. It requires exclusive access to advance its own position, while
retaining shared access to the object. Its yielded member borrows remain valid
for the source borrow, including after another advance; exhaustion is permanent.
The intrinsic lookup's result borrows only its receiver; the search-name text is
not retained. Array and scalar accessors likewise retain the source document
borrow, with no hidden ownership extraction.
Dynamic views do not implement ordinary equality. Compare selected scalar facts
or recursively compare values with an explicit object-order/numeric policy.

For example, this complete program inspects an unknown-schema member without
rounding its value through float64:

```meowy
debug : @"debug"
json : @"json"
memory : @"memory"

document : json.decode<json.Value>(
    "\{\"count\":9007199254740993\}", memory.heap, json.DefaultLimits)
| document <error> | debug.panic(document)
object : document.root().object()
| object <error> | debug.panic(object)
member : object.get("count")
| member <null> | debug.panic("Missing count")
number : member.number()
| number <error> | debug.panic(number)
count : number.to_integer<uint64>()
| count <error> | debug.panic(count)
debug.print(count)  # 9007199254740993 #
```

## Encode through an explicit writer

`json.write(write, value)` borrows the encodable value and returns `<null>`,
`json.EncodeError`, or `io.Error`. It accepts the typed schema family above,
unions whose every alternative is encodable, immutable slices of encodable
elements, and the dynamic Value/Array/Object/Number views. Encoding a named list
emits its initialized elements as an array, ignoring its lookup aliases. Records
must have a null primary. References are dereferenced as borrows, not serialized
as addresses; recursive reference types, raw pointers, functions, arbitrary
resource owners, errors, dates, and other unsupported types are static errors.
Passing a Document itself is unsupported; pass its `root()` borrow.

Null fields are written explicitly. Record fields use lexicographic UTF-8 name
order; dynamic object members retain parsed order. Arrays preserve element order.
Dynamic Number writes its original lexeme exactly. Integers use base-ten digits,
with a leading minus only for negative signed values. Typed finite floats use
the decimal representation with the fewest significant digits that round-trips to the same binary value
under the decoder's rounding rule; negative zero writes `-0`. Among equally short
representations choose the numerically closest, then an even final significand
digit on a tie. Exponent notation and insignificant JSON whitespace may vary;
applications needing byte-canonical JSON must specify a separate encoding.
Typed NaN/infinity produce EncodeError. Strings and names emit valid JSON escapes;
the choice between a scalar's literal UTF-8 and equivalent escapes is not fixed.

Encoding never invokes user getters, follows arbitrary owner internals, or
allocates intermediate text. Traversal requires no application allocator. It may
fail after writing a prefix, including when a later field contains a nonfinite
float. Writer errors retain the writer's committed progress semantics; `json.write`
does not retry the whole message. Use an explicit memory writer first when no
encoding failure may expose a partial external message.

## Error facts

JSON errors are nominal and allocation-free; all require explicit boxing for
erasure under the [library error classification](errors.md#choose-inline-storage-or-explicit-erasure).
Applications normally inspect them directly. `errors.code` returns respectively
`json.parse`, `json.type`, `json.number`, and `json.encode`; messages are static
explanations, not copies of the source input.

- ParseError exposes `position() <usize>`, `kind() <string>`, and
  `limit() <usize><null>`. Kinds are `"syntax"`, `"duplicate_name"`,
  `"schema"`, `"number"`, `"invalid_limits"`, `"bytes_limit"`,
  `"nodes_limit"`, or `"depth_limit"`. Limit is the configured bound only for
  a `*_limit` kind. Duplicate names point to the second name's opening quote.
- TypeError exposes `expected() <json.Kind>` and `actual() <json.Kind>`.
- NumberError exposes `kind()` as `"fractional"` or `"out_of_range"`.
  An integral value outside the target range is out_of_range; a nonintegral value
  is fractional even if its magnitude is also outside an integer target's range.
- EncodeError exposes `kind()` as `"nonfinite"`; unsupported source types were
  already rejected statically. It carries no borrowed pointer into a value after
  the call finishes.

The [stdlib source cases](../../programs/testing/tests/stdlib_test.mwy) exercise
dynamic lookup, exact numbers, limit boundaries, and custom CLI/I/O failures.
The [JSON report](../../programs/json-report/README.md) keeps a typed schema and
finishes encoding into caller storage before writing to stdout.
