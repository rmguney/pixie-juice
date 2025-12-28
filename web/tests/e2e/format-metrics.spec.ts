import { test, expect, Page } from '@playwright/test';
import fs from 'fs';
import path from 'path';

type MetricsRecord = {
  format: string;
  inputSize: number;
  outputSize: number;
  compressionPct: number;
  outputFormat: string;
  ok: boolean;
  error?: string;
};

type MetricsFile = {
  generatedAt: string;
  project: string;
  results: Record<string, MetricsRecord>;
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

function createDeterministicBmp(width: number = 256, height: number = 256): Uint8Array {
  const bytesPerPixel = 3;
  const rowSizeUnpadded = width * bytesPerPixel;
  const rowPadding = (4 - (rowSizeUnpadded % 4)) % 4;
  const rowSize = rowSizeUnpadded + rowPadding;
  const pixelDataSize = rowSize * height;
  const fileSize = 54 + pixelDataSize;

  const bmp = new Uint8Array(fileSize);
  const view = new DataView(bmp.buffer);

  bmp[0] = 0x42;
  bmp[1] = 0x4d;
  view.setUint32(2, fileSize, true);
  view.setUint32(10, 54, true);

  view.setUint32(14, 40, true);
  view.setInt32(18, width, true);
  view.setInt32(22, height, true);
  view.setUint16(26, 1, true);
  view.setUint16(28, 24, true);
  view.setUint32(34, pixelDataSize, true);

  let offset = 54;
  for (let y = height - 1; y >= 0; y--) {
    for (let x = 0; x < width; x++) {
      const stripe = ((x >> 4) + (y >> 4)) & 1;
      const r = stripe ? 220 : 30;
      const g = (x * 5) & 0xff;
      const b = (y * 3) & 0xff;
      bmp[offset++] = b;
      bmp[offset++] = g;
      bmp[offset++] = r;
    }
    for (let p = 0; p < rowPadding; p++) bmp[offset++] = 0;
  }

  return bmp;
}

function calculateCrc32(data: Uint8Array): number {
  let crc = 0xffffffff;
  for (let i = 0; i < data.length; i++) {
    crc ^= data[i];
    for (let j = 0; j < 8; j++) {
      const mask = -(crc & 1);
      crc = (crc >>> 1) ^ (0xedb88320 & mask);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function deflateStoredBlocks(data: Uint8Array): Uint8Array {
  // Stored blocks (no compression). Valid raw DEFLATE stream.
  const blockSize = 65535;
  const chunks: number[] = [];

  let offset = 0;
  while (offset < data.length) {
    const remaining = data.length - offset;
    const size = Math.min(blockSize, remaining);
    const isFinal = offset + size >= data.length;

    chunks.push(isFinal ? 0x01 : 0x00);
    chunks.push(size & 0xff, (size >> 8) & 0xff);
    const nlen = (~size) & 0xffff;
    chunks.push(nlen & 0xff, (nlen >> 8) & 0xff);

    for (let i = 0; i < size; i++) chunks.push(data[offset + i]);
    offset += size;
  }

  return new Uint8Array(chunks);
}

function adler32(data: Uint8Array): number {
  const MOD_ADLER = 65521;
  let a = 1;
  let b = 0;
  for (let i = 0; i < data.length; i++) {
    a = (a + data[i]) % MOD_ADLER;
    b = (b + a) % MOD_ADLER;
  }
  return ((b << 16) | a) >>> 0;
}

function zlibNoCompression(data: Uint8Array): Uint8Array {
  // zlib header for DEFLATE with 32K window, fastest algo.
  const header = new Uint8Array([0x78, 0x01]);
  const deflate = deflateStoredBlocks(data);
  const checksum = adler32(data);

  const out = new Uint8Array(header.length + deflate.length + 4);
  out.set(header, 0);
  out.set(deflate, header.length);
  // Adler32 is big-endian
  out[out.length - 4] = (checksum >>> 24) & 0xff;
  out[out.length - 3] = (checksum >>> 16) & 0xff;
  out[out.length - 2] = (checksum >>> 8) & 0xff;
  out[out.length - 1] = checksum & 0xff;
  return out;
}

function createPngChunk(type: string, payload: Uint8Array): Uint8Array {
  const typeBytes = new TextEncoder().encode(type);
  const length = payload.length;

  const chunk = new Uint8Array(8 + payload.length + 4);
  const view = new DataView(chunk.buffer);
  view.setUint32(0, length, false);
  chunk.set(typeBytes, 4);
  chunk.set(payload, 8);

  const crcData = new Uint8Array(typeBytes.length + payload.length);
  crcData.set(typeBytes, 0);
  crcData.set(payload, typeBytes.length);
  const crc = calculateCrc32(crcData);
  view.setUint32(8 + payload.length, crc, false);

  return chunk;
}

function createDeterministicPng(width: number = 128, height: number = 128): Uint8Array {
  const signature = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  const ihdr = new Uint8Array(13);
  const view = new DataView(ihdr.buffer);
  view.setUint32(0, width, false);
  view.setUint32(4, height, false);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 2; // color type RGB
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  const raw = new Uint8Array(height * (1 + width * 3));
  let p = 0;
  for (let y = 0; y < height; y++) {
    raw[p++] = 0; // filter type 0
    for (let x = 0; x < width; x++) {
      raw[p++] = (x * 3) & 0xff;
      raw[p++] = (y * 5) & 0xff;
      raw[p++] = ((x + y) * 2) & 0xff;
    }
  }

  const deflated = zlibNoCompression(raw);
  const idat = createPngChunk('IDAT', deflated);
  const ihdrChunk = createPngChunk('IHDR', ihdr);
  const iendChunk = createPngChunk('IEND', new Uint8Array(0));

  const out = new Uint8Array(signature.length + ihdrChunk.length + idat.length + iendChunk.length);
  let o = 0;
  out.set(signature, o); o += signature.length;
  out.set(ihdrChunk, o); o += ihdrChunk.length;
  out.set(idat, o); o += idat.length;
  out.set(iendChunk, o);
  return out;
}

function createMinimalGltfWithDataUri(): Uint8Array {
  const positions = new Float32Array([
    0.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
  ]);
  const indices = new Uint16Array([0, 1, 2]);

  const posBytes = new Uint8Array(positions.buffer);
  const idxBytes = new Uint8Array(indices.buffer);

  const bin = new Uint8Array(posBytes.length + idxBytes.length);
  bin.set(posBytes, 0);
  bin.set(idxBytes, posBytes.length);

  const b64 = Buffer.from(bin).toString('base64');
  const json = {
    asset: { version: '2.0' },
    buffers: [{ uri: `data:application/octet-stream;base64,${b64}`, byteLength: bin.length }],
    bufferViews: [
      { buffer: 0, byteOffset: 0, byteLength: posBytes.length, target: 34962 },
      { buffer: 0, byteOffset: posBytes.length, byteLength: idxBytes.length, target: 34963 },
    ],
    accessors: [
      {
        bufferView: 0,
        componentType: 5126,
        count: 3,
        type: 'VEC3',
        min: [0, 0, 0],
        max: [1, 1, 0],
      },
      {
        bufferView: 1,
        componentType: 5123,
        count: 3,
        type: 'SCALAR',
      },
    ],
    meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 1 }] }],
    nodes: [{ mesh: 0 }],
    scenes: [{ nodes: [0] }],
    scene: 0,
  };

  return new TextEncoder().encode(JSON.stringify(json));
}

