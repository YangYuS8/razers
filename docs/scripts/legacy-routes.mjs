// SPDX-License-Identifier: GPL-2.0-or-later
// Frozen compatibility manifest: these are URLs shipped by the mdBook site.
import { writeFile } from 'node:fs/promises';
export const legacyChapters = [
  'index', 'getting-started', 'application', 'cli', 'troubleshooting',
  'product-principles', 'safety', 'architecture', 'device-schema',
  'evidence-policy', 'provenance', 'ipc', 'localization', 'contributing',
  'releases', 'api',
];

export function legacyRedirects() {
  return Object.fromEntries(['en', 'zh-CN'].flatMap(locale =>
    legacyChapters.map(chapter => [
      `/${locale}/${chapter}.html`,
      `/${locale}/${chapter === 'index' ? '' : `${chapter}/`}`,
    ]),
  ));
}

export function legacyCompatibility() {
  return {
    name: 'razers-legacy-links',
    hooks: {
      'astro:build:done': async ({ dir }) => {
        for (const [from, to] of Object.entries(legacyRedirects())) {
          // Starlight already emits these exact index files. Do not overwrite them.
          if (from.endsWith('/index.html')) continue;
          const destination = `/razers${to}`;
          const locale = from.startsWith('/zh-CN/') ? 'zh-CN' : 'en';
          await writeFile(new URL(from.slice(1), dir),
            `<!doctype html><html lang="${locale}"><meta charset="utf-8">` +
            `<title>RazeRS</title><link rel="canonical" href="${destination}">` +
            `<meta http-equiv="refresh" content="0;url=${destination}">` +
            `<script>location.replace(${JSON.stringify(destination)}+location.hash)</script>` +
            `<body data-pagefind-ignore><a href="${destination}">Continue / 继续阅读</a></body></html>`);
        }
      },
    },
  };
}
