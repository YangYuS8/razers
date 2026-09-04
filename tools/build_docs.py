#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Build Starlight and rustdoc into one validated, static GitHub Pages artifact."""

from __future__ import annotations

import html
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_docs import check_chapters, check_site

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "target/site"


def run(*args: str, env: dict[str, str] | None = None) -> None:
    subprocess.run(args, cwd=ROOT, check=True, env=env)


def main() -> None:
    check_chapters(ROOT / "docs")
    docs_package = json.loads((ROOT / "docs/package.json").read_text())
    manager = docs_package["packageManager"]
    executable = os.environ.get("PNPM") or shutil.which("pnpm")
    pnpm = [executable] if executable else ["npm", "exec", "--yes", f"--package={manager}", "--", "pnpm"]
    version = subprocess.check_output([*pnpm, "--version"], text=True).strip()
    if version != manager.removeprefix("pnpm@").split("+")[0]:
        raise SystemExit(f"expected {manager}, got pnpm {version}")
    run(*pnpm, "--dir", "docs", "install", "--frozen-lockfile")
    # Only remove this fixed generated site, never a user-supplied output path.
    if SITE.exists():
        shutil.rmtree(SITE)
    SITE.mkdir(parents=True)
    run(*pnpm, "--dir", "docs", "run", "build",
        env={**os.environ, "ASTRO_TELEMETRY_DISABLED": "1"})
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
        "generator": "Astro Starlight",
        "astro": docs_package["dependencies"]["astro"],
        "starlight": docs_package["dependencies"]["@astrojs/starlight"],
        "package_manager": manager,
        "channel": "development",
    }, indent=2) + "\n")
    check_site(SITE)
    print(f"Validated bilingual Starlight + rustdoc site: {SITE}")


if __name__ == "__main__":
    main()