function createMinimalGlb(): Uint8Array {
  const positions = new Float32Array([
    0.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
  ]);
  const indices = new Uint16Array([0, 1, 2]);

  const posBytes = new Uint8Array(positions.buffer);
  const idxBytes = new Uint8Array(indices.buffer);
  const bin = new Uint8Array(posBytes.length + idxBytes.length);
  bin.set(posBytes, 0);
  bin.set(idxBytes, posBytes.length);

  const json = {
    asset: { version: '2.0' },
    buffers: [{ byteLength: bin.length }],
    bufferViews: [
      { buffer: 0, byteOffset: 0, byteLength: posBytes.length, target: 34962 },
      { buffer: 0, byteOffset: posBytes.length, byteLength: idxBytes.length, target: 34963 },
    ],
    accessors: [
      {
        bufferView: 0,
        componentType: 5126,
        count: 3,
        type: 'VEC3',
        min: [0, 0, 0],
        max: [1, 1, 0],
      },
      {
        bufferView: 1,
        componentType: 5123,
        count: 3,
        type: 'SCALAR',
      },
    ],
    meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 1 }] }],
    nodes: [{ mesh: 0 }],
    scenes: [{ nodes: [0] }],
    scene: 0,
  };

  const jsonText = JSON.stringify(json);
  const jsonBytesUnpadded = new TextEncoder().encode(jsonText);
  const jsonPad = (4 - (jsonBytesUnpadded.length % 4)) % 4;
  const jsonBytes = new Uint8Array(jsonBytesUnpadded.length + jsonPad);
  jsonBytes.set(jsonBytesUnpadded, 0);
  for (let i = jsonBytesUnpadded.length; i < jsonBytes.length; i++) jsonBytes[i] = 0x20;

  const binPad = (4 - (bin.length % 4)) % 4;
  const binBytes = new Uint8Array(bin.length + binPad);
  binBytes.set(bin, 0);

  const totalLength = 12 + 8 + jsonBytes.length + 8 + binBytes.length;
  const glb = new Uint8Array(totalLength);
  const view = new DataView(glb.buffer);

  glb[0] = 0x67;
  glb[1] = 0x6c;
  glb[2] = 0x54;
  glb[3] = 0x46;
  view.setUint32(4, 2, true);
  view.setUint32(8, totalLength, true);

  let offset = 12;
  view.setUint32(offset, jsonBytes.length, true);
  glb[offset + 4] = 0x4a;
  glb[offset + 5] = 0x53;
  glb[offset + 6] = 0x4f;
  glb[offset + 7] = 0x4e;
  offset += 8;
  glb.set(jsonBytes, offset);
  offset += jsonBytes.length;

  view.setUint32(offset, binBytes.length, true);
  glb[offset + 4] = 0x42;
  glb[offset + 5] = 0x49;
  glb[offset + 6] = 0x4e;
  glb[offset + 7] = 0x00;
  offset += 8;
  glb.set(binBytes, offset);

  return glb;
}

