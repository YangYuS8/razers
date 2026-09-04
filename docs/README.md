# Documentation development / 文档开发

The handbook uses pnpm + Astro Starlight. Rustdoc is built alongside it and the
combined static artifact is published to GitHub Pages. See the
[English maintenance guide](src/content/docs/en/localization.md) or
[中文维护指南](src/content/docs/zh-CN/localization.md) for prerequisites and policy.

From the repository root / 在仓库根目录执行：

```bash
pnpm --dir docs install --frozen-lockfile
pnpm --dir docs run dev
```

Full production checks / 完整产物检查：

```bash
python3 -m unittest discover -s tools/tests
pnpm --dir docs run test
pnpm --dir docs run check
python3 tools/build_docs.py
pnpm --dir docs exec playwright install chromium
pnpm --dir docs run test:site
pnpm --dir docs run preview
```

Preview: `http://localhost:4321/razers/`. `target/site/` is generated, never edited.
`src/content/docs/{en,zh-CN}/` contains paired chapters; `astro.config.mjs` owns
navigation; `scripts/legacy-routes.mjs` preserves the frozen list of old URLs.

预览地址如上。正文按相同文件名成对维护，导航只维护一份；旧链接清单不随新章节增长。
不要直接修改生成物，翻译检查不能代替对含义的审阅。
