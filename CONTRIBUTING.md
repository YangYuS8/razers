# Contributing to RazeRS

Thank you for helping build safe, verifiable support for Razer hardware.

## Development setup

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p razers-cli -- registry validate devices
cargo run -p razers-cli -- upstream validate
cargo run -p razers-cli -- upstream conflicts
```

All Rust source files should carry `SPDX-License-Identifier: GPL-2.0-or-later`.
Document the origin of protocol facts and device metadata using a pinned upstream
commit. Do not copy code from an incompatible license.
Use Conventional Commit subjects so the automated Release PR can derive versions
and changelog sections; see [`docs/releases.md`](docs/releases.md).

## Upstream data import

The catalogs under `data/upstream` are generated from the exact source commits
recorded in their importers. The OpenRazer catalog preserves USB identities, class
symbols, advertised methods, matrix dimensions, DPI limits, polling rates, and
derived feature hints. The OpenRGB catalog adds lighting matrix families,
transaction IDs, dimensions, zone symbols, and keyboard layout symbols. The iRazer
catalog supplies a second cross-platform inventory with capability labels and its
own upstream support claims; those labels never become RazeRS support claims.

Regenerate them from checkouts at their pinned commits with:

```bash
python3 tools/import_openrazer.py \
  --source /path/to/openrazer \
  --output data/upstream/openrazer-devices.toml
python3 tools/import_openrgb.py \
  --source /path/to/OpenRGB \
  --output data/upstream/openrgb-devices.toml
python3 tools/import_irazer.py \
  --source /path/to/iRazer \
  --output data/upstream/irazer-devices.toml
```

Do not edit generated catalogs by hand. Updating a pinned commit requires a
review of source licensing, importer output, and semantic changes. Imported facts
may establish experimental RazeRS support after reconciliation, a curated manifest,
and local unit or replay tests; project-owned hardware is not a mandatory gate. See
[`docs/evidence-policy.md`](docs/evidence-policy.md).

## Device contributions

A hardware-backed device contribution should include the applicable items below,
but upstream research, manifest, driver, and replay-test contributions do not require
the contributor to own the device:

- exact marketing name and connection mode;
- USB VID/PID or Bluetooth identity;
- operating system and firmware version;
- relevant HID interface descriptors and report lengths;
- capabilities tested, including failed and untested capabilities;
- a redacted diagnostic bundle or golden trace when available;
- source repository, pinned commit, path, symbol, and license for imported facts.

Never include serial numbers, account identifiers, hostnames, personal paths, or
unredacted packet captures in an issue or commit.

## Support states

- `detected`: identity is known; no command is claimed to work.
- `experimental`: an evidence-backed implementation is available, with visible
  limitations and opt-in where required; a RazeRS hardware record is optional.
- `verified`: the listed platform and firmware combination passed RazeRS hardware tests.
- `regressed`: a previously verified capability currently fails.
- `unsupported`: the device or capability is confirmed not to work.

Verification belongs to a capability and a platform/firmware combination. A device
working on one computer does not establish cross-platform support.

## Hardware safety

Normal builds must not expose arbitrary raw writes, firmware operations, or fuzzing
through public IPC. Persistent commands need an explicit warning and a verification
plan. See [`docs/safety.md`](docs/safety.md).
