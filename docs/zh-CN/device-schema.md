# 设备注册表格式 v1

`devices/*.toml` 中每个文件描述一个产品，`connections` 数组描述能暴露该产品的物理连接。
人工整理的清单与 `data/upstream` 自动目录不同：上游可为清单与实验性驱动提供证据，
但不能自动启用真实 I/O。差异按[证据政策](evidence-policy.md)调研；未解决字段保留并禁用。

解析器拒绝未知字段、重复标识、无效范围、未固定版本的证据，以及没有验证记录的
`verified` 声明。每次修改后运行：

```bash
cargo run -p razers-cli -- --lang zh-CN registry validate devices
```

## 产品字段

```toml
schema_version = 1
id = "razer.example-device"
display_name = "Razer Example Device"
kind = "mouse"

[support]
status = "detected"
notes = "仅识别标识；尚未验证硬件命令。"
```

`id` 是带命名空间的稳定标识，只允许小写 ASCII 字母、数字、点和连字符。
类别包括 `mouse`、`keyboard`、`headset`、`speaker`、`mouse-mat`、`laptop`、
`receiver`、`accessory`。字段与枚举值不随语言翻译，备注可以使用中文。

支持状态：`detected` 为已知身份；`experimental` 为有证据和回放测试、限制可见；
`verified` 为指定范围内 RazeRS 实机通过；`regressed` 为已发生回归；
`unsupported` 为确认不可用。实验性状态不要求维护者拥有该设备。

## 连接

```toml
[[connections]]
id = "wired"
role = "control"
transport = "usb-hid-feature"

[connections.match]
vid = 0x1532
pid = 0x0001
usage_page = 0xff00
usage = 0x0001
interface_number = 2

[connections.protocol]
family = "razer-report-90"
report_id = 0
transaction_id = 0x1f
response_delay_us = 600
busy_retries = 5

[connections.protocol.quirks]
include_report_id_in_payload = false
validate_response_crc = true
validate_command_echo = true
```

`usage_page`、`usage`、`interface_number` 在仅识别阶段可省略，但启用真实写入前
必须建立足够精确的接口匹配，避免打开普通输入接口或同一产品的其他逻辑设备。

## 能力与持久化

```toml
[capabilities.dpi]
status = "experimental"
driver = "dpi-u16-xy"
minimum = 100
maximum = 26000
step = 50
axes = "xy"
persistence = ["host-profile"]
```

能力定义语义行为并选择类型化驱动。每项能力有独立状态，产品状态不能隐藏部分支持。
持久化范围为 `session`、`host-profile`、`device-setting`、`onboard-profile-slot`。
重连后自动恢复主机配置不等于写入板载槽位。当前格式提供 DPI、回报率、灯光、电池类型；
新增字段需要解析校验、文档和版本决策。

## 证据

```toml
[[evidence]]
source = "OpenRazer"
repository = "openrazer/openrazer"
commit = "0123456789abcdef0123456789abcdef01234567"
path = "driver/example.c"
symbol = "USB_DEVICE_ID_RAZER_EXAMPLE"
license = "GPL-2.0-or-later"
```

每个清单都要求完整提交 SHA、准确文件与符号。证据说明事实来自哪里；
经过审阅的一组证据可支持实验性能力，但单独一条目录记录不够。

## 验证记录

`verified` 产品必须至少包含一条记录：

```toml
[[verification]]
platform = "linux-x86_64"
firmware = "1.03"
result = "passed"
capabilities = ["identity", "dpi", "polling-rate"]
notes = "通过有线控制接口测试。"
```

结论仅限所述平台、固件、连接和能力，不能扩大为所有设备和系统都已验证。
