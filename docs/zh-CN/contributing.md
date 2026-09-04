# 参与贡献

请先阅读[仓库贡献指南](https://github.com/YangYuS8/razers/blob/main/CONTRIBUTING.md)
与[安全报告政策](https://github.com/YangYuS8/razers/blob/main/SECURITY.md)。

保持传输、协议、能力、Agent 与 UI 的边界；提交聚焦的小变更，使用 Conventional Commits。
新增支持需要已审阅清单、固定来源、回放测试、失败处理和诚实的用户可见状态，
并不意味着维护者必须再买一份硬件重复测试。

```bash
cargo install mdbook --version 0.5.4 --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.85.0 check --workspace --all-features --locked
python tools/build_docs.py
```

mdBook 版本固定在 `tools/docs-tools.toml`，也可用 `MDBOOK` 指向对应版本的程序。
构建生成 `target/site/en`、`zh-CN` 和 `api`，不需要联网下载翻译。
使用 `python -m http.server 8000 --directory target/site`，然后打开
`http://localhost:8000/` 本地预览；线上 404 页按 `/razers/` 项目路径定位资源。
API 参考面向工作区库，命令行用法放在手册。
rustdoc 自带导航保留上游英文，项目 API 注释与入口提供中英双语。

英文与中文文案、章节请一起更新，遵循[翻译指南](localization.md)、[安全政策](safety.md)
和[证据政策](evidence-policy.md)。不要上传私人标识或真实输入流。
安全漏洞请使用仓库的 GitHub 私密漏洞报告入口，不要公开发 Issue。
