# JSON session report

[Worked programs](../README.md) · [JSON contracts](../../reference/stdlib/text-and-data.md#json-with-bounded-work-and-explicit-storage)

Decode a small session log into a closed record type, validate its application
rules, and publish both a human-readable summary and a JSON summary. The input
is a local literal, so the example needs no fixture path or external service.

From the repository root:

```sh
cd docs/programs/json-report
meowy check
meowy run
```

Program output, with insignificant JSON whitespace omitted:

```text
night shift: 3 sessions, 110 minutes
{"minutes":110,"sessions":3,"team":"night shift"}
```

[mod.mwy](mod.mwy) selects [main.mwy](main.mwy).
[report.mwy](report.mwy) exports the wire schema and a separate validation helper.
The schema permits at most eight sessions and requires each duration to fit
`uint16`. The application further requires nonempty trimmed names and durations
from 1 through 480 minutes. Widening each duration to `uint32` is explicit;
even eight maximum valid durations fit the accumulator.

`json.Document<report.Input>` owns decoding storage allocated through
`memory.heap`. Its root is borrowed, and the summary's `team` string retains
that borrow. The document therefore stays alive until formatting and encoding
finish; returning the borrowed summary after dropping the document would be
invalid. The document releases its allocation on every scope exit.

JSON encoding first writes into an inline 256-byte list. Leaving the `encode`
scope ends the writer's exclusive borrow before `encoded.slice()` is read.
Encoding failure can leave a prefix in this private buffer, but that prefix is
never sent to stdout. Once writing to stdout begins, an I/O failure can still
leave externally visible partial output.

Try a zero duration to exercise application validation, an unknown JSON field
to exercise strict schema decoding, or a ninth session to exceed the wire
capacity. A smaller output buffer exercises the encoder's bounded writer.
Failures are reported to stderr where possible and produce status 1; success
produces status 0. JSON whitespace is not used as an application contract, while
the summary record's field order is lexicographic as specified by `json.write`.
