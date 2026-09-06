import argparse
import os
import shlex
import signal
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-unknown-linux-gnu"


@dataclass(frozen=True)
class Check:
    name: str
    args: tuple[str, ...]


def plan(editor=None, compiler=False, strict=False, runtime=False):
    python = (sys.executable, "-B", "-X", "utf8", "-E")
    checks = [
        Check("tooling regressions", python + ("-m", "unittest", "discover", "-s", "tools/tests", "-p", "test_*.py")),
        Check("local documentation links", python + ("tools/check_links.py",)),
        Check("conformance catalog metadata", python + ("docs/conformance/check.py",)),
        Check("artifact schemas and example identities", python + ("docs/schemas/validate.py",)),
    ]
    if editor in ("vim", "both"):
        checks.append(Check("Vim runtime", ("vim", "-Nu", "NONE", "-i", "NONE", "-n", "-es", "-S", "editor/nvim/tests/run.vim")))
    if editor in ("nvim", "both"):
        checks.append(Check("Neovim runtime", ("env", "NVIM_LOG_FILE=" + os.devnull, "nvim", "--headless", "-u", "NONE", "-i", "NONE", "-n", "-S", "editor/nvim/tests/run.vim")))
    if runtime:
        checks.extend([
            Check("runtime harness regressions", python + ("-m", "unittest", "discover", "-s", "runtime/tests", "-p", "test_*.py")),
            Check("native cleanup, stacks, contexts and scheduling", python + ("runtime/check.py",)),
        ])
    if compiler:
        cargo = ("--locked", "--manifest-path", "compiler/Cargo.toml", "--target", TARGET, "--target-dir", "compiler/target")
        checks.extend([
            Check("Rust formatting", ("cargo", "fmt", "--manifest-path", "compiler/Cargo.toml", "--check")),
            Check("Rust lint", ("cargo", "clippy") + cargo + ("--all-targets", "--", "-D", "warnings")),
            Check("Rust and native regression tests", ("cargo", "test") + cargo),
            Check("compiler harness regressions", python + ("-m", "unittest", "discover", "-s", "compiler/tests", "-p", "test_*.py")),
            Check("bootstrap compiler build", ("cargo", "build") + cargo),
            Check("compiler conformance execution", python + ("compiler/tests/conformance.py", "--compiler", str(ROOT / "compiler/target" / TARGET / "debug/meowy")) + (("--strict",) if strict else ())),
        ])
    return checks


def run(check, root=ROOT, timeout=300):
    print(f"CHECK {check.name}: {shlex.join(check.args)}", flush=True)
    try:
        process = subprocess.Popen(check.args, cwd=root, start_new_session=True)
    except OSError as error:
        print(f"FAIL {check.name}: {error}", flush=True)
        return False
    try:
        code = process.wait(timeout=timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()
        if isinstance(error, KeyboardInterrupt):
            raise
        print(f"FAIL {check.name}: exceeded {timeout:g} seconds", flush=True)
        return False
    print(f"{'PASS' if code == 0 else 'FAIL'} {check.name}: exit {code}", flush=True)
    return code == 0


def main():
    parser = argparse.ArgumentParser(description="Verify repository contracts; compiler execution and editor runtimes are opt-in.")
    parser.add_argument("--editor", choices=("vim", "nvim", "both"))
    parser.add_argument("--compiler", action="store_true")
    parser.add_argument("--runtime", action="store_true", help="Check native cleanup, guarded stacks, contexts and bounded scheduling in debug, release and sanitizer builds.")
    parser.add_argument("--strict", action="store_true", help="Require every compiler conformance case; requires --compiler or --all.")
    parser.add_argument("--all", action="store_true", help="Include editor, compiler and native runtime verification.")
    parser.add_argument("--list", action="store_true", help="Print selected commands without running them.")
    parser.add_argument("--timeout", type=float, default=300, help="Maximum seconds for each command (default: 300).")
    args = parser.parse_args()
    if args.strict and not (args.compiler or args.all):
        parser.error("--strict requires --compiler or --all")
    if not 0 < args.timeout < float("inf"):
        parser.error("--timeout must be a finite positive number")
    compiler = args.compiler or args.all
    editor = "both" if args.all else args.editor
    runtime = args.runtime or args.all
    checks = plan(editor, compiler, args.strict, runtime)
    if args.list:
        for check in checks:
            print(f"{check.name}: {shlex.join(check.args)}")
        print("Plan only; no checks were run.")
        return 0
    if not compiler:
        print("Scope: repository contracts; no Meowy source will be compiled or executed.", flush=True)
    elif not args.strict:
        print("Scope: repository contracts and compiler bootstrap; unsupported language cases remain visible.", flush=True)
    else:
        print("Scope: repository contracts and full reference catalog execution.", flush=True)
    if runtime:
        print("Native cleanup, guarded stacks, contexts, child waits and owned task values are selected; automatic cancellation/scope-exit joins and DWARF unwinding remain unqualified.", flush=True)
    if not editor:
        print("Editor runtimes are not selected; use --editor both to include them.", flush=True)
    for check in checks:
        if not run(check, timeout=args.timeout):
            print("Verification stopped at the failed check; subsequent checks were not run.")
            return 1
    print(f"All {len(checks)} selected checks passed. This does not qualify the complete v0.0.1 release.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
