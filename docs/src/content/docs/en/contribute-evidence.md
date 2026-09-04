---
title: Contribute evidence without owning hardware
description: Turn an upstream implementation into a reviewable contribution without duplicating hardware testing.
---

You can contribute a source reference, a manifest, a protocol test, or a translation
without owning the device. Hardware reports are a separate, optional contribution.

1. From a source checkout, run `cargo run -p razers-cli -- upstream lookup 1532:0099`
   (substitute the device's VID:PID) to see existing sources.
2. Run `cargo run -p razers-cli -- upstream assess 1532:0099` and identify the
   specific missing or disputed field. A name difference alone is not proof of a conflict.
3. Find the upstream repository, exact commit, path, symbol, and license. Record
   connection mode, revision, units, and firmware scope when known. Do not guess unknown values.
4. If sources disagree, inspect their implementations and history; retain both
   claims and explain the reasoning. Do not select the newest number automatically.
5. Open an **Upstream evidence** issue, or a focused pull request. Include the
   relevant source links and a hardware-free replay test when contributing code.

Use **Device hardware report** only for actual test results. State working, failed,
and untested capabilities separately; redact serial numbers, personal paths, and
raw input. Never run unreviewed writes merely to complete a report.

An accepted source record does not automatically enable controls. See the
[evidence policy](/razers/evidence-policy/) for experimental support and scoped verification.
