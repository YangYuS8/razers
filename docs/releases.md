# Releases and dependency maintenance

RazeRS uses Conventional Commits and GitHub Actions for repository releases. The
workspace version in `Cargo.toml` is the single package version; individual crates
inherit it.

## Release flow

The `Release` workflow runs after every push to `main`. Release Please keeps one
`release-please--*` pull request current, updating the workspace version,
`Cargo.lock`, and this repository's `CHANGELOG.md`. It derives the next semantic version from
commit messages:

- `feat:` changes advance the minor version while RazeRS is pre-1.0;
- `fix:` and `perf:` changes advance the patch version;
- a Conventional Commit breaking-change marker advances the breaking version;
- documentation and maintenance-only commits remain in the log but do not create
  a release by themselves.

Merging the generated Release PR is the maintainer-owned release gate; users do not
need to choose release timing. The same workflow
then creates a `vX.Y.Z` tag and prerelease and builds for Linux x86-64 and ARM64,
Windows x86-64, and macOS x86-64 and ARM64. Each deterministic archive contains the
`razers` desktop application, its sibling `razers-agent`, the `razersctl` developer
CLI, and a SHA-256 file. The workspace is intentionally git-only and is not published
to crates.io during the pre-alpha phase.

Archives also include English/Chinese READMEs and the bundled font's OFL license
and provenance notice. Translation catalogs and the Chinese font are embedded, so
the installed application needs no translation downloads.

## Documentation publishing

The `Documentation` workflow builds both mdBooks and workspace library rustdoc,
checks matching chapter lists and generated local links, and rejects rustdoc
warnings on every PR. Pushes to `main` deploy the same output to GitHub Pages using
OIDC and the `github-pages` environment. Pull requests never receive Pages write
permission. This is the latest development documentation, not a version archive;
`build-info.json` records the source commit, workspace version, and mdBook version.
mdBook is pinned in `tools/docs-requirements.txt`.

If an artifact runner has a transient failure, rerun the failed jobs. The workflow
can also be dispatched with an existing `vX.Y.Z` tag to rebuild and replace that
release's assets without changing the version or changelog.

## Dependency updates

Dependabot checks Cargo dependencies and GitHub Actions weekly. Updates are grouped
to reduce notification noise. Cargo patch updates and GitHub Actions patch/minor
updates are set to auto-merge only after required CI passes. Cargo minor/major and
GitHub Actions major updates stay open for review.

Workflow actions are pinned to immutable commit SHAs. Dependabot updates those pins
and their human-readable version comments together.
