# Source provenance

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

[`data/upstream/openrazer-devices.toml`](../data/upstream/openrazer-devices.toml)
contains 267 USB identities extracted from the seven concrete device modules under
`daemon/openrazer_daemon/hardware`. The deterministic importer preserves each source
path and class symbol along with OpenRazer's advertised methods, matrix dimensions,
DPI limits, polling-rate lists, and conservative feature hints.

The importer at [`tools/import_openrazer.py`](../tools/import_openrazer.py) accepts
only a Git checkout at the pinned commit above. The Rust catalog parser rejects
duplicate VID/PID pairs, malformed provenance, invalid dimensions, and invalid
numeric data. This makes upstream refreshes reviewable instead of silently consuming
the latest branch.

OpenRazer is licensed GPL-2.0-or-later. RazeRS is also GPL-2.0-or-later and retains
file-level SPDX identifiers. This project currently contains a new Rust implementation
of the documented wire format, not a line-by-line translation of an upstream driver.

## Evidence versus verification

An upstream implementation is evidence that a protocol fact or device identity is
plausible. It is not a RazeRS hardware verification. Device manifests remain marked
`detected` or `experimental` until a named capability passes the project's own tests
on a recorded operating-system and firmware combination.

The generated catalog therefore stays separate from curated `devices/*.toml`
manifests. It can name an attached device and guide implementation, but it cannot
select a protocol driver or authorize hardware writes.

## OpenRGB lighting catalog

Lighting metadata is imported from OpenRGB commit:

```text
7fed68ccf1a2413b9bd38a70e266b12cb2d59c26
```

[`data/upstream/openrgb-devices.toml`](../data/upstream/openrgb-devices.toml)
contains the 196 entries referenced by OpenRGB's Razer device table. The importer
preserves the matrix protocol family, transaction ID, matrix dimensions, zone
symbols, PID symbol, and optional keyboard layout symbol from
[`RazerDevices.h`](https://github.com/CalcProgrammer1/OpenRGB/blob/7fed68ccf1a2413b9bd38a70e266b12cb2d59c26/Controllers/RazerController/RazerDevices.h)
and
[`RazerDevices.cpp`](https://github.com/CalcProgrammer1/OpenRGB/blob/7fed68ccf1a2413b9bd38a70e266b12cb2d59c26/Controllers/RazerController/RazerDevices.cpp).
Those files are GPL-2.0-or-later.

The two catalogs overlap on 172 USB identities. They currently contain 72 naming
differences and 18 matrix-dimension differences. RazeRS reports these disagreements
instead of choosing one source automatically; a curated manifest must resolve them
with device-specific evidence.

## iRazer cross-platform catalog

The iRazer catalog is imported from commit:

```text
7cc856ddd26edd9523a12a540b6d95a4ea3a54c4
```

[`data/upstream/irazer-devices.toml`](../data/upstream/irazer-devices.toml)
contains all 192 entries in iRazer's
[`DeviceCatalog.swift`](https://github.com/hanley-tech/iRazer/blob/7cc856ddd26edd9523a12a540b6d95a4ea3a54c4/Sources/iRazer/DeviceCatalog.swift).
The deterministic importer preserves USB identities, category, capability labels,
matrix family, transaction ID, and the source project's support label. iRazer is
MIT-licensed.

iRazer and OpenRGB overlap on 189 identities with no matrix-family or transaction-ID
disagreements at these pinned commits. iRazer additionally records the Nommo V2 Pro,
Nommo V2, and Nommo V2 X. An iRazer `supported` label is displayed explicitly as an
upstream claim and remains evidence-only in RazeRS until independently verified.
