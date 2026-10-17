import { test, expect } from '@playwright/test';
import { openApp } from './helpers';

test('once visited, the app opens and works offline, with the parts used before', async ({ page, context, browserName }) => {
  test.skip(browserName !== 'chromium', 'Playwright takes service workers offline in Chromium only');
  // The service worker exists in the production build only.
  test.skip(!process.env.CI, 'needs the production build (CI=1)');

  await openApp(page);
  // Installed (the app and the engine stored) and in control of the page.
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null);
  // Parts loaded on demand are stored when first used: SVGO, and the 3D preview.
  await page.evaluate(() => window.pixieJuice!.optimize(new TextEncoder().encode('<svg xmlns="http://www.w3.org/2000/svg"/>')));
  await page
    .locator('#file-input')
    .setInputFiles({ name: 'first.obj', mimeType: 'text/plain', buffer: Buffer.from('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n') });
  await expect(page.getByTestId('mesh-preview')).toHaveAttribute('aria-busy', 'false');

  // Only the service worker can answer for the page itself (the built files are also
  // in the HTTP cache once it has stored them, being immutable).
  await context.setOffline(true);
  const reloaded = await page.reload();
  expect(reloaded?.fromServiceWorker()).toBe(true);
  await openApp(page);
  await expect(page.getByRole('heading', { level: 1, name: 'Pixie Juice' })).toBeVisible();

  // SVGO and the engine, loaded into the workers only now.
  const svg = await page.evaluate(async () => {
    const input = new TextEncoder().encode('<svg xmlns="http://www.w3.org/2000/svg"><!-- note --><rect width="10" height="10"/></svg>');
    const out = await window.pixieJuice!.optimize(input);
    return { format: out.format, changed: out.changed };
  });
  expect(svg).toEqual({ format: 'svg', changed: true });

  // The 3D preview: three.js and the engine's mesh reader.
  await page
    .locator('#file-input')
    .setInputFiles({ name: 'tri.obj', mimeType: 'text/plain', buffer: Buffer.from('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n') });
  await expect(page.getByTestId('mesh-preview')).toHaveAttribute('aria-busy', 'false');
  await expect(page.getByText(/Cannot preview/)).toHaveCount(0);
});
