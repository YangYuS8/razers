---
title: "Releases and dependency maintenance"
description: "Automated releases, documentation deployments, dependency updates, and the remaining review gates."
---

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
Windows x86-64, and macOS x86-64 and ARM64. Installer downloads come first: Windows
NSIS setup, macOS DMG containing RazeRS.app, Debian packages, and Arch binary packages.
Deterministic portable archives remain available. All formats include `razers`,
its sibling `razers-agent`, the `razersctl` developer CLI, and separate SHA-256 files.
The workspace is intentionally git-only and is not published to crates.io during
the pre-alpha phase.

Archives also include English/Chinese READMEs and the bundled font's OFL license
and provenance notice. Translation catalogs and the Chinese font are embedded, so
the installed application needs no translation downloads.

## Installer automation and boundaries

`installers.yml` is a read-only reusable workflow shared by PR checks and release
builds. `tools/package_installers.py` derives the version from Cargo, uses the locked
`cargo-packager` helper in `tools/packaging`, and produces actual Arch packages with
the distribution's `makepkg`. The helper has a separate lockfile and stable toolchain;
it is not shipped and does not raise the application's MSRV. Do not fork an installer
template just to duplicate behavior the packaging tool already provides.

Each platform verifies package metadata, resources and checksums, runs the packaged
Agent and desktop executable through hardware-free `agent.info`, and exercises
installation, upgrade, removal and preference preservation. Upgrade fixtures use
synthetic old package metadata with current binaries; they validate installer mechanics,
not historical settings migrations. macOS copies/replaces/removes a bundle in a temporary
Applications folder and verifies its signature; this does not test interactive Finder
or Gatekeeper behavior. Linux also checks the desktop entry and Debian/Arch payload
equivalence. Pacman ownership tests use isolated roots on Ubuntu for both architectures;
a clean Arch x86-64 container additionally checks runtime dependencies. No attached
mouse or other physical device is required. Destructive lifecycle tests refuse to run
outside disposable GitHub-hosted runners.

Windows builds link the C runtime statically and check their imports, so the runner's
preinstalled Visual C++ runtime cannot hide a missing end-user prerequisite. The installer
is per-user and bilingual. macOS bundles target macOS 11+ and use ad-hoc signing only.
Neither Windows publisher signing nor Apple Developer ID/notarization is configured;
those need separately authorized credentials, not a bypass of OS security. Installation
never adds a daemon, autostart entry, updater, account or device-permission change.
Upgrades and removal retain user settings. APT/Pacman repositories and automatic desktop
updates are not part of this milestone.

The aggregate `Installers` check must pass alongside existing CI before merging.
Release publication waits for **all five** platform jobs, verifies the complete expected
asset set, uploads it, generates bilingual installer-first links, then downloads and
checks the published files again. Only the final publication job has release-write
permission; PR package jobs never do. The release entry can exist while those packages
are still building, so an empty Assets section is not a completed release.

To build locally after compiling the three release binaries for your target:

```bash
cargo build --locked --manifest-path tools/packaging/Cargo.toml --target-dir target/packaging-tool
python tools/package_installers.py --target x86_64-unknown-linux-gnu --packager target/packaging-tool/debug/razers-packaging
python tools/check_installers.py --target x86_64-unknown-linux-gnu
```

Local inspection only extracts and checks packages; it never installs them. Linux
packaging also needs `dpkg-deb`, `makepkg`, `fakeroot`, `bsdtar`, and `zstd`. On Windows use the
helper's `.exe` name and set the native build's target-specific Rust flags to
`-C target-feature=+crt-static`; on macOS set `MACOSX_DEPLOYMENT_TARGET=11.0` before
compiling. CI holds the authoritative platform commands.

## Documentation publishing

The `Documentation` workflow builds bilingual Starlight and workspace library rustdoc,
checks matching pages, translation companion edits, generated local links and anchors,
and rejects rustdoc warnings on every PR. It also runs browser regressions for
the direct root entrance, language switching, Chinese/English search, API navigation,
and mobile navigation.
Pushes to `main` deploy the same output to GitHub Pages using
OIDC and the `github-pages` environment. Pull requests never receive Pages write
permission. This is the latest development documentation, not a version archive;
`build-info.json` records the source commit, workspace version, documentation
framework versions, and package manager. These tools are pinned in `docs/package.json`
and `docs/pnpm-lock.yaml`; Node follows the LTS major in `docs/.node-version`.

Weekly external-link checks run separately from PR validation so a temporary
third-party outage cannot prevent an unrelated fix from merging. Failed scheduled
runs appear in GitHub Actions; browser failures retain diagnostic artifacts for seven
days. Translation checks detect missing companion edits, not incorrect meaning.
See [localization maintenance](/razers/localization/) for the authoring workflow.

If an artifact runner has a transient failure, rerun the failed jobs. The workflow
can also be dispatched with an existing `vX.Y.Z` tag to rebuild and replace that
release's assets without changing the version or changelog.
That dispatch supports tags containing the installer tooling; older archive-only
tags need their original workflow, not a compatibility layer in the current packager.

## Dependency updates

Dependabot checks workspace Cargo dependencies, the isolated packaging helper,
documentation npm dependencies, and GitHub Actions weekly.
Updates are grouped to reduce notification noise. Cargo/npm patch updates and
GitHub Actions patch/minor updates are set to auto-merge only after required CI
passes. Cargo/npm minor/major and Actions major updates stay open for review.
Documentation-only changes do not require an application version bump or a release.

Frozen pnpm installs and explicit native-build permissions protect reproducibility.
Do not disable supply-chain checks to make a newly published dependency install;
allow its required release-age window and verify it before updating the pin.
Major toolchain changes, licenses, support claims, security, and conflicting hardware
evidence still require judgment rather than automatic approval.

Workflow actions are pinned to immutable commit SHAs. Dependabot updates those pins
and their human-readable version comments together.
