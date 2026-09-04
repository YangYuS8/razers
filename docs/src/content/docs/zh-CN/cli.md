---
title: "命令行工具"
description: "使用 RazeRS 开发者命令行检查设备清单、上游证据与协议报文。"
---

`razersctl --help` 列出全部命令；`--lang en` 和 `--lang zh-CN` 选择输出语言。
语言选项可放在命令前后。命令名称、USB ID、十六进制报告、格式枚举值及上游数据
保持稳定；面向人的标签会翻译。脚本请显式使用 `--lang en`，不要依赖系统语言。
CLI 目前没有通用 JSON 输出模式。

```bash
razersctl --lang zh-CN registry validate devices
razersctl registry list devices --lang en
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

`report decode <HEX>` 可在不连接硬件的情况下检查报告。命令字节接受十进制或
`0x` 十六进制；报告允许空白、冒号和连字符分隔。非 ASCII 和不完整字节会被拒绝，
而不是造成程序崩溃。

注册表默认目录是 `./devices`；上游命令默认读取 `./data/upstream` 的三个目录文件。
请在源码仓库中运行，或显式传入路径。桌面 Agent 内置这些数据，不依赖源码目录。

`shortlist` 和 `conflicts` 只负责证据分类，不会启用硬件控制。
错误退出码为 2；错误上下文可翻译，必要的原始技术诊断保留原文。

`razers-agent --help --lang zh-CN` 可查看服务入口说明。
无论使用哪种语言，`razers-agent --stdio` 都使用相同的 [IPC 协议](/razers/zh-CN/ipc/)。
