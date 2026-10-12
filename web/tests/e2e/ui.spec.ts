import { test, expect, type Page } from '@playwright/test';
import { openApp } from './helpers';

/** A real PNG made in the page, handed to the file input as an upload. */
async function uploadPng(page: Page, name: string) {
  const bytes = await page.evaluate(async () =>
    Array.from(await window.testing.makeImage({ width: 120, height: 80, mime: 'image/png', noise: 30 })),
  );
  await page.locator('#file-input').setInputFiles({ name, mimeType: 'image/png', buffer: Buffer.from(bytes) });
}

test('loads without console errors and shows the drop zone', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => errors.push(error.message));
  await openApp(page);
  await expect(page.getByRole('heading', { level: 1, name: 'Pixie Juice' })).toBeVisible();
  await expect(page.getByRole('button', { name: /drop files/i })).toBeVisible();
  expect(errors).toEqual([]);
});

test('optimizes an upload end to end and names the download by its real format', async ({ page }) => {
  await openApp(page);
  await uploadPng(page, 'holiday photo.png');
  await expect(page.getByRole('button', { name: 'Preview holiday photo.png' })).toBeVisible();
  await expect(page.getByAltText('Preview of holiday photo.png')).toBeVisible();

  await page.getByLabel('Output format').selectOption('webp');
  await page.getByRole('button', { name: 'Optimize 1 file' }).click();
  await expect(page.getByRole('heading', { name: 'Results' })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Processed files' }).getByText('WEBP', { exact: true })).toBeVisible();

  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Download holiday photo.png' }).click();
  expect((await download).suggestedFilename()).toBe('holiday photo_juice.webp');

  // The optimized file can be previewed next to the original.
  await page.getByRole('tab', { name: 'Optimized' }).click();
  await expect(page.getByAltText('Preview of holiday photo_juice.webp')).toBeVisible();

  await page.getByRole('button', { name: 'Start over' }).click();
  await expect(page.getByRole('button', { name: /drop files/i })).toBeVisible();
});

test('changing the file list after processing discards the results', async ({ page }) => {
  await openApp(page);
  await uploadPng(page, 'first.png');
  await page.getByRole('button', { name: 'Optimize 1 file' }).click();
  await expect(page.getByRole('heading', { name: 'Results' })).toBeVisible();

  await uploadPng(page, 'second.png');
  await expect(page.getByRole('heading', { name: 'Results' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Optimize 2 files' }).click();
  await expect(page.getByRole('button', { name: 'Download second.png' })).toBeVisible();

  await page.getByRole('button', { name: 'Remove second.png' }).click();
  await expect(page.getByRole('button', { name: 'Optimize 1 file' })).toBeVisible();
});

test('previews TIFF through the engine decoder', async ({ page }) => {
  await openApp(page);
  const bytes = await page.evaluate(() => Array.from(window.testing.uncompressedTiff(64, 48)));
  await page.locator('#file-input').setInputFiles({ name: 'scan.tif', mimeType: 'image/tiff', buffer: Buffer.from(bytes) });
  await expect(page.getByRole('img', { name: 'Preview of scan.tif' })).toHaveJSProperty('width', 64);
});

test('shows the mesh detail control only for meshes', async ({ page }) => {
  await openApp(page);
  await uploadPng(page, 'icon.png');
  await expect(page.getByLabel('Mesh triangles kept')).toHaveCount(0);
  await page.locator('#file-input').setInputFiles({
    name: 'triangle.obj',
    mimeType: 'text/plain',
    buffer: Buffer.from('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n'),
  });
  await expect(page.getByLabel('Mesh triangles kept')).toBeVisible();
  await page.getByRole('button', { name: 'Preview triangle.obj' }).click();
  await expect(page.getByTestId('mesh-preview').locator('canvas')).toBeVisible();
  await expect(page.getByText(/Cannot preview/)).toHaveCount(0);
});

for (const viewport of [
  { name: 'mobile', width: 375, height: 812 },
  { name: 'tablet', width: 768, height: 1024 },
]) {
  test(`fits a ${viewport.name} viewport`, async ({ page }) => {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });
    await openApp(page);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    await expect(page.getByRole('button', { name: /drop files/i })).toBeVisible();
  });
}

test('the drop zone is keyboard reachable', async ({ page }) => {
  await openApp(page);
  const zone = page.getByRole('button', { name: /drop files/i });
  await zone.focus();
  await expect(zone).toBeFocused();
});
