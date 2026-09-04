#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Offline chapter-parity and generated local-link validation."""

from html.parser import HTMLParser
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit


def check_chapters(docs: Path) -> None:
    chapters = []
    for folder in [docs, docs / "zh-CN"]:
        paths = re.findall(r"\]\(([^)]+\.md)\)", (folder / "SUMMARY.md").read_text(encoding="utf-8"))
        if not paths or len(paths) != len(set(paths)):
            raise ValueError(f"empty or duplicate chapter list: {folder}")
        for path in paths:
            if not (folder / path).is_file():
                raise ValueError(f"missing chapter: {folder / path}")
        chapters.append(paths)
    if chapters[0] != chapters[1]:
        raise ValueError("English and Chinese chapter paths/order differ")


class Links(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.references: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        for key, value in attrs:
            if key in {"href", "src"} and value:
                self.references.append(value)


def check_site(site: Path) -> None:
    site = site.resolve()
    problems = []
    pages = list(site.rglob("*.html"))
    if not pages:
        raise ValueError("site has no HTML")
    for page in pages:
        links = Links()
        links.feed(page.read_text(encoding="utf-8"))
        for reference in links.references:
            url = urlsplit(reference)
            if url.scheme or url.netloc or not url.path:
                continue
            # mdBook's 404 <base> uses the GitHub Pages project mount.
            if url.path.startswith("/"):
                if not url.path.startswith("/razers/"):
                    problems.append(f"{page.relative_to(site)}: wrong site mount {reference}")
                    continue
                target = (site / unquote(url.path.removeprefix("/razers/"))).resolve()
            else:
                target = (page.parent / unquote(url.path)).resolve()
            if not target.is_relative_to(site):
                problems.append(f"{page.relative_to(site)}: outside site {reference}")
                continue
            if target.is_dir():
                target /= "index.html"
            if not target.is_file():
                problems.append(f"{page.relative_to(site)}: missing {reference}")
    if problems:
        raise ValueError("Broken local links:\n" + "\n".join(sorted(set(problems))))
    print(f"Checked local links in {len(pages)} HTML pages")
