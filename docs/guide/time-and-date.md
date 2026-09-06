# Time, calendars, and a small CLI

[Documentation index](../README.md) · [Standard library](../reference/stdlib/README.md) · [Complete calendar CLI](../programs/calendar-cli/main.mwy)

Use `time` for elapsed intervals and deadlines. Use `date` when a person means a
particular date, local hour, month, or year. A calendar selects date labels; a time
zone maps local time to an absolute timestamp. Keeping those choices explicit
makes timeouts and calendar arithmetic predictable.

## Give durations readable units

```meowy
time : @"time"

request_budget : time.Second.scale(5)
retention : time.Week.scale(2)
meeting_length : time.Hour.add(time.Minute.scale(30))
```

`time.Day` is exactly 24 hours and `time.Week` is exactly seven such days. All units
are ordinary Duration values, so `scale`, `add`, and dispatch compose without
special numeric operators. Even `budget:time.Hour.scale(2)` needs no spaces.
The [duration reference](../reference/stdlib/time-and-date.md#fixed-durations)
defines checked arithmetic and text such as `"2h30m"` or `"250ms"`.

## Measure work with a monotonic clock

```meowy
time : @"time"
debug : @"debug"

started : time.now()
time.sleep(time.Millisecond.scale(20))
debug.print(time.since(started))
```

The result is at least 20 milliseconds, subject to the selected clock's resolution
and scheduling delay. Changing the computer's civil clock cannot make this
measurement jump backward. Keep one deadline, `time.after(request_budget)`, when
several task or I/O operations share a budget. Creating a deadline does not start
a timer or a task. A [Timer or Ticker](../reference/stdlib/time-and-date.md#owned-timers-and-tickers)
is a separate owner with explicit allocation and cleanup.

## Make an invalid date an ordinary result

```meowy
date : @"date"
debug : @"debug"

january : date.make_date(2026, date.January, 31)
| january <error> | debug.panic(january)

february : january.add_months(1, date.Clamp)
| february <error> | debug.panic(february)
debug.print(february)   # 2026-02-28 #
```

`date.Reject` would return InvalidDate for February 31. `date.Carry` would produce
March 3 by carrying the day offset from February 1. These snippets panic only to
keep their fixed-input demonstrations short; applications normally match and
explain a failed user-supplied date, as the CLI below does.

For a recurring appointment on the 31st, retain that desired day. Repeatedly
clamping the previous result would change the desired day to the 28th.

## Choose between tomorrow and 24 hours later

This example crosses the spring clock transition in Europe/Zurich:

```meowy
date : @"date"
time : @"time"
debug : @"debug"

zone : date.zone("Europe/Zurich")
| zone <error> | debug.panic(zone)
day : date.make_date(2023, date.March, 25)
| day <error> | debug.panic(day)
noon : date.make_time(12, 0, 0, 0)
| noon <error> | debug.panic(noon)

start : date.resolve(date.civil(day, noon), zone, date.Strict)
| start <error> | debug.panic(start)

tomorrow : start.add_days(1, date.Strict)
| tomorrow <error> | debug.panic(tomorrow)
elapsed : start.add(time.Day)
| elapsed <error> | debug.panic(elapsed)

debug.print(tomorrow)  # 2023-03-26T12:00:00+02:00 #
debug.print(elapsed)   # 2023-03-26T13:00:00+02:00 #
```

Noon on the next calendar day is 23 elapsed hours away in this case, as in
[Go's calendar arithmetic example](https://pkg.go.dev/time#Time.AddDate).
`date.Strict` rejects a local time that repeats or does not exist; the
[resolution policies](../reference/stdlib/time-and-date.md#resolve-local-times-explicitly)
let the application handle those cases deliberately. Zone rules are pinned build
inputs, so the host's locale does not choose the meaning of this source.

## Change the calendar, preserve the day

```meowy
date : @"date"
debug : @"debug"

day : date.make_date(2026, date.February, 17)
| day <error> | debug.panic(day)
lunar : day.in_calendar(date.calendars.Chinese)
| lunar <error> | debug.panic(lunar)

debug.print(lunar)              # chinese:2026-M01-01 #
debug.print(day.same_day(lunar)) # true #
```

The date agrees with the [Hong Kong Observatory's 2026 table](https://www.hko.gov.hk/en/gts/time/calendar/pdf/files/2026e.pdf).
A Chinese leap month keeps its identity, such as `M06L`; it is not renamed month
seven. The [calendar reference](../reference/stdlib/calendars.md) defines Gregorian,
Julian, Hebrew, Chinese, Islamic civil, and Buddhist profiles, their year/month
conventions, and their supported ranges.

For an external format, choose a calendar-aware layout and supply storage:

```meowy
buffer <uint8[64]> := []
text : date.calendars.format_into(lunar, date.calendars.DateOnly, &!buffer)
| text <error> | debug.panic(text)
debug.print(text)
```

This fragment continues the preceding example. `text` borrows `buffer`; mutation
must wait until that view expires. An insufficient buffer yields an error without
partially replacing its contents. Gregorian RFC3339 output instead uses
`date.format_into` with a timestamp or Gregorian DateTime and `date.RFC3339`.

## Turn it into a command-line application

[Calendar CLI](../programs/calendar-cli/README.md) describes its options with `cli`,
reads argv through `process`, converts the date, and writes through `io` and `fmt`.
The parser produces typed fields and help/error values for explicit handling:

```console
$ meowy run docs/programs/calendar-cli/main.mwy -- --calendar chinese 2026-02-17
```

The application's output is:

```text
chinese:2026-M01-01
```

Use `--help` for generated usage, `--version` for its declared version, or `-v`
to also print the selected calendar rule version. A bad option, invalid Gregorian
date, or unsupported conversion range gives status 2; output and argv acquisition
failures give status 1. Help and successful conversion give status 0.

The command description and parse result use inline storage; argv acquisition
uses the explicit heap allocator and owns the strings until parsing and output
finish. Calendar conversion allocates nothing. The [CLI library reference](../reference/stdlib/cli.md)
covers custom parsers, validation, bounded repeated options, subcommands, global
options, and rendering help into a supplied writer.
