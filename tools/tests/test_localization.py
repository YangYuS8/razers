# SPDX-License-Identifier: GPL-2.0-or-later
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


class CatalogCoverage(unittest.TestCase):
    def test_static_presentation_keys_are_in_both_catalogs(self):
        pattern = r'\b(?:locale\(\)|locale)\s*\.\s*(?:text|format)\(\s*("(?:\\.|[^"\\])*")'
        for language in ["en", "zh-CN"]:
            catalog = json.loads((ROOT / f"crates/razers-i18n/locales/{language}.json").read_text(encoding="utf-8"))
            for file in (ROOT / "crates").rglob("*.rs"):
                if file.parent.parent.name == "razers-i18n":
                    continue  # This crate deliberately tests unknown-key fallback.
                for match in re.finditer(pattern, file.read_text(encoding="utf-8")):
                    key = json.loads(match[1])
                    self.assertIn(key, catalog, f"{language}: {file}: {key}")
