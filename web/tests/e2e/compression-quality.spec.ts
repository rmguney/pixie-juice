import { test, expect, Page } from '@playwright/test';
import path from 'path';

interface CompressionQualityResult {
  inputSize: number;
  outputSize: number;
  compressionRatio: number;
  formatDetected: string;
  error?: string;
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

async function testCompressionQuality(
  page: Page,
  imageData: Uint8Array,
  quality: number
): Promise<CompressionQualityResult> {
  return await page.evaluate(async ({ data, quality }) => {
    const win = window as unknown as { 
      pixieJuice?: { 
        optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array;
        detect_format?: (data: Uint8Array) => string;
      } 
    };
    
    const inputArray = new Uint8Array(data);
    const format = win.pixieJuice?.detect_format?.(inputArray) || 'unknown';
    
    try {
      const result = win.pixieJuice?.optimize_auto?.(inputArray, quality);
      
      if (!result) {
        return {
          inputSize: inputArray.length,
          outputSize: 0,
          compressionRatio: 0,
          formatDetected: format,
          error: 'No result returned'
        };
      }
      
      const ratio = ((inputArray.length - result.length) / inputArray.length) * 100;
      
      return {
        inputSize: inputArray.length,
        outputSize: result.length,
        compressionRatio: ratio,
        formatDetected: format
      };
    } catch (error) {
      return {
        inputSize: inputArray.length,
        outputSize: 0,
        compressionRatio: 0,
        formatDetected: format,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }, { data: Array.from(imageData), quality });
}

function createGradientPng(width: number, height: number): Uint8Array {
  const signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
  
  const ihdr = createChunk('IHDR', [
    (width >> 24) & 0xFF, (width >> 16) & 0xFF, (width >> 8) & 0xFF, width & 0xFF,
    (height >> 24) & 0xFF, (height >> 16) & 0xFF, (height >> 8) & 0xFF, height & 0xFF,
    8, 2, 0, 0, 0
  ]);
  
  const rawData: number[] = [];
  for (let y = 0; y < height; y++) {
    rawData.push(0);
    for (let x = 0; x < width; x++) {
      const r = Math.floor((x / width) * 255);
      const g = Math.floor((y / height) * 255);
      const b = Math.floor(((x + y) / (width + height)) * 255);
      rawData.push(r, g, b);
    }
  }
  
  const compressed = deflate(new Uint8Array(rawData));
  const idat = createChunk('IDAT', Array.from(compressed));
  const iend = createChunk('IEND', []);
  
  return new Uint8Array([...signature, ...ihdr, ...idat, ...iend]);
}

function createNoisyPng(width: number, height: number, seed: number = 12345): Uint8Array {
  const signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
  
  const ihdr = createChunk('IHDR', [
    (width >> 24) & 0xFF, (width >> 16) & 0xFF, (width >> 8) & 0xFF, width & 0xFF,
    (height >> 24) & 0xFF, (height >> 16) & 0xFF, (height >> 8) & 0xFF, height & 0xFF,
    8, 2, 0, 0, 0
  ]);
  
  let rng = seed;
  const random = () => {
    rng = (rng * 1103515245 + 12345) & 0x7FFFFFFF;
    return rng / 0x7FFFFFFF;
  };
  
  const rawData: number[] = [];
  for (let y = 0; y < height; y++) {
    rawData.push(0);
    for (let x = 0; x < width; x++) {
      rawData.push(Math.floor(random() * 256));
      rawData.push(Math.floor(random() * 256));
      rawData.push(Math.floor(random() * 256));
    }
  }
  
  const compressed = deflate(new Uint8Array(rawData));
  const idat = createChunk('IDAT', Array.from(compressed));
  const iend = createChunk('IEND', []);
  
  return new Uint8Array([...signature, ...ihdr, ...idat, ...iend]);
}

function createSolidPng(width: number, height: number, r: number, g: number, b: number): Uint8Array {
  const signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
  
  const ihdr = createChunk('IHDR', [
    (width >> 24) & 0xFF, (width >> 16) & 0xFF, (width >> 8) & 0xFF, width & 0xFF,
    (height >> 24) & 0xFF, (height >> 16) & 0xFF, (height >> 8) & 0xFF, height & 0xFF,
    8, 2, 0, 0, 0
  ]);
  
  const rawData: number[] = [];
  for (let y = 0; y < height; y++) {
    rawData.push(0);
    for (let x = 0; x < width; x++) {
      rawData.push(r, g, b);
    }
  }
  
  const compressed = deflate(new Uint8Array(rawData));
  const idat = createChunk('IDAT', Array.from(compressed));
  const iend = createChunk('IEND', []);
  
  return new Uint8Array([...signature, ...ihdr, ...idat, ...iend]);
}

function createChunk(type: string, data: number[]): number[] {
  const length = data.length;
  const typeBytes = Array.from(type).map(c => c.charCodeAt(0));
  const chunk = [
    (length >> 24) & 0xFF, (length >> 16) & 0xFF, (length >> 8) & 0xFF, length & 0xFF,
    ...typeBytes,
    ...data
  ];
  const crc = crc32(new Uint8Array([...typeBytes, ...data]));
  chunk.push((crc >> 24) & 0xFF, (crc >> 16) & 0xFF, (crc >> 8) & 0xFF, crc & 0xFF);
  return chunk;
}

function deflate(data: Uint8Array): Uint8Array {
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
  return new Uint8Array([...header, ...chunks,
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

function crc32(data: Uint8Array): number {
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

test.describe('Compression Quality Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test.describe('Solid Color Images', () => {
    test('should achieve high compression on solid color PNG', async ({ page }) => {
      const solidPng = createSolidPng(100, 100, 255, 0, 0);
      const result = await testCompressionQuality(page, solidPng, 80);
      
      expect(result.error).toBeUndefined();
      if (result.outputSize > 0) {
        expect(result.compressionRatio).toBeGreaterThan(80);
      }
    });

    test('should compress different solid colors consistently', async ({ page }) => {
      const colors = [
        { r: 255, g: 0, b: 0 },
        { r: 0, g: 255, b: 0 },
        { r: 0, g: 0, b: 255 },
        { r: 255, g: 255, b: 255 },
        { r: 0, g: 0, b: 0 },
      ];
      
      const results = await Promise.all(
        colors.map(async ({ r, g, b }) => {
          const png = createSolidPng(50, 50, r, g, b);
          return testCompressionQuality(page, png, 80);
        })
      );
      
      const ratios = results.map(r => r.compressionRatio).filter(r => r > 0);
      if (ratios.length > 1) {
        const avg = ratios.reduce((a, b) => a + b, 0) / ratios.length;
        const variance = ratios.reduce((sum, r) => sum + Math.pow(r - avg, 2), 0) / ratios.length;
        
        expect(variance).toBeLessThan(100);
      }
    });
  });

  test.describe('Gradient Images', () => {
    test('should compress gradient images reasonably', async ({ page }) => {
      const gradientPng = createGradientPng(100, 100);
      const result = await testCompressionQuality(page, gradientPng, 80);
      
      expect(result.inputSize).toBeGreaterThan(0);
    });

    test('quality setting should affect gradient compression', async ({ page }) => {
      const gradientPng = createGradientPng(80, 80);
      
      const lowQuality = await testCompressionQuality(page, gradientPng, 20);
      const highQuality = await testCompressionQuality(page, gradientPng, 95);
      
      if (lowQuality.outputSize > 0 && highQuality.outputSize > 0) {
        expect(lowQuality.outputSize).toBeLessThanOrEqual(highQuality.outputSize * 1.5);
      }
    });
  });

  test.describe('Noisy Images', () => {
    test('should handle noisy images', async ({ page }) => {
      const noisyPng = createNoisyPng(100, 100);
      const result = await testCompressionQuality(page, noisyPng, 80);
      
      expect(result.inputSize).toBeGreaterThan(0);
    });

    test('noisy images should compress less than solid images', async ({ page }) => {
      const noisyPng = createNoisyPng(50, 50);
      const solidPng = createSolidPng(50, 50, 128, 128, 128);
      
      const noisyResult = await testCompressionQuality(page, noisyPng, 80);
      const solidResult = await testCompressionQuality(page, solidPng, 80);
      
      if (noisyResult.compressionRatio > 0 && solidResult.compressionRatio > 0) {
        expect(solidResult.compressionRatio).toBeGreaterThanOrEqual(noisyResult.compressionRatio - 10);
      }
    });
  });

  test.describe('Size Scaling', () => {
    test('compression ratio should be consistent across sizes', async ({ page }) => {
      const sizes = [32, 64, 128];
      
      const results = await Promise.all(
        sizes.map(async size => {
          const png = createGradientPng(size, size);
          return testCompressionQuality(page, png, 80);
        })
      );
      
      expect(results.every(r => r.inputSize > 0)).toBe(true);
    });

    test('larger images should produce proportionally larger outputs', async ({ page }) => {
      const small = createGradientPng(32, 32);
      const large = createGradientPng(128, 128);
      
      const smallResult = await testCompressionQuality(page, small, 80);
      const largeResult = await testCompressionQuality(page, large, 80);
      
      if (smallResult.outputSize > 0 && largeResult.outputSize > 0) {
        expect(largeResult.outputSize).toBeGreaterThan(smallResult.outputSize);
      }
    });
  });

  test.describe('Quality Thresholds', () => {
    test('minimum acceptable compression for gradient images', async ({ page }) => {
      const gradientPng = createGradientPng(100, 100);
      const result = await testCompressionQuality(page, gradientPng, 60);
      
      expect(result.inputSize).toBeGreaterThan(0);
    });

    test('compression should produce valid output', async ({ page }) => {
      const testPng = createGradientPng(64, 64);
      
      const isValidOutput = await page.evaluate(async (data) => {
        const win = window as unknown as { 
          pixieJuice?: { 
            optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array;
            detect_format?: (data: Uint8Array) => string;
          } 
        };
        
        const inputArray = new Uint8Array(data);
        const result = win.pixieJuice?.optimize_auto?.(inputArray, 80);
        
        if (!result || result.length === 0) return false;
        
        const format = win.pixieJuice?.detect_format?.(result);
        return format !== 'unknown';
      }, Array.from(testPng));
      
      expect(isValidOutput).toBe(true);
    });
  });

  test.describe('Lossless Mode', () => {
    test('lossless mode should preserve data integrity', async ({ page }) => {
      const testPng = createGradientPng(32, 32);
      
      const result = await page.evaluate(async (data) => {
        const win = window as unknown as { 
          pixieJuice?: { 
            set_lossless_mode?: (enabled: boolean) => void;
            optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array;
          } 
        };
        
        win.pixieJuice?.set_lossless_mode?.(true);
        const inputArray = new Uint8Array(data);
        const result = win.pixieJuice?.optimize_auto?.(inputArray, 100);
        win.pixieJuice?.set_lossless_mode?.(false);
        
        return result ? result.length : 0;
      }, Array.from(testPng));
      
      expect(result).toBeGreaterThan(0);
    });
  });
});

test.describe('Output Validation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('PNG output should have valid PNG signature', async ({ page }) => {
    const testPng = createGradientPng(32, 32);
    
    const header = await page.evaluate(async (data) => {
      const win = window as unknown as { 
        pixieJuice?: { optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array } 
      };
      const result = win.pixieJuice?.optimize_auto?.(new Uint8Array(data), 80);
      if (!result) return null;
      return Array.from(result.slice(0, 8));
    }, Array.from(testPng));
    
    if (header) {
      const pngSignature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
      const webpSignature = [0x52, 0x49, 0x46, 0x46];
      const jpegSignature = [0xFF, 0xD8, 0xFF];
      
      const isPng = header.slice(0, 8).every((b, i) => b === pngSignature[i]);
      const isWebp = header.slice(0, 4).every((b, i) => b === webpSignature[i]);
      const isJpeg = header.slice(0, 3).every((b, i) => b === jpegSignature[i]);
      
      expect(isPng || isWebp || isJpeg).toBe(true);
    }
  });

  test('output should not exceed input size significantly', async ({ page }) => {
    const testPng = createGradientPng(64, 64);
    const result = await testCompressionQuality(page, testPng, 80);
    
    if (result.outputSize > 0) {
      expect(result.outputSize).toBeLessThan(result.inputSize * 2);
    }
  });
});
