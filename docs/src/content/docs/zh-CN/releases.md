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
macOS Intel/ARM64 五个平台。每包包含桌面、Agent、CLI、中英文说明和字体许可，
并附 SHA-256。核心硬件控制成熟前保留预发布标记；预 alpha 阶段不发布到 crates.io。

构建器临时失败时重跑失败作业。也可用已有标签手动触发 Release 工作流，
重建并替换该版本资产，不修改版本和变更日志。

## 依赖与文档

Dependabot 每周检查 Cargo、文档站 npm 依赖与 GitHub Actions，并分组减少通知。
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
