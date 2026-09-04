## Summary

Describe the boundary, protocol fact, device entry, or behavior changed.

## Evidence and verification

- Upstream repository and pinned commit:
- Hardware/platform/firmware tested (optional; not required for upstream evidence or documentation):
- Capabilities tested:

## Checklist

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo run -p razers-cli -- registry validate devices`
- [ ] New Rust files contain `SPDX-License-Identifier: GPL-2.0-or-later`
- [ ] Device identifiers, personal paths, tokens, and unrelated HID data are redacted
- [ ] Persistent or experimental writes include a safety and verification plan
- [ ] User-facing changes and translation reminders have been reviewed in both languages
- [ ] Documentation changes pass `python3 tools/build_docs.py` (links and anchors included)
- [ ] Changes to site behavior pass `pnpm --dir docs run test:site`
