---
title: 完成第一次只读体验
description: 不发送硬件指令，了解设备发现结果，并查看社区证据。
---

这个教程不需要你再买一台设备。没有连接雷蛇 USB 设备时，可走下面的空列表流程；
仍然可以检查上游证据。

1. 按[安装说明](/razers/zh-CN/getting-started/)启动 `razers`。
2. 选择 **English**、**简体中文**或跟随系统，然后刷新设备列表。
3. 如果出现设备，查看 USB 标识和证据来源。概览按产品标识归并接口，
   不代表物理设备数量。
4. 如果列表为空，查看[排障说明](/razers/zh-CN/troubleshooting/)中的连接提示。
   不要为了启用灰色按钮而安装内核驱动、以 root 运行或授予额外权限。
5. 留意控制面板当前不可用。这是现阶段的只读边界，不代表设备永远无法得到支持。

无需连接设备，也可以在源码仓库中查看社区证据：

```bash
cargo run --locked -p razers-cli -- --lang zh-CN upstream assess 1532:0099
```

结果比较固定版本的上游声明，不会发送报告，也不构成 RazeRS 实机验证。
接下来可以了解如何[贡献证据](/razers/zh-CN/contribute-evidence/)。
