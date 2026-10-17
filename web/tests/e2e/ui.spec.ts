import { readFileSync } from 'node:fs';
import { crc32 } from 'node:zlib';
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
  await page.getByRole('button', { name: 'Optimized', exact: true }).click();
  await expect(page.getByAltText('Preview of holiday photo_juice.webp')).toBeVisible();

  await page.getByRole('button', { name: 'Start over' }).click();
  await expect(page.getByRole('button', { name: /drop files/i })).toBeVisible();
});

test('compares the original and the optimized image on a split view', async ({ page }) => {
  await openApp(page);
  await uploadPng(page, 'photo.png');
  await page.getByLabel('Output format').selectOption('webp');
  await page.getByRole('button', { name: 'Optimize 1 file' }).click();

  // The split view opens after a run, the original left of the divider and the result right of it.
  await expect(page.getByRole('button', { name: 'Compare', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByAltText('Preview of photo.png')).toBeVisible();
  await expect(page.getByAltText('Preview of photo_juice.webp')).toBeVisible();

  // The divider follows a click, then the arrow keys.
  const slider = page.getByRole('slider', { name: 'Divider between original and optimized' });
  const area = (await slider.boundingBox())!;
  await slider.click({ position: { x: area.width / 4, y: area.height / 2 } });
  const value = Number(await slider.inputValue());
  expect(Math.abs(value - 25)).toBeLessThanOrEqual(1);
  const divider = (await page.locator('.compare-divider').boundingBox())!;
  expect(Math.abs(divider.x + divider.width / 2 - (area.x + area.width / 4))).toBeLessThanOrEqual(3);
  await slider.press('ArrowRight');
  const moved = await slider.inputValue();
  expect(Number(moved)).toBeGreaterThan(value);
  await expect(page.locator('.compare-after')).toHaveCSS('clip-path', new RegExp(`${moved}%`));
});

test('compares images the engine decodes for display', async ({ page }) => {
  await openApp(page);
  const bytes = await page.evaluate(() => Array.from(window.testing.uncompressedTiff(64, 48)));
  await page.locator('#file-input').setInputFiles({ name: 'scan.tif', mimeType: 'image/tiff', buffer: Buffer.from(bytes) });
  await expect(page.getByRole('img', { name: 'Preview of scan.tif' })).toHaveJSProperty('width', 64);
  await page.getByRole('button', { name: 'Optimize 1 file' }).click();
  await expect(page.getByRole('img', { name: 'Preview of scan.tif' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'Preview of scan_juice.tiff' })).toBeVisible();
});

test('meshes show one side at a time', async ({ page }) => {
  await openApp(page);
  await page.locator('#file-input').setInputFiles({
    name: 'triangle.obj',
    mimeType: 'text/plain',
    buffer: Buffer.from('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n'),
  });
  await page.getByRole('button', { name: 'Optimize 1 file' }).click();
  await expect(page.getByRole('heading', { name: 'Results' })).toBeVisible();
  await expect(page.getByRole('group', { name: 'Preview source' }).getByRole('button')).toHaveText(['Original', 'Optimized']);
  await expect(page.getByRole('button', { name: 'Original', exact: true })).toHaveAttribute('aria-pressed', 'true');
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

test('changing the file list during a run stops it without results', async ({ page }) => {
  await openApp(page);
  // zopfli on noisy PNGs keeps each file busy for a while, so most are still waiting when the list changes.
  const png = await page.evaluate(async () =>
    Array.from(await window.testing.makeImage({ width: 300, height: 300, mime: 'image/png', noise: 60 })),
  );
  const names = Array.from({ length: 10 }, (_, i) => `photo-${i}.png`);
  await page
    .locator('#file-input')
    .setInputFiles(names.map((name) => ({ name, mimeType: 'image/png', buffer: Buffer.from(png) })));
  // Counts the files the engine is asked to optimize, and notes whether results ever appear.
  await page.evaluate(() => {
    const seen = { calls: 0, results: false };
    (window as unknown as { seen: typeof seen }).seen = seen;
    const api = window.pixieJuice!;
    const optimize = api.optimize.bind(api);
    api.optimize = (...args) => {
      seen.calls++;
      return optimize(...args);
    };
    new MutationObserver(() => {
      seen.results ||= [...document.querySelectorAll('h2')].some((h) => h.textContent === 'Results');
    }).observe(document.body, { childList: true, subtree: true });
  });

  await page.getByRole('button', { name: 'Optimize 10 files' }).click();
  await expect(page.getByRole('progressbar', { name: 'Processing progress' })).toBeVisible();
  await page.getByRole('button', { name: 'Remove photo-9.png' }).click();

  // The files in progress finish, the rest never start, and no results for the old list appear.
  await expect(page.getByRole('button', { name: 'Optimize 9 files' })).toBeVisible({ timeout: 45_000 });
  const seen = await page.evaluate(() => (window as unknown as { seen: { calls: number; results: boolean } }).seen);
  expect(seen.results).toBe(false);
  expect(seen.calls).toBeLessThan(names.length);
});

test('Cancel stops a run, the files in progress too, and keeps what finished', async ({ page }) => {
  await openApp(page);
  const png = await page.evaluate(async () =>
    Array.from(await window.testing.makeImage({ width: 300, height: 300, mime: 'image/png', noise: 60 })),
  );
  const names = Array.from({ length: 10 }, (_, i) => `photo-${i}.png`);
  await page
    .locator('#file-input')
    .setInputFiles(names.map((name) => ({ name, mimeType: 'image/png', buffer: Buffer.from(png) })));
  await page.evaluate(() => {
    const seen = { calls: 0 };
    (window as unknown as { seen: typeof seen }).seen = seen;
    const api = window.pixieJuice!;
    const optimize = api.optimize.bind(api);
    api.optimize = (...args) => {
      seen.calls++;
      return optimize(...args);
    };
  });

  await page.getByRole('button', { name: 'Optimize 10 files' }).click();
  await expect(page.getByRole('progressbar', { name: 'Processing progress' })).toBeVisible();
  await page.getByRole('button', { name: 'Cancel' }).click();

  // Back right away (the files in progress were stopped, not awaited), with nothing new started.
  await expect(page.getByRole('progressbar', { name: 'Processing progress' })).toBeHidden({ timeout: 5_000 });
  const calls = await page.evaluate(() => (window as unknown as { seen: { calls: number } }).seen.calls);
  expect(calls).toBeLessThan(names.length);
  const results = page.getByRole('heading', { name: 'Results' });
  if (await results.isVisible()) {
    await expect(page.getByText(/not processed \(cancelled\)/)).toBeVisible();
  } else {
    await expect(page.getByRole('button', { name: 'Optimize 10 files' })).toBeVisible();
  }
  // The engine still works afterwards.
  const after = await page.evaluate(async (bytes) => (await window.pixieJuice!.optimize(new Uint8Array(bytes))).format, png);
  expect(after).toBe('png');
});

test('Resize appears for images and reaches the engine', async ({ page }) => {
  await openApp(page);
  await page.locator('#file-input').setInputFiles({ name: 'tri.obj', mimeType: 'text/plain', buffer: Buffer.from('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n') });
  await expect(page.getByLabel('Resize')).toHaveCount(0);
  await uploadPng(page, 'wide.png');
  await page.getByLabel('Resize').selectOption('512');
  await page.evaluate(() => {
    const api = window.pixieJuice!;
    const optimize = api.optimize.bind(api);
    api.optimize = (data, options, ...rest) => {
      (window as unknown as { maxSize?: number }).maxSize = options?.maxSize;
      return optimize(data, options, ...rest);
    };
  });
  await page.getByRole('button', { name: 'Optimize 2 files' }).click();
  await expect(page.getByRole('heading', { name: 'Results' })).toBeVisible();
  expect(await page.evaluate(() => (window as unknown as { maxSize?: number }).maxSize)).toBe(512);
});

test('a .bin goes with the .gltf that uses it, not on its own', async ({ page }) => {
  await openApp(page);
  await page.locator('#file-input').setInputFiles([
    { name: 'scene.bin', mimeType: 'application/octet-stream', buffer: Buffer.alloc(16) },
    { name: 'photo.png', mimeType: 'image/png', buffer: Buffer.from(await page.evaluate(async () => Array.from(await window.testing.makeImage({ width: 20, height: 20, mime: 'image/png' })))) },
  ]);
  await expect(page.getByRole('button', { name: 'Preview scene.bin' })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Selected files' }).getByText(/for a \.gltf/)).toBeVisible();
  await expect(page.getByRole('button', { name: 'Optimize 1 file' })).toBeVisible();
});

test('processes several files at once, results in list order', async ({ page }) => {
  await openApp(page);
  const concurrency = await page.evaluate(() => window.pixieJuice!.concurrency);
  const png = await page.evaluate(async () =>
    Array.from(await window.testing.makeImage({ width: 200, height: 200, mime: 'image/png', noise: 60 })),
  );
  const names = Array.from({ length: 6 }, (_, i) => `photo-${i}.png`);
  await page
    .locator('#file-input')
    .setInputFiles(names.map((name) => ({ name, mimeType: 'image/png', buffer: Buffer.from(png) })));
  await page.evaluate(() => {
    const seen = { most: 0 };
    (window as unknown as { seen: typeof seen }).seen = seen;
    new MutationObserver(() => {
      const active = document.querySelector('.progress-meta span')?.textContent?.split(', ').filter(Boolean) ?? [];
      seen.most = Math.max(seen.most, active.length);
    }).observe(document.body, { childList: true, subtree: true, characterData: true });
  });

  await page.getByRole('button', { name: 'Optimize 6 files' }).click();
  const results = page.getByRole('list', { name: 'Processed files' }).getByRole('listitem');
  await expect(results).toHaveCount(6, { timeout: 45_000 });
  expect(await results.locator('.row-name').allTextContents()).toEqual(names);
  const most = await page.evaluate(() => (window as unknown as { seen: { most: number } }).seen.most);
  expect(most).toBe(Math.min(concurrency, names.length));
});

/** The entries of a ZIP file, read from its central directory, with their stored data. */
function unzip(zip: Buffer): { name: string; data: Buffer; crc: number }[] {
  const end = zip.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  const count = zip.readUInt16LE(end + 10);
  let at = zip.readUInt32LE(end + 16);
  const entries = [];
  for (let i = 0; i < count; i++) {
    expect(zip.readUInt32LE(at)).toBe(0x02014b50);
    expect(zip.readUInt16LE(at + 10)).toBe(0); // stored
    const crc = zip.readUInt32LE(at + 16);
    const size = zip.readUInt32LE(at + 20);
    const nameLength = zip.readUInt16LE(at + 28);
    const name = zip.subarray(at + 46, at + 46 + nameLength).toString('utf8');
    const local = zip.readUInt32LE(at + 42);
    const start = local + 30 + zip.readUInt16LE(local + 26) + zip.readUInt16LE(local + 28);
    entries.push({ name, data: zip.subarray(start, start + size), crc });
    at += 46 + nameLength;
  }
  return entries;
}

test('Download all saves every result in one ZIP, each under its own name', async ({ page }) => {
  await openApp(page);
  await uploadPng(page, 'first.png');
  await uploadPng(page, 'second.png');
  await page.getByRole('button', { name: 'Optimize 2 files' }).click();
  await expect(page.getByRole('heading', { name: 'Results' })).toBeVisible();

  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Download all (ZIP)' }).click();
  expect((await download).suggestedFilename()).toBe('pixie-juice.zip');
  const entries = unzip(readFileSync((await (await download).path())!));
  expect(entries.map((entry) => entry.name).sort()).toEqual(['first_juice.png', 'second_juice.png']);
  for (const entry of entries) {
    expect(crc32(entry.data) >>> 0).toBe(entry.crc);
    expect(entry.data.subarray(0, 4).toString('latin1')).toBe('\x89PNG');
  }
});

test('Lossless refuses a conversion that would change pixels', async ({ page }) => {
  await openApp(page);
  const results = page.getByRole('list', { name: 'Processed files' });
  const convert = async (lossless: boolean) => {
    await uploadPng(page, 'photo.png');
    await page.getByLabel('Output format').selectOption('jpeg');
    await page.getByLabel('Lossless').setChecked(lossless);
    await page.getByRole('button', { name: 'Optimize 1 file' }).click();
    await expect(page.getByRole('heading', { name: 'Results' })).toBeVisible();
  };

  await convert(true);
  await expect(results).toContainText("Failed: JPEG can't hold this image losslessly");
  await page.getByRole('button', { name: 'Start over' }).click();

  await convert(false);
  await expect(results.getByText('JPG', { exact: true })).toBeVisible();
});

/** The PNG with a tEXt chunk added after its header. */
function withText(png: Buffer, keyword: string, text: string): Buffer {
  const body = Buffer.concat([Buffer.from('tEXt'), Buffer.from(`${keyword}\0${text}`, 'latin1')]);
  const chunk = Buffer.alloc(body.length + 8);
  chunk.writeUInt32BE(body.length - 4, 0);
  body.copy(chunk, 4);
  chunk.writeUInt32BE(crc32(body), body.length + 4);
  return Buffer.concat([png.subarray(0, 33), chunk, png.subarray(33)]);
}

test('Preserve metadata decides whether text chunks are kept', async ({ page }) => {
  await openApp(page);
  const png = await page.evaluate(async () =>
    Array.from(await window.testing.makeImage({ width: 120, height: 80, mime: 'image/png', noise: 0 })),
  );
  const tagged = withText(Buffer.from(png), 'Comment', 'kept by Preserve metadata');
  const optimize = async (preserve: boolean) => {
    await page.locator('#file-input').setInputFiles({ name: 'tagged.png', mimeType: 'image/png', buffer: tagged });
    await page.getByLabel('Preserve metadata').setChecked(preserve);
    await page.getByRole('button', { name: 'Optimize 1 file' }).click();
    const download = page.waitForEvent('download');
    await page.getByRole('button', { name: 'Download tagged.png' }).click();
    const bytes = readFileSync((await (await download).path())!);
    await page.getByRole('button', { name: 'Start over' }).click();
    return { smaller: bytes.length < tagged.length, text: bytes.includes('kept by Preserve metadata') };
  };

  expect(await optimize(false)).toEqual({ smaller: true, text: false });
  expect((await optimize(true)).text).toBe(true);
});

test('SVG goes through SVGO whatever the output format, and settings survive a run', async ({ page }) => {
  await openApp(page);
  const svg = '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><!-- c --><rect width="10" height="10"/></svg>';
  await page.locator('#file-input').setInputFiles({ name: 'icon.svg', mimeType: 'image/svg+xml', buffer: Buffer.from(svg) });
  await page.getByLabel('Output format').selectOption('webp');
  await page.getByLabel('Lossless').check();
  await page.getByRole('button', { name: 'Optimize 1 file' }).click();
  const results = page.getByRole('list', { name: 'Processed files' });
  await expect(results.getByText('SVG', { exact: true })).toBeVisible();
  await expect(results.getByText(/Failed/)).toHaveCount(0);

  // A new file brings the settings back as they were.
  await uploadPng(page, 'photo.png');
  await expect(page.getByLabel('Output format')).toHaveValue('webp');
  await expect(page.getByLabel('Lossless')).toBeChecked();
});

test('a file added again under the same name replaces the listed one', async ({ page }) => {
  await openApp(page);
  const upload = (text: string) =>
    page.locator('#file-input').setInputFiles({ name: 'tri.obj', mimeType: 'text/plain', buffer: Buffer.from(text) });
  await upload('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n');
  await upload('v 0 0 0\nv 1 0 0\nv 0 1 0\nv 1 1 0\nf 1 2 3\nf 2 4 3\n');
  await expect(page.getByRole('list', { name: 'Selected files' }).getByRole('listitem')).toHaveCount(1);
  await expect(page.getByRole('list', { name: 'Selected files' })).toContainText('48 B');
});

test('pasting adds files, and a second pasted "image.png" gets a name of its own', async ({ page }) => {
  await openApp(page);
  const paste = () =>
    page.evaluate(async () => {
      const png = await window.testing.makeImage({ width: 40, height: 30, mime: 'image/png' });
      const data = new DataTransfer();
      data.items.add(new File([png as BlobPart], 'image.png', { type: 'image/png' }));
      const event = new Event('paste', { bubbles: true, cancelable: true });
      Object.defineProperty(event, 'clipboardData', { value: data });
      document.body.dispatchEvent(event);
    });
  await paste();
  await expect(page.getByRole('button', { name: 'Preview image.png' })).toBeVisible();
  await paste();
  await expect(page.getByRole('button', { name: 'Preview image-2.png' })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Selected files' }).getByRole('listitem')).toHaveCount(2);
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
  await expect(page.getByTestId('mesh-preview')).toHaveAttribute('aria-busy', 'false');
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

test('the drop zone is keyboard reachable and opens the file chooser', async ({ page }) => {
  await openApp(page);
  const zone = page.getByRole('button', { name: /drop files/i });
  await zone.focus();
  await expect(zone).toBeFocused();
  for (const key of ['Enter', 'Space']) {
    const chooser = page.waitForEvent('filechooser');
    await page.keyboard.press(key);
    expect((await chooser).isMultiple()).toBe(true);
  }
});
