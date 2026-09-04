---
title: "Desktop and languages"
description: "Understand device discovery, switch the interface language, and interpret support and privacy states."
---

The overview groups HID interfaces by USB product identity, not by physical unit.
Two identical mice can currently appear as one identity. Refresh requests a new
descriptor snapshot through a private Agent child; it does not query device settings.

## Language selection

Choose **System default**, **English**, or **简体中文** in the Language menu.
Changes take effect without restarting. The choice is saved locally by the desktop
framework on autosave/normal exit. A command-line `--lang` selection overrides the
saved choice for that launch and becomes the selected preference.

Automatic selection uses the first nonempty value from `RAZERS_LANG`, `LC_ALL`,
`LC_MESSAGES`, `LANG`, then the native OS language. Chinese locale variants map to
Simplified Chinese; other unsupported languages fall back to English. `auto` is
skipped when resolving environment preferences.

```bash
razers --lang zh-CN
razers --lang en
RAZERS_LANG=zh-CN razers
```

Labels, support explanations, capability badges, empty/error states and tooltips
are translated. Device model names, identifiers, source symbols and original
diagnostic details are not rewritten. Errors have a translated summary and expandable
technical details. Unknown messages from a newer Agent fall back to their original text.
Old v1 Agents remain readable, but lack structured evidence counts for localization.

## Privacy and support

The font and device knowledge are embedded. Startup, discovery and language switching
do not need the internet. The Documentation link opens an external browser only when
you choose it. No account, advertisement or background upload is involved.

Known capabilities describe evidence, not available controls. Open controls stays
disabled in this milestone. See [support levels](/razers/en/evidence-policy/) before interpreting
`detected`, `experimental` or `verified` labels.
