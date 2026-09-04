---
title: "Architecture"
description: "How Transport, Protocol, Capability, Agent, and UI responsibilities fit together."
---

RazeRS is a cross-platform, user-space device control platform. Operating-system
HID drivers remain responsible for ordinary keyboard, mouse, and audio input. RazeRS
handles vendor-specific configuration such as lighting, DPI, polling rate, battery,
power management, equalizers, and eventually input actions.

## Boundaries

```text
Desktop UI / CLI / SDK
          |
          | versioned local IPC
          v
        Agent
  device manager, profiles, state cache, diagnostics
          |
          | semantic device commands
          v
Connection actors (one serialized worker per physical link)
          |
          v
Protocol and capability drivers
          |
          | fixed reports and packets
          v
Transport backends (USB HID, hidraw, IOHID, Windows HID, BLE)
```

Three boundaries are deliberately kept independent:

1. **Transport** defines how bytes move.
2. **Protocol** defines what those bytes mean.
3. **Capability** defines what a user can do.

A transport must never expose methods such as `set_dpi` or `set_static_color`.
Those operations belong to capability drivers backed by protocol implementations.

## Device model

```text
Product
  +-- Connection
        +-- Logical device
              +-- Capability
```

A product can combine several links. A wireless mouse may expose wired USB, a
receiver, and a charging dock; a speaker may combine USB audio, vendor HID, and BLE
lighting. A single physical receiver can also carry multiple logical devices. The
registry and future Agent must preserve these distinctions.

User interfaces render capability descriptors rather than device-specific pages.
Adding a product should normally mean adding a manifest, choosing existing drivers,
and attaching evidence and tests.

Source-derived catalogs and curated manifests are separate layers:

```text
Pinned upstream source -> generated evidence catalog -> evidence reconciliation
                                                        -> curated device manifest
                                                        -> typed driver + replay tests
                                                        -> experimental availability
                                                        -> optional RazeRS verification
```

The evidence catalog supplies known identities and reusable implementation results.
A reviewed, curated manifest may select protocol and capability drivers and enable
experimental behavior without requiring the maintainer to own that device. A
verification record makes the narrower, stronger claim that RazeRS itself passed on
a stated platform and firmware. See [`evidence-policy.md`](/razers/en/evidence-policy/).

## Concurrency model

`razers-i18n` provides offline English/Chinese presentation at the application and
CLI boundaries. Locale selection never changes transport bytes, IPC method names,
schema enums, source evidence, or hardware policy. User-readable text is translated
after receiving the Agent's structured response.

Vendor request-response exchanges are serialized per physical connection. The future
Agent may be asynchronous, but each connection actor owns a synchronous `ReportIo`
backend. This prevents one command from consuming another command's response and
makes protocol wait, retry, and cancellation behavior deterministic.

High-frequency reversible settings may coalesce to their most recent value. Persistent
writes and firmware operations must never be silently coalesced.

## Workspace evolution

The completed Milestone 0 foundation and the first read-only Milestone 1 slice contain:

```text
razers-types
razers-protocol-core
razers-protocol-razer90
razers-transport
razers-transport-hidapi
razers-device-registry
razers-ipc
razers-agent
razers-app
razers-cli
```

The Agent owns HID enumeration, curated manifests, reconciled evidence, and the
user-facing device summaries. The desktop application requests those summaries over
the versioned IPC boundary; it does not enumerate HID itself. Catalogs and manifests
are embedded at build time so release binaries do not depend on a working directory
or download device data at runtime. The Agent exposes no control until a typed,
replay-tested driver exists.

Expected later boundaries include protocol-family crates, platform transport crates,
profiles, actions, diagnostics, and capability-driven controls. New crates should be
introduced when a real boundary is exercised, not only to mirror a future directory
diagram.

## Local IPC direction

The first transport launches an Agent child with inherited standard-input and
standard-output pipes. There is no listening socket, port, or cross-user endpoint.
Messages use newline-delimited JSON-RPC 2.0 with an independent RazeRS protocol
version. The release archive places `razers-agent` beside the desktop executable; a
hidden self-hosted child mode keeps development builds usable when that sibling has
not been built yet.

A future persistent Agent may use a current-user Unix domain socket on Linux/macOS
and a current-user named pipe on Windows, but only after its ownership, permissions,
peer authentication, and lifecycle are implemented and tested. Normal IPC must never
expose arbitrary raw hardware writes. See [`ipc.md`](/razers/en/ipc/) for the current wire
contract.
