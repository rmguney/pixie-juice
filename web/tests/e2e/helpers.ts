import type { Page } from '@playwright/test';

/** Opens the app and waits until the engine worker reports ready. */
export async function openApp(page: Page): Promise<void> {
  await page.goto('/');
  await page.waitForFunction(() => window.pixieJuice !== undefined, null, { timeout: 30_000 });
  await page.evaluate(installPageHelpers);
}

interface PixelStats {
  width: number;
  height: number;
  meanError: number;
  maxAlphaError: number;
  colourful: boolean;
}

declare global {
  interface Window {
    testing: {
      makeImage(options: {
        width: number;
        height: number;
        mime: 'image/png' | 'image/jpeg' | 'image/webp';
        quality?: number;
        alpha?: boolean;
        noise?: number;
      }): Promise<Uint8Array>;
      /** Decodes with the browser, or with the engine for TIFF/TGA. */
      decode(data: Uint8Array): Promise<{ width: number; height: number; data: Uint8ClampedArray }>;
      compare(a: Uint8Array, b: Uint8Array): Promise<PixelStats>;
      uncompressedTiff(width: number, height: number): Uint8Array;
    };
  }
}

/** Runs in the page: deterministic test images and pixel comparisons. */
function installPageHelpers() {
  const decode = async (data: Uint8Array) => {
    const kind = await window.pixieJuice!.detect(data);
    if (kind === 'image:tiff' || kind === 'image:tga') {
      const image = await window.pixieJuice!.decodeImage(data);
      return { width: image.width, height: image.height, data: image.rgba };
    }
    const bitmap = await createImageBitmap(new Blob([data as BlobPart]), {
      premultiplyAlpha: 'none',
      colorSpaceConversion: 'none',
    });
    const canvas = document.createElement('canvas');
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const context = canvas.getContext('2d', { willReadFrequently: true })!;
    context.drawImage(bitmap, 0, 0);
    return { width: bitmap.width, height: bitmap.height, data: context.getImageData(0, 0, bitmap.width, bitmap.height).data };
  };

  window.testing = {
    async makeImage({ width, height, mime, quality, alpha = false, noise = 24 }) {
      let seed = 1234567;
      const random = () => {
        seed ^= seed << 13;
        seed ^= seed >>> 17;
        seed ^= seed << 5;
        return ((seed >>> 0) % 1000) / 1000;
      };
      const canvas = document.createElement('canvas');
      canvas.width = width;
      canvas.height = height;
      const context = canvas.getContext('2d')!;
      const image = context.createImageData(width, height);
      for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
          const i = (y * width + x) * 4;
          const n = (random() - 0.5) * noise;
          image.data[i] = (x / width) * 255 + n;
          image.data[i + 1] = (y / height) * 200 + 30 + n;
          image.data[i + 2] = 220 - (x / width) * 180 + n;
          image.data[i + 3] = alpha ? (((x >> 4) + (y >> 4)) % 3 === 0 ? 0 : 160 + ((x + y) % 96)) : 255;
        }
      }
      context.putImageData(image, 0, 0);
      const blob = await new Promise<Blob>((resolve, reject) =>
        canvas.toBlob((b) => (b ? resolve(b) : reject(new Error(`${mime} encoding unsupported`))), mime, quality),
      );
      return new Uint8Array(await blob.arrayBuffer());
    },

    decode,

    async compare(a, b) {
      const [x, y] = await Promise.all([decode(a), decode(b)]);
      if (x.width !== y.width || x.height !== y.height) {
        return { width: y.width, height: y.height, meanError: Infinity, maxAlphaError: 255, colourful: false };
      }
      let error = 0;
      let maxAlphaError = 0;
      let colourful = false;
      for (let i = 0; i < x.data.length; i += 4) {
        error += Math.abs(x.data[i] - y.data[i]) + Math.abs(x.data[i + 1] - y.data[i + 1]) + Math.abs(x.data[i + 2] - y.data[i + 2]);
        maxAlphaError = Math.max(maxAlphaError, Math.abs(x.data[i + 3] - y.data[i + 3]));
        if (y.data[i] !== y.data[i + 1] || y.data[i + 1] !== y.data[i + 2]) colourful = true;
      }
      return { width: y.width, height: y.height, meanError: error / ((x.data.length / 4) * 3), maxAlphaError, colourful };
    },

    uncompressedTiff(width, height) {
      // Baseline little-endian RGB TIFF: header, pixel strip, then one IFD.
      const pixels = width * height * 3;
      const entries: [number, number, number, number][] = [
        [256, 4, 1, width], [257, 4, 1, height], [258, 3, 1, 8], [259, 3, 1, 1], [262, 3, 1, 2],
        [273, 4, 1, 8], [277, 3, 1, 3], [278, 4, 1, height], [279, 4, 1, pixels],
      ];
      const ifd = 8 + pixels;
      const buffer = new ArrayBuffer(ifd + 2 + entries.length * 12 + 4);
      const view = new DataView(buffer);
      view.setUint16(0, 0x4949, true);
      view.setUint16(2, 42, true);
      view.setUint32(4, ifd, true);
      // A copy of the RGB channels of the same gradient makeImage draws, without noise.
      for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
          const at = 8 + (y * width + x) * 3;
          view.setUint8(at, Math.round((x / width) * 255));
          view.setUint8(at + 1, Math.round((y / height) * 200 + 30));
          view.setUint8(at + 2, Math.round(220 - (x / width) * 180));
        }
      }
      view.setUint16(ifd, entries.length, true);
      entries.forEach(([tag, type, count, value], i) => {
        const at = ifd + 2 + i * 12;
        view.setUint16(at, tag, true);
        view.setUint16(at + 2, type, true);
        view.setUint32(at + 4, count, true);
        if (type === 3) view.setUint16(at + 8, value, true);
        else view.setUint32(at + 8, value, true);
      });
      return new Uint8Array(buffer);
    },
  };
}
