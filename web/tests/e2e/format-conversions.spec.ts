import { test, expect, Page } from '@playwright/test';

type WasmAPI = {
  convert_to_webp: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_png: (data: Uint8Array) => Uint8Array;
  convert_to_jpeg: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_bmp: (data: Uint8Array) => Uint8Array;
  convert_to_gif: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_ico: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_tiff: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_svg: (data: Uint8Array, quality: number) => Uint8Array;
  convert_to_tga: (data: Uint8Array, quality: number) => Uint8Array;

  optimize_image: (data: Uint8Array, quality: number) => Uint8Array;
  detect_format: (data: Uint8Array) => string;

  is_webp: (data: Uint8Array) => boolean;
  is_ico: (data: Uint8Array) => boolean;
  is_tga: (data: Uint8Array) => boolean;
};

function createDeterministicBmp(width: number = 256, height: number = 256): Uint8Array {
  const bytesPerPixel = 3;
  const rowSizeUnpadded = width * bytesPerPixel;
  const rowPadding = (4 - (rowSizeUnpadded % 4)) % 4;
  const rowSize = rowSizeUnpadded + rowPadding;
  const pixelDataSize = rowSize * height;
  const fileSize = 54 + pixelDataSize;

  const bmp = new Uint8Array(fileSize);
  const view = new DataView(bmp.buffer);

  // BITMAPFILEHEADER
  bmp[0] = 0x42;
  bmp[1] = 0x4D;
  view.setUint32(2, fileSize, true);
  view.setUint32(10, 54, true);

  // BITMAPINFOHEADER
  view.setUint32(14, 40, true);
  view.setInt32(18, width, true);
  view.setInt32(22, height, true);
  view.setUint16(26, 1, true);
  view.setUint16(28, 24, true);
  view.setUint32(34, pixelDataSize, true);

  // Pixel data: bottom-up rows.
  let offset = 54;
  for (let y = height - 1; y >= 0; y--) {
    for (let x = 0; x < width; x++) {
      const stripe = ((x >> 4) + (y >> 4)) & 1;
      const r = stripe ? 220 : 30;
      const g = (x * 5) & 0xFF;
      const b = (y * 3) & 0xFF;
      bmp[offset++] = b;
      bmp[offset++] = g;
      bmp[offset++] = r;
    }
    for (let p = 0; p < rowPadding; p++) bmp[offset++] = 0;
  }

  return bmp;
}

async function evalConvert(
  page: Page,
  fnName: keyof WasmAPI,
  input: Uint8Array,
  quality?: number
): Promise<{ out: Uint8Array; outFormat: string; outSize: number; inSize: number }> {
  return await page.evaluate(
    async ({ fnName, input, quality }) => {
      const api = (window as unknown as { pixieJuice?: WasmAPI }).pixieJuice;
      if (!api) throw new Error('pixieJuice not available');

      const inputBytes = new Uint8Array(input);
      const fn = api[fnName];
      if (typeof fn !== 'function') throw new Error(`missing function: ${String(fnName)}`);

      const out = typeof quality === 'number'
        ? (fn as unknown as (d: Uint8Array, q: number) => Uint8Array)(inputBytes, quality)
        : (fn as unknown as (d: Uint8Array) => Uint8Array)(inputBytes);

      const outFormat = api.detect_format(out);
      return { out: Array.from(out), outFormat, outSize: out.length, inSize: inputBytes.length };
    },
    { fnName: fnName as string, input: Array.from(input), quality }
  ).then(r => ({
    out: new Uint8Array(r.out),
    outFormat: r.outFormat,
    outSize: r.outSize,
    inSize: r.inSize,
  }));
}

test.describe('Format Conversions (WASM)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForFunction(() => typeof (window as any).pixieJuice !== 'undefined');
  });

  test('all convert_to_* functions are present', async ({ page }) => {
    await page.evaluate(() => {
      const api = (window as any).pixieJuice;
      const required = [
        'convert_to_webp',
        'convert_to_png',
        'convert_to_jpeg',
        'convert_to_bmp',
        'convert_to_gif',
        'convert_to_ico',
        'convert_to_tiff',
        'convert_to_svg',
        'convert_to_tga',
        'detect_format',
        'optimize_image',
        'is_tga',
      ];

      for (const name of required) {
        if (typeof api?.[name] !== 'function') {
          throw new Error(`missing wasm export on window.pixieJuice: ${name}`);
        }
      }
    });
  });

  test('BMP -> WebP yields WebP output and should be smaller', async ({ page }) => {
    const input = createDeterministicBmp(256, 256);
    const { outFormat, outSize, inSize } = await evalConvert(page, 'convert_to_webp', input, 80);

    expect(outFormat.toLowerCase()).toContain('webp');
    expect(outSize).toBeGreaterThan(0);

    expect(outSize).toBeLessThan(inSize);
  });

  test('BMP -> JPEG yields JPEG output and should be smaller', async ({ page }) => {
    const input = createDeterministicBmp(256, 256);
    const { outFormat, outSize, inSize } = await evalConvert(page, 'convert_to_jpeg', input, 80);

    expect(outFormat.toLowerCase()).toContain('jpeg');
    expect(outSize).toBeGreaterThan(0);
    expect(outSize).toBeLessThan(inSize);
  });

  test('BMP -> PNG yields PNG output and should be smaller', async ({ page }) => {
    const input = createDeterministicBmp(256, 256);
    const { outFormat, outSize, inSize } = await evalConvert(page, 'convert_to_png', input);

    expect(outFormat.toLowerCase()).toContain('png');
    expect(outSize).toBeGreaterThan(0);
    expect(outSize).toBeLessThan(inSize);
  });

  test('optimize_image output format stays valid', async ({ page }) => {
    const input = createDeterministicBmp(256, 256);
    const result = await page.evaluate(({ input }) => {
      const api = (window as any).pixieJuice as WasmAPI;
      const inBytes = new Uint8Array(input);
      const out = api.optimize_image(inBytes, 80);
      return {
        inSize: inBytes.length,
        outSize: out.length,
        outFormat: api.detect_format(out),
        outLooksLikeWebP: api.is_webp(out),
        outLooksLikeIco: api.is_ico(out),
        outLooksLikeTga: api.is_tga(out),
      };
    }, { input: Array.from(input) });

    expect(result.outSize).toBeGreaterThan(0);
    expect(result.outFormat.length).toBeGreaterThan(0);
    expect(result.outFormat).not.toBe('unknown');

    expect(result.outSize).toBeLessThanOrEqual(result.inSize);
  });
});
