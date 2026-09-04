# Application icon / 应用图标

The icon reuses this project's own mark in `docs/public/favicon.svg`, under the
repository's GPL-2.0-or-later license. PNG/ICO files are committed build inputs;
the installer and application build do not need graphics tools.

图标复用本项目 `docs/public/favicon.svg` 中的标识，沿用 GPL-2.0-or-later。
PNG/ICO 作为构建输入提交，应用与安装包构建不需要图形工具。

Regenerate after changing the SVG / 修改 SVG 后重新生成：

```bash
rsvg-convert --width 1024 --height 1024 docs/public/favicon.svg --output assets/icons/razers.png
magick assets/icons/razers.png -define icon:auto-resize=256,128,64,48,32,16 assets/icons/razers.ico
```
