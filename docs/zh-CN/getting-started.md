# 快速开始

## 下载与启动

在 [GitHub Releases](https://github.com/YangYuS8/razers/releases) 选择对应平台：

| 系统 | 发行包目标 |
| --- | --- |
| Linux Intel/AMD 64 位 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| Windows Intel/AMD 64 位 | `x86_64-pc-windows-msvc` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

解压后，请将 `razers`、`razers-agent` 和 `razersctl` 放在同一目录
（Windows 文件名带 `.exe`），运行 `razers` 打开桌面界面。不要以 root 身份运行。
目前提供便携二进制，不是签名安装器或 macOS 应用包。系统可能显示信任提示；
请核对下载来源与校验和，不要为了运行程序而全局关闭系统安全机制。

Linux 发行包在 Ubuntu 24.04 上构建，需要兼容的 glibc、libudev 和桌面图形环境。
程序已内置中文字体，无需另外安装。

在下载文件所在目录校验：

```bash
sha256sum --check razers-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz.sha256
```

macOS 使用 `shasum -a 256 -c <校验文件>`；Windows 使用
`Get-FileHash <发行包> -Algorithm SHA256`，并与下载的校验文件比较。

## 从源码构建

需要 Rust 1.85 或更高版本。Debian/Ubuntu 还需 `pkg-config`、`libudev-dev`、
`libxkbcommon-dev`；Arch 对应 `pkgconf`、`systemd`、`libxkbcommon`。
Windows/macOS 原生构建需要对应平台的 SDK 和链接器。

```bash
git clone https://github.com/YangYuS8/razers.git
cd razers
cargo test --workspace --all-features --locked
cargo run -p razers-app -- --lang zh-CN
```

语言设置与功能边界请见[桌面应用](application.md)。
