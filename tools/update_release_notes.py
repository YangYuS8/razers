#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Add bilingual installer-first download links to an existing GitHub release."""

import argparse
from pathlib import Path
import subprocess
import tempfile

from package_installers import artifact_name, workspace_version

MARKER = "<!-- razers-downloads -->"


def download_section(version: str) -> str:
    base = f"https://github.com/YangYuS8/razers/releases/download/v{version}/"
    rows = [
        ("Windows x64", "x86_64-pc-windows-msvc", "exe", "Setup / 安装向导"),
        ("macOS Apple Silicon", "aarch64-apple-darwin", "dmg", "DMG"),
        ("macOS Intel", "x86_64-apple-darwin", "dmg", "DMG"),
        ("Debian / Ubuntu x64", "x86_64-unknown-linux-gnu", "deb", "DEB"),
        ("Debian / Ubuntu ARM64", "aarch64-unknown-linux-gnu", "deb", "DEB"),
        ("Arch Linux x64", "x86_64-unknown-linux-gnu", "pkg.tar.zst", "Pacman"),
        ("Arch Linux ARM64", "aarch64-unknown-linux-gnu", "pkg.tar.zst", "Pacman"),
    ]
    text = [MARKER, "## Install / 安装", "", "| System / 系统 | Download / 下载 |", "| --- | --- |"]
    for system, target, extension, label in rows:
        text.append(f"| {system} | [{label}]({base}{artifact_name(version, target, extension)}) |")
    text += [
        "", "Portable archives and SHA-256 files remain under Assets. Read the "
        "[installation guide](https://yangyus8.top/razers/getting-started/) for platform requirements, "
        "verification, upgrades and removal. Windows installers are unsigned; macOS apps are "
        "ad-hoc signed, not notarized. This remains a read-only preview, not hardware control.",
        "", "便携包和 SHA-256 校验文件保留在 Assets。平台要求、校验、升级与卸载请见"
        "[安装指南](https://yangyus8.top/razers/zh-CN/getting-started/)。"
        "Windows 安装器尚无发布者签名，macOS 应用仅有临时签名、未经公证。当前仍是只读预览，不提供硬件控制。",
        "", MARKER,
    ]
    return "\n".join(text)


def with_downloads(body: str, version: str) -> str:
    if MARKER in body:
        before, _, rest = body.partition(MARKER)
        _, separator, after = rest.partition(MARKER)
        if not separator:
            raise ValueError("unpaired download section marker")
        body = before + after
    return download_section(version) + "\n\n" + body.strip() + "\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    version = workspace_version()
    if args.tag != f"v{version}":
        parser.error("tag does not match Cargo.toml")
    body = subprocess.check_output(["gh", "release", "view", args.tag, "--json", "body", "--jq", ".body"], text=True)
    with tempfile.TemporaryDirectory(prefix="razers-notes-") as directory:
        notes = Path(directory) / "notes.md"
        notes.write_text(with_downloads(body, version), encoding="utf-8")
        subprocess.run(["gh", "release", "edit", args.tag, "--notes-file", str(notes)], check=True)
