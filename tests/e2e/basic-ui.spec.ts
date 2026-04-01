import { test, expect } from '@playwright/test';

/**
 * E2Eテスト - アプリ起動と基本UI検証
 */
test.describe('Factorio Mod Translator E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('アプリが正常に起動し、タイトルが表示される', async ({ page }) => {
    await expect(page.locator('#app-title')).toContainText('Factorio Mod Translator');
  });

  test('タブが正しく表示される', async ({ page }) => {
    const tabs = page.locator('#tabs .tab');
    await expect(tabs).toHaveCount(5);
    await expect(tabs.nth(0)).toHaveText('Modを選択');
    await expect(tabs.nth(1)).toHaveText('翻訳');
    await expect(tabs.nth(3)).toHaveText('履歴');
    await expect(tabs.nth(4)).toHaveText('設定');
  });

  test('Mod選択画面が表示される', async ({ page }) => {
    await expect(page.locator('#mod-selection')).toBeVisible();
    await expect(page.locator('#btn-browse-folder')).toBeVisible();
    await expect(page.locator('#btn-browse-zip')).toBeVisible();
  });

  test('タブ切り替えが動作する', async ({ page }) => {
    await page.locator('.tab[data-view="translation-preview"]').click();
    await expect(page.locator('#translation-preview')).toHaveClass(/active/);
    await expect(page.locator('#mod-selection')).not.toHaveClass(/active/);

    await page.locator('.tab[data-view="history"]').click();
    await expect(page.locator('#history')).toHaveClass(/active/);

    await page.locator('.tab[data-view="settings"]').click();
    await expect(page.locator('#settings')).toHaveClass(/active/);
  });

  test('設定画面で翻訳エンジンが選択できる', async ({ page }) => {
    await page.locator('.tab[data-view="settings"]').click();
    const engineSelect = page.locator('#engine-select');
    await expect(engineSelect).toBeVisible();
    await expect(engineSelect).toHaveValue('DeepL');
  });

  test('翻訳画面で言語選択が表示される', async ({ page }) => {
    await page.locator('.tab[data-view="translation-preview"]').click();
    await expect(page.locator('#src-lang-select')).toBeVisible();
    await expect(page.locator('#target-lang-select')).toBeVisible();
  });
});
