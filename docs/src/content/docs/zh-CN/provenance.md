---
title: "来源记录"
description: "固定的上游来源、协议差异与内置资产的许可记录。"
---

## 内置中文字体

Noto Sans SC 从 `notofonts/noto-cjk` 的固定提交
`f8d157532fbfaeda587e826d4cd5b21a49186f7c` 原样嵌入，单独遵循 SIL Open Font License 1.1，
不适用代码的 GPL 许可。[字体说明](https://github.com/YangYuS8/razers/blob/main/assets/fonts/README.md)
记录上游路径与 SHA-256；发行包保留原始 OFL。

协议事实和设备元数据固定到上游提交，确保未来仍能重现研究。

## OpenRazer

基线提交为 `6820f9da169d354bc7e6e93a0aa8683a6bb75792`。

- [razercommon.h](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razercommon.h)：90 字节报告、字段尺寸、状态及大端剩余包数。
- [razercommon.c](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razercommon.c)：字节 2 到 87（含）的 XOR 校验。
- [razermouse_driver.h](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/driver/razermouse_driver.h)：Basilisk V3 的 VID `0x1532`、PID `0x0099`。
- [mouse.py](https://github.com/openrazer/openrazer/blob/6820f9da169d354bc7e6e93a0aa8683a6bb75792/daemon/openrazer_daemon/hardware/mouse.py)：最高 26,000 DPI、1×11 灯光矩阵与能力列表。

`openrazer-devices.toml` 包含从七个具体设备模块提取的 267 个 USB 标识，保留文件、
类符号、方法、矩阵、DPI 上限、回报率与保守的能力提示。
[导入器](https://github.com/YangYuS8/razers/blob/main/tools/import_openrazer.py) 只接受上述提交，
Rust 解析器拒绝重复 VID/PID、无效来源、尺寸与数值，使刷新可以审阅而非盲目跟随最新分支。

OpenRazer 为 GPL-2.0-or-later；RazeRS 的 Rust 实现同样采用该许可并保留 SPDX。
当前为根据公开格式新写的实现，不是逐行翻译上游驱动。

## BUSY 行为差异

OpenRazer 请求循环接受合法 `BUSY` 响应，因为部分设备已完成命令。
独立 Windows 项目 opsrzr 在提交 `f4e9eabca19f721cf1bcb6ee8097d0748367cfe7` 的
[transport.rs](https://github.com/atv57/opsrzr/blob/f4e9eabca19f721cf1bcb6ee8097d0748367cfe7/crates/razer-hid/src/transport.rs#L164-L211)
选择重试。RazeRS 将其建模为每连接显式 `BusyHandling` 策略，默认接受 BUSY，避免重发
可能重复持久操作的写命令；只有已知安全的设备/命令才选择重试。
重试、短读、状态、回显、事务 ID 和包计数都通过内存回放验证。
opsrzr 对应 crate 是 GPL-2.0-only；此处仅用于印证差异，不复制或改许可。

## OpenRGB

基线提交 `7fed68ccf1a2413b9bd38a70e266b12cb2d59c26`。
目录包含 196 条设备表记录，保留矩阵协议族、事务 ID、矩阵尺寸、区域/PID 符号与可选布局。
来源为 GPL-2.0-or-later 的
[RazerDevices.h](https://github.com/CalcProgrammer1/OpenRGB/blob/7fed68ccf1a2413b9bd38a70e266b12cb2d59c26/Controllers/RazerController/RazerDevices.h) 和
[RazerDevices.cpp](https://github.com/CalcProgrammer1/OpenRGB/blob/7fed68ccf1a2413b9bd38a70e266b12cb2d59c26/Controllers/RazerController/RazerDevices.cpp)。

与 OpenRazer 重叠 172 个标识，其中有 72 项名称差异和 18 项矩阵差异。
RazeRS 报告差异，不静默择一；按语义、修订、源码历史、Issue、测试和额外实现调研。
购买设备不是默认解决途径。

## iRazer

基线提交 `7cc856ddd26edd9523a12a540b6d95a4ea3a54c4`。
从 MIT 许可的 [DeviceCatalog.swift](https://github.com/hanley-tech/iRazer/blob/7cc856ddd26edd9523a12a540b6d95a4ea3a54c4/Sources/iRazer/DeviceCatalog.swift)
导入全部 192 条记录，保留 USB 标识、类别、能力标签、矩阵协议族、事务 ID 和上游支持声明。
与 OpenRGB 重叠 189 个标识，在这些固定提交中未发现矩阵协议族/事务 ID 冲突，额外收录
Nommo V2 Pro、Nommo V2、Nommo V2 X。

iRazer 的 `supported` 始终归属 iRazer，但它是实验性支持决策的重要输入，不应因缺少
RazeRS 重复测试而丢弃。

## 证据、验证与字体

自动目录与人工清单分离，导入不能静默打开硬件。按[证据政策](/razers/zh-CN/evidence-policy/)
核对来源并通过回放后，可实现或发布实验性能力；RazeRS 实机验证是更强且范围明确的声明。

中文字体采用未修改的 Noto Sans SC，来源与 SHA-256 见
[字体记录](https://github.com/YangYuS8/razers/blob/main/assets/fonts/README.md)。
字体保持 SIL OFL 1.1，发行包携带独立的字体许可；不改变为代码的 GPL 许可。
