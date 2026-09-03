# Hardware safety policy

Razers is designed for reversible device configuration, but vendor commands can still
disconnect devices, corrupt persistent settings, or make firmware unusable. Safety is
an architectural requirement rather than a user-interface warning added later.

## Command risk classes

Every semantic command must eventually declare one of these classes:

| Risk | Meaning | Default release behavior |
| --- | --- | --- |
| `read-only` | Descriptor or state query | Allowed for matched devices |
| `reversible` | Volatile DPI, lighting, or similar state | Allowed only for supported capabilities |
| `persistent` | Device or onboard storage write | Explicit confirmation and verification required |
| `experimental-write` | Incompletely understood vendor command | Developer build and allowlist only |
| `firmware` | Bootloader or firmware operation | Not part of the normal Agent or IPC |

Unknown devices enter safe probing. Safe probing may enumerate descriptors and run
reviewed information queries, but it must not send persistent writes, brute-force
commands, fuzz packet fields, or perform firmware operations.

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

The current code can enumerate HID descriptors but does not open devices or write to
real hardware. Device paths and serial-number values are omitted from enumeration
results. `ReplayTransport` remains the only report I/O implementation supplied by the
workspace.
