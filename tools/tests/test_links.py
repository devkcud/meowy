import sys
import tempfile
import unittest
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check_links


class LinkTests(unittest.TestCase):
    def test_inline_links_skip_code_and_keep_line_numbers(self):
        text = "[one](one.md)\n`[fake](missing.md)`\n```md\n[fake](absent.md)\n```\n[real](<two words.md#name> \"Title\")\n"
        self.assertEqual([(1, "one.md"), (6, "two words.md#name")], list(check_links.links(text)))

    def test_fences_comments_and_nested_labels(self):
        text = "~~~md\n[fake](none)\n~~~\n<!-- [fake](none) -->\n[an [inner] label](name(test).md)\n[`code`](name.md)\n"
        self.assertEqual([(5, "name(test).md"), (6, "name.md")], list(check_links.links(text)))

    def test_heading_punctuation_unicode_and_duplicate_ids(self):
        text = "# A `name`!\n## A name\n### A name-1\n# Café & tea ###\n```\n# Ignore\n```\n"
        self.assertEqual({"a-name", "a-name-1", "a-name-1-1", "café--tea"}, check_links.anchors(text))

    def test_local_paths_and_anchors(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "docs").mkdir()
            (root / "docs/one.md").write_text("# Start\n## Next step\n## Next step\n", encoding="utf-8")
            (root / "two words.txt").write_text("fixture", encoding="utf-8")
            (root / "README.md").write_text("# Home\n[local](docs/one.md#next-step-1)\n[own](#home)\n[space](two%20words.txt)\n[remote](https://example.test/missing)\n[remote](//example.test/missing)\n", encoding="utf-8")
            self.assertEqual((2, 3, []), check_links.check(root))

    def test_broken_path_anchor_escape_and_symlink(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            root.mkdir()
            outside = Path(temp) / "outside.md"
            outside.write_text("# Outside", encoding="utf-8")
            (root / "linked.md").symlink_to(outside)
            (root / "README.md").write_text("# Home\n[missing](absent.md)\n[bad](#unknown)\n[outside](../outside.md)\n[linked](linked.md)\n", encoding="utf-8")
            files, count, errors = check_links.check(root)
            self.assertEqual((2, 4), (files, count))
            self.assertEqual(4, len(errors))
            self.assertIn("README.md:2", errors[0])
            self.assertIn("heading does not exist", errors[1])
            self.assertTrue(all("escapes repository" in error for error in errors[2:]))

    def test_runtime_documentation_is_checked(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "runtime").mkdir()
            (root / "runtime/README.md").write_text("[missing](absent.md)", encoding="utf-8")
            files, count, errors = check_links.check(root)
            self.assertEqual((1, 1), (files, count))
            self.assertEqual(1, len(errors))
            self.assertIn("runtime/README.md:1", errors[0])

    def test_generated_directories_are_excluded(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "compiler/target").mkdir(parents=True)
            (root / "compiler/target/ignored.md").write_text("[bad](missing.md)", encoding="utf-8")
            (root / "README.md").write_text("# Home", encoding="utf-8")
            self.assertEqual((1, 0, []), check_links.check(root))


if __name__ == "__main__":
    unittest.main()
