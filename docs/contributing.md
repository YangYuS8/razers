# Contributing

Read the [repository contribution guide](https://github.com/YangYuS8/razers/blob/main/CONTRIBUTING.md)
and [security policy](https://github.com/YangYuS8/razers/blob/main/SECURITY.md).

Keep Transport, Protocol, Capability, Agent and UI responsibilities separate.
Submit narrowly scoped Conventional Commits. New support needs a reviewed manifest,
pinned evidence, replay tests, failure handling and honest user-visible status—not
necessarily another maintainer-owned copy of the hardware.

```bash
cargo install mdbook --version 0.5.4 --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.85.0 check --workspace --all-features --locked
python tools/build_docs.py
```

Use the mdBook version pinned in `tools/docs-tools.toml` (or provide that
binary through `MDBOOK`). The build generates `target/site/en`, `zh-CN` and `api`
without fetching translations. Preview with
`python -m http.server 8000 --directory target/site`, then open
`http://localhost:8000/`. Hosted 404 pages use the `/razers/` project mount.
The API reference documents workspace libraries; executable usage belongs in the
handbook. Rustdoc's own navigation remains upstream English, with bilingual
project API comments and a bilingual entry page.

Update English and Chinese messages/chapters together. Follow the
[localization guide](localization.md), [safety policy](safety.md) and
[evidence policy](evidence-policy.md). Do not upload private identifiers or live
input traces. Security vulnerabilities should be reported privately using GitHub's
repository security reporting facility rather than a public issue.
