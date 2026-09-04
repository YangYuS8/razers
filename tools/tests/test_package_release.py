# SPDX-License-Identifier: GPL-2.0-or-later
import hashlib
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from package_release import copy_payload, write_tar


class ReleasePackaging(unittest.TestCase):
    def test_bilingual_notices_are_shipped_deterministically(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binaries = []
            for name in ["razers", "razers-agent", "razersctl"]:
                binary = root / name
                binary.write_bytes(b"test binary")
                binaries.append(binary)
            staging = root / "payload"
            staging.mkdir()
            copy_payload(staging, binaries)
            first, second = root / "first.tar.gz", root / "second.tar.gz"
            write_tar(first, staging)
            write_tar(second, staging)
            self.assertEqual(hashlib.sha256(first.read_bytes()).digest(), hashlib.sha256(second.read_bytes()).digest())
            with tarfile.open(first) as archive:
                for name in ["README.md", "README.zh-CN.md", "FONT-LICENSE.txt", "FONT-NOTICE.md", "razers", "razers-agent", "razersctl"]:
                    self.assertIn(name, archive.getnames())
