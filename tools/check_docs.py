#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Validate translation pairs, PR freshness, and generated local links/anchors."""

from __future__ import annotations

import argparse
from html.parser import HTMLParser
from pathlib import Path
import subprocess
import re
import os
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
CONTENT = Path("docs/src/content/docs")
LOCALES = ("en", "zh-CN")


def check_chapters(docs: Path) -> None:
    """Starlight page pairs replace the old SUMMARY.md lists."""
    content = docs / "src/content/docs"
    pages = [{path.relative_to(content / locale) for path in (content / locale).rglob("*")
              if path.suffix in {".md", ".mdx"}}
             for locale in LOCALES]
    if not pages[0] or pages[0] != pages[1]:
        raise ValueError(f"English/Chinese pages missing or unmatched: {pages[0] ^ pages[1]}")
    for locale in LOCALES:
        for path in sorted(pages[0]):
            text = (content / locale / path).read_text(encoding="utf-8")
            if not text.startswith("---\n") or "\ntitle:" not in text or "\ndescription:" not in text:
                raise ValueError(f"missing page title/description: {locale}/{path}")
    print(f"Checked {len(pages[0])} English/Chinese page pairs")


def translation_gaps(changed: set[str]) -> list[str]:
    gaps = set()
    for path in changed:
        for locale, other in [("en", "zh-CN"), ("zh-CN", "en")]:
            prefix = f"{CONTENT.as_posix()}/{locale}/"
            if path.startswith(prefix) and Path(path).suffix in {".md", ".mdx"}:
                counterpart = f"{CONTENT.as_posix()}/{other}/{path.removeprefix(prefix)}"
                if counterpart not in changed:
                    gaps.add(counterpart)
        if path in {"README.md", "README.zh-CN.md"}:
            counterpart = "README.zh-CN.md" if path == "README.md" else "README.md"
            if counterpart not in changed:
                gaps.add(counterpart)
    return sorted(gaps)


def check_translation_changes(base: str) -> None:
    # Compare content changes against the PR base, not timestamps; squash commits
    # must not make stale translations appear newer. Include both ends of renames.
    subprocess.run(["git", "rev-parse", "--verify", f"{base}^{{commit}}"], cwd=ROOT,
                   check=True, stdout=subprocess.DEVNULL)
    changed = set(subprocess.check_output(
        ["git", "diff", "--no-renames", "--name-only", base, "--", str(CONTENT),
         "README.md", "README.zh-CN.md"], cwd=ROOT, text=True).splitlines())
    gaps = translation_gaps(changed)
    for gap in gaps:
        message = f"Review the unchanged translation / 请检查未改动的译文: {gap}"
        if os.environ.get("GITHUB_ACTIONS") == "true":
            escaped = message.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
            print(f"::warning title=Translation review::{escaped}")
        else:
            print(f"WARNING: {message}")
    if not gaps:
        print("Changed documentation has matching language changes (semantic review still required)")


class Links(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.references: list[str] = []
        self.ids: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        for key, value in attrs:
            if key in {"href", "src"} and value:
                self.references.append(value)
            if tag == "option" and key == "value" and value and value.startswith("/razers/"):
                self.references.append(value)
            if value and (key == "id" or (tag == "a" and key == "name")):
                self.ids.add(value)


def check_site(site: Path) -> None:
    site = site.resolve()
    problems = []
    parsed = {}
    for page in site.rglob("*.html"):
        links = Links()
        links.feed(page.read_text(encoding="utf-8"))
        parsed[page] = links
    if not parsed:
        raise ValueError("site has no HTML")
    for page, links in parsed.items():
        for reference in links.references:
            url = urlsplit(reference)
            if url.scheme or url.netloc:
                if url.scheme not in {"https", "http"} or url.netloc != "yangyus8.top":
                    continue
            if not url.path:
                target = page
            elif url.path.startswith("/"):
                if url.path not in {"/razers", "/razers/"} and not url.path.startswith("/razers/"):
                    problems.append(f"{page.relative_to(site)}: wrong site mount {reference}")
                    continue
                target = (site / unquote(url.path.removeprefix("/razers").lstrip("/"))).resolve()
            else:
                target = (page.parent / unquote(url.path)).resolve()
            if not target.is_relative_to(site):
                problems.append(f"{page.relative_to(site)}: outside site {reference}")
                continue
            if target.is_dir():
                target /= "index.html"
            if not target.is_file():
                problems.append(f"{page.relative_to(site)}: missing {reference}")
            elif url.fragment and target in parsed:
                ids = parsed[target].ids
                # Browsers try the literal ID before percent-decoding it. Rustdoc
                # uses encoded IDs for generics and JS interprets source line ranges.
                if url.fragment in ids or unquote(url.fragment) in ids:
                    continue
                lines = re.fullmatch(r"(\d+)-(\d+)", url.fragment)
                if (target.is_relative_to(site / "api/src") and lines
                        and int(lines[1]) <= int(lines[2])
                        and lines[1] in ids and lines[2] in ids):
                    continue
                problems.append(f"{page.relative_to(site)}: missing anchor {reference}")
    if problems:
        raise ValueError("Broken local links:\n" + "\n".join(sorted(set(problems))))
    print(f"Checked links and anchors in {len(parsed)} HTML pages")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="Git base commit for translation-change checks")
    parser.add_argument("--site", type=Path, help="Generated site to check")
    args = parser.parse_args()
    check_chapters(ROOT / "docs")
    if args.base:
        check_translation_changes(args.base)
    if args.site:
        check_site(args.site)
