# Product principles

RazeRS is software for the person who owns the hardware. The application exists
to make device configuration calm, fast, understandable, and dependable—not to
sell another product or keep the user inside an ecosystem.

## User contract

- No advertising, sponsored placements, promotional notifications, or upsell
  surfaces.
- No account is required for local device control, profiles, updates, diagnostics,
  or community support data.
- No telemetry, analytics, crash upload, or device-data upload is enabled by
  default. Any future network action must state what leaves the computer and wait
  for explicit consent.
- Core functionality is never deliberately delayed, obscured, or fragmented to
  increase engagement.
- The interface must say what works now. Evidence-backed possibilities are not
  presented as usable controls, and experimental behavior is visibly labeled.

## Interaction standards

The common path should be obvious: open RazeRS, choose a device, adjust a setting,
and know whether it applied. Frequently used settings stay close to the device
overview. Advanced and dangerous operations may use progressive disclosure, but
remain searchable and documented rather than hidden.

Every control needs a current value, supported range or choices, persistence scope,
apply state, and a useful failure explanation. Reversible settings should preview
immediately and support undo. Persistent writes require an explicit summary and
confirmation. Unsupported capabilities explain why they are unavailable instead of
disappearing.

The application follows the system theme and scaling, supports keyboard navigation
and platform accessibility APIs, avoids motion that is necessary to understand
state, and does not rely on color alone. Empty, loading, permission-denied,
disconnected, partial-support, and error states are first-class screens.

## Functional completeness

The desktop application renders shared capability descriptors; it must not grow a
separate hand-written page for each product. A capability is user-complete only when
its UI, validation, protocol driver, replay tests, state/error reporting, persistence
behavior, and documentation agree.

Breadth comes from reusable manifests and drivers. Device-specific exceptions belong
in reviewed quirks with provenance, never in silent UI conditionals. This keeps a
one-maintainer project capable of supporting community-tested hardware without
buying every model.

## Current boundary

The first application slice is intentionally read-only. A private Agent child
discovers Razer HID descriptors, groups interfaces by USB product identity, and
returns display-ready summaries over inherited pipes. The desktop process displays
curated or imported knowledge and explains why controls are still locked. Neither
process opens a device, reads a serial-number value, sends a report, requires an
account, or performs a network request.
