# Device registry schema v1

Device manifests live in `devices/*.toml`. One file describes one product, while its
`connections` array describes the physical links that can expose that product.

These curated manifests are intentionally distinct from the generated, evidence-only
OpenRazer catalog in `data/upstream/openrazer-devices.toml`. Upstream data can seed a
manifest, but it does not establish a safe interface match, protocol selection, or
working RazeRS capability.

The registry parser rejects unknown fields, duplicate identifiers, invalid ranges,
unpinned evidence, and claims of verified support without a verification record.
Run this after every manifest edit:

```bash
cargo run -p razers-cli -- registry validate devices
```

## Required product fields

```toml
schema_version = 1
id = "razer.example-device"
display_name = "Razer Example Device"
kind = "mouse"

[support]
status = "detected"
notes = "Identity only; no hardware commands have been verified."
```

`id` is stable and namespaced. It accepts lowercase ASCII letters, digits, dots, and
hyphens. Product kinds currently include `mouse`, `keyboard`, `headset`, `speaker`,
`mouse-mat`, `laptop`, `receiver`, and `accessory`.

Support status is one of:

- `detected`: identity is known, without a working-command claim;
- `experimental`: opt-in behavior exists but is incompletely tested;
- `verified`: listed platform/firmware records passed hardware tests;
- `regressed`: behavior that previously passed currently fails;
- `unsupported`: the device or capability is confirmed not to work.

## Connections

```toml
[[connections]]
id = "wired"
role = "control"
transport = "usb-hid-feature"

[connections.match]
vid = 0x1532
pid = 0x0001
usage_page = 0xff00       # optional
usage = 0x0001            # optional
interface_number = 2      # optional

[connections.protocol]
family = "razer-report-90"
report_id = 0
transaction_id = 0x1f
response_delay_us = 600
busy_retries = 5

[connections.protocol.quirks]
include_report_id_in_payload = false
validate_response_crc = true
validate_command_echo = true
```

Matching must be narrow enough to avoid opening an ordinary input interface or a
different logical device exposed by the same product. Interface details may remain
absent while support is only `detected`, but must be established before live writes.

## Capabilities

Capabilities describe semantic behavior and select a typed driver. Each capability
has its own support state; product-level support must not hide partial results.

```toml
[capabilities.dpi]
status = "experimental"
driver = "dpi-u16-xy"
minimum = 100
maximum = 26000
step = 50
axes = "xy"
persistence = ["host-profile"]
```

Supported persistence scopes are `session`, `host-profile`, `device-setting`, and
`onboard-profile-slot`. Automatically restoring a host profile after reconnect is
not the same as writing an onboard slot.

The initial schema has typed definitions for DPI, polling rate, lighting, and battery.
Schema additions require parser validation, documentation, and a versioning decision.

## Evidence

Every manifest requires a pinned source:

```toml
[[evidence]]
source = "OpenRazer"
repository = "openrazer/openrazer"
commit = "0123456789abcdef0123456789abcdef01234567"
path = "driver/example.c"
symbol = "USB_DEVICE_ID_RAZER_EXAMPLE"
license = "GPL-2.0-or-later"
```

Use a complete commit SHA and exact path/symbol. Evidence states where a fact came
from; it does not by itself establish that RazeRS works with the device.

## Verification

A `verified` product must include at least one record:

```toml
[[verification]]
platform = "linux-x86_64"
firmware = "1.03"
result = "passed"
capabilities = ["identity", "dpi", "polling-rate"]
notes = "Tested over the wired control interface."
```

Verification is scoped to the stated platform, firmware, connection, and capabilities.
