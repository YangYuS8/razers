#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Build Starlight and rustdoc into one validated, static GitHub Pages artifact."""

from __future__ import annotations

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
    # Starlight owns every handbook page, including the root and API overview.
    # Only rustdoc's crate pages/assets are added; never overwrite a handbook page.
    if (doc_output / "index.html").exists():
        raise RuntimeError("rustdoc index would overwrite the Starlight API overview")
    shutil.copytree(doc_output, SITE / "api", dirs_exist_ok=True)
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
