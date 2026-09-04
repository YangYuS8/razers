# SPDX-License-Identifier: GPL-2.0-or-later
import sys
from pathlib import Path
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_docs import check_chapters, check_site
from localize_book import localize_book
import json


class DocumentationChecks(unittest.TestCase):
    def test_chinese_search_indexes_unsegmented_text_and_translates_chrome(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "SUMMARY.md").write_text("- [语言](index.md)", encoding="utf-8")
            (root / "index.html").write_text('<button title="Change theme">Light</button><main><h1>语言</h1><p>可以切换中文语言</p></main>', encoding="utf-8")
            localize_book(root, root)
            self.assertIn('title="切换主题"', (root / "index.html").read_text(encoding="utf-8"))
            data = json.loads((root / "search-data.json").read_text(encoding="utf-8"))
            self.assertIn("中文语言", data[0]["text"])
            self.assertNotIn("button", data[0]["text"])
            self.assertEqual(data[0]["path"], "index.html")

    def test_repository_chapters_match(self):
        check_chapters(Path(__file__).resolve().parents[2] / "docs")

    def test_broken_link_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "index.html").write_text('<a href="missing.html">broken</a>')
            with self.assertRaisesRegex(ValueError, "missing"):
                check_site(root)

    def test_directory_links_and_fragments(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "en").mkdir()
            (root / "index.html").write_text('<a href="en/#start">book</a>')
            (root / "en/index.html").write_text('<a href="../index.html">home</a>')
            check_site(root)
