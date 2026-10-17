// @vitest-environment node
import { crc32 as nodeCrc32 } from 'node:zlib';
import { describe, it, expect } from 'vitest';
import { crc32, zip } from '../../src/utils/zip';

describe('zip', () => {
  it('stores each file with its name, size and checksum, readable from the central directory', async () => {
    const files = [
      { name: 'first_juice.png', data: new Uint8Array([1, 2, 3, 4, 5]) },
      { name: 'ünïcode name.webp', data: new Uint8Array(1000).map((_, i) => i * 7) },
    ];
    const archive = new Uint8Array(await zip(files, new Date(2026, 9, 18, 12, 30, 10)).arrayBuffer());
    const view = new DataView(archive.buffer);

    const end = archive.length - 22;
    expect(view.getUint32(end, true)).toBe(0x06054b50);
    expect(view.getUint16(end + 10, true)).toBe(2);
    let at = view.getUint32(end + 16, true);
    for (const file of files) {
      expect(view.getUint32(at, true)).toBe(0x02014b50);
      expect(view.getUint16(at + 8, true) & 0x0800).toBe(0x0800); // UTF-8 names
      expect(view.getUint32(at + 16, true)).toBe(nodeCrc32(file.data) >>> 0);
      expect(view.getUint32(at + 24, true)).toBe(file.data.length);
      const nameLength = view.getUint16(at + 28, true);
      expect(new TextDecoder().decode(archive.subarray(at + 46, at + 46 + nameLength))).toBe(file.name);
      const local = view.getUint32(at + 42, true);
      expect(view.getUint32(local, true)).toBe(0x04034b50);
      const start = local + 30 + view.getUint16(local + 26, true);
      expect(archive.subarray(start, start + file.data.length)).toEqual(file.data);
      at += 46 + nameLength;
    }
  });

  it('computes the standard CRC-32', () => {
    expect(crc32(new TextEncoder().encode('123456789'))).toBe(0xcbf43926);
  });
});
