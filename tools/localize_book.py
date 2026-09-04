#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Localize mdBook chrome and generate a Chinese substring-search index."""

from html.parser import HTMLParser
import json
from pathlib import Path
import re


CHROME = {
    "Table of contents": "目录",
    "Toggle Table of Contents": "展开或收起目录",
    "Change theme": "切换主题",
    "Themes": "主题",
    "Print this book": "打印手册",
    "Git repository": "Git 仓库",
    "Suggest an edit": "建议修改",
    "Page navigation": "章节导航",
    "Next chapter": "下一章",
    "Previous chapter": "上一章",
    "Keyboard shortcuts": "键盘快捷键",
    "Document not found (404)": "页面不存在（404）",
    "Light": "浅色",
    "Rust": "锈色",
    "Coal": "煤黑",
    "Navy": "深蓝",
    "Ayu": "Ayu 暗色",
}


class MainText(HTMLParser):
    def __init__(self):
        super().__init__()
        self.in_main = False
        self.text = []

    def handle_starttag(self, tag, attrs):
        if tag == "main":
            self.in_main = True

    def handle_endtag(self, tag):
        if tag == "main":
            self.in_main = False

    def handle_data(self, data):
        if self.in_main:
            self.text.append(data)


def localize_book(source: Path, output: Path) -> None:
    for page in output.rglob("*.html"):
        content = page.read_text(encoding="utf-8")
        for english, chinese in CHROME.items():
            for attribute in ["title", "aria-label"]:
                content = content.replace(f'{attribute}="{english}"', f'{attribute}="{chinese}"')
            content = content.replace(f">{english}<", f">{chinese}<")
        for english, chinese in {
            "Press <kbd>←</kbd> or <kbd>→</kbd> to navigate between chapters": "按 <kbd>←</kbd> 或 <kbd>→</kbd> 切换章节",
            "Press <kbd>S</kbd> or <kbd>/</kbd> to search in the book": "按 <kbd>S</kbd> 或 <kbd>/</kbd> 搜索手册",
            "Press <kbd>?</kbd> to show this help": "按 <kbd>?</kbd> 显示帮助",
            "Press <kbd>Esc</kbd> to hide this help": "按 <kbd>Esc</kbd> 关闭帮助",
        }.items():
            content = content.replace(english, chinese)
        page.write_text(content, encoding="utf-8")
    entries = []
    for title, path in re.findall(r"\[([^\]]+)\]\(([^)]+\.md)\)", (source / "SUMMARY.md").read_text(encoding="utf-8")):
        destination = str(Path(path).with_suffix(".html"))
        parser = MainText()
        parser.feed((output / destination).read_text(encoding="utf-8"))
        entries.append({"title": title, "path": destination,
                        "text": " ".join(" ".join(parser.text).split())})
    (output / "search-data.json").write_text(json.dumps(entries, ensure_ascii=False), encoding="utf-8")
