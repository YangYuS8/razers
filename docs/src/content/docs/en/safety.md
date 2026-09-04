---
title: "Hardware safety policy"
description: "Risk levels, transport restrictions, privacy, and the boundaries of safe hardware experimentation."
---

RazeRS is designed for reversible device configuration, but vendor commands can still
disconnect devices, corrupt persistent settings, or make firmware unusable. Safety is
an architectural requirement rather than a user-interface warning added later.

## Command risk classes

Every semantic command must eventually declare one of these classes:

| Risk | Meaning | Default release behavior |
| --- | --- | --- |
| `read-only` | Descriptor or state query | Allowed for narrowly matched, evidence-backed devices |
| `reversible` | Volatile DPI, lighting, or similar state | Experimental opt-in from reconciled evidence; verified capabilities may be default |
| `persistent` | Device or onboard storage write | Explicit confirmation and verification required |
| `experimental-write` | Incompletely understood vendor command | Developer build and allowlist only |
| `firmware` | Bootloader or firmware operation | Not part of the normal Agent or IPC |

Unknown devices enter safe probing. Safe probing may enumerate descriptors and run
reviewed information queries, but it must not send persistent writes, brute-force
commands, fuzz packet fields, or perform firmware operations.

Upstream hardware results may justify typed read-only and reversible experimental
operations when their provenance is pinned and local replay tests cover the packet
and failure behavior. Project-owned hardware testing is not required for every
device; it remains required before making stronger RazeRS-specific verification
claims.

## Raw packet tooling

Arbitrary packet writes must remain unavailable in normal release builds and public
IPC. If introduced for research, raw tools require all of the following:

- an explicit developer build;
- a device and firmware allowlist;
- command rate limiting and bounded retries;
- a visible risk warning;
- an audit log stored locally;
- no automatic upload of packets or identifiers.

Fuzzing belongs against pure codecs and replay transports in CI. Live-device fuzzing
is outside the supported workflow.

## Diagnostics and privacy

Before sharing diagnostic output, redact:

- serial numbers and stable Bluetooth identifiers;
- hostnames, usernames, and personal filesystem paths;
- account or cloud identifiers;
- access tokens and application secrets;
- unrelated HID traffic and raw input content.

Diagnostic upload must always be an explicit user action. The project must never
collect or transmit device data by default.

## Current milestone guarantee

The current Agent can enumerate HID descriptors but does not open devices or write
to real hardware. Device paths and serial-number values are omitted before data
crosses the IPC boundary. `ReplayTransport` remains the only report I/O
implementation supplied by the workspace.
