---
title: Your first read-only session
description: Discover a device, understand the result, and inspect evidence without sending hardware commands.
---

This tutorial works without buying another device. If no Razer USB device is
connected, use the empty-state path below; evidence inspection still works.

1. Follow [installation](/razers/getting-started/) and open `razers`.
2. Select **English**, **简体中文**, or the system language. Refresh the device list.
3. If a device appears, compare its USB identity and displayed evidence sources.
   The overview groups interfaces by product identity, not by physical unit.
4. If the list is empty, read the connection hints in
   [troubleshooting](/razers/troubleshooting/). Do not install a kernel driver,
   run as root, or grant extra permissions just to unlock a disabled control.
5. Notice that the control panel is unavailable. This is the current read-only
   boundary, not evidence that your hardware cannot eventually be supported.

To inspect community evidence without any device, run this from a source checkout:

```bash
cargo run --locked -p razers-cli -- --lang en upstream assess 1532:0099
```

The result compares pinned upstream claims. It does not send a report or verify
RazeRS hardware support. Next, learn to [contribute evidence](/razers/contribute-evidence/).
