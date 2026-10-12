import { test, expect } from '@playwright/test';
import { openApp } from './helpers';

test.beforeEach(async ({ page }) => openApp(page));

test('reports its version and the commit it was built from', async ({ page }) => {
  const info = await page.evaluate(() => window.pixieJuice!.info());
  expect(info.version).toMatch(/^\d+\.\d+\.\d+$/);
  expect(info.commit).toMatch(/^([0-9a-f]{7,}(-dirty)?|unknown)$/);
  expect(info.timestamp).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);
  await expect(page.getByTestId('version')).toContainText(`v${info.version}`);
});

test('detects every supported format', async ({ page }) => {
  const kinds = await page.evaluate(async () => {
    const api = window.pixieJuice!;
    const png = await window.testing.makeImage({ width: 16, height: 16, mime: 'image/png' });
    const text = (s: string) => new TextEncoder().encode(s);
    const stl = new Uint8Array(84 + 50);
    new DataView(stl.buffer).setUint32(80, 1, true);
    const samples: Record<string, Uint8Array> = {
      png,
      jpeg: await window.testing.makeImage({ width: 16, height: 16, mime: 'image/jpeg' }),
      webp: await window.testing.makeImage({ width: 16, height: 16, mime: 'image/webp' }),
      gif: (await api.convert(png, 'gif')).data,
      bmp: (await api.convert(png, 'bmp')).data,
      tga: (await api.convert(png, 'tga')).data,
      tiff: (await api.convert(png, 'tiff')).data,
      ico: (await api.convert(png, 'ico')).data,
      svg: text('<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"/>'),
      obj: text('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n'),
      stl,
      ply: text('ply\nformat ascii 1.0\nend_header\n'),
      gltf: text('{"asset":{"version":"2.0"}}'),
      glb: new Uint8Array([0x67, 0x6c, 0x54, 0x46, 2, 0, 0, 0, 12, 0, 0, 0]),
      fbx: text('Kaydara FBX Binary  \0'),
      garbage: text('nothing recognisable here'),
    };
    const out: Record<string, string> = {};
    for (const [name, data] of Object.entries(samples)) out[name] = await api.detect(data);
    return out;
  });
  for (const image of ['png', 'jpeg', 'webp', 'gif', 'bmp', 'tga', 'tiff', 'ico', 'svg']) {
    expect(kinds[image], image).toBe(`image:${image}`);
  }
  for (const mesh of ['obj', 'stl', 'ply', 'gltf', 'glb', 'fbx']) {
    expect(kinds[mesh], mesh).toBe(`mesh:${mesh}`);
  }
  expect(kinds.garbage).toBe('unknown');
});

test('rejects empty, unknown and broken input with an error, then keeps working', async ({ page }) => {
  const outcome = await page.evaluate(async () => {
    const api = window.pixieJuice!;
    const attempt = async (bytes: number[]) => {
      try {
        await api.optimize(new Uint8Array(bytes));
        return 'ok';
      } catch (error) {
        return (error as Error).message;
      }
    };
    const errors = {
      empty: await attempt([]),
      garbage: await attempt([1, 2, 3, 4, 5, 6, 7, 8, 9]),
      truncatedPng: await attempt([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13, 0x49, 0x48, 0x44, 0x52]),
      truncatedJpeg: await attempt([0xff, 0xd8, 0xff, 0xe0, 0, 16]),
      truncatedGif: await attempt([...new TextEncoder().encode('GIF89a'), 1, 0, 1, 0]),
    };
    const png = await window.testing.makeImage({ width: 32, height: 32, mime: 'image/png' });
    const after = await api.optimize(png);
    return { errors, stillWorks: after.format === 'png' };
  });
  for (const [name, message] of Object.entries(outcome.errors)) {
    expect(message, name).not.toBe('ok');
  }
  expect(outcome.stillWorks).toBe(true);
});

test('keeps the console clean while processing', async ({ page }) => {
  const messages: string[] = [];
  page.on('console', (message) => messages.push(`${message.type()}: ${message.text()}`));
  await page.evaluate(async () => {
    const api = window.pixieJuice!;
    for (const mime of ['image/png', 'image/jpeg', 'image/webp'] as const) {
      await api.optimize(await window.testing.makeImage({ width: 120, height: 90, mime }));
    }
    await api.optimize(new TextEncoder().encode('v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n'));
  });
  expect(messages).toEqual([]);
});
