#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Package the same release binaries with cargo-packager and makepkg.

No installation, privilege escalation, or changes to the developer's system.
Versions and payload files come from Cargo and the portable packager respectively.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib

from package_release import BINARY_NAMES, NOTICES, binary_paths, write_checksum

ROOT = Path(__file__).resolve().parents[1]
APP_ID = "io.github.yangyus8.razers"
TARGETS = {
    "x86_64-unknown-linux-gnu": ("deb", "pkg.tar.zst"),
    "aarch64-unknown-linux-gnu": ("deb", "pkg.tar.zst"),
    "x86_64-pc-windows-msvc": ("exe",),
    "x86_64-apple-darwin": ("dmg",),
    "aarch64-apple-darwin": ("dmg",),
}
DEB_DEPENDS = [
    "libc6 (>= 2.39)", "libgcc-s1", "libudev1", "libxkbcommon0", "libxkbcommon-x11-0",
    "libwayland-client0", "libwayland-egl1", "libegl1", "libgl1", "libx11-6", "libx11-xcb1", "libxcb1",
    "libxcursor1", "libxrandr2", "libxi6",
]
ARCH_DEPENDS = [
    "glibc>=2.39", "gcc-libs", "systemd-libs", "libxkbcommon", "libxkbcommon-x11", "wayland",
    "libglvnd", "libx11", "libxcb", "libxcursor", "libxrandr", "libxi",
]


def workspace_version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as source:
        return tomllib.load(source)["workspace"]["package"]["version"]


def validate_version(version: str) -> None:
    # GitHub's prerelease flag is independent of the native package version.
    # Refuse unsupported SemVer suffixes instead of silently losing information.
    if re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", version) is None:
        raise ValueError(f"native installers require an X.Y.Z version: {version}")


def artifact_name(version: str, target: str, extension: str) -> str:
    if target not in TARGETS or extension not in TARGETS[target]:
        raise ValueError("unsupported installer target or format")
    validate_version(version)
    suffix = "-setup.exe" if extension == "exe" else f".{extension}"
    return f"razers-v{version}-{target}{suffix}"


def packager_config(target: str, version: str, output: Path) -> dict:
    if target not in TARGETS:
        raise ValueError(f"unsupported target: {target}")
    validate_version(version)
    config = {
        "name": "razers",
        "productName": "RazeRS",
        "version": version,
        "identifier": APP_ID,
        "publisher": "YangYuS8",
        "authors": ["YangYuS8 and RazeRS contributors"],
        "homepage": "https://github.com/YangYuS8/razers",
        "description": "Read-only Razer device overview and community support evidence",
        "category": "Utility",
        # License texts are shipped below as notices, not turned into mandatory
        # click-through pages (including an EULA gate before mounting a DMG).
        "binaries": [{"path": name, "main": name == "razers"} for name in BINARY_NAMES],
        "binariesDir": str(ROOT / "target" / target / "release"),
        "outDir": str(output.resolve()),
        "targetTriple": target,
        "icons": [str(ROOT / "assets/icons" / name)
                  for name in ("razers.png", "razers@2x.png", "razers.ico")],
        "resources": [
            {"src": str(ROOT / source), "target": f"notices/{name}"}
            for source, name in NOTICES.items()
        ],
    }
    if "linux" in target:
        config.update({
            "formats": ["deb"],
            "linux": {"generateDesktopEntry": False},
            "deb": {
                "packageName": "razers", "section": "utils", "depends": DEB_DEPENDS,
                "files": {
                    str(ROOT / "tools/packaging/razers.desktop"):
                        f"usr/share/applications/{APP_ID}.desktop",
                    str(ROOT / "LICENSE"): "usr/share/licenses/razers/LICENSE",
                },
            },
        })
    elif "windows" in target:
        config.update({
            "formats": ["nsis"],
            "windows": {"allowDowngrades": False},
            "nsis": {
                "installMode": "currentUser",
                "languages": ["English", "SimpChinese"],
                "displayLanguageSelector": False,
                "installerIcon": str(ROOT / "assets/icons/razers.ico"),
                # Omit appdataPaths: uninstall never offers to delete user data.
            },
        })
    else:
        config.update({
            "formats": ["dmg"],
            "macos": {"minimumSystemVersion": "11.0", "signingIdentity": "-"},
        })
    return config


def run(*args: str | Path, **kwargs) -> None:
    subprocess.run([str(arg) for arg in args], check=True, **kwargs)


