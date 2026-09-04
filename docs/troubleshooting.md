# Troubleshooting

## No devices found

Check the cable/receiver, reconnect and refresh. Bluetooth-only devices or links that
do not expose a Razer USB HID identity are outside current enumeration. An empty list
does not prove the hardware is unsupported. RazeRS does not install kernel drivers
or ask for root to list descriptors.

## Controls are unavailable

This is expected: the current build is descriptor-only. Imported support claims are
not working RazeRS controls. No extra permissions can unlock an unimplemented driver.

## Agent startup or protocol error

Keep all binaries from the same archive together. Do not mix old and new executables.
Refresh after replacing the complete package. Expand Technical details for the original
error; redact personal paths before reporting it. A missing sibling Agent uses the
desktop binary's child entry point as a development fallback.

## Language or font problems

Choose a language explicitly, or launch with `--lang zh-CN` / `--lang en`.
Environment variables can override native OS language in System default mode.
Chinese glyphs are embedded, so installing a system language pack is unnecessary.
If reporting a rendering issue, include the language, scaling, OS and a redacted screenshot.

## Trust prompts and older Linux systems

Current archives are not signed installers. Verify source and SHA-256 instead of
disabling OS security globally. A glibc mismatch may require building from source
on the target system. See [getting started](getting-started.md).

Report issues at [GitHub Issues](https://github.com/YangYuS8/razers/issues).
Never publish serial numbers, raw input traffic, access tokens or private diagnostics.
