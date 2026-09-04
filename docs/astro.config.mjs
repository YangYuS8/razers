// SPDX-License-Identifier: GPL-2.0-or-later
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { legacyCompatibility } from './scripts/legacy-routes.mjs';

export default defineConfig({
  site: 'https://yangyus8.top',
  base: '/razers',
  trailingSlash: 'always',
  outDir: '../target/site',
  integrations: [legacyCompatibility(), starlight({
    title: 'RazeRS',
    description: 'Local, ad-free Razer peripheral software. 本地运行、无广告的雷蛇外设管理项目。',
    defaultLocale: 'en',
    locales: {
      en: { label: 'English', lang: 'en' },
      'zh-CN': { label: '简体中文', lang: 'zh-CN' },
    },
    social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/YangYuS8/razers' }],
    editLink: { baseUrl: 'https://github.com/YangYuS8/razers/edit/main/' },
    lastUpdated: true,
    customCss: ['./src/styles/custom.css'],
    components: {
      PageTitle: './src/components/PageTitle.astro',
      Head: './src/components/Head.astro',
    },
    sidebar: [
      { label: 'Start here', translations: { 'zh-CN': '开始使用' }, items: [
        'index', 'getting-started', 'first-run',
      ] },
      { label: 'How-to guides', translations: { 'zh-CN': '操作指南' }, items: [
        'application', 'troubleshooting', 'contribute-evidence',
      ] },
      { label: 'Reference', translations: { 'zh-CN': '参考资料' }, items: [
        'cli', 'device-schema', 'ipc', 'api',
      ] },
      { label: 'Design and policies', translations: { 'zh-CN': '设计与政策' }, items: [
        'product-principles', 'architecture', 'safety', 'evidence-policy', 'provenance',
      ] },
      { label: 'Contribute', translations: { 'zh-CN': '参与贡献' }, items: [
        'contributing', 'localization', 'releases', 'community',
      ] },
    ],
  })],
});