def arch_recipe(version: str, target: str) -> str:
    validate_version(version)
    if "linux" not in target or target not in TARGETS:
        raise ValueError("Arch packages require a supported Linux target")
    arch = target.split("-", 1)[0]
    dependencies = " ".join(f"'{name}'" for name in ARCH_DEPENDS)
    return f"""# Generated by tools/package_installers.py; do not edit.
pkgname=razers
pkgver={version}
pkgrel=1
pkgdesc='Read-only Razer device overview and community support evidence'
arch=('{arch}')
url='https://github.com/YangYuS8/razers'
license=('GPL-2.0-or-later')
depends=({dependencies})
options=('!strip' '!debug')
package() {{
    cp -a "$startdir/payload/usr" "$pkgdir/usr"
}}
"""


def build_arch(deb: Path, target: str, version: str, directory: Path) -> Path:
    """Use the verified Debian data tree, not cargo-packager's PKGBUILD tarball."""
    directory.mkdir()
    payload = directory / "payload"
    run("dpkg-deb", "--extract", deb, payload)
    (directory / "PKGBUILD").write_text(arch_recipe(version, target), encoding="utf-8")
    # Ubuntu runners can repackage prebuilt binaries without an Arch root or root
    # privileges. Runtime dependencies are checked separately in the lifecycle job.
    config = directory / "makepkg.conf"
    config.write_text(
        "source /etc/makepkg.conf\nPKGEXT='.pkg.tar.zst'\n"
        f"CARCH='{target.split('-', 1)[0]}'\n", encoding="utf-8"
    )
    run("makepkg", "--nodeps", "--noconfirm", "--config", config, cwd=directory)
    packages = list(directory.glob("*.pkg.tar.zst"))
    if len(packages) != 1:
        raise RuntimeError(f"expected one Arch package, found {packages}")
    return packages[0]


def stage_macos_binaries(target: str, directory: Path) -> None:
    """Give each standalone tool a valid ad-hoc signature before bundling.

    cargo-packager 0.11.8 sorts signing targets by path depth only. Signing the
    bundle's main executable can seal the bundle before a same-depth sibling has
    been signed. This fails for unsigned Intel binaries (ARM linkers pre-sign
    theirs). Pre-sign staged copies, without changing the portable build outputs.
    """
    directory.mkdir()
    for source in binary_paths(target):
        destination = directory / source.name
        shutil.copy2(ROOT / source, destination)
        run("codesign", "--force", "--sign", "-", "--options", "runtime",
            "--timestamp=none", destination)


def build_installers(target: str, version: str, output: Path, packager: Path) -> list[Path]:
    validate_version(version)
    if target not in TARGETS:
        raise ValueError(f"unsupported target: {target}")
    for binary in binary_paths(target):
        if not (ROOT / binary).is_file():
            raise FileNotFoundError(binary)
    output.mkdir(parents=True, exist_ok=True)
    artifacts = []
    with tempfile.TemporaryDirectory(prefix="razers-packaging-") as temporary:
        work = Path(temporary)
        raw = work / "output"
        config_file = work / "packager.json"
        config = packager_config(target, version, raw)
        if "apple" in target:
            stage_macos_binaries(target, work / "binaries")
            config["binariesDir"] = str(work / "binaries")
        config_file.write_text(json.dumps(config, indent=2), encoding="utf-8")
        run(packager.resolve(), "--config", config_file)
        extension = TARGETS[target][0]
        packages = list(raw.glob(f"*.{extension}"))
        if len(packages) != 1:
            raise RuntimeError(f"expected one {extension} installer, found {packages}")
        sources = [(extension, packages[0])]
        if "linux" in target:
            sources.append(("pkg.tar.zst", build_arch(packages[0], target, version, work / "arch")))
        for extension, source in sources:
            destination = output / artifact_name(version, target, extension)
            shutil.copyfile(source, destination)
            write_checksum(destination)
            artifacts.append(destination)
            print(f"created {destination}", flush=True)
    return artifacts


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--tag", help="Must match Cargo.toml; defaults to its current version")
    parser.add_argument("--packager", type=Path, required=True)
    args = parser.parse_args()
    version = workspace_version()
    if args.tag is not None and args.tag != f"v{version}":
        parser.error("release tag does not match the Cargo workspace version")
    build_installers(args.target, version, ROOT / "dist", args.packager)


if __name__ == "__main__":
    main()
