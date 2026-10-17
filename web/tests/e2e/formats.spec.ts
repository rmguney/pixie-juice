import { readFileSync } from 'node:fs';
import { test, expect } from '@playwright/test';
import { openApp } from './helpers';

// HEIC and AVIF samples from libheif's tests; TIFFs in each extra codec, written by libtiff.
const fixture = (name: string) => Array.from(readFileSync(new URL(`./fixtures/${name}`, import.meta.url)));

test.beforeEach(async ({ page }) => openApp(page));

test('HEIC: kept as it is by Auto, converted to JPEG and BMP through the HEIC module', async ({ page }) => {
  const r = await page.evaluate(async (bytes) => {
    const api = window.pixieJuice!;
    const heic = new Uint8Array(bytes);
    const kept = await api.optimize(heic);
    const jpeg = await api.convert(heic, 'jpeg', { quality: 80 });
    const bmp = await api.convert(heic, 'bmp', { lossless: true });
    const decoded = await window.testing.decode(jpeg.data);
    return {
      kind: await api.detect(heic),
      kept: { format: kept.format, changed: kept.changed },
      jpeg: { format: jpeg.format, width: decoded.width, height: decoded.height },
      // The BMP (uncompressed, so quick) holds exactly what the HEIC decoded to.
      bmp: await window.testing.compare(heic, bmp.data),
    };
  }, fixture('rainbow-451x461.heic'));
  expect(r.kind).toBe('image:heic');
  expect(r.kept).toEqual({ format: 'heic', changed: false });
  expect(r.jpeg).toEqual({ format: 'jpeg', width: 451, height: 461 });
  expect(r.bmp.meanError).toBe(0);
  expect(r.bmp.colourful).toBe(true);
});

test('HEIC: alpha survives a conversion', async ({ page }) => {
  const r = await page.evaluate(async (bytes) => {
    // TIFF keeps alpha and encodes quickly (Deflate).
    const tiff = await window.pixieJuice!.convert(new Uint8Array(bytes), 'tiff');
    const { data } = await window.testing.decode(tiff.data);
    let transparent = 0;
    for (let i = 3; i < data.length; i += 4) if (data[i] < 128) transparent++;
    return { format: tiff.format, transparent };
  }, fixture('with-alpha-512x512.heic'));
  expect(r.format).toBe('tiff');
  expect(r.transparent).toBeGreaterThan(0);
});

test('HEIC: previewed through the engine, then converted from the UI', async ({ page }) => {
  await page
    .locator('#file-input')
    .setInputFiles({ name: 'IMG_0001.HEIC', mimeType: 'image/heic', buffer: Buffer.from(fixture('rainbow-451x461.heic')) });
  await expect(page.getByRole('img', { name: 'Preview of IMG_0001.HEIC' })).toBeVisible();
  await page.getByLabel('Output format').selectOption('jpeg');
  await page.getByRole('button', { name: 'Optimize 1 file' }).click();
  await expect(page.getByRole('heading', { name: 'Results' })).toBeVisible();
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Download IMG_0001.HEIC' }).click();
  expect((await download).suggestedFilename()).toBe('IMG_0001_juice.jpg');
});

test('AVIF: decoded by the browser and converted, or a clear error where the browser has no decoder', async ({ page }) => {
  const r = await page.evaluate(async (bytes) => {
    const api = window.pixieJuice!;
    const avif = new Uint8Array(bytes);
    const kept = await api.optimize(avif);
    // Playwright's WebKit build for Windows has no AVIF decoder; Safari and the other browsers do.
    const decodable = await createImageBitmap(new Blob([avif], { type: 'image/avif' })).then(
      (bitmap) => (bitmap.close(), true),
      () => false,
    );
    let converted: { format: string; meanError: number } | { error: string };
    try {
      const png = await api.convert(avif, 'png', { lossless: true });
      // The browser decodes both: the PNG holds what the AVIF decoded to.
      converted = { format: png.format, meanError: (await window.testing.compare(avif, png.data)).meanError };
    } catch (error) {
      converted = { error: String(error) };
    }
    return { kind: await api.detect(avif), kept: { format: kept.format, changed: kept.changed }, decodable, converted };
  }, fixture('avif32.avif'));
  expect(r.kind).toBe('image:avif');
  expect(r.kept).toEqual({ format: 'avif', changed: false });
  if (r.decodable) {
    expect(r.converted).toMatchObject({ format: 'png' });
    expect((r.converted as { meanError: number }).meanError).toBeLessThan(1);
  } else {
    expect(r.converted).toEqual({ error: expect.stringContaining('This browser could not decode the AVIF') });
  }
});

for (const codec of ['zstd', 'lzma', 'webp'] as const) {
  test(`TIFF compressed with ${codec}: decoded and optimized without loss`, async ({ page }) => {
    const r = await page.evaluate(async (bytes) => {
      const tiff = new Uint8Array(bytes);
      const out = await window.pixieJuice!.optimize(tiff, { lossless: true });
      return { format: out.format, ...(await window.testing.compare(tiff, out.data)) };
    }, fixture(`${codec}.tiff`));
    expect(r.format).toBe('tiff');
    expect(r.width).toBe(64);
    expect(r.meanError).toBe(0);
  });
}

test('TIFF compressed with JPEG: re-encoded at the chosen quality', async ({ page }) => {
  const r = await page.evaluate(async (bytes) => {
    const tiff = new Uint8Array(bytes);
    const out = await window.pixieJuice!.optimize(tiff, { quality: 60 });
    return { format: out.format, changed: out.changed, smaller: out.data.length < tiff.length, ...(await window.testing.compare(tiff, out.data)) };
  }, fixture('jpeg.tiff'));
  expect(r).toMatchObject({ format: 'tiff', changed: true, smaller: true, width: 64 });
  expect(r.meanError).toBeLessThan(6);
});
