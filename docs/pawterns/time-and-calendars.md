# Time, calendars, and tomorrow's surprises

[Pawterns](README.md) · Previous: [CLI apps and files](cli-and-files.md) · Next: [Tasks and channels](tasks-and-channels.md)

"Wait a second" is easy. "Same time tomorrow" is where the calendar starts
asking follow-up questions. meowy gives elapsed time, civil time, zones, and
calendars different values so those questions stay visible in your code.

Each complete entry below belongs in its own [small project](first-project.md).
The fixed fixtures panic on unexpected errors to keep the interesting operations
in view. For user-supplied dates or durations, use the recoverable status and
output handling in [CLI applications](cli-and-files.md).

## Give a timeout one clock

Suppose an operation gets six slices of 250 milliseconds. Start with exact
durations; floats need not attend this meeting.

Complete `main.mwy`:

```meowy
debug : @"debug"
time : @"time"

slice : time.parse_duration("250ms")
| slice <error> | debug.panic(slice)
budget : slice.try_scale(6)
| budget <time.RangeError> | debug.panic(budget)

parts : budget.split(time.Second)
debug.print("Budget: {budget}")
debug.print("Nanoseconds: {budget.nanoseconds()}")
debug.print("Whole seconds: {parts.whole}")
debug.print("Remainder: {parts.remainder}")
-> 0
```

Expected output:

```text
Budget: 1.5s
Nanoseconds: 1500000000
Whole seconds: 1
Remainder: 0.5s
```

`try_scale` reports overflow as a value. For a factor supplied by a person or a
configuration file, that is usually a more useful boundary than the checked
`scale`, which panics on runtime overflow. A splitting unit must be positive;
this fixture uses the known `time.Second`. Validate a user-supplied unit before
calling `split`, as the [duration CLI](../programs/duration-cli/README.md) does.

The duration parser accepts exact components such as `1h30m`, `250ms`, or `-1.5s`.
It rejects `1month`, embedded whitespace, excess nanosecond precision, and
out-of-range totals. `time.Day` means exactly 24 hours; a calendar month has no
duration constant hiding somewhere under the sofa.

To turn the computed budget into one actual deadline, insert this fragment
before the final `-> 0` above:

```meowy
started : time.now()
deadline : started.add(budget)
time.sleep_until(deadline)
elapsed : time.since(started)
debug.print("Elapsed: {elapsed}")
```

The last line varies. The wait cannot return before that deadline on the selected
clock; scheduling can make it later. Reuse `deadline` when several operations
share this budget. Recomputing `time.after(budget)` at every retry grants a fresh
budget every time, which is a delightful way to make a timeout never time out.

Elapsed measurements use a monotonic `time.Instant`; changing the civil clock
does not move this deadline. An Instant belongs to its runtime's clock domain
and is not a value to serialize for tomorrow's process. Save durations or civil
intent instead, according to what must survive a restart.

The duration-only program needs inline values, arithmetic, and output support;
it does not initialize a clock. The optional wait adds clock/wait support, but
outside an executor it blocks the calling host thread without creating a task.
Inside a task, it suspends that task and checks cancellation. An owned Timer or
Ticker is a separate choice with explicit allocator-backed state; see the
[ticker project](../programs/ticker/README.md).

