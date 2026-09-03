# RazeRS

[![CI](https://github.com/YangYuS8/razers/actions/workflows/ci.yml/badge.svg)](https://github.com/YangYuS8/razers/actions/workflows/ci.yml)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)

RazeRS is an experimental, cross-platform, user-space foundation for controlling
Razer peripherals. The project separates byte transport, vendor protocols, device
capabilities, and user-facing applications so that hardware support can be added
and verified without building a device-specific UI for every product.

> [!WARNING]
> RazeRS is pre-alpha software. It does not send commands to real hardware yet.
> The current source tree contains protocol codecs, replayable transport tests, a
> versioned device registry, pinned OpenRazer, OpenRGB, and iRazer evidence catalogs,
> privacy-preserving HID enumeration, an initial read-only desktop application, and
> developer CLI tools.

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
- Reuse upstream hardware results; investigate disagreements instead of requiring
  maintainers to repurchase every device.
- Safe-by-default tooling: unknown devices receive read-only probes only.
- A user-first application with no ads, required account, or default telemetry.

## Workspace

| Crate | Responsibility |
| --- | --- |
| `razers-types` | Shared identifiers, support states, and command risk levels |
| `razers-app` | Cross-platform, user-facing desktop application |
| `razers-protocol-core` | Safe 90-byte report encoding, decoding, and CRC validation |
| `razers-protocol-razer90` | Timed request-response exchange, validation, and explicit busy policy |
| `razers-transport` | OS-independent report I/O trait and deterministic replay transport |
| `razers-transport-hidapi` | Cross-platform, descriptor-only HID enumeration |
| `razers-device-registry` | TOML device schema, loading, and validation |
| `razers-cli` | Registry and packet inspection developer commands |

The architectural boundaries and roadmap are documented in
[`docs/architecture.md`](docs/architecture.md).
Release automation and dependency policy are documented in
[`docs/releases.md`](docs/releases.md).

## Quick start

The workspace requires Rust 1.85 or newer. Linux builds also need `pkg-config`
and the libudev development files (`libudev-dev` on Debian/Ubuntu; provided by
`systemd` on Arch Linux).

```bash
cargo test --workspace
cargo run -p razers-app
cargo run -p razers-cli -- registry validate devices
cargo run -p razers-cli -- upstream validate
cargo run -p razers-cli -- upstream stats
cargo run -p razers-cli -- upstream shortlist
cargo run -p razers-cli -- registry list devices
cargo run -p razers-cli -- devices devices
```

Inspect a registry entry:

```bash
cargo run -p razers-cli -- registry show razer.basilisk-v3 devices
```

Look up source-derived facts for any imported USB identity:

```bash
cargo run -p razers-cli -- upstream lookup 1532:0099
cargo run -p razers-cli -- upstream assess 1532:0099
```

`upstream shortlist` lists identities corroborated by multiple catalogs without a
material recorded conflict. `upstream conflicts` lists identities whose device
kind, matrix dimensions, or protocol parameters need targeted research. These are
triage results, not RazeRS support or hardware-verification claims.

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
- [x] Policy-driven 90-byte exchange with status, echo, and optional transaction checks
- [x] Device registry schema v1 and validation
- [x] Developer CLI for registry and packet inspection
- [x] CI, safety policy, contribution guide, and source provenance
- [x] Cross-platform HID enumeration without opening devices
- [x] Descriptor-only HID collection classification that never authorizes writes
- [x] Reproducible import of 267 OpenRazer, 196 OpenRGB, and 192 iRazer records
- [x] Cross-source comparison without silently resolving conflicting facts
- [x] Field-level evidence assessment, candidate shortlist, and conflict queue
- [x] Read-only desktop device overview with explicit privacy and support states
- [ ] Safe, read-only identification of the first physical device
- [ ] Agent and versioned local IPC
- [ ] Capability-driven desktop UI

## Source provenance and licensing

Protocol facts and device metadata are traced to pinned upstream sources. The
generated OpenRazer, OpenRGB, and iRazer catalogs are evidence-only: an entry does not claim
that RazeRS can control or has tested that device.
See [`docs/provenance.md`](docs/provenance.md) and the `evidence` entries in each
device manifest. The criteria for turning that evidence into experimental support
are documented in [`docs/evidence-policy.md`](docs/evidence-policy.md). Code in this
repository is licensed under GPL-2.0-or-later; see [`LICENSE`](LICENSE).

The application experience is governed by
[`docs/product-principles.md`](docs/product-principles.md), including the permanent
no-advertising, no-required-account, and privacy-by-default commitments.

Please read [`docs/safety.md`](docs/safety.md) before experimenting with hardware,
and [`CONTRIBUTING.md`](CONTRIBUTING.md) before submitting device support.
