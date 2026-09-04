# Application icon / 应用图标

The icon reuses this project's own mark in `docs/public/favicon.svg`, under the
repository's GPL-2.0-or-later license. PNG/ICO files are committed build inputs;
the installer and application build do not need graphics tools.

图标复用本项目 `docs/public/favicon.svg` 中的标识，沿用 GPL-2.0-or-later。
PNG/ICO 作为构建输入提交，应用与安装包构建不需要图形工具。

Regenerate after changing the SVG / 修改 SVG 后重新生成：

```bash
rsvg-convert --width 512 --height 512 docs/public/favicon.svg --output assets/icons/razers.png
rsvg-convert --width 1024 --height 1024 docs/public/favicon.svg --output assets/icons/razers@2x.png
magick assets/icons/razers@2x.png -define icon:auto-resize=256,128,64,48,32,16 assets/icons/razers.ico
```

The `@2x` suffix identifies the 1024-pixel Retina image to the macOS ICNS encoder.
`@2x` 后缀让 macOS ICNS 编码器正确识别 1024 像素的 Retina 图像。
