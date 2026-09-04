# RazeRS

[English](README.md) | 简体中文 | [文档站](https://yangyus8.top/razers/zh-CN/)

RazeRS 是实验性的跨平台、用户态雷蛇外设管理项目，以 Rust 实现共享核心。
传输、协议、设备能力与用户界面分层，尽可能复用 OpenRazer、OpenRGB、iRazer 的
硬件经验，不要求单人维护者购买并重复测试所有设备。

> 当前为只读预览：可枚举 HID 描述符、查看支持状态和社区证据，尚不提供真实硬件控制。
> “已知能力”不是“已实现控制”，上游记录也不是 RazeRS 实机验证。

## 面向用户

- 无广告、无需账号、默认无遥测或上传。
- 中英切换、跟随系统、记住语言选择、离线中文字体。
- 桌面通过私有 Agent 子进程通信，不暴露网络端口、设备路径或序列号值。
- 显示未知、部分支持、实验性和错误状态，不静默掩盖来源差异。

从 [Releases](https://github.com/YangYuS8/razers/releases) 下载对应系统归档，
解压后保持 `razers`、`razers-agent`、`razersctl` 同目录，运行 `razers`。
安装要求与安全提示见[快速开始](https://yangyus8.top/razers/zh-CN/getting-started/)。

## 从源码开始

需要 Rust 1.85+；Linux 需要 pkg-config、libudev 和 libxkbcommon 开发文件。

```bash
cargo test --workspace --all-features
cargo run -p razers-app -- --lang zh-CN
cargo run -p razers-cli -- --lang zh-CN help
cargo run -p razers-cli -- --lang zh-CN upstream assess 1532:0099
```

CLI 标识与协议保持不变，脚本可用 `--lang en` 固定输出语言。
[英文 README](README.md) 列出完整工作区和命令。
阅读[中文手册](https://yangyus8.top/razers/zh-CN/)和 [Rust API](https://yangyus8.top/razers/api/)。

## 维护和许可

CI 验证测试、最低 Rust 版本、中英文目录与文档；Actions 自动维护版本、变更日志、
依赖更新、五平台发行包和 GitHub Pages。发布时机由维护流程决定，硬件能力成熟前保留预发布。
双语 Starlight 手册与 rustdoc 在每个 PR 检查翻译同步、链接、锚点、搜索及移动端导航，
由 `main` 自动部署。文档依赖自动提出升级 PR，patch 更新可在必需检查通过后合并。

代码使用 GPL-2.0-or-later，见 [LICENSE](LICENSE)；内置中文字体保持 SIL OFL 1.1，
见[字体来源](assets/fonts/README.md)。项目独立于 Razer Inc.，不代表其官方立场。
贡献请先阅读[安全](docs/src/content/docs/zh-CN/safety.md)和[证据政策](docs/src/content/docs/zh-CN/evidence-policy.md)。

欢迎提问和提出功能建议，贡献上游证据不要求持有硬件。
请参阅[支持指南](SUPPORT.md)与[行为准则](CODE_OF_CONDUCT.md)。
