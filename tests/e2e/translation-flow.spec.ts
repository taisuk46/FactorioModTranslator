import { test, expect } from '@playwright/test';

/**
 * E2Eテスト - Mod翻訳フロー
 */
test.describe('Mod翻訳フロー', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('Modをロードして翻訳プレビューが表示される', async ({ page }) => {
    await page.locator('#btn-browse-folder').click();

    await page.waitForTimeout(500);

    await page.locator('.tab[data-view="translation-preview"]').click();

    const translationList = page.locator('#translation-list');
    await expect(translationList).toBeVisible();

    const fileHeaders = translationList.locator('.file-header');
    await expect(fileHeaders.first()).toContainText('test.cfg');
  });

  test('翻訳実行ボタンが表示される', async ({ page }) => {
    await page.locator('#btn-browse-folder').click();
    await page.waitForTimeout(500);
    await page.locator('.tab[data-view="translation-preview"]').click();

    await expect(page.locator('#btn-translate')).toBeVisible();
    await expect(page.locator('#btn-translate')).toContainText('Translate All');
  });

  test('保存ボタンが表示される', async ({ page }) => {
    await page.locator('#btn-browse-folder').click();
    await page.waitForTimeout(500);
    await page.locator('.tab[data-view="translation-preview"]').click();

    await expect(page.locator('#btn-save-mod')).toBeVisible();
    await expect(page.locator('#btn-save-mod')).toContainText('Save Translated Mod');
  });

  test('言語選択オプションが正しい', async ({ page }) => {
    await page.locator('.tab[data-view="translation-preview"]').click();

    const targetLangSelect = page.locator('#target-lang-select');
    await expect(targetLangSelect.locator('option[value="ja"]')).toHaveText('Japanese');
    await expect(targetLangSelect.locator('option[value="en"]')).toHaveText('English');
  });
});
