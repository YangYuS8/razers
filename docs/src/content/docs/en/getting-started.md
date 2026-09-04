---
title: "Getting started"
description: "Download, verify, and start the read-only preview, or build RazeRS from source."
---

## Download and install

Use the **Install** links in [GitHub Releases](https://github.com/YangYuS8/razers/releases).
The prerelease label describes feature maturity, not the availability of installers.
Older releases may only have portable archives. The current app is still a read-only
preview; installing it does not enable DPI, lighting or button controls.

| System | Recommended format | Filename target |
| --- | --- | --- |
| Windows Intel/AMD 64-bit | `-setup.exe` | `x86_64-pc-windows-msvc` |
| macOS Apple Silicon | `.dmg` | `aarch64-apple-darwin` |
| macOS Intel | `.dmg` | `x86_64-apple-darwin` |
| Debian/Ubuntu Intel/AMD 64-bit | `.deb` | `x86_64-unknown-linux-gnu` |
| Debian/Ubuntu ARM64 | `.deb` | `aarch64-unknown-linux-gnu` |
| Arch Linux Intel/AMD 64-bit | `.pkg.tar.zst` | `x86_64-unknown-linux-gnu` |
| Arch Linux ARM64 | `.pkg.tar.zst` | `aarch64-unknown-linux-gnu` |

Every package includes the desktop app, its private Agent, the developer CLI,
English/Chinese resources, offline Chinese fonts, and license notices. There is no
advertising, account requirement, autostart entry, system service, or device-permission
change. Do not run the application as root.

### Windows

Run the `-setup.exe` installer and open **RazeRS** from the Start menu. It installs
for your current user without an administrator prompt. The wizard follows the system
language (English or Simplified Chinese). Close RazeRS before running a newer installer
to upgrade; uninstall through **Settings → Apps → Installed apps**. User preferences
are retained during upgrades and removal. The app and installer are not yet signed
with a publisher certificate; SmartScreen or organizational policies can block them.

### macOS

Open the DMG and drag **RazeRS.app** onto **Applications**, then eject the disk image.
Launch it from Applications. To upgrade, quit the app and replace that bundle with the
new one; to uninstall, move RazeRS.app to the Trash. Preferences are kept separately.
Builds target macOS 11 or later and the matching Intel/Apple Silicon architecture.
The app has an ad-hoc signature, **not** a Developer ID signature or Apple notarization.
Gatekeeper may block the download. After verifying its origin, follow Apple's
[instructions for opening a trusted downloaded app](https://support.apple.com/en-us/102445).
Managed devices may require administrator approval.

### Linux

Debian packages target Ubuntu 24.04 or a compatible newer userspace, such as Debian 13:
glibc 2.39+, libudev, and a working Wayland/X11 graphics session are required. Older
Ubuntu/Debian systems are not covered. From the download directory, substitute the
actual version and architecture:

```bash
sudo apt install ./razers-vX.Y.Z-x86_64-unknown-linux-gnu.deb
```

On Arch Linux, use its actual binary package, not the portable archive:

```bash
sudo pacman -U ./razers-vX.Y.Z-x86_64-unknown-linux-gnu.pkg.tar.zst
```

Launch **RazeRS** from your application menu. Run the same installation command with
the newer file to upgrade. Remove with `sudo apt remove razers` or `sudo pacman -R razers`;
your preferences remain. Package managers resolve runtime dependencies. Downloads
are not currently served through an APT/Pacman repository, so desktop updates are not
installed automatically. Keep your existing signature policy; these local packages
have SHA-256 checksums, not a distribution repository signature.

CI checks Linux binaries on both architectures and dependency installation in an
Arch Linux x86-64 container. ARM64 Pacman install/upgrade/removal is tested in an
isolated package root on Ubuntu ARM64, not on an Arch Linux ARM device. Fedora and
other distributions have no native installer in this set.

## Verify downloads

Download the matching `.sha256` file and verify from the download directory:

```bash
sha256sum --check razers-vX.Y.Z-x86_64-unknown-linux-gnu.deb.sha256
```

On macOS use `shasum -a 256 -c <checksum-file>`; on Windows compare
`Get-FileHash <download> -Algorithm SHA256` with the downloaded checksum.
A checksum detects changed bytes; it is not a publisher signature. Verify the GitHub
repository and release as well. Do not globally disable system security to launch RazeRS.

## Portable alternative

The `.zip` (Windows) and `.tar.gz` (Linux/macOS) assets remain available. Extract and
keep `razers`, `razers-agent`, and `razersctl` together (`.exe` on Windows); run `razers`.
Portable archives have no installer, application-menu registration, bundled updater,
or automatic runtime-dependency installation. They still use the user's normal
preferences directory; “portable” does not mean all settings stay beside the executable.

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
