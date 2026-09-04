# SPDX-License-Identifier: GPL-2.0-or-later
import json
import os
from pathlib import Path
import subprocess
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_installers import deb_members, inspect_deb, require_hosted_runner, run_nsis, verify_checksum
from package_installers import (
    APP_ID, TARGETS, arch_recipe, artifact_name, packager_config, validate_version,
)
from package_release import BINARY_NAMES, NOTICES, write_checksum
from update_release_notes import MARKER, with_downloads
from verify_release import expected_assets, verify_release


class InstallerConfiguration(unittest.TestCase):
    def test_every_target_uses_one_version_and_sibling_binaries(self):
        for target in TARGETS:
            with self.subTest(target=target):
                config = packager_config(target, "0.4.0", Path("output"))
                self.assertEqual(config["version"], "0.4.0")
                self.assertEqual(config["identifier"], APP_ID)
                self.assertEqual(config["productName"], "RazeRS")
                self.assertEqual([b["path"] for b in config["binaries"]], list(BINARY_NAMES))
                self.assertEqual(sum(b["main"] for b in config["binaries"]), 1)
                self.assertEqual({r["target"] for r in config["resources"]},
                                 {"notices/" + name for name in NOTICES.values()})
                self.assertNotIn("licenseFile", config, "do not turn notices into a click-through EULA")
                json.dumps(config)

    def test_windows_is_bilingual_per_user_and_preserves_appdata(self):
        config = packager_config("x86_64-pc-windows-msvc", "0.4.0", Path("out"))
        self.assertEqual(config["nsis"]["installMode"], "currentUser")
        self.assertEqual(config["nsis"]["languages"], ["English", "SimpChinese"])
        self.assertFalse(config["nsis"]["displayLanguageSelector"])
        self.assertFalse(config["windows"]["allowDowngrades"])
        self.assertNotIn("appdataPaths", config["nsis"])
        self.assertNotIn("preinstallSection", config["nsis"])
        self.assertNotIn("template", config["nsis"])

    def test_linux_uses_stable_package_and_desktop_identity(self):
        for target in TARGETS:
            if "linux" not in target:
                continue
            config = packager_config(target, "0.4.0", Path("out"))
            self.assertEqual(config["deb"]["packageName"], "razers")
            self.assertIn(f"usr/share/applications/{APP_ID}.desktop", config["deb"]["files"].values())
            self.assertFalse(config["linux"]["generateDesktopEntry"])
            recipe = arch_recipe("0.4.0", target)
            self.assertIn(f"arch=('{target.split('-', 1)[0]}')", recipe)
            self.assertIn("pkgver=0.4.0", recipe)
            self.assertIn('"$startdir/payload/usr"', recipe)
            self.assertNotIn("install=", recipe)

    def test_linux_declares_libraries_loaded_only_when_the_gui_starts(self):
        config = packager_config("x86_64-unknown-linux-gnu", "0.4.0", Path("out"))
        self.assertTrue({"libxkbcommon-x11-0", "libx11-xcb1", "libwayland-egl1"}
                        <= set(config["deb"]["depends"]))
        self.assertIn("'libxkbcommon-x11'", arch_recipe("0.4.0", "x86_64-unknown-linux-gnu"))

    def test_no_silent_version_coercion_or_path_injection(self):
        for version in ["v0.4.0", "0.4", "0.04.0", "0.4.0-beta.1", "0.4.0+build", "../0.4.0", "0.4.0\n"]:
            with self.subTest(version=version), self.assertRaises(ValueError):
                validate_version(version)
        with self.assertRaises(ValueError):
            packager_config("../../invalid", "0.4.0", Path("out"))
        with self.assertRaises(ValueError):
            artifact_name("0.4.0", "x86_64-apple-darwin", "exe")

    def test_macos_signing_claim_is_only_ad_hoc(self):
        config = packager_config("aarch64-apple-darwin", "0.4.0", Path("out"))
        self.assertEqual(config["macos"]["signingIdentity"], "-")
        self.assertNotIn("notarizationCredentials", config["macos"])

    def test_large_png_icons_have_retina_density_for_icns(self):
        config = packager_config("aarch64-apple-darwin", "0.4.0", Path("out"))
        for name in config["icons"]:
            path = Path(name)
            if path.suffix != ".png":
                continue
            width, height = struct.unpack(">II", path.read_bytes()[16:24])
            self.assertEqual(width, height)
            if width == 1024:
                self.assertTrue(path.stem.endswith("@2x"), "1024px ICNS requires Retina density")
            else:
                self.assertIn(width, [16, 32, 48, 64, 128, 256, 512])


