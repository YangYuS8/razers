#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Build both mdBooks and rustdoc into the repository's generated target/site."""

from __future__ import annotations

import html
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_docs import check_chapters, check_site
from localize_book import localize_book

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "target/site"


def run(*args: str, env: dict[str, str] | None = None) -> None:
    subprocess.run(args, cwd=ROOT, check=True, env=env)


def main() -> None:
    check_chapters(ROOT / "docs")
    mdbook = os.environ.get("MDBOOK", "mdbook")
    pinned = (ROOT / "tools/docs-requirements.txt").read_text().split("mdbook=")[1].strip()
    version = subprocess.check_output([mdbook, "--version"], text=True).strip()
    if version != f"mdbook v{pinned}":
        raise SystemExit(f"expected mdbook v{pinned}, got {version}")
    # Only remove this fixed generated site, never a user-supplied output path.
    if SITE.exists():
        shutil.rmtree(SITE)
    SITE.mkdir(parents=True)
    for language, source in [("en", "docs"), ("zh-CN", "docs/zh-CN")]:
        run(mdbook, "build", source, "--dest-dir", str(SITE / language))
    localize_book(ROOT / "docs/zh-CN", SITE / "zh-CN")
    # Isolate and refresh generated API docs so removed crates cannot survive a rebuild.
    doc_target = ROOT / "target/docs-build"
    doc_output = doc_target / "doc"
    if doc_output.exists():
        shutil.rmtree(doc_output)
    run("cargo", "doc", "--workspace", "--lib", "--all-features", "--no-deps", "--locked",
        "--target-dir", str(doc_target),
        env={**os.environ, "RUSTDOCFLAGS": "-D warnings"})
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"], cwd=ROOT, text=True))
    shutil.copytree(doc_output, SITE / "api")
    packages = sorted((package for package in metadata["packages"]
                       if package["id"] in metadata["workspace_members"]), key=lambda package: package["name"])
    links = []
    for package in packages:
        for target in package["targets"]:
            if "lib" not in target["kind"] or not target["doc"]:
                continue
            # cargo doc skips a same-named binary when a library is documented.
            destination = f'{target["name"].replace("-", "_")}/index.html'
            if (SITE / "api" / destination).exists():
                link = f'<li><a href="{destination}">{html.escape(package["name"])}</a> — {html.escape(package.get("description") or "")}</li>'
                if link not in links:
                    links.append(link)
    (SITE / "api/index.html").write_text(
        '<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width">'
        '<title>RazeRS Rust API / Rust API 参考</title><link rel="stylesheet" href="../site.css">'
        '<main><nav><a href="../en/">English handbook</a> · <a href="../zh-CN/" lang="zh-CN">中文手册</a></nav>'
        '<h1>RazeRS Rust API</h1><p lang="zh-CN">工作区 API 参考。符号保持稳定，说明按原始源码生成。</p>'
        '<p>Generated from the same commit as the handbook. Pre-alpha APIs may change.</p><ul>'
        + "".join(links) + '</ul></main></html>', encoding="utf-8")
    for name in ["index.html", "site.css"]:
        shutil.copyfile(ROOT / ".github/pages" / name, SITE / name)
    (SITE / ".nojekyll").touch()
    (SITE / "build-info.json").write_text(json.dumps({
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "version": tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"],
        "mdbook": pinned,
    }, indent=2) + "\n")
    check_site(SITE)
    print(f"Validated bilingual mdBook + rustdoc site: {SITE}")


if __name__ == "__main__":
    main()
