# 本地 IPC 协议

RazeRS 协议版本 1 使用按行分隔的 [JSON-RPC 2.0](https://www.jsonrpc.org/specification)。
每行一个请求或响应，本传输版本不接受批量数组。
桌面创建私有 Agent 子进程，通过继承管道通信；没有监听 Socket 或网络端口。
父进程关闭输入后 Agent 退出。发行包把 `razers-agent` 放在 `razers` 旁；
开发构建缺少该文件时可启动桌面程序的隐藏 Agent 子进程入口。

## 版本与方法

信封固定为 `"jsonrpc":"2.0"`；每个方法的参数对象包含 `"protocol_version":1`，
成功结果再次返回 RazeRS 协议版本。两种版本相互独立。
不支持的版本返回服务器错误 `-32001`，`error.data` 包含期望值和收到值。
新增兼容字段无需升级协议，不兼容的方法或字段变更必须升级。

`agent.info` 返回 Agent 版本、协议版本、访问模式和传输方式。
`devices.list` 枚举描述符并返回脱敏摘要。没有打开硬件或发送报告的方法。

```json
{"jsonrpc":"2.0","method":"devices.list","params":{"protocol_version":1},"id":1}
```

使用 JSON-RPC 标准的解析、无效请求、方法、参数和内部错误码。
通知不含 `id`，不返回响应。请求 ID 可为字符串、数字或 null，桌面使用整数。

## 翻译与隐私

协议方法、字段、错误码和现有英文兼容字段不受 UI/CLI 语言影响。
设备摘要新增可选 `evidence_source_count`，用于界面翻译相互印证的来源数量；
旧客户端忽略新字段，新客户端可读取缺少该字段的旧 Agent。
型号与原始来源事实保持原样，设备路径和序列号值绝不放进 IPC 结果。

当前没有可供其他进程发现的公共端点。持久用户级 Socket/命名管道推迟到各平台访问控制
和对端验证可测试后再引入。
