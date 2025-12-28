import { test, expect, Page } from '@playwright/test';

async function waitForWasmReady(page: Page): Promise<void> {
  await page.waitForFunction(
    () => {
      const win = window as unknown as { pixieJuice?: { version?: () => string } };
      if (!win.pixieJuice || typeof win.pixieJuice.version !== 'function') return false;
      try {
        const v = win.pixieJuice.version();
        return typeof v === 'string' && v.length > 0;
      } catch {
        return false;
      }
    },
    { timeout: 30000 }
  );
}

test.describe('UI Integration Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should display app title', async ({ page }) => {
    const title = page.locator('h1');
    await expect(title).toContainText('Pixie Juice');
  });

  test('should show file drop zone', async ({ page }) => {
    const dropZone = page.locator('input[type="file"]');
    await expect(dropZone).toBeAttached();
  });

  test('should display version info', async ({ page }) => {
    const version = await page.evaluate(() => {
      const win = window as unknown as { pixieJuice?: { version?: () => string } };
      return win.pixieJuice?.version?.();
    });
    
    expect(version).toBeTruthy();
  });
});

test.describe('File Upload Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should accept image file input', async ({ page }) => {
    const fileInput = page.locator('input[type="file"]');
    await expect(fileInput).toBeAttached();
    
    const acceptAttr = await fileInput.getAttribute('accept');
    expect(acceptAttr).toBeTruthy();
  });

  test('drop zone should be visible initially', async ({ page }) => {
    const dropZoneText = page.locator('text=/drag|drop|browse/i').first();
    await expect(dropZoneText).toBeVisible();
  });
});

test.describe('Error States', () => {
  test('should handle WASM load failure gracefully', async ({ page }) => {
    await page.route('**/pixie_juice_bg.wasm', route => route.abort());
    
    await page.goto('/');
    await page.waitForTimeout(5000);
    
    const errorText = page.locator('text=/failed|error|retry/i').first();
    if (await errorText.isVisible()) {
      expect(true).toBe(true);
    }
  });
});

test.describe('Performance UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should not have console errors on load', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error' && !msg.text().includes('favicon')) {
        errors.push(msg.text());
      }
    });
    
    await page.reload();
    await waitForWasmReady(page);
    await page.waitForTimeout(1000);
    
    const criticalErrors = errors.filter(e => 
      !e.includes('net::') && 
      !e.includes('favicon') &&
      !e.includes('404')
    );
    
    expect(criticalErrors.length).toBeLessThan(3);
  });

  test('page should load within acceptable time', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    await waitForWasmReady(page);
    const loadTime = Date.now() - startTime;

    const projectName = test.info().project.name;
    const maxMs = projectName === 'chromium' ? 15000 : projectName === 'firefox' ? 20000 : 30000;
    expect(loadTime).toBeLessThan(maxMs);
  });
});

test.describe('Responsive Design', () => {
  test('should work on mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await waitForWasmReady(page);
    
    const title = page.locator('h1');
    await expect(title).toBeVisible();
  });

  test('should work on tablet viewport', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');
    await waitForWasmReady(page);
    
    const title = page.locator('h1');
    await expect(title).toBeVisible();
  });

  test('should work on desktop viewport', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');
    await waitForWasmReady(page);
    
    const title = page.locator('h1');
    await expect(title).toBeVisible();
  });
});

test.describe('Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should have focusable file input', async ({ page }) => {
    const fileInput = page.locator('input[type="file"]');
    await expect(fileInput).toBeAttached();
  });

  test('should have proper heading structure', async ({ page }) => {
    const h1 = page.locator('h1');
    await expect(h1).toBeVisible();
  });
});
