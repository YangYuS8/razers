# 翻译与文档维护

英文使用 mdBook 内置搜索；中文使用本地生成的全文子串索引，支持查找连续中文句子中的词语。
搜索完全在浏览器本地完成，不向第三方发送关键词。中文目录、主题和快捷键帮助在构建时翻译；
rustdoc 自带导航仍为英文，项目 API 注释提供双语说明。

`razers-i18n/locales/en.json` 与 `zh-CN.json` 是内置的 gettext 风格翻译目录：
英文源文案用作键，值是显示模板。占位符为 `{0}`、`{1}` 等，数字格式由调用者预先处理。
参数只插入一次，设备名称中的花括号不会再次当成模板解析。

请在同一 PR 中添加两种翻译。测试要求键集合和占位符集合完全一致。
未知键回退到英文原文，但不能以此为由漏译。按完整句子翻译，不拼接英文词片段。
数量使用完整的标签式表达，避免把英文复数后缀带入中文。

桌面保存 `auto`、`en`、`zh-CN` 选择，而不是只保存解析后的语言。
协议字段、能力标识、来源声明与技术诊断不会在传输层翻译。
v1 新增可选的 `evidence_source_count` 字段支持本地化的证据数量，同时兼容旧客户端。

## 构建文档站

按照 `tools/docs-requirements.txt` 固定的版本安装 mdBook：

```bash
cargo install mdbook --version 0.5.4 --locked
python tools/build_docs.py
python -m http.server 8000 --directory target/site
```

打开 `http://localhost:8000/`。CI 使用同一构建：先生成两套 mdBook，再执行
`cargo doc --workspace --all-features --no-deps --locked`，并将 rustdoc 警告视为错误。
构建后检查中英章节对齐及本地 HTML 链接（包括 API 链接）。Rust API 符号保持稳定，
同一 rustdoc 页面可包含双语的 crate、模块和 API 说明。

英文正文位于 `docs/`，中文位于 `docs/zh-CN/`，两套 SUMMARY 的章节路径必须一致。
不要加入远程字体、统计脚本或追踪服务。切换语言时保留当前章节，丢弃可能不同的标题锚点。

Documentation 工作流验证每个 PR，不授予 PR 部署权限。
只有 `main` 的推送或在 `main` 上手动运行才会通过官方 Pages 产物与 OIDC 部署动作，
发布到 `github-pages` 环境。站点包含仓库路径下的 `/en/`、`/zh-CN/` 和 `/api/`。
