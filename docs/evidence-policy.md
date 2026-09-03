# Upstream evidence policy

RazeRS is designed to be maintainable by a small community. It does not require a
project maintainer to buy and retest every device that established open-source
drivers already support. Published implementation experience is reusable
engineering evidence when its origin, meaning, and license are clear.

## What upstream evidence can establish

Pinned upstream implementations may supply device identities, connection and
protocol parameters, capability ranges, lighting geometry, command behavior,
timing, quirks, and expected responses. Those facts may be used to:

- generate and review candidate device manifests;
- implement typed Rust protocol and capability drivers;
- construct golden packets and replay tests without hardware;
- ship an `experimental` capability when the evidence and safety conditions below
  are satisfied.

A separate RazeRS hardware test is valuable, but is not a prerequisite for every
implementation. The `verified` state is reserved for the stronger claim that a
specific RazeRS build passed on a recorded platform, connection, and firmware.

## Evidence threshold

An upstream-backed capability may become `experimental` when:

1. every material fact points to an exact repository, commit, path, and symbol;
2. a mature upstream implementation identifies the exact device, or two
   independent implementations agree on the fact;
3. protocol encoding, response validation, limits, and failure paths have local
   unit or replay tests;
4. live I/O is limited to a narrowly matched vendor interface;
5. reversible writes require an experimental opt-in until stronger evidence or
   community reports establish safe defaults.

Persistent storage, firmware, bootloader operations, and incompletely understood
raw writes are never enabled solely from catalog metadata.

## Resolving disagreements

No source has global priority. Source relevance is field-specific: a lighting
implementation can be stronger evidence for matrix geometry, while a device driver
may be stronger evidence for DPI limits or response timing.

When sources disagree:

1. normalize names, units, matrix orientation, connection mode, hardware revision,
   and firmware scope before treating values as contradictory;
2. inspect the exact implementations plus their commit history, issues, pull
   requests, release notes, tests, and available redacted traces;
3. seek another independent implementation or vendor documentation;
4. record every claim and the rationale for the selected value in the curated
   manifest or provenance notes;
5. model real variants as separate connections, revisions, or quirks instead of
   forcing one value across all hardware;
6. if the conflict remains unresolved, mark only the affected field or capability
   disputed and keep its live operation disabled.

Agreement is strong evidence, not an automatic merge rule. Disagreement triggers
research, not a requirement that the maintainer purchase the device.

## Confidence and user-facing claims

- `detected`: the identity is known, but no RazeRS command is offered.
- `experimental`: an evidence-backed RazeRS implementation and replay tests exist;
  project-owned hardware testing is optional and any limitations are visible.
- `verified`: a recorded RazeRS hardware result exists for the listed platform,
  firmware, connection, and capabilities.
- `regressed`: behavior that previously worked now fails.
- `unsupported`: evidence confirms that the device or capability cannot work in
  the stated scope.

Community verification adds coverage and may promote support, but a missing RazeRS
hardware record must never erase credible upstream testing or force duplicate
research.
