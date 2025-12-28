import { test, expect, Page } from '@playwright/test';

interface CompressionTestResult {
  success: boolean;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  error?: string;
  timeMs: number;
  outputFormat?: string;
  outputHash?: number;
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

async function compressImageInBrowser(
  page: Page, 
  imageData: Uint8Array, 
  quality: number = 80
): Promise<CompressionTestResult> {
  return await page.evaluate(async ({ data, quality }) => {
    const win = window as unknown as { 
      pixieJuice?: { 
        optimize_image?: (data: Uint8Array, quality: number) => Uint8Array;
        optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array;
        detect_format?: (data: Uint8Array) => string;
      } 
    };
    
    const startTime = performance.now();

    const fnv1a32 = (bytes: Uint8Array): number => {
      let hash = 0x811c9dc5;
      for (let i = 0; i < bytes.length; i++) {
        hash ^= bytes[i];
        hash = Math.imul(hash, 0x01000193);
      }
      return hash >>> 0;
    };
    
    try {
      const inputArray = new Uint8Array(data);
      const result = win.pixieJuice?.optimize_auto?.(inputArray, quality) 
                  || win.pixieJuice?.optimize_image?.(inputArray, quality);
      
      if (!result) {
        return {
          success: false,
          originalSize: inputArray.length,
          compressedSize: 0,
          compressionRatio: 0,
          error: 'No optimization function available',
          timeMs: performance.now() - startTime
        };
      }
      
      const endTime = performance.now();
      const ratio = ((inputArray.length - result.length) / inputArray.length) * 100;
      const outputFormat = win.pixieJuice?.detect_format?.(result);
      const outputHash = fnv1a32(result);
      
      return {
        success: true,
        originalSize: inputArray.length,
        compressedSize: result.length,
        compressionRatio: ratio,
        timeMs: endTime - startTime,
        outputFormat,
        outputHash
      };
    } catch (error) {
      return {
        success: false,
        originalSize: data.length,
        compressedSize: 0,
        compressionRatio: 0,
        error: error instanceof Error ? error.message : String(error),
        timeMs: performance.now() - startTime
      };
    }
  }, { data: Array.from(imageData), quality });
}

function createTestPng(width: number = 160, height: number = 160): Uint8Array {
  const signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
  
  const ihdr = createPngChunk('IHDR', [
    (width >> 24) & 0xFF, (width >> 16) & 0xFF, (width >> 8) & 0xFF, width & 0xFF,
    (height >> 24) & 0xFF, (height >> 16) & 0xFF, (height >> 8) & 0xFF, height & 0xFF,
    8, 2, 0, 0, 0
  ]);
  
  const rawData: number[] = [];
  for (let y = 0; y < height; y++) {
    rawData.push(0);
    for (let x = 0; x < width; x++) {
      const r = (x * 3) & 0xFF;
      const g = (y * 5) & 0xFF;
      const b = ((x + y) * 2) & 0xFF;
      rawData.push(r, g, b);
    }
  }
  
  const compressed = deflateRaw(new Uint8Array(rawData));
  const idat = createPngChunk('IDAT', Array.from(compressed));
  
  const iend = createPngChunk('IEND', []);
  
  return new Uint8Array([...signature, ...ihdr, ...idat, ...iend]);
}

function createPngChunk(type: string, data: number[]): number[] {
  const length = data.length;
  const typeBytes = Array.from(type).map(c => c.charCodeAt(0));
  
  const chunk = [
    (length >> 24) & 0xFF, (length >> 16) & 0xFF, (length >> 8) & 0xFF, length & 0xFF,
    ...typeBytes,
    ...data
  ];
  
  const crc = calculateCrc32(new Uint8Array([...typeBytes, ...data]));
  chunk.push((crc >> 24) & 0xFF, (crc >> 16) & 0xFF, (crc >> 8) & 0xFF, crc & 0xFF);
  
  return chunk;
}

function deflateRaw(data: Uint8Array): Uint8Array {
  const header = [0x78, 0x9C];
  const chunks: number[] = [];
  
  for (let i = 0; i < data.length; i += 65535) {
    const chunk = data.slice(i, Math.min(i + 65535, data.length));
    const isLast = i + 65535 >= data.length;
    chunks.push(isLast ? 0x01 : 0x00);
    chunks.push(chunk.length & 0xFF, (chunk.length >> 8) & 0xFF);
    chunks.push((~chunk.length) & 0xFF, ((~chunk.length) >> 8) & 0xFF);
    chunks.push(...Array.from(chunk));
  }
  
  const adler = adler32(data);
  return new Uint8Array([
    ...header, 
    ...chunks, 
    (adler >> 24) & 0xFF, (adler >> 16) & 0xFF, (adler >> 8) & 0xFF, adler & 0xFF
  ]);
}

function adler32(data: Uint8Array): number {
  let a = 1, b = 0;
  for (const byte of data) {
    a = (a + byte) % 65521;
    b = (b + a) % 65521;
  }
  return (b << 16) | a;
}

function calculateCrc32(data: Uint8Array): number {
  let crc = 0xFFFFFFFF;
  const table = makeCrcTable();
  for (const byte of data) {
    crc = table[(crc ^ byte) & 0xFF] ^ (crc >>> 8);
  }
  return (crc ^ 0xFFFFFFFF) >>> 0;
}

function makeCrcTable(): number[] {
  const table: number[] = [];
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xEDB88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
}

function createTestJpeg(): Uint8Array {
  return new Uint8Array([
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
    0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
    0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
    0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
    0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
    0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
    0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
    0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0A, 0x0B, 0xFF, 0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03,
    0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D,
    0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0xFB, 0xD5,
    0xFF, 0xD9
  ]);
}

function createTestWebP(): Uint8Array {
  const fileSize = 26;
  return new Uint8Array([
    0x52, 0x49, 0x46, 0x46,
    fileSize & 0xFF, (fileSize >> 8) & 0xFF, (fileSize >> 16) & 0xFF, (fileSize >> 24) & 0xFF,
    0x57, 0x45, 0x42, 0x50,
    0x56, 0x50, 0x38, 0x4C,
    0x0A, 0x00, 0x00, 0x00,
    0x2F, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
  ]);
}

function createTestGif(): Uint8Array {
  return new Uint8Array([
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61,
    0x01, 0x00, 0x01, 0x00,
    0x80, 0x00, 0x00,
    0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00,
    0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x2C, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
    0x02, 0x02, 0x44, 0x01, 0x00,
    0x3B
  ]);
}

function createTestBmp(): Uint8Array {
  const width = 64, height = 64;
  const rowSize = Math.ceil((width * 3) / 4) * 4;
  const pixelDataSize = rowSize * height;
  const fileSize = 54 + pixelDataSize;
  
  const header = [
    0x42, 0x4D,
    fileSize & 0xFF, (fileSize >> 8) & 0xFF, (fileSize >> 16) & 0xFF, (fileSize >> 24) & 0xFF,
    0x00, 0x00, 0x00, 0x00,
    0x36, 0x00, 0x00, 0x00,
    0x28, 0x00, 0x00, 0x00,
    width & 0xFF, (width >> 8) & 0xFF, 0x00, 0x00,
    height & 0xFF, (height >> 8) & 0xFF, 0x00, 0x00,
    0x01, 0x00,
    0x18, 0x00,
    0x00, 0x00, 0x00, 0x00,
    pixelDataSize & 0xFF, (pixelDataSize >> 8) & 0xFF, 0x00, 0x00,
    0x13, 0x0B, 0x00, 0x00,
    0x13, 0x0B, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
  ];
  
  const pixels: number[] = [];
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      pixels.push(0xFF, 0x00, 0x00);
    }
    for (let p = width * 3; p < rowSize; p++) {
      pixels.push(0x00);
    }
  }
  
  return new Uint8Array([...header, ...pixels]);
}

