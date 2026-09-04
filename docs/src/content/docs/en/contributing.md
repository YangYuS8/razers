---
title: "Contributing"
description: "Set up development, validate a change, and contribute code, translations, or upstream research."
---

Read the [repository contribution guide](https://github.com/YangYuS8/razers/blob/main/CONTRIBUTING.md)
and [security policy](https://github.com/YangYuS8/razers/blob/main/SECURITY.md).

Keep Transport, Protocol, Capability, Agent and UI responsibilities separate.
Submit narrowly scoped Conventional Commits. New support needs a reviewed manifest,
pinned evidence, replay tests, failure handling and honest user-visible status—not
necessarily another maintainer-owned copy of the hardware.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.85.0 check --workspace --all-features --locked
pnpm --dir docs install --frozen-lockfile
python3 tools/build_docs.py
pnpm --dir docs exec playwright install chromium
pnpm --dir docs run test:site
```

Use the Node LTS major in `docs/.node-version` and pnpm pinned in
`docs/package.json`. The build generates the English handbook at `target/site/`,
Chinese under `zh-CN/`, and rustdoc crate pages under `api/`, without fetching
translations. Preview with `pnpm --dir docs run preview`, then
open `http://localhost:4321/razers/`. Hosted 404 pages use the `/razers/` project mount.
The API reference documents workspace libraries; executable usage belongs in the
handbook. Rustdoc's own navigation remains upstream English, with bilingual
project API comments and Starlight API overviews in both languages.

Update English and Chinese messages/chapters together. Follow the
[localization guide](/razers/localization/), [safety policy](/razers/safety/) and
[evidence policy](/razers/evidence-policy/). Do not upload private identifiers or live
input traces. Security vulnerabilities should be reported privately using GitHub's
repository security reporting facility rather than a public issue.

You can [contribute upstream evidence without hardware](/razers/contribute-evidence/).
For questions, feature requests, and conduct expectations, see
[community and support](/razers/community/).
