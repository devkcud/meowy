# Pack light, keep the useful bits

[Pawterns](README.md) · [Ownership](ownership.md) · [Optimization reference](../reference/optimization.md)

The greeting is ready to leave your laptop. This is where three different
questions tend to get stuffed into one suitcase: how large is the executable,
what else does it need to run, and how much memory does it use while running?
Measure them separately. A tiny file can still eat the machine's lunch.

## Use a duration without inviting every calendar

You need a five-second budget. The program can use a duration without creating
timers or bringing a named-zone database to the party.

In a fresh project, use this complete `main.mwy`:

```meowy
debug : @"debug"
time : @"time"

budget : time.Second.scale(5)
debug.print(budget.nanoseconds())
```

Use this complete `mod.mwy`:

```meowy
-> build : {
    -> entry : "./main.mwy"
    -> profile : "release"
    -> optimize : "size"
    -> debug_info : "separate"
    -> link : {
        -> dead_strip : true
    }
}
```

From that project:

```sh
meowy run
meowy build --output build/duration --report
```

The program prints `5000000000`. The build writes the executable plus
`build/duration.build.json` and `build/duration.link.map`. Look at the report's
`retention`, `sections`, and `runtime` evidence: integer output still needs
formatting and an output path, while this duration calculation does not require
clock initialization, timer storage, or calendar conversion rules.

The import makes values available. Reachable operations decide what stays in the
binary, with observable initialization and cleanup preserved. A constant duration
can fold away; the spelling `@"time"` is not an instruction to glue every stdlib
package into your executable.

For a useful contrast, inspect a build of the
[calendar CLI](../programs/calendar-cli/README.md). Its runtime calendar selection
must support the choices it advertises. Choosing Chinese directly has a different
retention surface from accepting an unrestricted calendar name. See
[the precise retention rules](../reference/optimization.md#what-using-time-actually-retains).

**Watch the units:** separate debug information reduces the executable's file
size. It does not prove a smaller heap peak. Keep the matching debug companion
for investigating this exact binary later.

## Send the binary to an older machine

“Works on my CPU” is a surprisingly literal failure mode. Pick the deployment
target and its CPU baseline explicitly, then keep native dependencies honest too.

For the duration project above, this is a replacement `mod.mwy` for a toolchain
that supplies the named Linux target, its static runtime, and ThinLTO support:

```meowy
-> build : {
    -> entry : "./main.mwy"
    -> profile : "release"
    -> target : "x86_64-unknown-linux-musl"
    -> cpu : "baseline"
    -> optimize : "size"
    -> jobs : 1
    -> debug_info : "separate"
    -> link : {
        -> mode : "static"
        -> dead_strip : true
        -> lto : "thin"
        -> icf : "safe"
    }
}
```

Build with:

```sh
meowy build --output build/duration --report
```

The example target is a deliberate choice, not an installed-target promise.
Unavailable target/runtime/LTO support produces `E507`; select supplied inputs
that match the intended machine. For another architecture or ABI, change the
target accordingly. The report records the expanded CPU requirements, runtime,
and linker identity. Compare those requirements with the physical CPU or VM's
exposed features before deployment.

`jobs : 1` limits compiler and LTO concurrency on the builder. It does not turn
the application into a one-worker executor. The greeting has no executor at all.
ThinLTO may expose useful cross-module optimizations, but “enable optimization”
is not a scientific measurement of either file size or build-time memory.

Try one setting at a time and compare reports for the same inputs. `safe` folding
must preserve observable function identity. Separate symbols preserve a useful
debug artifact without copying every debugger byte into the deployment executable.

See [build policy](../reference/optimization.md#select-build-policy-in-modmwy)
and [old machines and VMs](../reference/optimization.md#docker-images-old-machines-and-virtual-machines).

## Give a container a budget it cannot sweet-talk

For the verified static Linux binary above, with no external application data
requirements, save this `Dockerfile` beside `mod.mwy`:

```dockerfile
FROM scratch
COPY build/duration /duration
ENTRYPOINT ["/duration"]
```

In a Docker environment matching that binary's platform:

```sh
docker build -t paw-duration:local .
docker run --rm --memory=128m --memory-swap=128m --cpus=1 paw-duration:local
```

The expected application output is still `5000000000`. This image packages a
prebuilt binary; it does not compile the program. `scratch` supplies an empty
filesystem, so verify the runtime closure before choosing it. A shared-linked
binary needs its compatible loader and libraries. Applications using files,
certificates, or foreign-library data need those inputs included deliberately.

The equal memory and memory-swap settings disable container swap. This is an
illustrative test envelope, not a claim that every application fits inside
128 MiB. Build caches, reports, replay capsules, and separate symbols can stay
in build/diagnostic storage instead of the final image.

The duration program is deliberately too small to teach much about sustained
memory pressure. Apply the same budgeting process to a real workload such as
[streaming file I/O](cli-and-files.md) or [bounded task batches](tasks-and-channels.md):
fix input sizes, cap admitted work, and include slow consumers and retained results
when measuring the peak. A queue full of eight owner handles can own eight large
buffers. The pointers do not make the payload bytes disappear. Nice try, though.

Handle fallible allocation failures where the API returns them. An OS or cgroup
kill cannot promise cleanup or an in-process diagnostic capsule, so examine host
evidence too. On Linux cgroup v2, memory usage and OOM events belong to the process's
actual cgroup; a suspicious exit code alone is not a diagnosis.

See [memory pressure](../reference/optimization.md#under-severe-memory-pressure)
and the [deployment reference](../reference/optimization.md#docker-images-old-machines-and-virtual-machines)
for measurement details and primary platform sources. A successful tiny run checks
that deployment path; it says nothing about your application's worst input.
