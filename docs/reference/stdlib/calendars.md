# Calendar systems

[Library index](README.md) · [Time and date](time-and-date.md) · [Practical guide](../../guide/time-and-date.md)

`@"calendars"` supplies calendar values and calendar-specific operations.
`date.calendars` re-exports the same module, so `date.calendars.Chinese` and
`calendars.Chinese` identify the same rules. A calendar is an ordinary immutable
value, with a stable identifier, rule version, and supported range.

A calendar answers which year, month, and day label a civil day. A time zone
answers how a local clock reading relates to a timestamp. Choosing Chinese month
names does not choose Beijing time, and choosing Hebrew dates does not make a
clock switch days at sunset. These are separate operations with separate inputs.

## Calendars are views of a day

A `date.Date` stores a day number and a calendar identity. Day number zero is
Gregorian 1970-01-01; consecutive numbers denote consecutive civil days. It is
not a Unix timestamp divided by 86,400: converting a timestamp to a local day
first requires a time zone.

`day.in_calendar(calendar)` returns the same day in another calendar, or
`date.RangeError` if it is outside that calendar's support. `day.same_day(other)`
compares day numbers; `==` also compares the calendar's identity and rule version.
`days_since` and `compare` operate on day numbers across calendar views.
`weekday()` is unchanged. `iso_week()` always reports the Gregorian ISO week-year,
not a lunisolar year with an ISO-looking number.

The common absolute range is Gregorian 0001-01-01 through 9999-12-31; individual
calendars may support a subset. `calendar.range()` returns inclusive `first` and
`last` Gregorian Dates. Invalid fields and unsupported ranges are reported rather
than extrapolated silently.

| Value                    | Stable ID         | Calendar contract                                                                                 |
| ------------------------ | ----------------- | ------------------------------------------------------------------------------------------------- |
| `calendars.Gregorian`    | `"gregorian"`     | Proleptic Gregorian; CE years, no regional reform gap                                             |
| `calendars.Julian`       | `"julian"`        | Proleptic Julian; CE years and a leap day every fourth year                                       |
| `calendars.Hebrew`       | `"hebrew"`        | Fixed arithmetic Hebrew calendar; Anno Mundi years and explicit leap months                       |
| `calendars.Chinese`      | `"chinese"`       | Versioned Chinese lunisolar conversion data, with related Gregorian years and leap-month identity |
| `calendars.IslamicCivil` | `"islamic-civil"` | Tabular civil Hijri calendar, Friday epoch and the specified 30-year leap cycle                   |
| `calendars.Buddhist`     | `"buddhist"`      | Proleptic Gregorian month/day rules with Buddhist Era year = CE year + 543                        |

`calendars.lookup(id)` returns `<calendars.Calendar><calendars.UnknownCalendar>`.
`calendar.id()` and `.version()` return static strings. The identifiers select
specific rule sets, not whatever calendar a machine's locale happens to prefer.
The supported positive-year interval for each arithmetic calendar is intersected
with the common absolute range. No constructor guesses an era from a negative
number or supplies a hidden Gregorian/Julian cutover date.