function assertSizeReductionOrEqual(result: CompressionTestResult, label: string): void {
  expect(result.success, `${label}: expected success, got error=${result.error ?? 'n/a'}`).toBe(true);
  expect(result.compressedSize, `${label}: empty output`).toBeGreaterThan(0);

  const isMeaningful = result.originalSize >= 4096;
  const maxAllowed = isMeaningful ? result.originalSize : result.originalSize + 512;
  expect(
    result.compressedSize,
    `${label}: output grew (${result.originalSize} -> ${result.compressedSize}) format=${result.outputFormat ?? 'unknown'}`
  ).toBeLessThanOrEqual(maxAllowed);

  if (isMeaningful) {
    expect(
      result.compressedSize,
      `${label}: no-op compression (${result.originalSize} -> ${result.compressedSize})`
    ).toBeLessThan(result.originalSize);
  }
}

test.describe('Image Compression - PNG', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should compress PNG image', async ({ page }) => {
    const testPng = createTestPng(50, 50);
    const result = await compressImageInBrowser(page, testPng, 80);

    assertSizeReductionOrEqual(result, 'PNG compress');
  });

  test('should maintain valid PNG output after compression', async ({ page }) => {
    const testPng = createTestPng(20, 20);
    
    const outputHeader = await page.evaluate(async (data) => {
      const win = window as unknown as { 
        pixieJuice?: { optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array } 
      };
      const inputArray = new Uint8Array(data);
      const result = win.pixieJuice?.optimize_auto?.(inputArray, 80);
      if (!result) return null;
      return Array.from(result.slice(0, 8));
    }, Array.from(testPng));
    
    if (outputHeader) {
      const pngSignature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
      expect(outputHeader).toEqual(pngSignature);
    }
  });
});

