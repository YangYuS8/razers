## Summary

Describe the boundary, protocol fact, device entry, or behavior changed.

## Evidence and verification

- Upstream repository and pinned commit:
- Hardware/platform/firmware tested:
- Capabilities tested:

## Checklist

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo run -p razers-cli -- registry validate devices`
- [ ] New Rust files contain `SPDX-License-Identifier: GPL-2.0-or-later`
- [ ] Device identifiers, personal paths, tokens, and unrelated HID data are redacted
- [ ] Persistent or experimental writes include a safety and verification plan
