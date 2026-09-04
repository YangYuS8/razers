// SPDX-License-Identifier: GPL-2.0-or-later
import { expect, test } from '@playwright/test';
test('root is the handbook itself, even without JavaScript', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ baseURL, javaScriptEnabled: false });
  try {
    const page = await context.newPage();
    const response = await page.goto('/razers/');
    expect(response?.status()).toBe(200);
    expect(response?.request().redirectedFrom()).toBeNull();
    await expect(page).toHaveURL(/\/razers\/$/);
    await expect(page.getByRole('heading', { name: 'Welcome to RazeRS', exact: true })).toBeVisible();
    await expect(page.getByRole('combobox', { name: 'Select language' })).toBeVisible();
    await expect(page.locator('meta[http-equiv="refresh"]')).toHaveCount(0);
    await expect(page.getByRole('navigation', { name: 'Documentation languages and API' })).toHaveCount(0);
    await page.getByRole('link', { name: 'installation', exact: true }).click();
    await expect(page).toHaveURL(/\/razers\/getting-started\/$/);
    await expect(page.getByRole('heading', { name: 'Getting started', exact: true })).toBeVisible();
  } finally {
    await context.close();
  }
});

test('both languages render useful content with a development warning', async ({ page }) => {
  for (const [path, locale, title, warning] of [
    ['/razers/', 'en', 'Welcome to RazeRS', 'Development docs'],
    ['/razers/zh-CN/', 'zh-CN', '欢迎使用 RazeRS', '开发版文档'],
  ]) {
    await page.goto(path);
    await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible();
    await expect(page.locator('.development-notice')).toContainText(warning);
    await expect(page.locator('html')).toHaveAttribute('lang', locale);
  }
});

test('language switch preserves the current chapter in both directions', async ({ page }) => {
  await page.goto('/razers/architecture/');
  await page.getByRole('combobox', { name: 'Select language' }).selectOption('/razers/zh-CN/architecture/');
  await expect(page).toHaveURL(/\/razers\/zh-CN\/architecture\/$/);
  await expect(page.getByRole('heading', { name: '架构', exact: true })).toBeVisible();
  await page.getByRole('combobox', { name: '选择语言' }).selectOption('/razers/architecture/');
  await expect(page).toHaveURL(/\/razers\/architecture\/$/);
  await expect(page.getByRole('heading', { name: 'Architecture', exact: true })).toBeVisible();
});

test('home language switch and brand links stay within the handbook', async ({ page }) => {
  await page.goto('/razers/');
  await page.getByRole('combobox', { name: 'Select language' }).selectOption('/razers/zh-CN/');
  await expect(page).toHaveURL(/\/razers\/zh-CN\/$/);
  await expect(page.getByRole('heading', { name: '欢迎使用 RazeRS', exact: true })).toBeVisible();
  await page.getByRole('link', { name: 'RazeRS', exact: true }).click();
  await expect(page).toHaveURL(/\/razers\/zh-CN\/$/);
  await page.getByRole('combobox', { name: '选择语言' }).selectOption('/razers/');
  await expect(page).toHaveURL(/\/razers\/$/);
  await expect(page.getByRole('heading', { name: 'Welcome to RazeRS', exact: true })).toBeVisible();
  await page.getByRole('link', { name: 'RazeRS', exact: true }).click();
  await expect(page).toHaveURL(/\/razers\/$/);
});

test('edit links point to actual paired Markdown and MDX sources', async ({ page }) => {
  for (const [route, source] of [
    ['/razers/', 'en/index.md'],
    ['/razers/zh-CN/', 'zh-CN/index.md'],
    ['/razers/api/', 'en/api.mdx'],
    ['/razers/zh-CN/api/', 'zh-CN/api.mdx'],
  ]) {
    await page.goto(route);
    await expect(page.locator('a[href*="github.com/YangYuS8/razers/edit/"]'))
      .toHaveAttribute('href', `https://github.com/YangYuS8/razers/edit/main/docs/src/content/docs/${source}`);
  }
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
    const links = page.locator('.pagefind-ui__result-link');
    const result = links.filter({ hasText: title }).first();
    await expect(links.first()).toBeVisible();
    // New handbook chapters can move a relevant result beyond Pagefind's first
    // five entries. Exercise real pagination; still require the specific page.
    for (let pages = 0; pages < 5 && !(await result.isVisible()); pages++) {
      const more = page.getByRole('dialog').getByRole('button', { name: '加载更多结果', exact: true });
      if (!(await more.isVisible())) break;
      const before = await links.count();
      await more.click();
      await expect.poll(() => links.count()).toBeGreaterThan(before);
    }
    await expect(result).toBeVisible();
    await result.click();
    await expect(page).toHaveURL(/\/razers\/zh-CN\//);
    await expect(page.getByRole('heading', { name: title, exact: true }).first()).toBeVisible();
  });
}

test('English full-text search works', async ({ page }) => {
  await page.goto('/razers/');
  await page.getByRole('button', { name: /Search/ }).click();
  await page.getByRole('dialog').getByRole('textbox', { name: 'Search', exact: true }).fill('firmware');
  await expect(page.locator('.pagefind-ui__result-link').first()).toBeVisible();
});

test('released desktop Help entrance opens the root handbook', async ({ page }) => {
  await page.goto('/razers/en/');
  await expect(page).toHaveURL(/\/razers\/$/);
  await expect(page.getByRole('heading', { name: 'Welcome to RazeRS', exact: true })).toBeVisible();
});

test('both API overviews use Starlight and link directly to rustdoc', async ({ page }) => {
  for (const [path, title, search] of [
    ['/razers/api/', 'Rust API reference', 'Search'],
    ['/razers/zh-CN/api/', 'Rust API 参考', '搜索'],
  ]) {
    await page.goto(path);
    await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: new RegExp(search) })).toBeVisible();
    await expect(page.getByRole('link', { name: 'razers-transport', exact: true }))
      .toHaveAttribute('href', '/razers/api/razers_transport/index.html');
    await page.getByRole('link', { name: 'razers-transport', exact: true }).click();
    await expect(page).toHaveURL(/\/razers\/api\/razers_transport\/index\.html$/);
    await expect(page.locator('#main-content')).toContainText('razers_transport');
  }
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
  await page.getByRole('navigation', { name: 'Documentation languages / 文档语言' })
    .getByRole('link', { name: '简体中文', exact: true }).click();
  await expect(page).toHaveURL(/\/razers\/zh-CN\/$/);
  await expect(page.getByRole('heading', { name: '欢迎使用 RazeRS', exact: true })).toBeVisible();
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

test('mobile root entrances expose content and the site menu', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const [path, title, menu, firstRun] of [
    ['/razers/', 'Welcome to RazeRS', 'Menu', 'Your first read-only session'],
    ['/razers/zh-CN/', '欢迎使用 RazeRS', '菜单', '完成第一次只读体验'],
  ]) {
    await page.goto(path);
    await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
    await page.screenshot({ path: testInfo.outputPath(`${menu}-root.png`), fullPage: true });
    await page.getByRole('button', { name: menu, exact: true }).click();
    await page.getByRole('link', { name: firstRun, exact: true }).click();
    await expect(page.getByRole('heading', { name: firstRun, exact: true })).toBeVisible();
  }
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
