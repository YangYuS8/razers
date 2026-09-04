---
title: "Rust API reference"
description: "Generated Rust API references and executable examples for workspace libraries."
---

Open the [generated rustdoc index](/razers/api/) for all workspace crates.
The API is pre-alpha and may change between minor releases. Names and protocol
identifiers stay in their original form; bilingual prose describes their intent.

Start with `razers_types` for stable identifiers and safety classes,
`razers_device_registry` for manifest/evidence parsing, `razers_protocol_core` for
the 90-byte codec, and `razers_transport` for replay. `razers_protocol_razer90`
implements validated exchanges; `razers_ipc` defines the local wire contract;
`razers_agent` and `razers_app` own process boundaries; `razers_i18n` owns presentation.

Rustdoc is generated from the same commit as this handbook with warnings treated as
errors. It does not include dependency documentation or claim hardware verification.
