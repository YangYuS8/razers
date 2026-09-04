// SPDX-License-Identifier: GPL-2.0-or-later
import { defineCollection } from 'astro:content';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
  docs: defineCollection({
    // Astro's default slugger lowercases paths, but our published locale is zh-CN.
    loader: docsLoader({ generateId: ({ entry }) => entry.replace(/\.(md|mdx)$/, '').replace(/\/index$/, '') }),
    schema: docsSchema(),
  }),
  i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
};