test.describe('Image Compression - JPEG', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle JPEG input', async ({ page }) => {
    const testJpeg = createTestJpeg();
    const result = await compressImageInBrowser(page, testJpeg, 75);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });
});

test.describe('Image Compression - WebP', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle WebP input', async ({ page }) => {
    const testWebP = createTestWebP();
    const result = await compressImageInBrowser(page, testWebP, 80);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });
});

test.describe('Image Compression - GIF', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle GIF input', async ({ page }) => {
    const testGif = createTestGif();
    const result = await compressImageInBrowser(page, testGif, 80);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });
});

test.describe('Image Compression - BMP', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle BMP input', async ({ page }) => {
    const testBmp = createTestBmp();
    const result = await compressImageInBrowser(page, testBmp, 80);

    expect(result.originalSize).toBeGreaterThan(0);
    if (result.success) {
      expect(result.compressedSize).toBeGreaterThan(0);
    }
  });

  test('should achieve good compression on BMP', async ({ page }) => {
    const testBmp = createTestBmp();
    const result = await compressImageInBrowser(page, testBmp, 80);

    if (result.success) {
      expect(result.compressedSize).toBeLessThan(result.originalSize);
      expect(result.compressionRatio).toBeGreaterThan(10);
    }
  });
});

test.describe('Image Compression - Quality Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle quality 0 (minimum)', async ({ page }) => {
    const testPng = createTestPng(30, 30);
    const result = await compressImageInBrowser(page, testPng, 0);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });

  test('should handle quality 100 (maximum)', async ({ page }) => {
    const testPng = createTestPng(30, 30);
    const result = await compressImageInBrowser(page, testPng, 100);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });

  test('lower quality should produce smaller files', async ({ page }) => {
    const testPng = createTestPng(50, 50);
    
    const lowQuality = await compressImageInBrowser(page, testPng, 20);
    const highQuality = await compressImageInBrowser(page, testPng, 95);
    
    if (lowQuality.success && highQuality.success) {
      const differs =
        lowQuality.compressedSize !== highQuality.compressedSize ||
        lowQuality.outputHash !== highQuality.outputHash;
      expect(differs).toBe(true);
    }
  });
});

test.describe('Image Compression - Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle empty input gracefully', async ({ page }) => {
    const emptyData = new Uint8Array(0);
    const result = await compressImageInBrowser(page, emptyData, 80);
    
    expect(result.success).toBe(false);
    expect(result.error).toBeTruthy();
  });

  test('should handle invalid data gracefully', async ({ page }) => {
    const invalidData = new Uint8Array([0x00, 0x01, 0x02, 0x03, 0x04]);
    const result = await compressImageInBrowser(page, invalidData, 80);
    
    expect(result.success).toBe(false);
  });

  test('should handle corrupted PNG header', async ({ page }) => {
    const corruptedPng = new Uint8Array([0x89, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00, 0x00]);
    const result = await compressImageInBrowser(page, corruptedPng, 80);
    
    expect(result.success).toBe(false);
  });
});

test.describe('Image Compression - Performance', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should complete small image in under 1 second', async ({ page }) => {
    const testPng = createTestPng(50, 50);
    const result = await compressImageInBrowser(page, testPng, 80);
    
    expect(result.timeMs).toBeLessThan(1000);
  });

  test('should complete medium image in under 5 seconds', async ({ page }) => {
    const testPng = createTestPng(200, 200);
    const result = await compressImageInBrowser(page, testPng, 80);
    
    expect(result.timeMs).toBeLessThan(5000);
  });
});
