---
title: "架构"
description: "了解传输、协议、能力、Agent 与用户界面之间的职责边界。"
---

RazeRS 是跨平台、用户态的设备控制平台。操作系统 HID 驱动继续负责普通键盘、鼠标和
音频输入；RazeRS 面向灯光、DPI、回报率、电池、电源管理、均衡器及未来的输入动作。

## 职责边界

桌面 UI 通过版本化本地 IPC 请求 Agent；Agent 未来负责设备管理、配置、状态缓存和
诊断。语义设备命令进入每条物理连接独占的串行工作器，再由协议/能力驱动转换为固定
报告，交给 USB HID、hidraw、IOHID、Windows HID 或未来 BLE 传输后端。

三层保持独立：Transport 定义字节如何传输；Protocol 定义字节的意义；Capability
定义用户能做什么。传输层不得暴露 `set_dpi`、`set_static_color` 等语义操作。

## 设备模型

模型为 `Product → Connection → Logical device → Capability`。
一个产品可包含有线 USB、无线接收器、充电底座等多条连接；一个接收器也可能承载多个
逻辑设备。音箱可能同时拥有 USB 音频、厂商 HID 和 BLE 灯光。注册表与 Agent 必须
保留这些差异，不能把产品、连接和逻辑设备混为一谈。

界面应渲染共享的能力描述符，而不是每个型号单独编写页面。新增产品通常应通过清单、
已有驱动、证据与测试完成。

证据流程为：固定上游源码 → 生成证据目录 → 差异核对 → 人工审阅清单 → 类型化驱动与
回放测试 → 实验性可用 → 可选的 RazeRS 实机验证。上游目录不直接启用硬件操作。
符合[证据政策](/razers/zh-CN/evidence-policy/)的能力不要求维护者购买每个设备；`verified`
才表示 RazeRS 在指定平台与固件上实测通过。

## 并发与持久化

每条物理连接串行进行厂商请求/响应交换，防止一个命令消费另一个命令的响应。
未来 Agent 可以是异步的，但每个连接工作器独占同步 `ReportIo` 后端，使等待、重试和
取消行为可确定地测试。高频、可撤销设置可合并为最新值；持久写入和固件操作不得静默合并。

## 当前工作区

基础库包括 `razers-types`、`razers-protocol-core`、`razers-protocol-razer90`、
`razers-transport`、`razers-transport-hidapi`、`razers-device-registry`。
应用边界包括 `razers-ipc`、`razers-agent`、`razers-app`、`razers-cli`，翻译由
`razers-i18n` 提供。

Agent 拥有描述符枚举、内置清单与证据汇总；UI 只请求已脱敏的摘要，不自行枚举 HID。
目录与字体在编译时内置，运行不依赖工作目录或下载。当前没有打开设备的传输实现；
控制功能仍需类型化驱动与回放测试。

## 本地 IPC

当前以私有 Agent 子进程和继承的标准输入/输出管道通信，不监听端口或 Socket。
消息为按行分隔的 JSON-RPC 2.0，另有独立的 RazeRS 协议版本。
发行包把 `razers-agent` 放在桌面程序旁；开发环境缺少该程序时，桌面程序可启动自己的
隐藏子进程入口。

未来持久 Agent 可采用 Unix 域 Socket 或 Windows 命名管道，但必须先实现并测试
所有权、权限、对端身份和生命周期。普通 IPC 不得暴露任意原始硬件写入。
详见 [IPC 契约](/razers/zh-CN/ipc/)。只有实际需要新边界时才新增 crate。
