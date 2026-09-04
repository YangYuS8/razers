// SPDX-License-Identifier: GPL-2.0-or-later
import { expect, test } from '@playwright/test';
import legacyAnchors from './fixtures/mdbook-anchors.json' with { type: 'json' };

test('all pre-migration heading anchors remain addressable', async ({ request }) => {
  for (const [route, anchors] of Object.entries(legacyAnchors.headings)) {
    const response = await request.get(route);
    expect(response.ok(), route).toBe(true);
    const html = await response.text();
    for (const anchor of anchors) expect(html, `${route}#${anchor}`).toContain(`id="${anchor}"`);
  }
});

test('both languages render useful content with a development warning', async ({ page }) => {
  for (const [locale, title, warning] of [
    ['en', 'Welcome to RazeRS', 'Development docs'],
    ['zh-CN', '欢迎使用 RazeRS', '开发版文档'],
  ]) {
    await page.goto(`/razers/${locale}/`);
    await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible();
    await expect(page.locator('.development-notice')).toContainText(warning);
    await expect(page.locator('html')).toHaveAttribute('lang', locale);
  }
});

test('language switch preserves the current chapter', async ({ page }) => {
  await page.goto('/razers/en/architecture/');
  await page.getByRole('combobox', { name: 'Select language' }).selectOption('/razers/zh-CN/architecture/');
  await expect(page).toHaveURL(/\/razers\/zh-CN\/architecture\/$/);
  await expect(page.getByRole('heading', { name: '架构', exact: true })).toBeVisible();
});

for (const [query, title] of [
  ['回报率', '架构'],
  ['设备设置', '桌面应用与语言'],
  ['DPI', '来源记录'],
]) {
  test(`Chinese search handles ${query}`, async ({ page }) => {
    await page.goto('/razers/zh-CN/');
    await page.getByRole('button', { name: /搜索|Search/ }).click();
    await page.getByRole('dialog').getByRole('textbox', { name: '搜索', exact: true }).fill(query);
    await expect(page.locator('.pagefind-ui__result-link').filter({ hasText: title }).first()).toBeVisible();
  });
}

test('English full-text search works', async ({ page }) => {
  await page.goto('/razers/en/');
  await page.getByRole('button', { name: /Search/ }).click();
  await page.getByRole('dialog').getByRole('textbox', { name: 'Search', exact: true }).fill('firmware');
  await expect(page.locator('.pagefind-ui__result-link').first()).toBeVisible();
});

test('legacy bookmarks and API entry remain usable', async ({ page }) => {
  await page.goto('/razers/zh-CN/getting-started.html#下载与启动');
  await expect(page).toHaveURL(/\/razers\/zh-CN\/getting-started\/#/);
  await expect(page.getByRole('heading', { name: '快速开始', exact: true })).toBeVisible();
  await expect(page.locator('[id="下载与启动"]')).toBeVisible();
  for (const [locale, fragment] of [
    ['en', 'localization-and-documentation-maintenance'],
    ['zh-CN', '翻译与文档维护'],
    ['en', 'build-the-site'],
    ['zh-CN', '构建文档站'],
  ]) {
    await page.goto(`/razers/${locale}/localization.html#${fragment}`);
    await expect(page).toHaveURL(new RegExp(`/razers/${locale}/localization/#`));
    await expect(page.locator(`[id="${fragment}"]`)).toBeAttached();
  }
  await page.goto('/razers/api/');
  await expect(page.getByRole('link', { name: 'razers-transport', exact: true })).toBeVisible();
});

test('Chinese search labels and empty results are translated', async ({ page }) => {
  await page.goto('/razers/zh-CN/');
  await page.getByRole('button', { name: '搜索', exact: true }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog.getByRole('search', { name: '搜索本站' })).toBeVisible();
  await dialog.getByRole('textbox', { name: '搜索', exact: true }).fill('qzxvwjkqzxvwjk');
  await expect(dialog.getByText('未找到“qzxvwjkqzxvwjk”的结果', { exact: true })).toBeVisible();
  await dialog.getByRole('button', { name: '清除', exact: true }).click();
  await expect(dialog.getByRole('textbox')).toHaveValue('');
  await page.keyboard.press('Escape');
  await expect(dialog).not.toBeVisible();
});

test('not-found page offers both handbook entrances', async ({ page }) => {
  await page.goto('/razers/404.html');
  await expect(page.getByRole('heading', { name: 'Page not found / 页面未找到', exact: true })).toBeVisible();
  await expect(page.getByRole('link', { name: '中文首页', exact: true })).toHaveAttribute('href', '/razers/zh-CN/');
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute('content', 'noindex');
});

test('mobile layout has navigation and no horizontal page overflow', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const chapter of ['device-schema', 'cli', 'first-run']) {
    await page.goto(`/razers/zh-CN/${chapter}/`);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), chapter).toBe(true);
  }
  await expect(page.getByRole('heading', { name: '完成第一次只读体验', exact: true })).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath('mobile-handbook.png'), fullPage: true });
  await page.getByRole('button', { name: /菜单|Menu/ }).click();
  await expect(page.getByRole('link', { name: '不持有硬件也能贡献证据', exact: true })).toBeVisible();
});

test('reading and search load no third-party resources', async ({ page }, testInfo) => {
  const requested: string[] = [];
  const errors: string[] = [];
  page.on('request', request => requested.push(request.url()));
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/razers/zh-CN/first-run/');
  const origin = new URL(page.url()).origin;
  await page.getByRole('combobox', { name: '选择主题' }).selectOption('light');
  await page.screenshot({ path: testInfo.outputPath('desktop-handbook.png'), fullPage: true });
  await page.getByRole('button', { name: '搜索', exact: true }).click();
  await page.getByRole('dialog').getByRole('textbox', { name: '搜索', exact: true }).fill('DPI');
  await expect(page.locator('.pagefind-ui__result-link').first()).toBeVisible();
  expect(requested.filter(url => /^https?:/.test(url) && new URL(url).origin !== origin)).toEqual([]);
  expect(errors).toEqual([]);
});
