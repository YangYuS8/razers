// SPDX-License-Identifier: GPL-2.0-or-later
import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { legacyChapters, legacyRedirects } from '../scripts/legacy-routes.mjs';

test('every shipped mdBook page has a same-language redirect', () => {
  const redirects = legacyRedirects();
  assert.equal(Object.keys(redirects).length, legacyChapters.length * 2);
  for (const locale of ['en', 'zh-CN']) {
    assert.equal(redirects[`/${locale}/index.html`], `/${locale}/`);
    for (const chapter of legacyChapters.filter(name => name !== 'index')) {
      assert.equal(redirects[`/${locale}/${chapter}.html`], `/${locale}/${chapter}/`);
    }
  }
});

test('custom interface translations have matching keys and placeholders', async () => {
  const read = async locale => JSON.parse(await readFile(new URL(`../src/content/i18n/${locale}.json`, import.meta.url), 'utf8'));
  const en = await read('en');
  const zh = await read('zh-CN');
  assert.deepEqual(Object.keys(en).sort(), Object.keys(zh).sort());
  const placeholders = text => [...text.matchAll(/\[[A-Z_]+\]|\{\{\w+\}\}/g)].map(match => match[0]).sort();
  for (const key of Object.keys(en)) {
    assert.ok(en[key].trim() && zh[key].trim(), key);
    assert.deepEqual(placeholders(en[key]), placeholders(zh[key]), key);
  }
});