The Julian value covers the familiar Roman-era Julian system. It does not claim
to reconstruct every irregular pre-Julian Roman calendar. Buddhist here names the
stated proleptic rule, not every historical regional New Year convention. Islamic
civil uses leap years 2, 5, 7, 10, 13, 16, 18, 21, 24, 26, and 29 in each 30-year
cycle, with its civil epoch; it does not predict local moon-sighting decisions.
These distinctions are consistent with the separate calendar identities in
[Unicode calendar data](https://github.com/unicode-org/cldr/blob/main/common/bcp47/calendar.xml).

## Keep month identity separate from position

`date.Month` is an opaque month code with a base number and a leap marker.
`date.month_code("M06L")` returns that code or `date.InvalidDate` for an invalid
spelling. Codes use `M01` through `M13`, optionally followed by `L`; whether a code
actually exists depends on the selected calendar and year. `.code()` returns a
static spelling, `.number()` its base number, and `.is_leap()` the marker.

`date.January` through `date.December` are convenient Gregorian names for the
ordinary `M01` through `M12` codes. Code equality does not assert that two calendar
months have the same dates or cultural meaning. For other calendars, prefer the
explicit code or the calendar's own naming data.

An ordinal is a position in one particular year. Chinese `M06L` follows `M06` and
can occupy ordinal seven; subsequent ordinary months keep their own month codes.
The fixed Hebrew convention numbers ordinary codes from Tishri as `M01` through
Elul as `M12`; Adar I is `M05L`, and Adar/Adar II is `M06`. Neither scheme renames
a leap month to the next ordinary month. This separation of code, leap marker,
and ordinal follows the model exposed by [ICU4X dates](https://icu4x.unicode.org/2_1/tsdoc/classes/Date.html).

| API                                                                           | Result                                                 | Contract                                                    |
| ----------------------------------------------------------------------------- | ------------------------------------------------------ | ----------------------------------------------------------- |
| `date.from_calendar(calendar, year <int32>, month <date.Month>, day <uint8>)` | `date.Date` or `date.InvalidDate` or `date.RangeError` | Validate fields in the selected calendar                    |
| `day.calendar()`                                                              | `Calendar`                                             | Read the day’s calendar identity                            |
| `day.year()`, `.month()`, `.day()`                                            | `int32`, `date.Month`, `uint8`                         | Read calendar-specific fields                               |
| `day.month_ordinal()`                                                         | `uint8`                                                | One-based position in the calendar year                     |
| `day.months_in_year()`, `.days_in_year()`                                     | `uint8`, `uint16`                                      | Read this year's actual sizes                               |
| `calendars.months(calendar, year <int32>)`                                    | `date.Month[13]` or `date.RangeError`                  | Month codes in actual chronological order                   |
| `day.in_calendar(calendar)`                                                   | `date.Date` or `date.RangeError`                       | Preserve the day, change its calendar view                  |
| `civil.in_calendar(calendar)`                                                 | `date.Civil` or `date.RangeError`                      | Convert the date and keep its time of day                   |
| `value.in_calendar(calendar)` on a DateTime                                   | `date.DateTime` or `date.RangeError`                   | Preserve timestamp and zone, change the local calendar view |

`date.make_date(year, month, day)` remains a convenient Gregorian constructor.
All calendar values, dates, and month codes are immutable, copyable, and require
no heap allocation. A month list is bounded by thirteen for these built-in rule
sets. A year whose complete metadata is unavailable reports `RangeError` instead
of returning a shortened list pretending to describe the whole year.

## Chinese lunisolar dates

Chinese dates carry both an ordinary month number and whether that month is the
intercalary occurrence. Calendar years are identified here by the Gregorian year
in which that Chinese New Year occurs. This is a deliberate related-year
convention, not a claim that a sexagenary cycle alone identifies an absolute year.
Unicode distinguishes [related years and cyclic years](https://unicode.org/reports/tr35/tr35-dates.html#Date_Field_Symbol_Table)
for this reason.

The foundational Chinese profile uses pinned conversion data with the
Hong Kong Observatory's modern civil-day convention. Its advertised absolute
range is Gregorian 1901-01-01 through 2100-12-31. The
[Observatory's conversion tables](https://www.hko.gov.hk/en/gts/time/conversion.htm)
cover those years and document possible future revisions near astronomical
boundaries. The data version and digest therefore form part of calendar identity;
results outside the range return `RangeError`, and updates cannot silently change
a saved replay's interpretation.

For example, Gregorian 2026-02-17 is Chinese related year 2026, `M01`, day 1,
as shown in the [2026 conversion table](https://www.hko.gov.hk/en/gts/time/calendar/pdf/files/2026e.pdf).
This fragment imports the values it uses and checks both fallible steps:

```meowy
date : @"date"
debug : @"debug"

gregorian : date.make_date(2026, date.February, 17)
| gregorian <error> | debug.panic(gregorian)

lunar : gregorian.in_calendar(date.calendars.Chinese)
| lunar <error> | debug.panic(lunar)

debug.print(lunar.month().code())   # M01 #
debug.print(lunar.day())            # 1 #
```

`calendars.chinese_cycle(day)` accepts a Chinese Date and returns a record with
`year <uint8>`, `stem <uint8>`, and `branch <uint8>`, or
`calendars.CalendarMismatch`. They are one-based positions in cycles of 60, 10,
and 12. Related year 1984 anchors cycle year 1; 2026 has cycle year 43, stem 3,
and branch 7. Those positions support presentation without making zodiac text
part of the storage type or guessing a locale.

Astronomical rules determine the profile's civil-day conversion data; they do
not make an individual Date contain an ephemeris or run a floating-point lunar
simulation. Choosing another geographic or observational convention requires a
different explicitly versioned calendar profile, not the host's time zone.

## Calendar arithmetic stays in its calendar

`add_days` moves through day numbers and retains the chosen calendar view.
`add_months` walks actual month occurrences, including an intercalary month.
Chinese `M06` followed by `M06L` is one month step; adding twelve months does not
necessarily reach the same month code in the next year.

`add_years` changes the calendar year and attempts to retain the month code.
If that month code does not exist in the target year, it returns `InvalidDate`.
The `Reject`, `Clamp`, and `Carry` month policies resolve an out-of-range day
within an existing target month; they never silently erase a leap marker or choose
a replacement month. For an anniversary in a missing leap month, the application
must explicitly choose the ordinary month or another recurrence rule.

DateTime calendar operations change the local calendar date, preserve the local
time, and apply the explicit time-zone gap/fold policy. The calendar does not
select that zone. Civil dates in Hebrew and Islamic views are mapped using this
library's midnight-based civil-day coordinate; determining a religious sunset
boundary needs a separate location and observational calculation.

## Calendar-aware text and serialization

The `date` module's ISO/RFC text functions accept Gregorian views. Convert to
Gregorian explicitly before using them on another calendar. A Chinese month/day
must never be emitted as an unmarked Gregorian-looking protocol date.

`calendars.format_into(value, layout, &!buffer)` handles a Date, Civil, or DateTime
in its current calendar. It uses the date formatter's buffer and failure rules,
with `%Y` as the full calendar year, `%m` as the two-digit base month number,
`%L` as `L` for a leap month or empty otherwise, and `%C` as the calendar ID.
The ordinary clock and offset directives keep their existing meaning. `%b` and
`%B` require explicit localized month data and are rejected by this locale-free
formatter. A leap month requires `%L`; omitting its identity is a `FormatError`.

`calendars.DateOnly` is `"%C:%Y-M%m%L-%d"`. Its output for the example is
`chinese:2026-M01-01`. Non-Gregorian diagnostic Date display uses that form;
Civil and DateTime append `T` and their ordinary clock/offset display. This
makes calendar identity visible in errors and logs without allocating a string.

`calendars.parse_date(text, layout, calendar)` returns a Date or `date.ParseError`
or `date.RangeError`. It requires year, month, and day; `%L` may be absent only
for an ordinary month, and a supplied `%C` must match the selected calendar ID.
There is no locale-driven calendar switch. The selected calendar version remains
an explicit input, even when the text includes its stable ID.

For durable storage, preserve the day number when the observed day is the fact,
or preserve calendar ID, rule version, year convention, month code, and day when
the calendar expression itself is the fact. A lunar anniversary cannot be reduced
to a fixed number of seconds. Timestamp serialization remains calendar-independent.
