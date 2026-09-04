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
`src/content/docs/{en,zh-CN}/` contains paired Markdown/MDX chapters;
`astro.config.mjs` owns navigation. English is served directly at `/razers/`, Chinese
at `/razers/zh-CN/`. Both API overviews use Starlight and generate their crate list
from Cargo metadata. The full build adds rustdoc's crate pages below `/razers/api/`.

预览地址如上。Markdown/MDX 正文按相同文件名成对维护，导航只维护一份。
根地址直接打开英文正文，中文位于 `/razers/zh-CN/`；API 总览使用同一套 Starlight 界面，
crate 清单由 Cargo 自动生成，完整构建将 rustdoc 的库页面放在 `/razers/api/` 下。
不要直接修改生成物，翻译检查不能代替对含义的审阅。
