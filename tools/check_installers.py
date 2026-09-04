#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Inspect release payloads, or exercise installers on disposable hosted CI only."""

from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
import io
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tarfile
import tempfile

from package_installers import (
    APP_ID, ROOT, TARGETS, artifact_name, build_installers, run, workspace_version,
)
from package_release import BINARY_NAMES, NOTICES


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def captured(*args: str | Path, **kwargs) -> str:
    return subprocess.check_output([str(arg) for arg in args], text=True, encoding="utf-8", **kwargs)


def verify_checksum(artifact: Path) -> None:
    expected = f"{hashlib.sha256(artifact.read_bytes()).hexdigest()}  {artifact.name}\n"
    require(artifact.with_name(artifact.name + ".sha256").read_text(encoding="ascii") == expected,
            f"checksum or checksum filename mismatch: {artifact.name}")


def smoke_binaries(directory: Path, version: str) -> None:
    """Both real executables answer agent.info, without enumerating/opening HID."""
    suffix = ".exe" if sys.platform == "win32" else ""
    request = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "agent.info",
                          "params": {"protocol_version": 1}}) + "\n"
    for name, flag in [("razers-agent", "--stdio"), ("razers", "--agent-stdio")]:
        result = subprocess.run([str(directory / (name + suffix)), flag], input=request,
                                capture_output=True, text=True, encoding="utf-8", timeout=30, check=True)
        response = json.loads(result.stdout)
        require(response.get("id") == 1 and "error" not in response, "agent.info response failed")
        info = response["result"]
        require(info["protocol_version"] == 1 and info["agent_version"] == version,
                "installed Agent protocol/version mismatch")
        require(info["access_mode"] == "descriptor-only", "unexpected hardware access mode")
    for language in ["en", "zh-CN"]:
        output = captured(directory / f"razersctl{suffix}", "--lang", language, "help")
        require("razersctl" in output and len(output) > 100, "installed CLI help is incomplete")


def verify_notices(directory: Path) -> None:
    for source, name in NOTICES.items():
        require((directory / name).read_bytes() == (ROOT / source).read_bytes(),
                f"missing or changed notice: {name}")


def deb_members(artifact: Path) -> dict[str, bytes]:
    """Read the small ar wrapper without requiring dpkg on a developer machine."""
    stream = io.BytesIO(artifact.read_bytes())
    require(stream.read(8) == b"!<arch>\n", "not a Debian ar package")
    members = {}
    while header := stream.read(60):
        require(len(header) == 60 and header[58:60] == b"`\n", "invalid ar header")
        name = header[:16].decode("ascii").strip().rstrip("/")
        size = int(header[48:58])
        data = stream.read(size)
        require(len(data) == size, "truncated ar member")
        members[name] = data
        if size % 2:
            stream.read(1)
    require(members.get("debian-binary") == b"2.0\n", "unsupported Debian package format")
    return members


def inspect_deb(artifact: Path, target: str, version: str, destination: Path) -> None:
    members = deb_members(artifact)
    control = next(data for name, data in members.items() if name.startswith("control.tar."))
    with tarfile.open(fileobj=io.BytesIO(control)) as archive:
        names = {name.removeprefix("./") for name in archive.getnames()}
        require(not names.intersection({"preinst", "postinst", "prerm", "postrm"}),
                "install hooks require explicit review")
        entry = next(member for member in archive.getmembers() if member.name.removeprefix("./") == "control")
        text = archive.extractfile(entry).read().decode()
        arch = "amd64" if target.startswith("x86_64") else "arm64"
        for field in ["Package: razers", f"Version: {version}", f"Architecture: {arch}"]:
            require(field in text.splitlines(), f"incorrect Debian field: {field}")
    data = next(data for name, data in members.items() if name.startswith("data.tar."))
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for member in archive.getmembers():
            path = member.name.removeprefix("./").rstrip("/")
            require(path == "usr" or path.startswith("usr/"), f"unexpected installed path: {path}")
        archive.extractall(destination, filter="data")
    verify_linux_tree(destination)


def verify_linux_tree(root: Path) -> None:
    for name in BINARY_NAMES:
        require(os.access(root / "usr/bin" / name, os.X_OK), f"missing executable: {name}")
    require((root / f"usr/share/applications/{APP_ID}.desktop").read_bytes()
            == (ROOT / "tools/packaging/razers.desktop").read_bytes(), "desktop entry mismatch")
    require(bool(list((root / "usr/share/icons").rglob("razers.png"))), "missing Linux app icon")
    verify_notices(root / "usr/lib/razers/notices")


