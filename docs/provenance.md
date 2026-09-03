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

OpenRazer is licensed GPL-2.0-or-later. RazeRS is also GPL-2.0-or-later and retains
file-level SPDX identifiers. This project currently contains a new Rust implementation
of the documented wire format, not a line-by-line translation of an upstream driver.

## Evidence versus verification

An upstream implementation is evidence that a protocol fact or device identity is
plausible. It is not a RazeRS hardware verification. Device manifests remain marked
`detected` or `experimental` until a named capability passes the project's own tests
on a recorded operating-system and firmware combination.
