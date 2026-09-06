import subprocess
import signal
import sys
import unittest
from pathlib import Path
from unittest import mock


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check


class CheckTests(unittest.TestCase):
    def test_arbitrary_failure_does_not_count_as_fatal_panic(self):
        for code in (0, 1, 3, -signal.SIGSEGV):
            with self.subTest(code=code):
                result = subprocess.CompletedProcess(["binary"], code, "", "panic[P008]")
                with self.assertRaises(RuntimeError):
                    check.check_fatal(result, False)

    def test_abort_requires_complete_cleanup_evidence(self):
        result = subprocess.CompletedProcess(["binary"], -signal.SIGABRT, "", "panic[P008]")
        with self.assertRaisesRegex(RuntimeError, "evidence differs"):
            check.check_fatal(result, True)

    def test_original_panic_is_not_lost(self):
        error = ("release-trigger\npanic[P008]: panic during cleanup\n"
                 "original: panic P006: body failed\ncleanup: newer\nsecond: P006: release failed\n")
        result = subprocess.CompletedProcess(["binary"], -signal.SIGABRT, "", error)
        check.check_fatal(result, True)
        with self.assertRaises(RuntimeError):
            check.check_fatal(result, False)

    def test_wrong_toolchain_fails_before_build(self):
        result = subprocess.CompletedProcess(["clang"], 0, "clang version 21.1.8\n", "")
        with mock.patch.object(check, "invoke", return_value=result) as invoke:
            with self.assertRaisesRegex(RuntimeError, "requires Clang 22.1.8"):
                check.check("clang", Path("/tmp"))
        invoke.assert_called_once_with(["clang", "--version"])

    def test_timeout_kills_group_and_reaps_process(self):
        process = mock.Mock(pid=1234)
        process.communicate.side_effect = [subprocess.TimeoutExpired("runtime", 60), (b"", b"")]
        with mock.patch.object(check.subprocess, "Popen", return_value=process) as popen:
            with mock.patch.object(check.os, "killpg") as kill:
                with self.assertRaises(subprocess.TimeoutExpired):
                    check.invoke(["runtime"])
        self.assertTrue(popen.call_args.kwargs["start_new_session"])
        kill.assert_called_once_with(1234, signal.SIGKILL)
        self.assertEqual(process.communicate.call_count, 2)


if __name__ == "__main__":
    unittest.main()
