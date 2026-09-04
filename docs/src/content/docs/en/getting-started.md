---
title: "Getting started"
description: "Download, verify, and start the read-only preview, or build RazeRS from source."
---

## Download

Choose an archive from [GitHub Releases](https://github.com/YangYuS8/razers/releases):

| System | Archive target |
| --- | --- |
| Linux Intel/AMD 64-bit | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| Windows Intel/AMD 64-bit | `x86_64-pc-windows-msvc` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

Extract the archive and keep `razers`, `razers-agent`, and `razersctl` together
(`.exe` on Windows). Run `razers` to open the desktop app. Do not run it as root.
Archives are portable binaries, not signed installers or macOS application bundles.
Operating-system trust prompts may appear; verify the download source and checksum.
Do not globally disable system security to launch the application.

Linux builds use Ubuntu 24.04 and require a compatible glibc, libudev and desktop
graphics environment. Chinese fonts are bundled; they do not need a separate install.

Verify the archive from the directory containing it:

```bash
sha256sum --check razers-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz.sha256
```

On macOS use `shasum -a 256 -c <checksum-file>`; on Windows compare
`Get-FileHash <archive> -Algorithm SHA256` with the downloaded checksum.

## Build from source

Rust 1.85 or newer is required. On Debian/Ubuntu install `pkg-config`, `libudev-dev`
and `libxkbcommon-dev`. On Arch these are supplied by `pkgconf`, `systemd` and
`libxkbcommon`. Native Windows/macOS builds require their usual platform SDK/linker.

```bash
git clone https://github.com/YangYuS8/razers.git
cd razers
cargo test --workspace --all-features --locked
cargo run -p razers-app -- --lang en
```

See [the application guide](/razers/application/) for language selection and current limits.
