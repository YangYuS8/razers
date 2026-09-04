// SPDX-License-Identifier: GPL-2.0-or-later
import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { workspaceLibraries } from '../scripts/workspace-libraries.mjs';

test('API library list is generated from documented workspace targets', () => {
  assert.ok(workspaceLibraries.length > 0);
  assert.equal(new Set(workspaceLibraries.map(item => item.href)).size, workspaceLibraries.length);
  assert.ok(workspaceLibraries.some(item => item.name === 'razers-transport'));
  for (const item of workspaceLibraries) {
    assert.match(item.name, /^razers-/);
    assert.match(item.href, /^\/razers\/api\/razers_\w+\/index\.html$/);
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