def inspect_arch(artifact: Path, target: str, version: str, destination: Path) -> None:
    entries = captured("tar", "--zstd", "-tf", artifact).splitlines()
    for name in entries:
        name = name.removeprefix("./").rstrip("/")
        require(name in {"usr", ".PKGINFO", ".BUILDINFO", ".MTREE"} or name.startswith("usr/"),
                f"unexpected Arch package path: {name}")
        require(".." not in Path(name).parts, "unsafe Arch package path")
    info = captured("tar", "--zstd", "-xOf", artifact, ".PKGINFO")
    for field in ["pkgname = razers", f"pkgver = {version}-1", f"arch = {target.split('-', 1)[0]}"]:
        require(field in info.splitlines(), f"incorrect Arch package field: {field}")
    destination.mkdir()
    run("tar", "--zstd", "-xf", artifact, "-C", destination)
    verify_linux_tree(destination)


@contextmanager
def mounted_dmg(artifact: Path):
    with tempfile.TemporaryDirectory(prefix="razers-dmg-") as directory:
        mount = Path(directory) / "mount"
        run("hdiutil", "attach", "-readonly", "-nobrowse", "-mountpoint", mount, artifact)
        try:
            yield mount
        finally:
            run("hdiutil", "detach", mount)


def inspect_app(bundle: Path, version: str) -> None:
    with (bundle / "Contents/Info.plist").open("rb") as stream:
        info = plistlib.load(stream)
    require(info["CFBundleIdentifier"] == APP_ID, "macOS bundle identifier mismatch")
    require(info["CFBundleShortVersionString"] == version, "macOS bundle version mismatch")
    require(info["CFBundleExecutable"] == "razers", "macOS entry point mismatch")
    require(bool(list((bundle / "Contents/Resources").glob("*.icns"))), "missing macOS app icon")
    verify_notices(bundle / "Contents/Resources/notices")
    for name in BINARY_NAMES:
        require(os.access(bundle / "Contents/MacOS" / name, os.X_OK), f"missing bundle executable: {name}")
    run("codesign", "--verify", "--deep", "--strict", bundle)


def inspect_all(target: str, output: Path) -> None:
    version = workspace_version()
    with tempfile.TemporaryDirectory(prefix="razers-inspect-") as directory:
        work = Path(directory)
        for extension in TARGETS[target]:
            artifact = output / artifact_name(version, target, extension)
            verify_checksum(artifact)
            if extension == "deb":
                inspect_deb(artifact, target, version, work / "deb")
                smoke_binaries(work / "deb/usr/bin", version)
            elif extension == "pkg.tar.zst":
                inspect_arch(artifact, target, version, work / "arch")
                smoke_binaries(work / "arch/usr/bin", version)
                for path in (work / "deb/usr").rglob("*"):
                    if path.is_file():
                        other = work / "arch" / path.relative_to(work / "deb")
                        require(other.read_bytes() == path.read_bytes(), "Debian/Arch payload differs")
            elif extension == "dmg":
                with mounted_dmg(artifact) as mount:
                    require((mount / "Applications").is_symlink(), "DMG has no Applications shortcut")
                    inspect_app(mount / "RazeRS.app", version)
                    smoke_binaries(mount / "RazeRS.app/Contents/MacOS", version)
            else:
                require(artifact.read_bytes().startswith(b"MZ"), "invalid Windows installer")
    print(f"Payload inspection passed: {target}", flush=True)


def require_hosted_runner() -> None:
    require(os.environ.get("GITHUB_ACTIONS") == "true"
            and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted",
            "installation lifecycle tests are restricted to disposable GitHub-hosted runners")


@contextmanager
def user_settings_marker():
    if sys.platform == "win32":
        directory = Path(os.environ["APPDATA"]) / APP_ID / "data"
    elif sys.platform == "darwin":
        directory = Path.home() / "Library/Application Support" / APP_ID
    else:
        directory = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share")) / APP_ID
    directory.mkdir(parents=True, exist_ok=True)
    marker = directory / "app.ron"
    # Never overwrite settings, even if invoked on a contaminated runner.
    with marker.open("x", encoding="utf-8") as output:
        output.write('{"language": "zh-CN"}\n')
    expected = marker.read_bytes()
    try:
        yield lambda: require(marker.read_bytes() == expected, "installer changed user preferences")
    finally:
        marker.unlink(missing_ok=True)


