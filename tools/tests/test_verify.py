import contextlib
import io
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import verify


class VerifyTests(unittest.TestCase):
    def test_default_plan_does_not_execute_compiler_or_editor(self):
        checks = verify.plan()
        self.assertEqual(4, len(checks))
        self.assertTrue(all(check.args[0] == sys.executable for check in checks))
        self.assertTrue(all("compiler/" not in arg for check in checks for arg in check.args))
        self.assertTrue(all("-E" in check.args for check in checks))

    def test_selected_tools_and_strict_catalog(self):
        checks = verify.plan("both", True, True)
        self.assertIn("Vim runtime", [check.name for check in checks])
        self.assertIn("Neovim runtime", [check.name for check in checks])
        self.assertEqual("--strict", checks[-1].args[-1])
        self.assertLess([check.name for check in checks].index("bootstrap compiler build"), [check.name for check in checks].index("compiler conformance execution"))

    def test_cargo_build_and_harness_share_explicit_target_location(self):
        with patch.dict(os.environ, {"CARGO_TARGET_DIR": "/elsewhere", "CARGO_BUILD_TARGET": "other-target"}):
            checks = verify.plan(compiler=True)
        build = next(check for check in checks if check.name == "bootstrap compiler build")
        target = build.args[build.args.index("--target") + 1]
        base = build.args[build.args.index("--target-dir") + 1]
        self.assertEqual(str(verify.ROOT / base / target / "debug/meowy"), checks[-1].args[-1])

    def test_subprocess_uses_repository_directory(self):
        with tempfile.TemporaryDirectory() as temp, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temp)
            check = verify.Check("working directory", (sys.executable, "-c", "from pathlib import Path; Path('result').write_text('done')"))
            self.assertTrue(verify.run(check, root=root))
            self.assertEqual("done", (root / "result").read_text())

    def test_failure_missing_tool_and_timeout_fail(self):
        checks = [
            verify.Check("failed process", (sys.executable, "-c", "raise SystemExit(7)")),
            verify.Check("missing process", ("/nonexistent/meowy-verification-tool",)),
            verify.Check("timeout", (sys.executable, "-c", "import time; time.sleep(10)")),
        ]
        with contextlib.redirect_stdout(io.StringIO()):
            for check in checks:
                with self.subTest(check=check.name):
                    self.assertFalse(verify.run(check, timeout=0.1))

    def test_optimized_environment_does_not_disable_validator_assertions(self):
        check = verify.Check("assertions", verify.plan()[0].args[:5] + ("-c", "import sys; sys.exit(0 if __debug__ else 1)"))
        with patch.dict(os.environ, {"PYTHONOPTIMIZE": "2"}), contextlib.redirect_stdout(io.StringIO()):
            self.assertTrue(verify.run(check))

    def test_list_is_read_only_and_strict_requires_compiler(self):
        for args, code in [(["--all", "--strict", "--list"], 0), (["--strict"], 2), (["--timeout", "nan"], 2)]:
            with self.subTest(args=args):
                result = subprocess.run([sys.executable, "-B", str(verify.ROOT / "tools/verify.py"), *args], capture_output=True, text=True, cwd="/tmp")
                self.assertEqual(code, result.returncode)
                if code == 0:
                    self.assertIn("Plan only; no checks were run.", result.stdout)
                    self.assertIn("--strict", result.stdout)

    def test_verification_stops_after_failure(self):
        with patch.object(sys, "argv", ["verify.py"]), patch.object(verify, "run", return_value=False) as run, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(1, verify.main())
            self.assertEqual(1, run.call_count)


if __name__ == "__main__":
    unittest.main()
