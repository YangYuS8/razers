---
title: "版本发布与依赖维护"
description: "版本发布、文档部署与依赖升级的自动化流程，以及仍需审阅的边界。"
---

RazeRS 使用 Conventional Commits 与 GitHub Actions。
`Cargo.toml` 的 workspace 版本是唯一包版本来源，所有 crate 继承它。

## 发布流程

每次推送 `main`，Release Please 维护一个发布 PR，自动更新版本、`Cargo.lock`、
`CHANGELOG.md` 与版本文件。1.0 之前 `feat:` 推进 minor，`fix:`/`perf:` 推进 patch，
破坏性变更标记推进对应版本；单纯文档和维护不单独触发发行。

发布 PR 是唯一决策关口，由维护流程在完整、已验证的里程碑时合并，无需用户逐次判断。
随后自动创建 `vX.Y.Z` 标签与预发布，构建 Linux x86-64/ARM64、Windows x86-64、
macOS Intel/ARM64 五个平台。下载优先展示安装包：Windows NSIS 安装向导、包含
RazeRS.app 的 macOS DMG、Debian 包和 Arch 二进制包，同时保留确定性便携归档。
每包包含桌面、Agent、CLI、中英文说明和字体许可，并另附 SHA-256 校验文件。
核心硬件控制成熟前保留预发布标记；预 alpha 阶段不发布到 crates.io。

构建器临时失败时重跑失败作业。也可用已有标签手动触发 Release 工作流，
重建并替换该版本资产，不修改版本和变更日志。
此入口支持包含安装包工具的标签；较早的纯归档标签需使用当时的工作流，
不在当前打包器中额外维护兼容层。

## 安装包自动化与边界

PR 检查与发行构建共用只读的 `installers.yml` 工作流。
`tools/package_installers.py` 从 Cargo 读取版本，通过 `tools/packaging` 中锁定的
`cargo-packager` 辅助工具打包，并使用发行版标准 `makepkg` 生成真正可安装的 Arch 包。
辅助工具单独维护锁文件、使用 stable 工具链，不随应用分发，也不提高应用 MSRV。
不为重复现成打包器已提供的行为而复制维护整份安装器模板。

每个平台检查包元数据、资源与校验和，通过无需硬件的 `agent.info` 运行包内 Agent
与桌面可执行文件，并测试安装、升级、卸载及偏好保留。升级测试用当前程序搭配模拟旧版
包元数据，验证安装器机制，不代表已验证历史设置迁移。macOS 在临时 Applications
目录复制、替换、删除应用包并验证签名，不模拟 Finder 交互或 Gatekeeper 行为。
Linux 还检查桌面入口和 Debian/Arch 内容一致性。两种架构均在 Ubuntu 的隔离根目录
中测试 Pacman 文件所有权，另用干净的 Arch x86-64 容器检查运行依赖。
这些检查不需要接入鼠标或其他实体设备；有安装/卸载动作的测试拒绝在 GitHub 托管的
一次性 runner 以外运行。

Windows 静态链接 C 运行库，并检查导入项，避免 runner 已安装的 Visual C++ 运行库
掩盖用户机器缺少依赖的问题。安装器仅面向当前用户，支持中英文。macOS 应用包面向
macOS 11+，仅使用临时签名。Windows 发布者签名、Apple Developer ID 与公证尚未配置，
需要另行授权的凭据，不能用关闭系统安全机制来替代。安装不会添加后台服务、自启动项、
更新器、账号要求或设备权限改动；升级与卸载保留设置。
APT/Pacman 软件仓库与桌面自动升级不属于本轮范围。

合并前除了原有 CI，还必须通过汇总的 `Installers` 检查。发布等待**全部五个**平台
通过，核对预期资产集合，再上传、生成双语安装下载入口，并重新下载校验已发布文件。
只有最终发布作业具有发行写权限，PR 打包作业没有。构建期间 release 条目可能已经
存在，空的 Assets 不代表发布完成。

为当前目标编译三份 release 程序后，可本地构建与检查：

```bash
cargo build --locked --manifest-path tools/packaging/Cargo.toml --target-dir target/packaging-tool
python tools/package_installers.py --target x86_64-unknown-linux-gnu --packager target/packaging-tool/debug/razers-packaging
python tools/check_installers.py --target x86_64-unknown-linux-gnu
```

本地检查只解包验证，不安装。Linux 打包还需 `dpkg-deb`、`makepkg`、`fakeroot`、`bsdtar` 和 `zstd`。
Windows 辅助工具名带 `.exe`，编译应用时需将目标专用 Rust flags 设为
`-C target-feature=+crt-static`；macOS 编译前设置 `MACOSX_DEPLOYMENT_TARGET=11.0`。
完整的平台命令以 CI 为准。

## 依赖与文档

Dependabot 每周检查工作区 Cargo、独立打包辅助工具、文档站 npm 依赖与 GitHub Actions，
并分组减少通知。
Cargo/npm patch 和 Actions patch/minor 在必需 CI 通过后可自动合并；
Cargo/npm minor/major 与 Actions major 留待审阅。Actions 固定到不可变提交 SHA，
Dependabot 同时维护固定值和版本注释。

冻结的 pnpm 安装和显式原生构建许可用于保证可重复构建。
不要为安装刚发布的依赖而关闭供应链检查，应等待所需的发布时间窗口并验证后更新固定值。
工具链重大升级、许可、支持声明、安全和有冲突的硬件证据仍需判断，不能自动批准。

## 文档发布

Documentation 工作流在每个 PR 构建双语 Starlight 和库 rustdoc，检查页面配对、
翻译对应文件的改动、本地链接及锚点，拒绝 rustdoc 警告，并执行语言切换、中英文搜索、
根入口、API 导航与移动端导航的浏览器回归测试。
只有 `main` 通过 OIDC 和 `github-pages` 环境部署，PR 不持有 Pages 写权限。
单纯文档变更不需要提升应用版本或发布应用。

`docs/package.json` 与 `docs/pnpm-lock.yaml` 固定文档工具及依赖，Node 使用
`docs/.node-version` 中的 LTS 主版本。`build-info.json` 记录源码提交、工作区版本、
文档框架版本与包管理器。本站描述开发进度，历史版本请查看对应 tag 和发行日志，
本站目前不提供逐版本归档。

外部链接每周独立检查，第三方暂时不可用不会阻塞无关 PR；失败会显示在 GitHub Actions。
浏览器测试失败时保留七天诊断产物。翻译检查只能发现漏同步，不能判断译文含义。
日常维护见[翻译与文档维护](/razers/zh-CN/localization/)。
