#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Gate publication on the complete installer/portable matrix and its checksums."""

import argparse
from pathlib import Path
import tarfile
import zipfile

from check_installers import require, verify_checksum
from package_installers import ROOT, TARGETS, artifact_name, workspace_version
from package_release import BINARY_NAMES, NOTICES


def expected_assets(version: str) -> set[str]:
    assets = set()
    for target, formats in TARGETS.items():
        assets.update(artifact_name(version, target, extension) for extension in formats)
        extension = ".zip" if "windows" in target else ".tar.gz"
        assets.add(f"razers-v{version}-{target}{extension}")
    return assets | {name + ".sha256" for name in assets}


def verify_release(directory: Path, version: str) -> None:
    actual = {path.name for path in directory.iterdir()}
    expected = expected_assets(version)
    require(actual == expected,
            f"incomplete release; missing={sorted(expected - actual)}, unexpected={sorted(actual - expected)}")
    for name in sorted(actual):
        if name.endswith(".sha256"):
            continue
        artifact = directory / name
        verify_checksum(artifact)
        if name.endswith((".zip", ".tar.gz")):
            suffix = ".exe" if name.endswith(".zip") else ""
            expected_files = set(NOTICES.values()) | {name + suffix for name in BINARY_NAMES}
            if suffix:
                with zipfile.ZipFile(artifact) as archive:
                    require(set(archive.namelist()) == expected_files, "portable ZIP payload mismatch")
                    for source, notice in NOTICES.items():
                        require(archive.read(notice) == (ROOT / source).read_bytes(), "portable ZIP notice mismatch")
            else:
                with tarfile.open(artifact) as archive:
                    require(set(archive.getnames()) == expected_files, "portable tar payload mismatch")
                    for source, notice in NOTICES.items():
                        require(archive.extractfile(notice).read() == (ROOT / source).read_bytes(), "portable tar notice mismatch")
    print(f"Verified complete release: {len(expected)} assets, all SHA-256 checksums and portable payloads")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--tag")
    args = parser.parse_args()
    version = workspace_version()
    if args.tag is not None and args.tag != f"v{version}":
        parser.error("tag does not match Cargo.toml")
    verify_release(args.directory, version)
