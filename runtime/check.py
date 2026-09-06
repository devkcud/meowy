import argparse
import os
import re
import signal
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent
VERSION = "22.1.8"
CASES = 14
STACK_CASES = 10


def invoke(args, timeout=60, env=None):
    process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               cwd=ROOT, env=env, start_new_session=True)
    try:
        out, err = process.communicate(timeout=timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.communicate()
        raise
    return subprocess.CompletedProcess(args, process.returncode,
                                       out.decode("utf-8", errors="replace"),
                                       err.decode("utf-8", errors="replace"))


def require(result, code=0):
    if result.returncode != code:
        raise RuntimeError(f"command failed (exit {result.returncode}): {' '.join(map(str, result.args))}\n"
                           f"{result.stdout}{result.stderr}")


def check_fatal(result, panicked):
    require(result, -signal.SIGABRT)
    original = "panic P006: body failed" if panicked else "complete"
    expected = ("release-trigger\npanic[P008]: panic during cleanup\n"
                f"original: {original}\ncleanup: newer\nsecond: P006: release failed\n")
    if result.stdout or result.stderr != expected:
        raise RuntimeError(f"fatal cleanup evidence differs:\n{result.stdout}{result.stderr}")


def check_guard(result, high):
    require(result, -signal.SIGSEGV)
    side = "high" if high else "low"
    expected = f"guard-{side}: SEGV_ACCERR at expected address on alternate stack\n"
    if result.stdout or result.stderr != expected:
        raise RuntimeError(f"guard fault evidence differs:\n{result.stdout}{result.stderr}")


def check_cases(result, count, suite):
    require(result)
    if (result.stderr or not result.stdout.endswith(f"{count} {suite} cases passed\n") or
            result.stdout.count("PASS ") != count):
        raise RuntimeError(f"{suite} case evidence differs:\n{result.stdout}{result.stderr}")


def check(clang, directory, sanitizers=True):
    version = invoke([clang, "--version"])
    require(version)
    if not re.search(rf"\bclang version {re.escape(VERSION)}\b", version.stdout):
        raise RuntimeError(f"runtime prototype requires Clang {VERSION}; got {version.stdout.strip()}")
    profiles = [("debug", ["-O0", "-g"]), ("release", ["-O2", "-DNDEBUG"])]
    if sanitizers:
        profiles.append(("sanitized", ["-O1", "-g", "-fsanitize=address,undefined",
                                        "-fno-sanitize-recover=all", "-fno-omit-frame-pointer"]))
    env = dict(os.environ, ASAN_OPTIONS="detect_leaks=1:halt_on_error=1:abort_on_error=1",
               UBSAN_OPTIONS="halt_on_error=1:print_stacktrace=1")
    for name, flags in profiles:
        binary = directory / f"cleanup-{name}"
        args = [clang, "-std=c++20", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                "-fno-exceptions", "-fno-rtti", "-I", str(ROOT / "include"),
                *flags, str(ROOT / "src/cleanup.cpp"), str(ROOT / "tests/cleanup.cpp"),
                "-o", str(binary)]
        print(f"CHECK runtime cleanup: {name}", flush=True)
        require(invoke(args))
        result = invoke([str(binary)], env=env)
        check_cases(result, CASES, "cleanup")
        check_fatal(invoke([str(binary), "--fatal-normal"], env=env), False)
        check_fatal(invoke([str(binary), "--fatal-panic"], env=env), True)
        print(f"PASS runtime cleanup: {name}; {CASES} cases and 2 fatal subprocesses", flush=True)
        stack = directory / f"stack-{name}"
        args = [clang, "-std=c++20", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                "-fno-exceptions", "-fno-rtti", "-I", str(ROOT / "include"),
                *flags, str(ROOT / "src/cleanup.cpp"), str(ROOT / "src/stack_memory.cpp"),
                str(ROOT / "tests/stack_memory.cpp"), "-Wl,--wrap=mmap,--wrap=mprotect,--wrap=munmap",
                "-o", str(stack)]
        print(f"CHECK runtime stack allocation: {name}", flush=True)
        require(invoke(args))
        check_cases(invoke([str(stack)], env=env), STACK_CASES, "stack allocation")
        admission = invoke([str(stack), "--os-admission-failure"], env=env)
        require(admission)
        if admission.stderr or admission.stdout != "PASS kernel admission refusal: ENOMEM without owner\n":
            raise RuntimeError(f"kernel admission evidence differs:\n{admission.stdout}{admission.stderr}")
        guard_env = dict(env, ASAN_OPTIONS=env["ASAN_OPTIONS"] + ":handle_segv=0")
        check_guard(invoke([str(stack), "--guard-low"], env=guard_env), False)
        check_guard(invoke([str(stack), "--guard-high"], env=guard_env), True)
        print(f"PASS runtime stack allocation: {name}; {STACK_CASES} cases, kernel refusal and 2 guard subprocesses", flush=True)
    if not sanitizers:
        print("Sanitizers were explicitly disabled; sanitizer behavior was not checked.")
    print("Cleanup and guarded allocation only; context switching, suspension and DWARF unwinding are not qualified.")


def main():
    parser = argparse.ArgumentParser(description="Build and check bounded native cleanup and guarded stack allocation.")
    parser.add_argument("--clang", default="/usr/bin/clang++", help=f"Clang {VERSION} executable")
    parser.add_argument("--build-dir", type=Path, help="Keep binaries in this directory; default uses temporary storage")
    parser.add_argument("--no-sanitizers", action="store_true", help="Run debug/release only; explicitly omit sanitizer validation")
    args = parser.parse_args()
    try:
        if args.build_dir:
            directory = args.build_dir.resolve()
            directory.mkdir(parents=True, exist_ok=True)
            check(args.clang, directory, not args.no_sanitizers)
        else:
            with tempfile.TemporaryDirectory(prefix="meowy-runtime-") as temp:
                check(args.clang, Path(temp), not args.no_sanitizers)
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        print(f"FAIL runtime prototype: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