See [fixed durations](../reference/stdlib/time-and-date.md#fixed-durations),
[clock domains and waits](../reference/stdlib/time-and-date.md#clocks-deadlines-and-waits),
and [task deadlines](tasks-and-channels.md).

## Tomorrow is not always 24 hours away

You promise lunch at noon tomorrow. Nobody asked for lunch at 13:00 because the
clock changed overnight. Choose calendar arithmetic when the promise is about
the local hour.

This complete `main.mwy` uses the same pinned Zurich transition fixture as the
[time guide](../guide/time-and-date.md#choose-between-tomorrow-and-24-hours-later):

```meowy
date : @"date"
debug : @"debug"
time : @"time"

zone : date.zone("Europe/Zurich")
| zone <error> | debug.panic(zone)
local : date.parse_civil("2023-03-25T12:00:00", date.ISODateTime)
| local <error> | debug.panic(local)
start : date.resolve(local, zone, date.Strict)
| start <error> | debug.panic(start)

tomorrow : start.add_days(1, date.Strict)
| tomorrow <error> | debug.panic(tomorrow)
after_a_day : start.add(time.Day)
| after_a_day <error> | debug.panic(after_a_day)
elapsed : tomorrow.timestamp().since(start.timestamp())
| elapsed <error> | debug.panic(elapsed)

debug.print("Tomorrow: {tomorrow}")
debug.print("After 24 hours: {after_a_day}")
debug.print("Until tomorrow: {elapsed}")
-> 0
```

Expected output under the documented zone rules:

```text
Tomorrow: 2023-03-26T12:00:00+02:00
After 24 hours: 2023-03-26T13:00:00+02:00
Until tomorrow: 23h
```

The named zone is an explicit input. Parsing a timestamp with `+01:00` creates a
fixed offset, which cannot recover the identity `Europe/Zurich` or its changing
rules. The host's `TZ`, locale, or current location does not select this zone.

For external local-time input, `date.Strict` can produce `date.Ambiguous` when
an hour repeats, or `date.Nonexistent` when an hour is skipped. The former carries
`earlier` and `later` candidate DateTimes; the latter carries the requested Civil,
zone, and exact gap Duration. A date outside the representable range also fails.
These are decisions to explain to the caller, not hours to silently guess.

If your application has an explicit policy to choose the later occurrence of a
repeated hour and reject skipped hours, replace only the resolution call with:

```meowy
start : date.resolve(local, zone, date.Later)
```

Retain the error guard. `Later` resolves a fold; it does not make gaps disappear.
Pass the same deliberate policy to subsequent calendar operations when that is
the recurrence rule. `date.Strict`, `Earlier`, and `Later` are ordinary values,
so a helper can accept the chosen policy without growing a new syntax feature.

For an observed event, persist its `date.Timestamp`. For a future local
appointment, preserve the Civil, zone name, and resolution policy. Preserve the
zone rule version too when the exact resolution must remain reproducible; rules
are versioned build data and can change with an explicit toolchain/library update.

Date, Civil, Timestamp, Zone, and DateTime values are inline and copyable. Named
zone rules have program lifetime; date construction and lookup do not allocate a
heap object per appointment. Calendar operations do not start timers or threads.
An unrestricted runtime zone lookup can retain the complete promised lookup
dataset in the binary, while UTC and fixed offsets need no named-zone database.

See [local-time resolution](../reference/stdlib/time-and-date.md#resolve-local-times-explicitly),
[timestamps and zones](../reference/stdlib/time-and-date.md#timestamps-and-zones),
and [binary retention](../reference/optimization.md).

## Send a date to the Chinese calendar

Same day, different labels. No time machine and, happily, no background lunar
thread. Select a calendar value, convert the view, and keep its identity when
you serialize it.

Complete `main.mwy`:

```meowy
calendars : @"calendars"
date : @"date"
debug : @"debug"

day : date.make_date(2026, date.February, 17)
| day <error> | debug.panic(day)
lunar : day.in_calendar(calendars.Chinese)
| lunar <error> | debug.panic(lunar)

buffer <uint8[64]> := []
text : calendars.format_into(lunar, calendars.DateOnly, &!buffer)
| text <error> | debug.panic(text)

debug.print(text)
debug.print("Same day: {day.same_day(lunar)}")
debug.print("Month code: {lunar.month().code()}")
-> 0
```

Expected output:

```text
chinese:2026-M01-01
Same day: true
Month code: M01
```

This conversion follows the [documented Chinese fixture and data profile](../reference/stdlib/calendars.md#chinese-lunisolar-dates).
Its related year is the Gregorian year in which that Chinese New Year occurs.
The pinned profile supports Gregorian 1901-01-01 through 2100-12-31; going beyond
that range returns `date.RangeError` rather than inventing a plausible moon.

Use `same_day` for this comparison. Ordinary equality also compares calendar
identity and rule version, so two labels for the same day need not be equal
values. Changing the calendar also does not select a time zone: Chinese calendar
fields do not automatically imply Beijing time.

Leap months retain a code such as `M06L`. They are not renamed to ordinary month
seven merely because they occupy the seventh position in a particular year.
`calendars.DateOnly` includes the calendar ID and leap marker; omitting the marker
when formatting a leap month is an error. Use this calendar-aware formatter, or
convert back to Gregorian before producing a Gregorian protocol date.

For a lunar anniversary, preserve the calendar ID, rule version, related-year
convention, month code, and day. Adding a fixed `time.Year` cannot help: no such
duration exists. Calendar `add_years` can itself fail if the target year lacks
the requested leap month. The application must choose what that anniversary
means in such a year.

All date/calendar values here are inline. Formatting writes into 64 caller-owned
bytes and returns a view borrowing them; keep the buffer unchanged while `text`
is in use. A formatting-capacity failure leaves the original buffer unchanged.
Direct use of `calendars.Chinese` retains its needed rules and helpers; it does
not require every other calendar in the executable. A CLI accepting unrestricted
runtime calendar IDs must retain the support it advertises.

The [calendar CLI](../programs/calendar-cli/README.md) turns this into a typed
`--calendar` option with range errors reported as application failures. See
[calendar-aware text](../reference/stdlib/calendars.md#calendar-aware-text-and-serialization)
and [how calendar data reaches a binary](../reference/optimization.md).
