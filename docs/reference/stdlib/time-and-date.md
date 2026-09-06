# Time, dates, and calendars

[Documentation index](../../README.md) · [Practical guide](../../guide/time-and-date.md) · [Library index](README.md)

`@"time"` supplies elapsed-time values, clocks, waits, and timers. `@"date"`
supplies civil dates, times of day, timestamps, time zones, and calendar arithmetic.
Both are foundational modules. Their names, units, months, and policy values are
ordinary bindings; neither module adds syntax or reserved words.

The ergonomics take inspiration from [Go's time package](https://pkg.go.dev/time).
Meowy keeps monotonic instants and civil timestamps in separate types, with
explicit storage and typed failures. Duration operations are ordinary calls,
so they also compose through dispatch without adding operator overloading.

## Choose the right value

| Type                        | Meaning                                      | Representation and ownership                                   |
| --------------------------- | -------------------------------------------- | -------------------------------------------------------------- |
| `time.Duration`             | A signed, fixed elapsed interval             | Opaque value containing an `<int64>` nanosecond count          |
| `time.Instant`              | A position on this runtime's monotonic clock | Opaque inline tick value and clock-domain identity             |
| `date.Date`                 | A civil day viewed in a selected calendar    | Inline day number and calendar identity; no clock or time zone |
| `date.Time`                 | A time of day, with nanoseconds              | Validated inline value, no date or time zone                   |
| `date.Civil`                | A date paired with a time of day             | Inline `Date` and `Time`; not yet an instant                   |
| `date.Timestamp`            | An absolute position on the Unix time scale  | Signed seconds and a normalized nanosecond fraction            |
| `date.Zone`                 | UTC, a fixed offset, or versioned IANA rules | Immutable inline descriptor; rule data has program lifetime    |
| `date.DateTime`             | A timestamp viewed in a zone and calendar    | Inline `Timestamp`, `Zone`, and calendar identity              |
| `time.Timer`, `time.Ticker` | An owned source of timed wakeups             | Move-only handles with explicit allocator-backed state         |

Duration, Instant, Date, Time, Civil, Timestamp, Zone, and DateTime satisfy
`memory.Copy`, `tasks.Send`, and `tasks.Sync`. Timer and Ticker remain move-only. Their
ordinary layout is target-defined, not a serialization format or a C ABI. A
`DateTime` does not contain a monotonic reading, and an `Instant` cannot be cast
or formatted into a calendar date. Neither duration units nor type aliases erase
these distinctions.

The listed copyable domain types support ordinary `==` and `!=`; their identity
rules are specified below. Duration equality compares nanoseconds, and Instant
equality compares the clock-domain identity and tick value, returning false for
different domains. Ordered Instant operations still require one clock domain.
Time equality compares its four validated components; Timestamp equality compares
normalized seconds and nanoseconds. Month, Weekday, and MonthPolicy values also
support equality, comparing month code/leap marker, weekday number, and selected
policy respectively. Ordinary ordering/arithmetic operators remain unavailable;
use the named operations. Timer and Ticker do not support ordinary equality.
Month, Weekday, and MonthPolicy have `memory.Copy`, `tasks.Send`, and `tasks.Sync`;
their values contain no external resource or non-static borrow.

## Fixed durations

These constants all have type `<time.Duration>`:

| Value              | Exact length       |
| ------------------ | ------------------ |
| `time.Nanosecond`  | 1 nanosecond       |
| `time.Microsecond` | 1,000 nanoseconds  |
| `time.Millisecond` | 1,000 microseconds |
| `time.Second`      | 1,000 milliseconds |
| `time.Minute`      | 60 seconds         |
| `time.Hour`        | 60 minutes         |
| `time.Day`         | 24 hours           |
| `time.Week`        | 7 fixed days       |
| `time.Zero`        | Zero nanoseconds   |

A duration is bounded by the signed 64-bit nanosecond range, approximately 292
years in either direction. `Day` and `Week` are useful for timeouts and elapsed
budgets; they do not mean the next calendar day or week. There is no `time.Month`
or `time.Year`, since those have no fixed length. Calendar months and years belong
to date operations.

```meowy
time : @"time"

budget : time.Hour.scale(2).add(time.Minute.scale(30))
retention : time.Week.scale(2)
deadline : time.after(time.Second)
```

Meowy has no implicit operator overloading. `time.Hour.scale(2)` keeps a duration
typed as a duration; `2 * time.Hour` is not numeric multiplication. Each duration
member has an equivalent module function: `time.scale(time.Hour, 2)` and
`time.Hour.(time.scale, 2)` call the same operation. Aliasing those functions or
unit values preserves their behavior.

| API                                                                | Result                                           | Contract                                                   |
| ------------------------------------------------------------------ | ------------------------------------------------ | ---------------------------------------------------------- |
| `time.ns(count <int64>)`                                           | `Duration`                                       | Construct an exact signed nanosecond interval              |
| `time.ms(count <uint64>)`                                          | `Duration`                                       | Construct nonnegative milliseconds; checked multiplication |
| `duration.nanoseconds()`                                           | `int64`                                          | Extract the exact stored count                             |
| `duration.add(other)`, `.sub(other)`                               | `Duration`                                       | Checked addition/subtraction                               |
| `duration.scale(factor <int64>)`                                   | `Duration`                                       | Checked integer multiplication                             |
| `duration.divide(divisor <int64>)`                                 | `Duration`                                       | Integer division, truncating toward zero                   |
| `duration.negate()`, `.abs()`                                      | `Duration`                                       | Checked sign change; the minimum count cannot be negated   |
| `duration.compare(other)`                                          | `int32`                                          | `-1`, `0`, or `1` in elapsed-length order                  |
| `duration.whole(unit)`                                             | `int64`                                          | Count whole positive units, truncating toward zero         |
| `duration.split(unit)`                                             | Record `{ whole <int64>; remainder <Duration> }` | Exact quotient and same-sign remainder for a positive unit |
| `duration.try_add(other)`, `.try_sub(other)`, `.try_scale(factor)` | `Duration` or `time.RangeError`                  | Recoverable overflow, with operands unchanged              |
| `time.parse_duration(text)`                                        | `Duration` or `time.ParseError`                  | Parse the exact duration grammar below                     |

Member equivalents pass the duration as the first argument; for example,
`time.add(duration, other)`. Arithmetic never wraps or silently uses floating
point. An invalid checked operation uses `E107` when established statically or
`P002` when dependent on runtime input. Division by zero, a nonpositive unit, and
range overflow follow that rule. A checked constructor remains distinct from a
parser returning an error value for malformed external text.

All fixed-duration constructors, arithmetic, scalar accessors, comparisons and
`time.parse_duration` above satisfy `core.Pure`. They can run during required
evaluation with constant inputs and the same checked failures. This does not
grant purity to clock reads, sleep, timers, tickers or deadline operations.

Duration text has an optional leading sign followed by one or more decimal
number/unit components, such as `"2h30m"`, `"250ms"`, or `"-1.5s"`. Units are
`ns`, `us`, `ms`, `s`, `m`, `h`, `d`, and `w`; `d` and `w` have the fixed lengths
above. A fractional component must have a digit on both sides of its decimal
point and denote an exact number of nanoseconds. Whitespace, exponents, embedded
signs, unitless values other than `"0"`, excess precision, and overflow are errors.
Components may repeat and are summed exactly before the final range check.

The diagnostic display uses one sign and descending `w`, `d`, `h`, `m`, `s`
components, with a trimmed nanosecond fraction on seconds; zero prints `0s`.
Thus 2.5 hours prints `2h30m` and one millisecond prints `0.001s`. Display streams
without allocating a string. `time.format_into(duration, &!buffer)` instead writes
into a caller-owned `<uint8[N]>` list and returns a borrowed `<string>` or
`time.FormatError`, following the buffer rules below.

## Clocks, deadlines, and waits

| API                         | Result                 | Contract                                                               |
| --------------------------- | ---------------------- | ---------------------------------------------------------------------- |
| `time.now()`                | `Instant`              | Read the monotonic clock                                               |
| `time.after(duration)`      | `Instant`              | Sample once and add the duration; this creates a deadline, not a timer |
| `instant.add(duration)`     | `Instant`              | Shift a monotonic instant with checked range                           |
| `instant.since(earlier)`    | `Duration`             | Signed monotonic difference; checked duration range                    |
| `instant.compare(other)`    | `int32`                | `-1`, `0`, or `1` within the same clock domain                         |
| `time.since(start)`         | `Duration`             | `time.now().since(start)`                                              |
| `time.until(deadline)`      | `Duration`             | `deadline.since(time.now())`; negative after expiry                    |
| `time.sleep(duration)`      | `null` on continuation | Wait for a relative interval                                           |
| `time.sleep_until(instant)` | `null` on continuation | Wait against an existing monotonic deadline                            |
| `time.clock_info()`         | Record                 | `resolution <Duration>` and `includes_suspend <boolean>`               |

The monotonic clock is nondecreasing and unaffected by setting the civil clock.
Its epoch has no public calendar meaning. A program with reachable clock, wait,
or timer operations must initialize its monotonic source before any module
initializer uses it and before program entry. Using only duration values and
arithmetic does not initialize a clock or start an executor. The clock's
resolution and whether it advances during machine suspend are target/runtime
properties exposed by `clock_info` and recorded in diagnostics. Nanosecond
representation does not promise nanosecond hardware resolution.

An `Instant` is valid only in its originating runtime clock domain, including
that runtime's child tasks. There is no portable byte constructor, Unix conversion,
or serialization API for it. An unchecked host adapter must validate the clock
domain before constructing a safe Instant. Forging an instant from another
runtime violates that adapter's safety contract; the public API supplies no such
conversion.

`after` accepts negative durations, producing a past deadline. Nonpositive sleeps
and already-expired waits return immediately, after checking pending cancellation.
A positive sleep returns no earlier than its deadline on the selected clock;
scheduling and cleanup may delay return. Inside a task, waits suspend that task
and are cancellation points. Outside an executor, sleep blocks the calling host
thread without starting an executor or background task.

Use the same `Instant` to give several operations one overall budget. Passing a
`date.Timestamp` or `date.DateTime` to `job.deadline` is a type error. Task deadline
inheritance, cancellation, and mandatory joining remain governed by
[tasks and channels](../tasks-and-channels.md#deadlines-and-cancellation).

## Owned timers and tickers

| API                               | Result                                                      | Contract                                                            |
| --------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------- |
| `time.timer(duration, allocator)` | `Timer` or `memory.AllocationFailure` or `time.RangeError`  | Allocate and arm one relative wakeup                                |
| `timer.wait()`                    | `Instant` or `time.Stopped`                                 | Wait for and consume the armed event; return its scheduled deadline |
| `timer.reset(duration)`           | `null` or `time.RangeError`                                 | Replace any pending event with a fresh relative deadline            |
| `timer.stop()`                    | `null`                                                      | Disarm and discard a pending event; idempotent                      |
| `time.ticker(period, allocator)`  | `Ticker` or `memory.AllocationFailure` or `time.RangeError` | Allocate periodic state; period must be positive                    |
| `ticker.next()`                   | `Tick` or `time.Stopped`                                    | Wait for the next periodic observation                              |
| `ticker.reset(period)`            | `null` or `time.RangeError`                                 | Discard pending ticks; anchor a new positive period at now          |
| `ticker.stop()`                   | `null`                                                      | Disarm and discard pending ticks; idempotent                        |
| `timer.close()`, `ticker.close()` | `null`                                                      | Consume the owner and release state                                 |

Operations require exclusive access to their owner. Handles may move between tasks
but cannot be used concurrently through a shared borrow; a child waiting on a
handle is cancelled through its task. Waits acknowledge cancellation with ordinary
unwinding. Dropping an owner disarms it and releases its state without requiring a
final tick or allocating during cleanup. Its allocator must outlive it.

After a timer event is consumed, another `wait()` returns `Stopped` until reset.
Stopped tickers behave similarly. Reset validates before changing state; failure
retains the original schedule. Nonpositive timer intervals arm an immediately due
event. No callback, detached child, or implicit channel is created.

`Tick` contains `scheduled <Instant>`, `observed <Instant>`, and
`skipped <uint64>`. Ticks stay anchored to the original schedule. A slow caller
receives the most recent due slot, with the number of omitted slots since its
last observation; the first slot has index one. Missed ticks are coalesced rather
than queued without bound. Reset starts a new anchor and count. Scheduler waits
use the owner's allocated state; constructors expose the allocation cost.

## Civil dates and times

Gregorian convenience constructors cover years 1 through 9999. Other calendar
views, including Chinese and Hebrew leap months, follow the
[calendar-system contract](calendars.md). Months have validated day counts and
calendar-specific leap rules. Times use hours 0–23, minutes and seconds
0–59, and nanoseconds 0–999,999,999. There is no leap-second or `24:00` value;
constructors reject them rather than normalizing into another date.

`date.January` through `date.December` name ordinary `<date.Month>` codes for
Gregorian use; their `.number()` results are 1–12. Other calendars retain an
explicit leap-month marker. `date.Monday` through `date.Sunday` are `<date.Weekday>` values,
with `.number()` results 1–7. `date.month(number <uint8>)` returns a `Month` or
`date.InvalidDate`; this convenience constructor accepts 1–12 and creates ordinary
month codes. Use `date.month_code` for the more general calendar codes. These are
namespaced values, not numeric aliases or keywords.

| API                                                                                 | Result                                            | Contract                                                                       |
| ----------------------------------------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------ |
| `date.make_date(year <uint16>, month <Month>, day <uint8>)`                         | `Date` or `date.InvalidDate`                      | Validate the complete calendar date                                            |
| `date.make_time(hour <uint8>, minute <uint8>, second <uint8>, nanosecond <uint32>)` | `Time` or `date.InvalidTime`                      | Validate the time of day                                                       |
| `date.civil(day <Date>, clock <Time>)`                                              | `Civil`                                           | Pair values without selecting a zone                                           |
| `day.year()`, `.month()`, `.day()`                                                  | `int32`, `Month`, `uint8`                         | Read components in the selected calendar                                       |
| `day.weekday()`, `.day_of_year()`                                                   | `Weekday`, `uint16`                               | Read weekday and 1-based day of year                                           |
| `day.iso_week()`                                                                    | Record `{ year <uint16>; week <uint8> }`          | ISO week-year and week number; weeks start Monday, week one contains January 4 |
| `day.days_in_month()`                                                               | `uint8`                                           | Read this month's valid day count                                              |
| `day.add_days(count <int32>)`, `.add_weeks(count <int32>)`                          | `Date` or `date.RangeError`                       | Advance through calendar dates; one week is seven calendar days                |
| `day.add_months(count <int32>, policy)`, `.add_years(count <int32>, policy)`        | `Date` or `date.InvalidDate` or `date.RangeError` | Calendar arithmetic with an explicit end-of-month policy                       |
| `day.days_since(earlier <Date>)`                                                    | `int32`                                           | Signed calendar-day difference                                                 |
| `clock.hour()`, `.minute()`, `.second()`, `.nanosecond()`                           | `uint8`, `uint8`, `uint8`, `uint32`               | Read time-of-day components                                                    |
| `civil.date()`, `.time()`                                                           | `Date`, `Time`                                    | Read the paired components                                                     |
| `value.compare(other)`                                                              | `int32`                                           | Same-type `Date`, `Time`, or `Civil` comparison; `-1`, `0`, or `1`             |

Month/year operations accept the ordinary `<date.MonthPolicy>` values
`date.Reject`, `date.Clamp`, and `date.Carry`. Reject reports an invalid target
like February 31. Clamp chooses that month's final day. Carry counts the original
day offset from the target month's first day: January 31 plus one month becomes
March 3 in a non-leap year. Nothing changes the input date in place.

For a monthly recurrence, keep the original desired day as part of the recurrence
rule. Repeatedly adding a clamped month to its previous result can drift from the
31st to the 28th. A fixed `time.Duration` cannot encode this calendar policy.

## Timestamps and zones

`date.Timestamp` uses the Unix epoch, `1970-01-01T00:00:00Z`, with no distinct leap
seconds. Its range corresponds to UTC years 1–9999. The nanosecond fraction is
always nonnegative and less than one second: half a second before the epoch is
`seconds = -1`, `nanoseconds = 500_000_000`.

| API                                                | Result                                                | Contract                                                    |
| -------------------------------------------------- | ----------------------------------------------------- | ----------------------------------------------------------- |
| `date.now()`                                       | `Timestamp` or `date.ClockError` or `date.RangeError` | Read the civil clock, independent of the monotonic clock    |
| `date.unix(seconds <int64>, nanoseconds <uint32>)` | `Timestamp` or `date.RangeError`                      | Validate seconds and an already normalized fraction         |
| `stamp.unix_seconds()`, `.nanosecond()`            | `int64`, `uint32`                                     | Read normalized epoch components                            |
| `stamp.add(duration)`                              | `Timestamp` or `date.RangeError`                      | Advance by a fixed elapsed interval on the Unix scale       |
| `stamp.since(earlier)`                             | `Duration` or `time.RangeError`                       | Signed difference, possibly too large for a duration        |
| `stamp.compare(other)`                             | `int32`                                               | Compare absolute timestamps                                 |
| `date.zone(name <string>)`                         | `Zone` or `date.UnknownZone`                          | Find an exact IANA name in the bundled rule database        |
| `date.fixed_zone(offset_seconds <int32>)`          | `Zone` or `date.InvalidOffset`                        | Validate an offset strictly between -24 and +24 hours       |
| `date.in_zone(stamp, zone)`                        | `DateTime` or `date.RangeError`                       | View the same timestamp in a zone with a Gregorian calendar |
| `value.in_zone(zone)`                              | `DateTime` or `date.RangeError`                       | Change a DateTime's view without changing its timestamp     |
| `value.timestamp()`, `.zone()`                     | `Timestamp`, `Zone`                                   | Read a DateTime's identity and view                         |
| `value.date()`, `.time()`, `.offset_seconds()`     | `Date`, `Time`, `int32`                               | Read resolved local components and UTC offset               |
| `value.same_instant(other)`                        | `boolean`                                             | Compare timestamps while ignoring the selected zones        |

`date.UTC` is a `Zone` value. Named zones use the versioned
[IANA Time Zone Database](https://www.iana.org/time-zones), which records changing
civil-clock rules. `date.ZoneDatabaseVersion` identifies the bundled release;
build records also retain its content digest. Lookup performs no network or
ambient filesystem access, allocates no heap storage, and never consults `TZ` or
a machine-local default. Named-zone lookup makes the bundled rules explicit build
inputs. Updating them is an explicit toolchain/library change; future civil-time
results may change with those rules.

Zone equality compares a defined identity, not merely today's UTC offset. UTC is
the singleton also returned by `date.fixed_zone(0)`. Other fixed zones compare
their exact offset seconds. Named-zone identity consists of its canonical IANA
target name and bundled rule-data version/digest; IANA link names resolve to
that same target identity. Named zones remain distinct from UTC/fixed zones even
when all their offsets happen to agree. Thus a named `Etc/UTC` view can denote
the same instant as a `date.UTC` view without being an equal DateTime. No host
name canonicalization or case folding participates in lookup.

Build input identity is distinct from binary retention. Importing `date`, using
`date.UTC` or a fixed offset, or reading `ZoneDatabaseVersion` does not by itself
require the named-zone database in the executable. A reachable `date.zone(name)`
with an unconstrained runtime name retains lookup support for every bundled name;
optimization cannot silently make a supported name become `UnknownZone`. A
constant lookup or a runtime name proven to come from a finite set may retain
less data only with equivalent behavior for every reachable use.
[Memory and binary optimization](../optimization.md)
explains how these dependencies connect to the linker and to static storage.

A fixed offset does not express daylight-saving transitions. An RFC timestamp's
numeric offset cannot recover an IANA zone name. Persist a `Timestamp` for an
observed event, and a `Civil` plus zone name and resolution policy for an intended
future local appointment. Record the rule version when a particular resolution
must remain reproducible.

`==` on the same date/time value type compares its complete value. In particular,
DateTime equality includes calendar, zone, and rule-data identity as well as the
timestamp. Date equality includes the selected calendar; use `same_day` to compare
days across calendar views. Use `same_instant` for differently zoned views of one event. There
is no implicit ordering or subtraction between different date/time types.
Civil equality compares its complete Date and Time, including the Date's calendar
identity. These are semantic value comparisons, never comparisons of padding or
descriptor addresses.

### Resolve local times explicitly

`date.resolve(civil, zone, policy)` returns a `DateTime`, `date.Ambiguous`,
`date.Nonexistent`, or `date.RangeError`. A clock change can repeat a local time
or skip it entirely. Resolution retains the Civil's calendar; changing zones also
retains that view, subject to its supported range. A `<date.ResolvePolicy>` is an ordinary record with:

```meowy
-> fold : "reject"
-> gap : "reject"
```

`fold` accepts `"reject"`, `"earlier"`, or `"later"` by absolute timestamp.
`gap` accepts `"reject"` or `"shift_forward"`; shifting advances the requested
civil time by the transition's exact gap length before resolution. An unknown
policy value is a type error: the record's fields have closed string-literal
unions. `date.Strict` rejects both cases; `date.Earlier` and `date.Later` resolve
folds while still rejecting gaps.

`Ambiguous` carries the two candidate DateTimes as `earlier` and `later`.
`Nonexistent` carries the requested `civil`, `zone`, and `gap <time.Duration>`.
The caller can inspect or retry with a deliberate policy. A constructor never
silently picks one occurrence of a repeated hour.

`value.add(duration)` advances a DateTime's timestamp by a fixed duration and
recomputes its local view. It returns `DateTime` or `date.RangeError`.
`value.add_days(count, resolve_policy)` keeps the time of day while changing the
calendar date, then resolves again; it returns the same alternatives as `resolve`.
`add_weeks(count, resolve_policy)` does the same with seven-day steps.
`add_months(count, month_policy, resolve_policy)` and
`add_years(count, month_policy, resolve_policy)` additionally accept an end-of-month
policy and can return `InvalidDate`. There is no implicit policy argument.

Consequently, adding `time.Day` can change the local hour, while adding one
calendar day can span fewer or more than 24 elapsed hours. Go's
[calendar arithmetic example](https://pkg.go.dev/time#Time.AddDate) illustrates
this distinction; meowy makes the gap/fold choice explicit in the call.

## Parse and format text

`date.parse(text, layout)` returns a `DateTime` or `date.ParseError` and requires
a complete date, clock, and numeric UTC offset. `date.parse_civil`,
`date.parse_date`, and `date.parse_time` return `Civil`, `Date`, and `Time`
respectively, or `ParseError`, and require exactly their corresponding components.
There is no guessed current year, local zone, midnight, or locale.
These functions use Gregorian dates. Use the [calendar-aware text API](calendars.md#calendar-aware-text-and-serialization)
to parse Chinese, Hebrew, or another calendar's fields explicitly.

These four parsing items satisfy `core.Pure`: input text and layout completely
determine their result under the bundled rules, and they allocate no runtime
owner. They are suitable for pure CLI validators and required evaluation with
constant inputs. Writer/buffer formatting has its separately documented effects.

Layouts are ordinary `<string>` values with these directives:

| Directive        | Meaning                                                                                                   |
| ---------------- | --------------------------------------------------------------------------------------------------------- |
| `%Y`, `%m`, `%d` | Four-digit year, two-digit month, two-digit day                                                           |
| `%H`, `%M`, `%S` | Two-digit 24-hour hour, minute, second                                                                    |
| `%N`             | Exactly nine nanosecond digits, without a decimal point                                                   |
| `%f`             | Optional decimal point plus 1–9 fraction digits; formatting omits zero fractions and trims trailing zeros |
| `%:z`            | `Z` or a signed `HH:MM` UTC offset; formatting uses `Z` for zero                                          |
| `%z`             | Signed `HHMM` offset                                                                                      |
| `%a`, `%A`       | Short or full English weekday name                                                                        |
| `%b`, `%B`       | Short or full English month name                                                                          |
| `%%`             | Literal percent sign                                                                                      |

`date.DateOnly` is `"%Y-%m-%d"`; `date.TimeOnly` is `"%H:%M:%S%f"`;
`date.ISODateTime` is `"%Y-%m-%dT%H:%M:%S%f"`;
`date.RFC3339` is `"%Y-%m-%dT%H:%M:%S%f%:z"`. These values follow ordinary lookup.
Short English names are `Mon` through `Sun` and `Jan` through `Dec`. Layout
literals and names match exactly and case-sensitively. Numeric fields are ASCII.
Weekday names must agree with the parsed date, and duplicate or incompatible
component directives are errors. Unknown directives are never printed literally.

Parsing consumes the whole input and validates dates, ranges, and offsets.
Nanoseconds default to zero only when no fraction is supplied. The supported
[RFC 3339 profile](https://www.rfc-editor.org/rfc/rfc3339) uses uppercase `T`/`Z`,
years 1–9999, and seconds 0–59. It rejects leap seconds, unknown offset `-00:00`,
and excess fractional precision rather than guessing or truncating. A parsed
numeric offset creates a fixed-offset zone, not a guessed named zone. This
profile is deliberately narrower than every representation permitted by the RFC.

`date.format_into(value, layout, &!buffer)` accepts a Date, Time, Civil, Timestamp,
or DateTime and returns `<string><date.FormatError>`. A Timestamp formats as UTC.
A non-Gregorian Date, Civil, or DateTime must first be converted to Gregorian,
or formatted with `calendars.format_into` instead.
A layout requesting a component absent from the value fails; a numeric offset
with sub-minute precision cannot be silently rounded into `%z` or `%:z`.
Date-only and time-only layouts can select components from a larger value.

The generic buffer is a caller-owned `<uint8[N]>` bounded list. The formatter
validates and measures before writing; failure retains its original bytes and
length and reports the required capacity when applicable. Success replaces the
list contents, returns a borrowed UTF-8 view, and allocates no heap storage.
The buffer cannot be mutated or dropped while that view is live. There is no
implicit allocating `format()` result. Diagnostic printing can stream the default
form directly: DateOnly for Date, TimeOnly for Time, ISODateTime for Civil, and
RFC3339 for Gregorian Timestamp/DateTime views. Other calendar views display
their calendar ID and month code as specified in the calendar chapter.
A historical sub-minute offset uses `±HH:MM:SS`
in diagnostic display, explicitly outside RFC3339.

## Failures, storage, and replay

Invalid calendar data, parsing, missing zones, ambiguous local times, clock read
failures, and output capacity are recoverable error values. Error descriptors
store fixed-size numeric facts and offsets, not an implicitly allocated copy of
the original input. `ParseError` contains a 1-based UTF-8 byte `position` and a
static reason; `FormatError` identifies the unsupported field or required capacity.
Range errors report the operation and representable bounds. All these concrete
error values are allocation-free and transferable.

Date/time construction, lookup, parsing, arithmetic, and comparison allocate no
heap storage. Timer/ticker constructors expose their allocator. Formatting borrows
caller storage. The calendar library does not run timers or create threads in
order to represent or manipulate a date.

Clock reads and wakeups are observable inputs. With `--record-replay`, captures
record monotonic and civil readings, clock-domain mapping, timer delivery, task
cancellation, and the zone/calendar data identities needed by the failure. Replay uses
recorded reads and a virtual clock; it does not sleep for the original wall-clock
interval or substitute the host's current civil time. Missing required events
make replay incomplete. Build-only isolation does not claim deterministic timing.
See [replay fidelity](../diagnostics.md#replay-fidelity).

The [ticker project](../../programs/ticker/README.md) observes coalesced wakeups
with explicit timer ownership. The [duration CLI](../../programs/duration-cli/README.md)
parses durations and checks arithmetic before formatting a result.
