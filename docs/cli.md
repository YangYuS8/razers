# Command-line tools

`razersctl --help` lists all commands; `--lang en` and `--lang zh-CN` select output
language. Options may precede or follow the command. Command names, USB IDs, hex
reports, schema tokens and upstream source data remain stable. Human-readable labels
are localized; scripts should explicitly use `--lang en` instead of parsing an OS-dependent
language. The CLI currently has no general JSON output mode.

```bash
razersctl --lang en registry validate devices
razersctl registry list devices --lang zh-CN
razersctl registry show razer.basilisk-v3 devices
razersctl upstream validate
razersctl upstream stats
razersctl upstream lookup 1532:0099
razersctl upstream assess 1532:0099
razersctl upstream shortlist
razersctl upstream conflicts
razersctl devices devices
razersctl report encode 0x00 0x81 0000
```

`report decode <HEX>` inspects a report without hardware. Numeric command bytes accept
decimal or `0x` hexadecimal. Report hex accepts whitespace, `:` and `-` separators.
Non-ASCII or incomplete bytes are rejected instead of panicking.

Registry commands default to `./devices`. Upstream commands default to the three
catalogs under `./data/upstream`. Run these developer commands from a source checkout
or pass explicit paths. The desktop Agent embeds its catalogs and does not need a checkout.

`shortlist` and `conflicts` classify evidence; neither enables hardware controls.
Errors use exit code 2, translated context and original diagnostic detail where useful.

`razers-agent --help --lang zh-CN` explains the service entry point.
`razers-agent --stdio` always speaks the same [IPC protocol](ipc.md), regardless of locale.
