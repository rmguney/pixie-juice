import { test, expect, Page } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const FIXTURES_DIR = path.join(__dirname, '../fixtures');

interface CompressionResult {
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  format: string;
}

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

async function uploadFile(page: Page, filePath: string): Promise<void> {
  const fileInput = page.locator('input[type="file"]');
  await fileInput.setInputFiles(filePath);
}

async function getProcessedResult(page: Page): Promise<CompressionResult | null> {
  const resultText = await page.locator('[data-testid="compression-result"]').textContent();
  if (!resultText) return null;
  
  const match = resultText.match(/(\d+(?:\.\d+)?)\s*%/);
  if (!match) return null;
  
  return {
    originalSize: 0,
    compressedSize: 0,
    compressionRatio: parseFloat(match[1]),
    format: 'unknown'
  };
}

test.describe('WASM Module Loading', () => {
  test('should load WASM module successfully', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const version = await page.evaluate(() => {
      const win = window as unknown as { pixieJuice?: { version?: () => string } };
      return win.pixieJuice?.version?.();
    });
    
    expect(version).toBeTruthy();
    expect(version).toMatch(/^\d+\.\d+\.\d+$/);
  });

  test('should not show loading state after WASM loads', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const loadingIndicator = page.locator('text=Loading WASM');
    await expect(loadingIndicator).not.toBeVisible();
  });

  test('should expose all required WASM functions', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const functions = await page.evaluate(() => {
      const win = window as unknown as { pixieJuice?: Record<string, unknown> };
      if (!win.pixieJuice) return [];
      
      return Object.keys(win.pixieJuice).filter(
        key => typeof win.pixieJuice![key] === 'function'
      );
    });
    
    const requiredFunctions = [
      'optimize_image',
      'optimize_mesh',
      'optimize_auto',
      'detect_format',
      'version',
    ];
    
    for (const fn of requiredFunctions) {
      expect(functions).toContain(fn);
    }
  });
});

test.describe('Format Detection', () => {
  test('should detect PNG format', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const format = await page.evaluate(async () => {
      const win = window as unknown as { pixieJuice?: { detect_format?: (data: Uint8Array) => string } };
      const pngHeader = new Uint8Array([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
      return win.pixieJuice?.detect_format?.(pngHeader);
    });
    
    expect(format).toContain('Png');
  });

  test('should detect JPEG format', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const format = await page.evaluate(async () => {
      const win = window as unknown as { pixieJuice?: { detect_format?: (data: Uint8Array) => string } };
      const jpegHeader = new Uint8Array([
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 
        0x4A, 0x46, 0x49, 0x46, 0x00, 0x01
      ]);
      return win.pixieJuice?.detect_format?.(jpegHeader);
    });
    
    expect(format?.toLowerCase()).toMatch(/jpeg|jpg|image/);
  });

  test('should detect WebP format', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const format = await page.evaluate(async () => {
      const win = window as unknown as { pixieJuice?: { detect_format?: (data: Uint8Array) => string } };
      const webpHeader = new Uint8Array([
        0x52, 0x49, 0x46, 0x46, 
        0x00, 0x00, 0x00, 0x00, 
        0x57, 0x45, 0x42, 0x50  
      ]);
      return win.pixieJuice?.detect_format?.(webpHeader);
    });
    
    expect(format).toContain('WebP');
  });

  test('should detect GIF format', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const format = await page.evaluate(async () => {
      const win = window as unknown as { pixieJuice?: { detect_format?: (data: Uint8Array) => string } };
      const gifHeader = new Uint8Array([
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61,
        0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00,
        0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00
      ]);
      return win.pixieJuice?.detect_format?.(gifHeader);
    });
    
    expect(format?.toLowerCase()).toMatch(/gif|image/);
  });

  test('should return unknown for invalid data', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const format = await page.evaluate(async () => {
      const win = window as unknown as { pixieJuice?: { detect_format?: (data: Uint8Array) => string } };
      const randomData = new Uint8Array([0x00, 0x01, 0x02, 0x03]);
      return win.pixieJuice?.detect_format?.(randomData);
    });
    
    expect(format).toBe('unknown');
  });
});

test.describe('Performance Metrics', () => {
  test('should return performance metrics', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    const metrics = await page.evaluate(() => {
      const win = window as unknown as { pixieJuice?: { get_performance_metrics?: () => unknown } };
      return win.pixieJuice?.get_performance_metrics?.();
    });
    
    expect(metrics).toBeTruthy();
    expect(typeof metrics).toBe('object');
  });

  test('should reset performance stats', async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
    
    await page.evaluate(() => {
      const win = window as unknown as { pixieJuice?: { reset_performance_stats?: () => void } };
      win.pixieJuice?.reset_performance_stats?.();
    });
    
    const metrics = await page.evaluate(() => {
      const win = window as unknown as { 
        pixieJuice?: { 
          get_performance_metrics?: () => { images_processed?: number } 
        } 
      };
      return win.pixieJuice?.get_performance_metrics?.();
    });
    
    expect(metrics?.images_processed).toBe(0);
  });
});
