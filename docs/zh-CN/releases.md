# 版本发布与依赖维护

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

Dependabot 每周检查 Cargo 与 GitHub Actions 并分组减少通知。
Cargo patch 和 Actions patch/minor 在必需 CI 通过后可自动合并；
Cargo minor/major 与 Actions major 留待审阅。Actions 固定到不可变提交 SHA，
Dependabot 同时维护固定值和版本注释。

mdBook 固定在 `tools/docs-requirements.txt`，升级时须验证两种语言与本地链接。
Documentation 工作流在每个 PR 构建并验证 mdBook/rustdoc，只有 `main` 部署 Pages。
本站描述 `main`，历史版本请查看对应 tag 和发行日志。
