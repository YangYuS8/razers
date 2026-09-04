---
title: "Source provenance"
description: "Pinned upstream sources, protocol differences, and the licensing of bundled assets."
---

RazeRS uses published upstream projects as evidence for protocol facts and device
metadata. Source evidence is pinned so that a future contributor can reproduce the
research even after upstream changes.

## OpenRazer baseline

Initial protocol and Basilisk V3 facts were checked against OpenRazer commit:

```text
6820f9da169d354bc7e6e93a0aa8683a6bb75792
```

Relevant upstream locations:

- [`driver/razercommon.h`](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razercommon.h)
  defines the 90-byte report layout, field sizes, status values, and big-endian
  `remaining_packets` field.
- [`driver/razercommon.c`](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razercommon.c)
  documents the XOR checksum over byte positions 2 through 87 inclusive.
- [`driver/razermouse_driver.h`](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razermouse_driver.h)
  identifies the Basilisk V3 as USB PID `0x0099` under Razer VID `0x1532`.
- [`daemon/openrazer_daemon/hardware/mouse.py`](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/daemon/openrazer_daemon/hardware/mouse.py)
  records its 26,000 DPI maximum, 1x11 lighting matrix, and upstream feature list.

### Generated evidence catalog

[`data/upstream/openrazer-devices.toml`](https://github.com/YangYuS8/razers/blob/main/data/upstream/openrazer-devices.toml)
contains 267 USB identities extracted from the seven concrete device modules under
`daemon/openrazer_daemon/hardware`. The deterministic importer preserves each source
path and class symbol along with OpenRazer's advertised methods, matrix dimensions,
DPI limits, polling-rate lists, and conservative feature hints.

The importer at [`tools/import_openrazer.py`](https://github.com/YangYuS8/razers/blob/main/tools/import_openrazer.py) accepts
only a Git checkout at the pinned commit above. The Rust catalog parser rejects
duplicate VID/PID pairs, malformed provenance, invalid dimensions, and invalid
numeric data. This makes upstream refreshes reviewable instead of silently consuming
the latest branch.

OpenRazer is licensed GPL-2.0-or-later. RazeRS is also GPL-2.0-or-later and retains
file-level SPDX identifiers. This project currently contains a new Rust implementation
of the documented wire format, not a line-by-line translation of an upstream driver.

### Classic request-response behavior

The validated exchange layer follows the status values and response matching fields
in OpenRazer's
[`razercommon.h`](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razercommon.h)
and the request loop in
[`razermouse_driver.c`](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razermouse_driver.c#L115-L177).
OpenRazer accepts a valid `BUSY` response because some devices complete the command
despite that status. The independent Windows project opsrzr, at commit
`f4e9eabca19f721cf1bcb6ee8097d0748367cfe7`, instead
[`retries BUSY responses`](https://github.com/atv57/opsrzr/blob/f4e9eabca19f721cf1bcb6ee8097d0748367cfe7/crates/razer-hid/src/transport.rs#L164-L211).

RazeRS preserves that disagreement as an explicit per-connection `BusyHandling`
policy. Accepting busy is the conservative default because resending a write may
repeat a persistent operation; retrying must be opted into for a device and command
known to be safe. All retry, short-read, status, command-echo, transaction-ID, and
packet-counter paths are tested against the in-memory replay transport.

The relevant opsrzr crate is GPL-2.0-only, so its implementation is not copied or
relicensed here. RazeRS's new implementation remains GPL-2.0-or-later; opsrzr is used
only to corroborate the existence of the alternative behavior.

## Evidence versus verification

An upstream implementation is reusable engineering evidence, including evidence of
real-world hardware behavior. It is not labeled as a RazeRS hardware verification,
but a separate project-owned device test is not required before implementing or
shipping an `experimental` capability.

The generated catalog stays separate from curated `devices/*.toml` manifests so
that imports cannot silently enable hardware operations. A reviewed manifest may
select a typed driver after source reconciliation and replay testing. RazeRS
verification records add platform and firmware-specific confidence rather than
acting as a prerequisite for all useful support. The acceptance and disagreement
rules are defined in [`evidence-policy.md`](/razers/en/evidence-policy/).

## OpenRGB lighting catalog

Lighting metadata is imported from OpenRGB commit:

```text
7fed68ccf1a2413b9bd38a70e266b12cb2d59c26
```

[`data/upstream/openrgb-devices.toml`](https://github.com/YangYuS8/razers/blob/main/data/upstream/openrgb-devices.toml)
contains the 196 entries referenced by OpenRGB's Razer device table. The importer
preserves the matrix protocol family, transaction ID, matrix dimensions, zone
symbols, PID symbol, and optional keyboard layout symbol from
[`RazerDevices.h`](https://github.com/CalcProgrammer1/OpenRGB/blob/7fed68ccf1a2413b9bd38a70e266b12cb2d59c26/Controllers/RazerController/RazerDevices.h)
and
[`RazerDevices.cpp`](https://github.com/CalcProgrammer1/OpenRGB/blob/7fed68ccf1a2413b9bd38a70e266b12cb2d59c26/Controllers/RazerController/RazerDevices.cpp).
Those files are GPL-2.0-or-later.

The two catalogs overlap on 172 USB identities. They currently contain 72 naming
differences and 18 matrix-dimension differences. RazeRS reports these disagreements
instead of choosing one source automatically. Resolution examines semantics,
revisions, source history, issues, tests, and additional implementations; purchasing
the device is not the default resolution mechanism.

## iRazer cross-platform catalog

The iRazer catalog is imported from commit:

```text
7cc856ddd26edd9523a12a540b6d95a4ea3a54c4
```

[`data/upstream/irazer-devices.toml`](https://github.com/YangYuS8/razers/blob/main/data/upstream/irazer-devices.toml)
contains all 192 entries in iRazer's
[`DeviceCatalog.swift`](https://github.com/hanley-tech/iRazer/blob/7cc856ddd26edd9523a12a540b6d95a4ea3a54c4/Sources/iRazer/DeviceCatalog.swift).
The deterministic importer preserves USB identities, category, capability labels,
matrix family, transaction ID, and the source project's support label. iRazer is
MIT-licensed.

iRazer and OpenRGB overlap on 189 identities with no matrix-family or transaction-ID
disagreements at these pinned commits. iRazer additionally records the Nommo V2 Pro,
Nommo V2, and Nommo V2 X. An iRazer `supported` label remains attributed to iRazer,
but it is a meaningful input to RazeRS experimental-support decisions rather than a
claim that must be discarded until repeated locally.

## Bundled Chinese font

Noto Sans SC is embedded, unmodified, from `notofonts/noto-cjk` commit
`f8d157532fbfaeda587e826d4cd5b21a49186f7c`, under SIL Open Font License 1.1.
It is not covered by the code's GPL license. The [font notice](https://github.com/YangYuS8/razers/blob/main/assets/fonts/README.md)
records the source path and SHA-256; release archives retain the original OFL.