function createMinimalSvg(): Uint8Array {
  const svg =
    '<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256">' +
    '<rect width="256" height="256" fill="#ff0000" />' +
    '<circle cx="128" cy="128" r="64" fill="#0000ff" />' +
    '</svg>';
  return new TextEncoder().encode(svg);
}

function createMinimalTiffRgb(width: number = 64, height: number = 64): Uint8Array {
  const bytesPerPixel = 3;
  const pixelBytes = width * height * bytesPerPixel;
  const numIfdEntries = 10;

  const ifdOffset = 8;
  const ifdSize = 2 + 12 * numIfdEntries + 4;
  const bitsOffset = ifdOffset + ifdSize;
  const bitsSize = 3 * 2;
  const pixelOffset = bitsOffset + bitsSize;
  const total = pixelOffset + pixelBytes;

  const tiff = new Uint8Array(total);
  const view = new DataView(tiff.buffer);

  // Header: II, 42, offset to IFD
  tiff[0] = 0x49;
  tiff[1] = 0x49;
  view.setUint16(2, 42, true);
  view.setUint32(4, ifdOffset, true);

  let off = ifdOffset;
  view.setUint16(off, numIfdEntries, true);
  off += 2;

  const writeEntry = (tag: number, fieldType: number, count: number, valueOrOffset: number) => {
    view.setUint16(off + 0, tag, true);
    view.setUint16(off + 2, fieldType, true);
    view.setUint32(off + 4, count, true);
    view.setUint32(off + 8, valueOrOffset, true);
    off += 12;
  };

  writeEntry(256, 4, 1, width);
  writeEntry(257, 4, 1, height);
  writeEntry(258, 3, 3, bitsOffset);
  writeEntry(259, 3, 1, 1);
  writeEntry(262, 3, 1, 2);
  writeEntry(273, 4, 1, pixelOffset);
  writeEntry(277, 3, 1, 3);
  writeEntry(278, 4, 1, height);
  writeEntry(279, 4, 1, pixelBytes);
  writeEntry(284, 3, 1, 1);

  view.setUint32(off, 0, true);

  view.setUint16(bitsOffset + 0, 8, true);
  view.setUint16(bitsOffset + 2, 8, true);
  view.setUint16(bitsOffset + 4, 8, true);

  let p = pixelOffset;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const r = (x * 3) & 0xff;
      const g = (y * 5) & 0xff;
      const b = ((x + y) * 2) & 0xff;
      tiff[p++] = r;
      tiff[p++] = g;
      tiff[p++] = b;
    }
  }

  return tiff;
}

