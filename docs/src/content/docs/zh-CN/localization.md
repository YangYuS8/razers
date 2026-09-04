---
title: "翻译与文档维护"
description: "通过自动检查维护内置翻译和中英文 Starlight 手册。"
---

`razers-i18n/locales/en.json` 与 `zh-CN.json` 是内置的 gettext 风格翻译目录：
英文源文案用作键，值是显示模板。占位符为 `{0}`、`{1}` 等，数字格式由调用者预先处理。
参数只插入一次，设备名称中的花括号不会再次当成模板解析。

请在同一 PR 中添加两种翻译。测试要求键集合和占位符集合完全一致。
未知键回退到英文原文，但不能以此为由漏译。按完整句子翻译，不拼接英文词片段。
数量使用完整的标签式表达，避免把英文复数后缀带入中文。

桌面保存 `auto`、`en`、`zh-CN` 选择，而不是只保存解析后的语言。
协议字段、能力标识、来源声明与技术诊断不会在传输层翻译。
v1 新增可选的 `evidence_source_count` 字段支持本地化的证据数量，同时兼容旧客户端。

## 手册维护流程

英文和中文正文分别位于 `docs/src/content/docs/en/` 与
`docs/src/content/docs/zh-CN/`，文件名一一对应。每页在 YAML frontmatter 中填写
`title` 和 `description`。导航仅在 `docs/astro.config.mjs` 维护一次，并提供分组名翻译，
不再分别维护两份 SUMMARY。链接使用 `/razers/en/getting-started/` 或
`/razers/zh-CN/getting-started/` 等路径。切换语言保留章节，但不保留可能不同的标题锚点。

Pagefind 在构建时索引两种语言，搜索在浏览器本地运行，支持中文分词，不向服务发送查询。
浏览器回归测试覆盖中英文关键词、语言切换、旧链接及移动端导航。
rustdoc 保留独立搜索和上游英文导航，项目 API 注释与 API 入口提供双语说明。
不要加入远程字体、统计脚本、运行时翻译服务或追踪。

每个 PR 拒绝缺少对应语言页面的改动，并在只有一侧正文变化时发出审阅提醒；
根目录的两份 README 也会检查单侧改动。不需要手工维护另一份翻译状态表。
此检查只能发现漏同步，不能证明语义一致，仍需审阅译文。
如果只是单语言错字，检查另一语言后可以保留提醒，无需通过无意义改动、状态注释或特殊标签消除它。

<span id="构建文档站" aria-hidden="true" data-pagefind-ignore></span>

## 构建与预览

准备 `docs/.node-version` 指定的 Node LTS 主版本，以及 `docs/package.json` 中固定的
pnpm 版本。rustdoc 还需要 Rust 和工作区对应平台的构建依赖。在仓库根目录执行：

```bash
pnpm --dir docs install --frozen-lockfile
pnpm --dir docs run check
python3 -m unittest discover -s tools/tests
python3 tools/build_docs.py
pnpm --dir docs exec playwright install chromium
pnpm --dir docs run test:site
pnpm --dir docs run preview --host 127.0.0.1
```

打开 `http://localhost:4321/razers/`。仅编辑手册时可用 `pnpm --dir docs run dev`；
API 与双语站点入口由完整的 Python 构建组装。`target/site/` 是生成物，不要直接编辑。
若未全局安装 pnpm，`build_docs.py` 可通过 npm 调用固定版本，不更改全局工具配置。

完整构建使用冻结的依赖锁文件，生成 Starlight 后执行
`cargo doc --workspace --lib --all-features --no-deps --locked`，拒绝 rustdoc 警告，
并验证生成的本地链接与标题锚点，包括 API 链接。新增库 API 示例应尽量使用可执行的 rustdoc 测试。

## 文档技术选型记录

2026 年 9 月，我们将 mdBook 替换为 pnpm + Astro Starlight，统一双语导航、搜索与响应式阅读体验。
代价是增加 Node 工具链与依赖锁文件，收益是移除自行维护的中文搜索及界面翻译适配器。
rustdoc 继续生成 API，GitHub Pages 继续托管站点。既有 `/en/*.html`、`/zh-CN/*.html`
与 `/api/` 链接保留兼容，新链接使用目录式章节地址。旧地址清单是固定的兼容数据，
不是另一份需要随新章节更新的目录。

只有 `main` 的推送或在 `main` 上手动运行才通过 `github-pages` 环境、官方 Pages
产物与 OIDC 动作部署，PR 验证不持有部署凭据。站点跟随开发进度，不是发行版文档归档。
依赖和部署自动化见[维护政策](/razers/zh-CN/releases/)。
