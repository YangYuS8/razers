---
title: "Localization and documentation maintenance"
description: "Maintain the embedded translations and bilingual Starlight handbook with automated checks."
---

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

## Handbook workflow

English and Chinese chapters use matching filenames under
`docs/src/content/docs/en/` and `docs/src/content/docs/zh-CN/`. Every page has a
`title` and `description` in YAML frontmatter. Navigation is maintained once in
`docs/astro.config.mjs`, with translated group labels; there are no separate SUMMARY files.
Markdown and MDX pages are both checked. English source IDs omit the `en/` prefix,
so the site root serves the actual English handbook; Chinese lives at `/razers/zh-CN/`.
Use links such as `/razers/getting-started/` or `/razers/zh-CN/getting-started/`.
The language selector retains the chapter, but not translated heading anchors.

Pagefind indexes both languages during the build. Search runs locally in the
browser, including Chinese segmentation, without sending queries to a service.
Browser tests exercise the root entrance, Chinese and English queries, language
switching, and mobile navigation. Both API overviews use Starlight; their library
list comes from Cargo metadata, not a hand-maintained HTML index. Individual rustdoc
crate pages retain their own search and upstream English navigation with bilingual
project API comments.
Do not add remote fonts, analytics, runtime translation services, or tracking.

Every PR rejects missing language pages and warns when only one language file
changes. The same change check covers the two root READMEs. There is no manually
maintained translation-status ledger. This detects missing companion edits, not
semantic equivalence: reviewers must still check meaning. A warning can legitimately
remain for a language-only typo after reviewing the companion; no artificial edit,
status comment, or special label is required to silence it.

## Build and preview

Install the Node LTS major in `docs/.node-version` and the exact pnpm version in
`docs/package.json`. Rust and the workspace's platform build dependencies are also
needed for rustdoc. Run from the repository root:

```bash
pnpm --dir docs install --frozen-lockfile
pnpm --dir docs run check
python3 -m unittest discover -s tools/tests
python3 tools/build_docs.py
pnpm --dir docs exec playwright install chromium
pnpm --dir docs run test:site
pnpm --dir docs run preview --host 127.0.0.1
```

Open `http://localhost:4321/razers/`. For quick handbook-only editing, use
`pnpm --dir docs run dev`; the root and API overviews already work in development.
The full Python build adds rustdoc's crate pages. `target/site/` is generated; never
edit it directly.
If pnpm is not installed globally, `build_docs.py` can invoke its exact pinned
version through npm without changing your global tool configuration.

The full build uses the frozen dependency lockfile, builds Starlight and
`cargo doc --workspace --lib --all-features --no-deps --locked`, rejects rustdoc
warnings, and validates generated local links and heading anchors, including API
links. New library API examples should be executable rustdoc tests where practical.

## Documentation stack decision

In September 2026 we replaced mdBook with pnpm + Astro Starlight for a unified
bilingual navigation, search, and responsive reader experience. This adds a Node
toolchain and dependency lockfile, but removes our custom Chinese search and UI
translation adapters. Rustdoc remains the API generator and GitHub Pages remains
the host. The root directly renders the handbook using Starlight's root locale,
with no language chooser landing page or separate navigation design.

During this early project stage, clarity and low maintenance take priority over
historical documentation URLs. We removed the mdBook redirects, heading aliases,
and frozen compatibility fixtures; current links and user journeys remain tested.
The one `/en/` redirect exists because the released v0.3.0 desktop Help button uses
it. It is not a promise to preserve old chapter URLs or anchors. Add compatibility
only for a demonstrated dependency, using the smallest solution that serves it.

Only `main` or a manual run on `main` deploys through the `github-pages` environment
using the official Pages artifact and OIDC actions. PR validation has no deployment
credentials. The site follows development, not a release archive. See the
[maintenance policy](/razers/releases/) for dependency and publishing automation.
