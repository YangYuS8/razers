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

Merging the generated Release PR is the only manual release gate. The same workflow
then creates a `vX.Y.Z` tag and prerelease, builds `razersctl` for Linux x86-64 and
ARM64, Windows x86-64, and macOS x86-64 and ARM64. Each deterministic archive also
contains the `razers` desktop application and a SHA-256 file. The workspace is
intentionally git-only and is not published to crates.io during the pre-alpha phase.

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
