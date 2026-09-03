# Architecture

Razers is a cross-platform, user-space device control platform. Operating-system
HID drivers remain responsible for ordinary keyboard, mouse, and audio input. Razers
handles vendor-specific configuration such as lighting, DPI, polling rate, battery,
power management, equalizers, and eventually input actions.

## Boundaries

```text
Desktop UI / CLI / SDK
          |
          | versioned local IPC (future milestone)
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

## Concurrency model

Vendor request-response exchanges are serialized per physical connection. The future
Agent may be asynchronous, but each connection actor owns a synchronous `ReportIo`
backend. This prevents one command from consuming another command's response and
makes protocol wait, retry, and cancellation behavior deterministic.

High-frequency reversible settings may coalesce to their most recent value. Persistent
writes and firmware operations must never be silently coalesced.

## Workspace evolution

Milestone 0 intentionally contains only foundational crates:

```text
razers-types
razers-protocol-core
razers-transport
razers-device-registry
razers-cli
```

Expected later boundaries include protocol-family crates, platform transport crates,
an Agent core and executable, versioned IPC, profiles, actions, diagnostics, and a
capability-driven desktop application. New crates should be introduced when a real
boundary is exercised, not only to mirror a future directory diagram.

## Local IPC direction

The Agent will use a versioned local protocol: Unix domain sockets on Linux/macOS and
named pipes on Windows. JSON-RPC is the preferred initial encoding because it is easy
to inspect and supports third-party clients. IPC must be restricted to the current
user and must not expose raw hardware writes in normal builds.
