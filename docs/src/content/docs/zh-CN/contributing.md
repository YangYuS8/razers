---
title: "参与贡献"
description: "准备开发环境、验证改动，并贡献代码、翻译或上游研究。"
---

请先阅读[仓库贡献指南](https://github.com/YangYuS8/razers/blob/main/CONTRIBUTING.md)
与[安全报告政策](https://github.com/YangYuS8/razers/blob/main/SECURITY.md)。

保持传输、协议、能力、Agent 与 UI 的边界；提交聚焦的小变更，使用 Conventional Commits。
新增支持需要已审阅清单、固定来源、回放测试、失败处理和诚实的用户可见状态，
并不意味着维护者必须再买一份硬件重复测试。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.85.0 check --workspace --all-features --locked
pnpm --dir docs install --frozen-lockfile
python3 tools/build_docs.py
pnpm --dir docs exec playwright install chromium
pnpm --dir docs run test:site
```

使用 `docs/.node-version` 中的 Node LTS 主版本和 `docs/package.json` 固定的 pnpm 版本。
构建在 `target/site/` 根目录生成英文手册，在 `zh-CN/` 下生成中文手册，
并在 `api/` 下补入 rustdoc 的库页面，不需要联网下载翻译。
使用 `pnpm --dir docs run preview`，然后打开
`http://localhost:4321/razers/` 本地预览；线上 404 页按 `/razers/` 项目路径定位资源。
API 参考面向工作区库，命令行用法放在手册。
rustdoc 自带导航保留上游英文，项目 API 注释提供双语说明，总览在中英文 Starlight 手册内呈现。

英文与中文文案、章节请一起更新，遵循[翻译指南](/razers/zh-CN/localization/)、[安全政策](/razers/zh-CN/safety/)
和[证据政策](/razers/zh-CN/evidence-policy/)。不要上传私人标识或真实输入流。
安全漏洞请使用仓库的 GitHub 私密漏洞报告入口，不要公开发 Issue。

你可以[不持有硬件也贡献上游证据](/razers/zh-CN/contribute-evidence/)。
提问、功能建议与行为准则见[社区与支持](/razers/zh-CN/community/)。
