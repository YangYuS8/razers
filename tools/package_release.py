#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later

"""Create a deterministic razersctl release archive and SHA-256 file."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import os
import re
import shutil
import tarfile
import tempfile
import zipfile
from pathlib import Path


TAG_PATTERN = re.compile(r"v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?")
FIXED_ZIP_TIME = (1980, 1, 1, 0, 0, 0)


def binary_path(target: str) -> Path:
    suffix = ".exe" if "windows" in target else ""
    return Path("target") / target / "release" / f"razersctl{suffix}"


def copy_payload(staging: Path, binary: Path) -> None:
    destination = staging / binary.name
    shutil.copyfile(binary, destination)
    destination.chmod(0o755)
    for name in ("README.md", "LICENSE", "CHANGELOG.md"):
        shutil.copyfile(name, staging / name)


def write_tar(archive: Path, staging: Path) -> None:
    with archive.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as output:
                for path in sorted(staging.iterdir(), key=lambda item: item.name):
                    info = output.gettarinfo(str(path), arcname=path.name)
                    info.mtime = 0
                    info.uid = 0
                    info.gid = 0
                    info.uname = ""
                    info.gname = ""
                    with path.open("rb") as source:
                        output.addfile(info, source)


def write_zip(archive: Path, staging: Path) -> None:
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as output:
        for path in sorted(staging.iterdir(), key=lambda item: item.name):
            info = zipfile.ZipInfo(path.name, FIXED_ZIP_TIME)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = (0o755 if path.suffix == ".exe" else 0o644) << 16
            output.writestr(info, path.read_bytes())


def write_checksum(archive: Path) -> Path:
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    checksum = archive.with_name(f"{archive.name}.sha256")
    checksum.write_text(f"{digest}  {archive.name}\n", encoding="ascii")
    return checksum


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", required=True)
    parser.add_argument("--target", required=True)
    args = parser.parse_args()

    if TAG_PATTERN.fullmatch(args.tag) is None:
        raise SystemExit(f"invalid release tag: {args.tag}")
    binary = binary_path(args.target)
    if not binary.is_file():
        raise SystemExit(f"release binary not found: {binary}")

    output_directory = Path("dist")
    output_directory.mkdir(exist_ok=True)
    extension = ".zip" if "windows" in args.target else ".tar.gz"
    archive = output_directory / f"razers-{args.tag}-{args.target}{extension}"

    with tempfile.TemporaryDirectory(prefix="razers-release-") as temporary:
        staging = Path(temporary)
        copy_payload(staging, binary)
        if extension == ".zip":
            write_zip(archive, staging)
        else:
            write_tar(archive, staging)

    checksum = write_checksum(archive)
    print(f"created {archive} and {checksum}")


if __name__ == "__main__":
    main()