def windows_registry() -> dict[str, str] | None:
    import winreg
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,
                            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\RazeRS",
                            0, winreg.KEY_READ | winreg.KEY_WOW64_64KEY) as key:
            return {name: winreg.QueryValueEx(key, name)[0]
                    for name in ["DisplayVersion", "InstallLocation", "UninstallString"]}
    except FileNotFoundError:
        return None


def run_nsis(executable: Path, final_argument: str) -> None:
    # NSIS requires /D= and _?= at the end, WITHOUT surrounding quotes even when
    # the directory contains spaces. subprocess's normal list quoting is wrong
    # for these two special parameters. No command shell is involved.
    require(not any(character in final_argument for character in '\r\n"'), "invalid NSIS directory")
    command = subprocess.list2cmdline([str(executable), "/S"]) + " " + final_argument
    subprocess.run(command, check=True, timeout=180)


def lifecycle_windows(old: Path, current: Path, work: Path, check_settings) -> None:
    require(windows_registry() is None, "RazeRS is already installed on this runner")
    destination = work / "Installed RazeRS"
    for installer, version in [(old, "0.0.0"), (current, workspace_version())]:
        run_nsis(installer, f"/D={destination}")
        registration = windows_registry()
        require(registration is not None and registration["DisplayVersion"] == version,
                "Windows installation/upgrade registration is incorrect")
        require(Path(registration["InstallLocation"].strip('"')) == destination,
                "Windows installer used the wrong directory")
        verify_notices(destination / "notices")
        smoke_binaries(destination, workspace_version())
        check_settings()
    # Hosted runners have VC++ installed already. Inspect imports as well so a
    # missing runtime cannot be masked by the runner's developer environment.
    inspector = shutil.which("llvm-readobj") or r"C:\Program Files\LLVM\bin\llvm-readobj.exe"
    for name in BINARY_NAMES:
        imports = captured(inspector, "--coff-imports", destination / f"{name}.exe").lower()
        require("vcruntime" not in imports and "msvcp" not in imports,
                "Windows binary depends on a non-bundled Visual C++ runtime")
    # The Start Menu shortcut must resolve to the GUI, with its real icon.
    shortcut = json.loads(captured("powershell", "-NoProfile", "-Command",
        "$w = New-Object -ComObject WScript.Shell; "
        "$links = Get-ChildItem ([Environment]::GetFolderPath('StartMenu')) -Recurse -Filter RazeRS.lnk; "
        "if (!$links) { throw 'Missing Start Menu shortcut' }; "
        "$s = $w.CreateShortcut($links[0].FullName); "
        "$v = (Get-Item $env:RAZERS_TEST_EXE).VersionInfo; "
        "@{TargetPath=$s.TargetPath; ProductName=$v.ProductName; ProductVersion=$v.ProductVersion} | ConvertTo-Json -Compress",
        env=dict(os.environ, RAZERS_TEST_EXE=str(destination / "razers.exe"))))
    # WScript expands an 8.3 path such as RUNNER~1. Compare file identity, not
    # different spellings of the same installed executable.
    require(Path(shortcut["TargetPath"]).samefile(destination / "razers.exe"), "wrong Start Menu target")
    require(shortcut["ProductName"] == "RazeRS" and shortcut["ProductVersion"] == workspace_version(),
            "Windows executable resource version mismatch")
    foreign = destination / "unrelated-file.txt"
    foreign.write_text("keep me", encoding="utf-8")
    # _?= runs synchronously instead of launching a temporary uninstaller child.
    uninstaller = work / "remove-razers.exe"
    shutil.copyfile(destination / "uninstall.exe", uninstaller)
    run_nsis(uninstaller, f"_?={destination}")
    require(windows_registry() is None, "Windows uninstall registration remains")
    for name in BINARY_NAMES:
        require(not (destination / f"{name}.exe").exists(), f"uninstall left {name}")
    require(foreign.read_text(encoding="utf-8") == "keep me", "uninstaller removed an unrelated file")
    require(not (destination / "notices").exists(), "Windows uninstall left package resources")
    run("powershell", "-NoProfile", "-Command",
        "$links = Get-ChildItem ([Environment]::GetFolderPath('StartMenu')) -Recurse -Filter RazeRS.lnk; "
        "if ($links) { throw 'Uninstall left Start Menu shortcut' }; "
        "$desktop = Join-Path ([Environment]::GetFolderPath('Desktop')) 'RazeRS.lnk'; "
        "if (Test-Path $desktop) { throw 'Uninstall left desktop shortcut' }")
    check_settings()


