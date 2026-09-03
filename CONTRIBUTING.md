# Contributing to Razers

Thank you for helping build safe, verifiable support for Razer hardware.

## Development setup

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p razers-cli -- registry validate devices
```

All Rust source files should carry `SPDX-License-Identifier: GPL-2.0-or-later`.
Document the origin of protocol facts and device metadata using a pinned upstream
commit. Do not copy code from an incompatible license.

## Device contributions

A device contribution should include:

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
- `experimental`: an explicit opt-in capability is available but incompletely tested.
- `verified`: the listed platform and firmware combination passed hardware tests.
- `regressed`: a previously verified capability currently fails.
- `unsupported`: the device or capability is confirmed not to work.

Verification belongs to a capability and a platform/firmware combination. A device
working on one computer does not establish cross-platform support.

## Hardware safety

Normal builds must not expose arbitrary raw writes, firmware operations, or fuzzing
through public IPC. Persistent commands need an explicit warning and a verification
plan. See [`docs/safety.md`](docs/safety.md).
