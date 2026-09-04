# SPDX-License-Identifier: GPL-2.0-or-later
import sys
import io
from contextlib import redirect_stdout
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_docs import check_chapters, check_site, check_translation_changes, translation_gaps


class DocumentationChecks(unittest.TestCase):
    def test_unpaired_language_change_requires_review(self):
        self.assertEqual(translation_gaps({"docs/src/content/docs/en/safety.md"}),
                         ["docs/src/content/docs/zh-CN/safety.md"])
        self.assertEqual(translation_gaps({"docs/src/content/docs/zh-CN/safety.md"}),
                         ["docs/src/content/docs/en/safety.md"])

    def test_paired_changes_and_unrelated_files_pass(self):
        self.assertEqual(translation_gaps({"docs/src/content/docs/en/safety.md",
                                          "docs/src/content/docs/zh-CN/safety.md",
                                          "docs/astro.config.mjs"}), [])
        self.assertEqual(translation_gaps({"README.md"}), ["README.zh-CN.md"])

    def test_repository_chapters_match(self):
        check_chapters(Path(__file__).resolve().parents[2] / "docs")

    def test_single_language_edits_warn_without_requiring_artificial_changes(self):
        output = io.StringIO()
        with patch("check_docs.subprocess.run"), patch("check_docs.subprocess.check_output",
                return_value="docs/src/content/docs/en/safety.md\n"), \
                patch.dict("os.environ", {"GITHUB_ACTIONS": "true"}), redirect_stdout(output):
            check_translation_changes("base")
        self.assertIn("::warning title=Translation review::", output.getvalue())
        self.assertIn("docs/src/content/docs/zh-CN/safety.md", output.getvalue())

    def test_missing_language_page_blocks_validation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            english = root / "src/content/docs/en"
            english.mkdir(parents=True)
            (english / "index.md").write_text('---\ntitle: Home\ndescription: Home\n---\n')
            with self.assertRaisesRegex(ValueError, "missing or unmatched"):
                check_chapters(root)

    def test_broken_link_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "index.html").write_text('<a href="missing.html">broken</a>')
            with self.assertRaisesRegex(ValueError, "missing"):
                check_site(root)

    def test_language_selector_destinations_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "index.html").write_text('<select><option value="/razers/zh-CN/404/">中文</option></select>')
            with self.assertRaisesRegex(ValueError, "missing /razers/zh-CN/404/"):
                check_site(root)

    def test_directory_links_and_fragments(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "en").mkdir()
            (root / "index.html").write_text('<a href="en/#start">book</a>')
            (root / "en/index.html").write_text('<h1 id="start">Start</h1><a href="../index.html">home</a>')
            check_site(root)

    def test_missing_anchor_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "index.html").write_text('<a href="#missing">broken</a>')
            with self.assertRaisesRegex(ValueError, "missing anchor"):
                check_site(root)

    def test_percent_encoded_chinese_anchor(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "index.html").write_text('<h1 id="语言">语言</h1><a href="#%E8%AF%AD%E8%A8%80">link</a>')
            check_site(root)

    def test_wrong_mount_and_escaped_path_are_rejected(self):
        for href in ["/en/index.html", "../outside.html", "/razers/%2e%2e/outside.html"]:
            with self.subTest(href=href), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "index.html").write_text(f'<a href="{href}">broken</a>')
                with self.assertRaises(ValueError):
                    check_site(root)

    def test_same_site_absolute_links_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "index.html").write_text('<a href="https://yangyus8.top/razers/missing/">broken</a>')
            with self.assertRaisesRegex(ValueError, "missing"):
                check_site(root)

    def test_literal_percent_encoded_rustdoc_ids(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "index.html").write_text(
                '<h2 id="impl-Borrow%3CT%3E">impl</h2><a href="#impl-Borrow%3CT%3E">link</a>')
            check_site(root)

    def test_rustdoc_source_ranges_require_existing_ordered_endpoints(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "api/src/example"
            source.mkdir(parents=True)
            (source / "lib.rs.html").write_text('<a id=10>10</a><a id=12>12</a>')
            (root / "index.html").write_text('<a href="api/src/example/lib.rs.html#10-12">source</a>')
            check_site(root)
            for fragment in ["10-13", "12-10"]:
                (root / "index.html").write_text(f'<a href="api/src/example/lib.rs.html#{fragment}">bad</a>')
                with self.assertRaisesRegex(ValueError, "missing anchor"):
                    check_site(root)
