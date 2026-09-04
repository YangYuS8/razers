# Localization and documentation maintenance

`razers-i18n/locales/en.json` and `zh-CN.json` are embedded gettext-style catalogs:
English source messages are keys; values are display templates. Placeholders use
`{0}`, `{1}`, etc. Callers format numeric values before insertion. Arguments are
inserted once, so braces in a device name never become template syntax.

Add both translations in the same PR. Workspace tests require identical key sets and
placeholder multisets. Unknown keys fall back to the original English message; do
not use that fallback as a reason to omit a translation. Complete sentences are
translated rather than assembled from English word fragments. Count labels are
language-neutral constructions, avoiding English plural suffixes in Chinese.

The desktop persists `auto`, `en`, or `zh-CN` separately from the resolved locale.
Protocol fields, capability identifiers, source claims and diagnostic data are not
translated on the wire. The optional `evidence_source_count` v1 field supports numeric
evidence messages while preserving compatibility with older clients.

## Build the site

Install mdBook using the version in `tools/docs-requirements.txt`:

```bash
cargo install mdbook --version 0.5.4 --locked
python tools/build_docs.py
python -m http.server 8000 --directory target/site
```

Open `http://localhost:8000/`. The same build is used by CI: English and Chinese
mdBook chapters, then `cargo doc --workspace --lib --all-features --no-deps --locked` with
rustdoc warnings denied. A post-build checker verifies language chapter parity and
local HTML links, including API links. API symbols are stable; crate/module and
API prose can include both languages in one rustdoc page.

English chapters live directly under `docs/`; Chinese chapters under `docs/zh-CN/`.
Keep their SUMMARY paths identical. Do not add remote fonts, analytics or tracking.
The toolbar preserves the chapter when switching language and drops translated anchors.
English uses mdBook's built-in search. Chinese uses a locally generated substring
index so words inside unsegmented Chinese text are searchable. Both are entirely
client-side and do not send queries to third parties. Chinese navigation, themes
and keyboard help are localized during the build. Rustdoc's upstream navigation
remains English; project API comments are bilingual.

The Documentation workflow validates every PR without deployment credentials.
Only `main` or a manual run on `main` deploys to the `github-pages` environment using
the official Pages artifact and OIDC deployment actions. It publishes `/en/`,
`/zh-CN/` and `/api/` beneath the repository site path.
