# RazeRS

[![CI](https://github.com/YangYuS8/razers/actions/workflows/ci.yml/badge.svg)](https://github.com/YangYuS8/razers/actions/workflows/ci.yml)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)

RazeRS is an experimental, cross-platform, user-space foundation for controlling
Razer peripherals. The project separates byte transport, vendor protocols, device
capabilities, and user-facing applications so that hardware support can be added
and verified without building a device-specific UI for every product.

> [!WARNING]
> RazeRS is pre-alpha software. It does not send commands to real hardware yet.
> The current release contains protocol codecs, replayable transport tests, a
> versioned device registry, privacy-preserving HID enumeration, and developer
> CLI tools.

RazeRS is an independent community project. It is not affiliated with, endorsed
by, or sponsored by Razer Inc. Razer and related product names are trademarks of
their respective owners.

## Design goals

- User-space first; standard operating-system HID drivers continue to handle input.
- One shared Rust core for Linux, Windows, and macOS.
- Capability-driven devices instead of hard-coded product pages.
- Product, connection, logical device, and capability are distinct concepts.
- One serialized worker per physical connection when live I/O is introduced.
- Evidence-backed support levels with platform and firmware-specific verification.
- Safe-by-default tooling: unknown devices receive read-only probes only.

## Workspace

| Crate | Responsibility |
| --- | --- |
| `razers-types` | Shared identifiers, support states, and command risk levels |
| `razers-protocol-core` | Safe 90-byte report encoding, decoding, and CRC validation |
| `razers-transport` | OS-independent report I/O trait and deterministic replay transport |
| `razers-transport-hidapi` | Cross-platform, descriptor-only HID enumeration |
| `razers-device-registry` | TOML device schema, loading, and validation |
| `razers-cli` | Registry and packet inspection developer commands |

The architectural boundaries and roadmap are documented in
[`docs/architecture.md`](docs/architecture.md).

## Quick start

The workspace requires Rust 1.85 or newer. Linux builds also need `pkg-config`
and the libudev development files (`libudev-dev` on Debian/Ubuntu; provided by
`systemd` on Arch Linux).

```bash
cargo test --workspace
cargo run -p razers-cli -- registry validate devices
cargo run -p razers-cli -- registry list devices
cargo run -p razers-cli -- devices devices
```

Inspect a registry entry:

```bash
cargo run -p razers-cli -- registry show razer.basilisk-v3 devices
```

Encode and decode a protocol packet without touching hardware:

```bash
packet_hex=$(cargo run --quiet -p razers-cli -- report encode 0x00 0x81 0000)
cargo run -p razers-cli -- report decode "$packet_hex"
```

Run `cargo run -p razers-cli -- help` for the complete command list.

## Current status

Milestone 0 is complete, and descriptor-only enumeration begins Milestone 1:

- [x] Rust workspace and architectural boundaries
- [x] Explicit 90-byte report codec and checksum tests
- [x] Replay transport for hardware-free tests
- [x] Device registry schema v1 and validation
- [x] Developer CLI for registry and packet inspection
- [x] CI, safety policy, contribution guide, and source provenance
- [x] Cross-platform HID enumeration without opening devices
- [ ] Safe, read-only identification of the first physical device
- [ ] Agent and versioned local IPC
- [ ] Capability-driven desktop UI

## Source provenance and licensing

Protocol facts and initial device metadata are traced to pinned upstream sources.
See [`docs/provenance.md`](docs/provenance.md) and the `evidence` entries in each
device manifest. Code in this repository is licensed under
GPL-2.0-or-later; see [`LICENSE`](LICENSE).

Please read [`docs/safety.md`](docs/safety.md) before experimenting with hardware,
and [`CONTRIBUTING.md`](CONTRIBUTING.md) before submitting device support.
