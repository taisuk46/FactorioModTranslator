import { test, expect } from '@playwright/test';

/**
 * E2Eテスト - 用語集機能
 * 注: 用語集タブは初期状態で非表示(display: none)のため、DOM存在確認のみ行う
 */
test.describe('用語集機能', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('用語集タブがDOMに存在する', async ({ page }) => {
    const glossaryTab = page.locator('.tab[data-view="glossary"]');
    await expect(glossaryTab).toBeAttached();
    await expect(glossaryTab).toContainText('用語集');
  });

  test('用語集セクションがDOMに存在する', async ({ page }) => {
    const glossarySection = page.locator('#glossary');
    await expect(glossarySection).toBeAttached();
  });

  test('用語追加ボタンがDOMに存在する', async ({ page }) => {
    const addBtn = page.locator('#btn-add-glossary');
    await expect(addBtn).toBeAttached();
    await expect(addBtn).toContainText('+ New Term');
  });
});