function createFbxHeaderStub(): Uint8Array {
  const header = 'Kaydara FBX Binary  \0\x1a\0';
  const bytes = new Uint8Array(32);
  const enc = new TextEncoder().encode(header);
  bytes.set(enc.slice(0, Math.min(enc.length, bytes.length)));
  return bytes;
}

test.describe('Format Metrics (WASM)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('writes per-format compression metrics', async ({ page }, testInfo) => {
    const bmp = createDeterministicBmp(256, 256);
    const png = createDeterministicPng(128, 128);
    const gltf = createMinimalGltfWithDataUri();
    const glb = createMinimalGlb();
    const svg = createMinimalSvg();
    const tiff = createMinimalTiffRgb(64, 64);
    const fbx = createFbxHeaderStub();

    const metrics = await page.evaluate(
      async ({ bmp, png, gltf, glb, svg, tiff, fbx }) => {
        const api = (window as any).pixieJuice as any;
        if (!api) throw new Error('pixieJuice not available');

        const safeCall = (fn: (() => Uint8Array) | null, inputSize: number): { ok: boolean; out?: Uint8Array; err?: string } => {
          if (!fn) return { ok: false, err: 'missing function' };
          try {
            const out = fn();
            if (!(out instanceof Uint8Array)) return { ok: false, err: 'non-Uint8Array result' };
            if (out.length === 0) return { ok: false, err: 'empty output' };
            return { ok: true, out };
          } catch (e) {
            return { ok: false, err: e instanceof Error ? e.message : String(e) };
          }
        };

        const detect = (bytes: Uint8Array): string => {
          try {
            const f = api.detect_format?.(bytes);
            return typeof f === 'string' ? f : 'unknown';
          } catch {
            return 'unknown';
          }
        };

        const pct = (inSize: number, outSize: number): number => {
          if (inSize <= 0) return 0;
          return ((inSize - outSize) / inSize) * 100;
        };

        const results: Record<string, any> = {};

        const bmpBytes = new Uint8Array(bmp);
        results.bmp = (() => {
          const out = safeCall(() => (api.optimize_auto?.(bmpBytes, 80) ?? api.optimize_image?.(bmpBytes, 80)), bmpBytes.length);
          if (!out.ok) return { format: 'bmp', inputSize: bmpBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return {
            format: 'bmp',
            inputSize: bmpBytes.length,
            outputSize: out.out!.length,
            compressionPct: pct(bmpBytes.length, out.out!.length),
            outputFormat: detect(out.out!),
            ok: true,
          };
        })();

        const jpeg = safeCall(() => api.convert_to_jpeg?.(bmpBytes, 80), bmpBytes.length);
        results.jpeg = (() => {
          if (!jpeg.ok) return { format: 'jpeg', inputSize: bmpBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: jpeg.err };
          const input = jpeg.out!;
          const out = safeCall(() => (api.optimize_jpeg?.(input, 80) ?? api.optimize_image?.(input, 80)), input.length);
          if (!out.ok) return { format: 'jpeg', inputSize: input.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'jpeg', inputSize: input.length, outputSize: out.out!.length, compressionPct: pct(input.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        results.png = (() => {
          const input = new Uint8Array(png);
          const out = safeCall(() => (api.optimize_png?.(input, 80) ?? api.optimize_image?.(input, 80)), input.length);
          if (!out.ok) return { format: 'png', inputSize: input.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'png', inputSize: input.length, outputSize: out.out!.length, compressionPct: pct(input.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        const webp = safeCall(() => api.convert_to_webp?.(bmpBytes, 80), bmpBytes.length);
        results.webp = (() => {
          if (!webp.ok) return { format: 'webp', inputSize: bmpBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: webp.err };
          const input = webp.out!;
          const out = safeCall(() => (api.optimize_webp?.(input, 80) ?? api.optimize_image?.(input, 80)), input.length);
          if (!out.ok) return { format: 'webp', inputSize: input.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'webp', inputSize: input.length, outputSize: out.out!.length, compressionPct: pct(input.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        const gif = safeCall(() => api.convert_to_gif?.(bmpBytes, 80), bmpBytes.length);
        results.gif = (() => {
          if (!gif.ok) return { format: 'gif', inputSize: bmpBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: gif.err };
          const input = gif.out!;
          const out = safeCall(() => (api.optimize_gif?.(input, 80) ?? api.optimize_image?.(input, 80)), input.length);
          if (!out.ok) return { format: 'gif', inputSize: input.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'gif', inputSize: input.length, outputSize: out.out!.length, compressionPct: pct(input.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        results.ico = (() => {
          const pngBytes = new Uint8Array(png);

          const icoHeaderSize = 6;
          const dirEntrySize = 16;
          const imageOffset = icoHeaderSize + dirEntrySize;
          const icoBytes = new Uint8Array(imageOffset + pngBytes.length);
          const view = new DataView(icoBytes.buffer);
          view.setUint16(0, 0, true);
          view.setUint16(2, 1, true);
          view.setUint16(4, 1, true);

          icoBytes[6] = 32;
          icoBytes[7] = 32;
          icoBytes[8] = 0;
          icoBytes[9] = 0;
          view.setUint16(10, 1, true);
          view.setUint16(12, 32, true);
          view.setUint32(14, pngBytes.length, true);
          view.setUint32(18, imageOffset, true);

          icoBytes.set(pngBytes, imageOffset);

          const out = safeCall(() => (api.optimize_ico?.(icoBytes, 80) ?? api.optimize_image?.(icoBytes, 80)), icoBytes.length);
          if (!out.ok) return { format: 'ico', inputSize: icoBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'ico', inputSize: icoBytes.length, outputSize: out.out!.length, compressionPct: pct(icoBytes.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        const tga = safeCall(() => api.convert_to_tga?.(bmpBytes, 80), bmpBytes.length);
        results.tga = (() => {
          if (!tga.ok) return { format: 'tga', inputSize: bmpBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: tga.err };
          const input = tga.out!;
          const out = safeCall(() => (api.optimize_tga?.(input, 80) ?? api.optimize_image?.(input, 80)), input.length);
          if (!out.ok) return { format: 'tga', inputSize: input.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'tga', inputSize: input.length, outputSize: out.out!.length, compressionPct: pct(input.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        results.tiff = (() => {
          const tiffBytes = new Uint8Array(tiff);
          const optimized = safeCall(() => (api.optimize_image?.(tiffBytes, 80) ?? tiffBytes), tiffBytes.length);
          if (!optimized.ok) return { format: 'tiff', inputSize: tiffBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: optimized.err };

          // Metadata stripping is optional; only run it if it returns a valid-looking TIFF.
          if (typeof api.strip_tiff_metadata_simd === 'function') {
            const stripped = safeCall(() => api.strip_tiff_metadata_simd(tiffBytes, true), tiffBytes.length);
            if (stripped.ok && detect(stripped.out!) === 'image:Tiff') {
              const strippedOptimized = safeCall(() => (api.optimize_image?.(stripped.out!, 80) ?? stripped.out!), stripped.out!.length);
              if (strippedOptimized.ok) {
                return {
                  format: 'tiff',
                  inputSize: stripped.out!.length,
                  outputSize: strippedOptimized.out!.length,
                  compressionPct: pct(stripped.out!.length, strippedOptimized.out!.length),
                  outputFormat: detect(strippedOptimized.out!),
                  ok: true,
                };
              }
            }
          }

          return {
            format: 'tiff',
            inputSize: tiffBytes.length,
            outputSize: optimized.out!.length,
            compressionPct: pct(tiffBytes.length, optimized.out!.length),
            outputFormat: detect(optimized.out!),
            ok: true,
          };
        })();

        results.svg = (() => {
          const svgBytes = new Uint8Array(svg);
          const out = safeCall(() => (api.optimize_image?.(svgBytes, 80) ?? api.optimize_auto?.(svgBytes, 80)), svgBytes.length);
          if (!out.ok) return { format: 'svg', inputSize: svgBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'svg', inputSize: svgBytes.length, outputSize: out.out!.length, compressionPct: pct(svgBytes.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        results.gltf = (() => {
          const gltfBytes = new Uint8Array(gltf);
          const out = safeCall(() => (api.optimize_gltf?.(gltfBytes, 80) ?? api.optimize_mesh?.(gltfBytes, 80) ?? api.optimize_auto?.(gltfBytes, 80)), gltfBytes.length);
          if (!out.ok) return { format: 'gltf', inputSize: gltfBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'gltf', inputSize: gltfBytes.length, outputSize: out.out!.length, compressionPct: pct(gltfBytes.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        results.glb = (() => {
          const glbBytes = new Uint8Array(glb);
          const out = safeCall(() => (api.optimize_mesh?.(glbBytes, 80) ?? api.optimize_auto?.(glbBytes, 80)), glbBytes.length);
          if (!out.ok) return { format: 'glb', inputSize: glbBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'glb', inputSize: glbBytes.length, outputSize: out.out!.length, compressionPct: pct(glbBytes.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        results.fbx = (() => {
          const fbxBytes = new Uint8Array(fbx);
          const out = safeCall(() => (api.optimize_fbx?.(fbxBytes, 80) ?? api.optimize_mesh?.(fbxBytes, 80) ?? api.optimize_auto?.(fbxBytes, 80)), fbxBytes.length);
          if (!out.ok) return { format: 'fbx', inputSize: fbxBytes.length, outputSize: 0, compressionPct: 0, outputFormat: 'unknown', ok: false, error: out.err };
          return { format: 'fbx', inputSize: fbxBytes.length, outputSize: out.out!.length, compressionPct: pct(fbxBytes.length, out.out!.length), outputFormat: detect(out.out!), ok: true };
        })();

        return results;
      },
      {
        bmp: Array.from(bmp),
        png: Array.from(png),
        gltf: Array.from(gltf),
        glb: Array.from(glb),
        svg: Array.from(svg),
        tiff: Array.from(tiff),
        fbx: Array.from(fbx),
      }
    );

    const out: MetricsFile = {
      generatedAt: new Date().toISOString(),
      project: testInfo.project.name,
      results: metrics as Record<string, MetricsRecord>,
    };

    const outDir = path.join(process.cwd(), 'tests', 'fixtures', 'generated');
    fs.mkdirSync(outDir, { recursive: true });
    const outPath = path.join(outDir, `format-metrics.${testInfo.project.name}.json`);
    fs.writeFileSync(outPath, JSON.stringify(out, null, 2));

    testInfo.attach('format-metrics', { path: outPath, contentType: 'application/json' });

    for (const [key, r] of Object.entries(out.results)) {
      expect(r.inputSize, `${key}: input size`).toBeGreaterThan(0);
      if (r.ok) {
        expect(r.outputSize, `${key}: output size`).toBeGreaterThan(0);
        expect(r.outputFormat, `${key}: output format`).not.toBe('unknown');
      }
    }
  });
});
