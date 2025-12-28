import { test, expect, Page } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const BASELINE_FILE = path.join(__dirname, '../fixtures/regression-baseline.json');

interface RegressionBaseline {
  version: string;
  timestamp: string;
  results: {
    [testName: string]: {
      compressionRatio: number;
      timeMs: number;
      outputSize: number;
      inputSize: number;
      outputHash?: number;
    };
  };
}

interface TestResult {
  success: boolean;
  compressionRatio: number;
  timeMs: number;
  outputSize: number;
  inputSize: number;
  outputHash?: number;
  error?: string;
}

const REGRESSION_TOLERANCE = {
  compressionRatio: 5,
  timeMs: 50,
};

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

function loadBaseline(): RegressionBaseline | null {
  try {
    if (fs.existsSync(BASELINE_FILE)) {
      return JSON.parse(fs.readFileSync(BASELINE_FILE, 'utf-8'));
    }
  } catch {
    console.warn('Could not load regression baseline');
  }
  return null;
}

function saveBaseline(baseline: RegressionBaseline): void {
  const dir = path.dirname(BASELINE_FILE);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
  fs.writeFileSync(BASELINE_FILE, JSON.stringify(baseline, null, 2));
}

async function runCompressionTest(
  page: Page,
  testData: Uint8Array,
  quality: number = 80
): Promise<TestResult> {
  return await page.evaluate(async ({ data, quality }) => {
    const win = window as unknown as { 
      pixieJuice?: { 
        optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array 
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
    const inputArray = new Uint8Array(data);
    
    try {
      const result = win.pixieJuice?.optimize_auto?.(inputArray, quality);
      const endTime = performance.now();

      if (!result) {
        return {
          success: false,
          compressionRatio: 0,
          timeMs: endTime - startTime,
          outputSize: 0,
          inputSize: inputArray.length,
          error: 'optimize_auto returned null/undefined'
        };
      }

      const ratio = ((inputArray.length - result.length) / inputArray.length) * 100;
      return {
        success: true,
        compressionRatio: ratio,
        timeMs: endTime - startTime,
        outputSize: result.length,
        inputSize: inputArray.length,
        outputHash: fnv1a32(result)
      };
    } catch (error) {
      return {
        success: false,
        compressionRatio: 0,
        timeMs: performance.now() - startTime,
        outputSize: 0,
        inputSize: inputArray.length,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }, { data: Array.from(testData), quality });
}

function createDeterministicPng(): Uint8Array {
  const width = 64, height = 64;
  const signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
  
  const ihdr = createChunk('IHDR', [
    0, 0, 0, width,
    0, 0, 0, height,
    8, 2, 0, 0, 0
  ]);
  
  const rawData: number[] = [];
  for (let y = 0; y < height; y++) {
    rawData.push(0);
    for (let x = 0; x < width; x++) {
      rawData.push((x * 4) % 256);
      rawData.push((y * 4) % 256);
      rawData.push(((x + y) * 2) % 256);
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
  return new Uint8Array([
    ...header, ...chunks,
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

function createDeterministicObj(): Uint8Array {
  const obj = `# Regression test OBJ
o TestCube
v -1.0 -1.0 1.0
v -1.0 1.0 1.0
v 1.0 1.0 1.0
v 1.0 -1.0 1.0
v -1.0 -1.0 -1.0
v -1.0 1.0 -1.0
v 1.0 1.0 -1.0
v 1.0 -1.0 -1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 -1.0
f 1//1 2//1 3//1 4//1
f 5//2 8//2 7//2 6//2
`;
  return new TextEncoder().encode(obj);
}

test.describe('Regression Tests', () => {
  let baseline: RegressionBaseline | null = null;
  const currentResults: RegressionBaseline['results'] = {};

  test.beforeAll(() => {
    baseline = loadBaseline();
  });

  test.afterAll(async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto('/');
    await waitForWasmReady(page);
    
    const version = await page.evaluate(() => {
      const win = window as unknown as { pixieJuice?: { version?: () => string } };
      return win.pixieJuice?.version?.() || 'unknown';
    });
    
    const newBaseline: RegressionBaseline = {
      version,
      timestamp: new Date().toISOString(),
      results: currentResults
    };
    
    if (process.env.UPDATE_BASELINE === 'true') {
      saveBaseline(newBaseline);
      console.log('Baseline updated');
    }
    
    await page.close();
  });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('PNG compression regression', async ({ page }) => {
    const testPng = createDeterministicPng();
    const result = await runCompressionTest(page, testPng, 80);
    
    currentResults['png_compression_80'] = result;
    
    if (baseline?.results['png_compression_80']) {
      const baselineResult = baseline.results['png_compression_80'];
      const ratioDiff = Math.abs(result.compressionRatio - baselineResult.compressionRatio);
      
      expect(ratioDiff).toBeLessThan(REGRESSION_TOLERANCE.compressionRatio);
    }
    
    expect(result.inputSize).toBeGreaterThan(0);
  });

  test('PNG high quality regression', async ({ page }) => {
    const testPng = createDeterministicPng();
    const result = await runCompressionTest(page, testPng, 95);
    
    currentResults['png_compression_95'] = result;
    
    if (baseline?.results['png_compression_95']) {
      const baselineResult = baseline.results['png_compression_95'];
      const ratioDiff = Math.abs(result.compressionRatio - baselineResult.compressionRatio);
      
      expect(ratioDiff).toBeLessThan(REGRESSION_TOLERANCE.compressionRatio);
    }
  });

  test('PNG low quality regression', async ({ page }) => {
    const testPng = createDeterministicPng();
    const result = await runCompressionTest(page, testPng, 20);
    
    currentResults['png_compression_20'] = result;
    
    if (baseline?.results['png_compression_20']) {
      const baselineResult = baseline.results['png_compression_20'];
      const ratioDiff = Math.abs(result.compressionRatio - baselineResult.compressionRatio);
      
      expect(ratioDiff).toBeLessThan(REGRESSION_TOLERANCE.compressionRatio);
    }
  });

  test('OBJ mesh regression', async ({ page }) => {
    const testObj = createDeterministicObj();
    const result = await runCompressionTest(page, testObj, 80);
    
    currentResults['obj_mesh_compression'] = result;
    
    if (baseline?.results['obj_mesh_compression']) {
      const baselineResult = baseline.results['obj_mesh_compression'];
      const ratioDiff = Math.abs(result.compressionRatio - baselineResult.compressionRatio);
      
      expect(ratioDiff).toBeLessThan(REGRESSION_TOLERANCE.compressionRatio);
    }
  });

  test('Performance regression - small image', async ({ page }) => {
    const testPng = createDeterministicPng();
    const result = await runCompressionTest(page, testPng, 80);
    
    currentResults['perf_small_image'] = result;
    
    if (baseline?.results['perf_small_image']) {
      const baselineTime = baseline.results['perf_small_image'].timeMs;
      const allowedTime = baselineTime * (1 + REGRESSION_TOLERANCE.timeMs / 100);
      
      expect(result.timeMs).toBeLessThan(allowedTime);
    }
    
    expect(result.timeMs).toBeLessThan(2000);
  });

  test('Consistent output size', async ({ page }) => {
    const testPng = createDeterministicPng();
    
    const results = await Promise.all([
      runCompressionTest(page, testPng, 80),
      runCompressionTest(page, testPng, 80),
      runCompressionTest(page, testPng, 80),
    ]);
    
    const sizes = results.map(r => r.outputSize);
    const allSame = sizes.every(s => s === sizes[0]);
    
    expect(allSame).toBe(true);
  });

  test('Quality setting affects output', async ({ page }) => {
    const testPng = createDeterministicPng();
    
    const lowQuality = await runCompressionTest(page, testPng, 20);
    const highQuality = await runCompressionTest(page, testPng, 95);
    
    currentResults['quality_comparison_low'] = lowQuality;
    currentResults['quality_comparison_high'] = highQuality;

    expect(lowQuality.success).toBe(true);
    expect(highQuality.success).toBe(true);

    const differs =
      lowQuality.outputSize !== highQuality.outputSize ||
      lowQuality.outputHash !== highQuality.outputHash;
    expect(differs).toBe(true);
  });
});

test.describe('Snapshot Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('WASM module exports snapshot', async ({ page }) => {
    const exports = await page.evaluate(() => {
      const win = window as unknown as { pixieJuice?: Record<string, unknown> };
      if (!win.pixieJuice) return [];
      return Object.keys(win.pixieJuice).sort();
    });
    
    expect(exports).toMatchObject(expect.arrayContaining([
      'optimize_image',
      'optimize_mesh',
      'optimize_auto',
      'detect_format',
      'version'
    ]));
  });

  test('Performance metrics structure snapshot', async ({ page }) => {
    const metrics = await page.evaluate(() => {
      const win = window as unknown as { 
        pixieJuice?: { get_performance_metrics?: () => Record<string, unknown> } 
      };
      const result = win.pixieJuice?.get_performance_metrics?.();
      if (!result) return null;
      return Object.keys(result).sort();
    });
    
    if (metrics) {
      expect(metrics).toEqual(expect.arrayContaining([
        'images_processed',
        'meshes_processed'
      ]));
    }
  });
});