def lifecycle_linux(old: dict, current: dict, work: Path, check_settings) -> None:
    require(subprocess.run(["dpkg-query", "-W", "-f=${Status}", "razers"],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode != 0,
            "RazeRS already has a dpkg record on this runner")
    for package, version in [(old["deb"], "0.0.0"), (current["deb"], workspace_version())]:
        run("sudo", "apt-get", "install", "--yes", package)
        require(captured("dpkg-query", "-W", "-f=${Version}", "razers") == version,
                "Debian installed version mismatch")
        verify_linux_tree(Path("/"))
        smoke_binaries(Path("/usr/bin"), workspace_version())
        check_settings()
    run("sudo", "apt-get", "remove", "--yes", "razers")
    for name in BINARY_NAMES:
        require(not (Path("/usr/bin") / name).exists(), f"Debian uninstall left {name}")
    check_settings()

    # Exercise pacman's real ownership database in an isolated root. --nodeps is
    # intentional: Ubuntu is not an Arch root. An additional Arch container test
    # below validates actual runtime dependencies on x86_64.
    pacman_root = work / "pacman-root"
    database = pacman_root / "var/lib/pacman"
    database.mkdir(parents=True)
    config = work / "pacman.conf"
    config.write_text("[options]\nSigLevel = Never\n", encoding="utf-8")
    base = ["sudo", "pacman", "--config", config, "--root", pacman_root,
            "--dbpath", database, "--logfile", work / "pacman.log", "--noconfirm"]
    try:
        for package, version in [(old["pkg.tar.zst"], "0.0.0"),
                                 (current["pkg.tar.zst"], workspace_version())]:
            run(*base, "--upgrade", "--nodeps", "--noscriptlet", package)
            require(captured(*base, "--query", "razers").strip() == f"razers {version}-1",
                    "Arch installed version mismatch")
            verify_linux_tree(pacman_root)
            smoke_binaries(pacman_root / "usr/bin", workspace_version())
            check_settings()
        run(*base, "--remove", "razers")
        for name in BINARY_NAMES:
            require(not (pacman_root / "usr/bin" / name).exists(), f"Arch uninstall left {name}")
        check_settings()
    finally:
        # Allow TemporaryDirectory to remove only its own test data.
        run("sudo", "chown", "-R", f"{os.getuid()}:{os.getgid()}", pacman_root)


def lifecycle_macos(old: Path, current: Path, work: Path, check_settings) -> None:
    applications = work / "Applications"
    applications.mkdir()
    bundle = applications / "RazeRS.app"
    for installer, version in [(old, "0.0.0"), (current, workspace_version())]:
        with mounted_dmg(installer) as mount:
            # Finder's Replace operation is represented by replacing this owned,
            # temporary bundle. This does not simulate Gatekeeper or notarization.
            if bundle.exists():
                shutil.rmtree(bundle)
            run("ditto", mount / "RazeRS.app", bundle)
        inspect_app(bundle, version)
        smoke_binaries(bundle / "Contents/MacOS", workspace_version())
        check_settings()
    shutil.rmtree(bundle)
    require(not bundle.exists(), "macOS application removal failed")
    check_settings()


def lifecycle(target: str, output: Path, packager: Path) -> None:
    require_hosted_runner()
    with tempfile.TemporaryDirectory(prefix="razers-lifecycle-") as directory:
        work = Path(directory)
        # Synthetic old metadata around today's binaries tests installer upgrade
        # mechanics; it is not a claim of compatibility with a historical binary.
        build_installers(target, "0.0.0", work / "old", packager)
        old = {ext: work / "old" / artifact_name("0.0.0", target, ext) for ext in TARGETS[target]}
        current = {ext: output / artifact_name(workspace_version(), target, ext) for ext in TARGETS[target]}
        with user_settings_marker() as check_settings:
            if "windows" in target:
                lifecycle_windows(old["exe"], current["exe"], work, check_settings)
            elif "apple" in target:
                lifecycle_macos(old["dmg"], current["dmg"], work, check_settings)
            else:
                lifecycle_linux(old, current, work, check_settings)
    print(f"Installation, upgrade, removal and settings preservation passed: {target}", flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--lifecycle", action="store_true")
    parser.add_argument("--packager", type=Path)
    args = parser.parse_args()
    if args.lifecycle:
        require_hosted_runner()
        if args.packager is None:
            parser.error("--lifecycle requires --packager")
    inspect_all(args.target, ROOT / "dist")
    if args.lifecycle:
        lifecycle(args.target, ROOT / "dist", args.packager)


if __name__ == "__main__":
    main()
