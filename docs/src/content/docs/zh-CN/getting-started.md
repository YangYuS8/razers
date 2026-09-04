---
title: "快速开始"
description: "下载、校验并启动只读预览，或从源码构建 RazeRS。"
---

## 下载与安装

使用 [GitHub Releases](https://github.com/YangYuS8/razers/releases) 中的 **安装** 下载入口。
“预发布”表示功能成熟度，不表示不能提供安装包；较早版本可能仍只有便携归档。
当前应用仍是只读预览，安装后不会解锁 DPI、灯效或按键控制。

| 系统 | 推荐格式 | 文件名目标 |
| --- | --- | --- |
| Windows Intel/AMD 64 位 | `-setup.exe` | `x86_64-pc-windows-msvc` |
| macOS Apple Silicon | `.dmg` | `aarch64-apple-darwin` |
| macOS Intel | `.dmg` | `x86_64-apple-darwin` |
| Debian/Ubuntu Intel/AMD 64 位 | `.deb` | `x86_64-unknown-linux-gnu` |
| Debian/Ubuntu ARM64 | `.deb` | `aarch64-unknown-linux-gnu` |
| Arch Linux Intel/AMD 64 位 | `.pkg.tar.zst` | `x86_64-unknown-linux-gnu` |
| Arch Linux ARM64 | `.pkg.tar.zst` | `aarch64-unknown-linux-gnu` |

每个包都包含桌面应用、私有 Agent、开发者 CLI、中英文资源、离线中文字体和许可说明。
不加入广告、账号要求、自启动项或系统服务，不修改设备权限。请勿以 root 身份运行应用。

### Windows

运行 `-setup.exe` 安装向导，再从开始菜单打开 **RazeRS**。默认仅安装到当前用户，
无需管理员权限；向导跟随系统语言，支持英文与简体中文。升级前退出 RazeRS，运行新版
安装器即可；卸载使用 **设置 → 应用 → 已安装的应用**。升级和卸载均保留用户偏好。
应用与安装器目前尚无发布者证书签名，SmartScreen 或组织策略可能阻止运行。

### macOS

打开 DMG，把 **RazeRS.app** 拖入 **Applications（应用程序）**，再推出磁盘映像。
从“应用程序”启动。升级时退出应用，用新版替换原应用包；卸载时将 RazeRS.app 移到
废纸篓。偏好设置另行保存，不随应用包删除。需要 macOS 11 或更新版本，并选择对应
Intel / Apple Silicon 架构。应用只有临时签名，**没有** Developer ID 发布者签名或
Apple 公证，Gatekeeper 可能阻止打开。核对来源后，可按照 Apple 的
[受信任应用打开说明](https://support.apple.com/zh-cn/102445)操作；受管理的设备可能需要管理员批准。

### Linux

Debian 包面向 Ubuntu 24.04 或兼容的较新用户空间，例如 Debian 13：需要 glibc 2.39+、
libudev 与正常的 Wayland/X11 图形会话，不覆盖旧版 Ubuntu/Debian。
在下载目录执行，替换实际版本和架构：

```bash
sudo apt install ./razers-vX.Y.Z-x86_64-unknown-linux-gnu.deb
```

Arch Linux 使用可直接安装的二进制包，而非便携归档：

```bash
sudo pacman -U ./razers-vX.Y.Z-x86_64-unknown-linux-gnu.pkg.tar.zst
```

从应用菜单启动 **RazeRS**。升级时对新版文件运行相同安装命令；卸载使用
`sudo apt remove razers` 或 `sudo pacman -R razers`，保留用户偏好。
运行依赖由包管理器解析。目前没有 APT/Pacman 软件仓库，因此桌面更新不会自动安装。
保持现有签名策略；这些本地安装包附带 SHA-256 校验和，没有发行版软件仓库签名。

CI 检查两种架构的 Linux 程序，并在 Arch Linux x86-64 容器中验证运行依赖安装。
ARM64 的 Pacman 安装、升级和卸载在 Ubuntu ARM64 的隔离包根目录中验证，
不等于已在 Arch Linux ARM 设备上实测。当前安装包集合不覆盖 Fedora 等其他发行版。

## 校验下载

下载同名 `.sha256` 文件，在下载目录校验：

```bash
sha256sum --check razers-vX.Y.Z-x86_64-unknown-linux-gnu.deb.sha256
```

macOS 使用 `shasum -a 256 -c <校验文件>`；Windows 使用
`Get-FileHash <发行包> -Algorithm SHA256`，并与下载的校验文件比较。
校验和用于发现字节变化，不等于发布者签名；同时核对 GitHub 仓库与发行版本。
不要为了运行 RazeRS 而全局关闭系统安全机制。

## 便携包选项

仍提供 Windows `.zip` 与 Linux/macOS `.tar.gz`。解压后保持 `razers`、`razers-agent`、
`razersctl` 同目录（Windows 文件名带 `.exe`），运行 `razers`。
便携包不提供安装向导、应用菜单注册、内置更新器或运行依赖自动安装。
它仍使用正常的用户偏好目录；“便携”不表示所有设置都存放在可执行文件旁边。

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

语言设置与功能边界请见[桌面应用](/razers/zh-CN/application/)。