class InstallerValidation(unittest.TestCase):
    def test_nsis_directory_parameter_stays_last_and_unquoted(self):
        for prefix in ["/D=", "_?="]:
            with patch("check_installers.subprocess.run") as execute:
                run_nsis(Path("C:/Downloaded Files/setup.exe"), prefix + "C:/Installed RazeRS")
                command = execute.call_args.args[0]
                self.assertTrue(command.endswith(prefix + "C:/Installed RazeRS"))
                self.assertNotIn("shell", execute.call_args.kwargs)

    def test_lifecycle_cannot_run_on_a_developer_machine(self):
        for env in [{}, {"GITHUB_ACTIONS": "true"},
                    {"GITHUB_ACTIONS": "true", "RUNNER_ENVIRONMENT": "self-hosted"}]:
            with patch.dict(os.environ, env, clear=True), self.assertRaises(RuntimeError):
                require_hosted_runner()
        with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "RUNNER_ENVIRONMENT": "github-hosted"}, clear=True):
            require_hosted_runner()

    def test_checksum_detects_modified_package_and_wrong_basename(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "razers.deb"
            artifact.write_bytes(b"package")
            checksum = write_checksum(artifact)
            verify_checksum(artifact)
            checksum.write_text(checksum.read_text().replace("razers.deb", "other.deb"))
            with self.assertRaises(RuntimeError):
                verify_checksum(artifact)
            write_checksum(artifact)
            artifact.write_bytes(b"tampered")
            with self.assertRaises(RuntimeError):
                verify_checksum(artifact)

    def test_all_installers_archives_and_checksums_are_required(self):
        assets = expected_assets("0.4.0")
        self.assertEqual(len(assets), 24)
        self.assertIn("razers-v0.4.0-x86_64-pc-windows-msvc-setup.exe", assets)
        self.assertIn("razers-v0.4.0-aarch64-unknown-linux-gnu.pkg.tar.zst.sha256", assets)
        self.assertFalse(any("0.0.0" in name for name in assets))
        with tempfile.TemporaryDirectory() as directory, self.assertRaisesRegex(RuntimeError, "incomplete release"):
            verify_release(Path(directory), "0.4.0")

    def test_release_links_are_bilingual_and_idempotent(self):
        first = with_downloads("## Changes\n\nA feature.", "0.4.0")
        self.assertEqual(with_downloads(first, "0.4.0"), first)
        self.assertEqual(first.count(MARKER), 2)
        self.assertEqual(first.count("## Changes"), 1)
        self.assertIn("安装向导", first)
        self.assertIn("not notarized", first)
        self.assertIn("aarch64-apple-darwin.dmg", first)
        with self.assertRaises(ValueError):
            with_downloads(MARKER + "broken", "0.4.0")

    def test_malformed_deb_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "broken.deb"
            for data in [b"bad", b"!<arch>\ntruncated"]:
                artifact.write_bytes(data)
                with self.assertRaises(RuntimeError):
                    deb_members(artifact)


@unittest.skipUnless(os.environ.get("RAZERS_TEST_PACKAGER"), "optional local packaging-tool integration")
class RealDebPackaging(unittest.TestCase):
    def test_mac_bundle_icons_encode_with_the_same_packager(self):
        with tempfile.TemporaryDirectory(prefix="razers-app-test-") as directory:
            work = Path(directory)
            for name in BINARY_NAMES:
                (work / name).write_bytes(b"fixture binary")
            config = packager_config("aarch64-apple-darwin", "0.4.0", work / "out")
            config["binariesDir"] = str(work)
            config["formats"] = ["app"]
            # This cross-platform fixture checks the real icon encoder and bundle
            # layout, not Mach-O validity or signing (covered by native CI).
            config["macos"].pop("signingIdentity")
            path = work / "packager.json"
            path.write_text(json.dumps(config), encoding="utf-8")
            subprocess.run([str(Path(os.environ["RAZERS_TEST_PACKAGER"]).resolve()), "--config", str(path)], check=True)
            bundle = work / "out/RazeRS.app/Contents"
            for name in BINARY_NAMES:
                self.assertTrue((bundle / "MacOS" / name).is_file())
            icon, = (bundle / "Resources").glob("*.icns")
            self.assertTrue(icon.read_bytes().startswith(b"icns"))

    def test_packager_accepts_configuration_and_produces_installable_payload(self):
        with tempfile.TemporaryDirectory(prefix="razers-deb-test-") as directory:
            work = Path(directory)
            binaries = work / "binaries"
            binaries.mkdir()
            for name in BINARY_NAMES:
                binary = binaries / name
                binary.write_bytes(b"#!/bin/sh\nexit 0\n")
                binary.chmod(0o755)
            target = "x86_64-unknown-linux-gnu"
            config = packager_config(target, "0.4.0", work / "out")
            config["binariesDir"] = str(binaries)
            path = work / "packager.json"
            path.write_text(json.dumps(config), encoding="utf-8")
            subprocess.run([str(Path(os.environ["RAZERS_TEST_PACKAGER"]).resolve()), "--config", str(path)], check=True)
            artifact, = (work / "out").glob("*.deb")
            inspect_deb(artifact, target, "0.4.0", work / "extracted")
