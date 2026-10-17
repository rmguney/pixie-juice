// A ZIP archive of files stored as they are (the results are compressed
// already), so a batch downloads as one file.

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  return table;
})();

export function crc32(data: Uint8Array): number {
  let crc = 0xffffffff;
  for (let i = 0; i < data.length; i++) crc = CRC_TABLE[(crc ^ data[i]) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

/** Time and date in MS-DOS format, as ZIP stores them. */
function dosTime(date: Date): [number, number] {
  const time = (date.getHours() << 11) | (date.getMinutes() << 5) | (date.getSeconds() >> 1);
  const day = ((Math.max(date.getFullYear(), 1980) - 1980) << 9) | ((date.getMonth() + 1) << 5) | date.getDate();
  return [time, day];
}

export function zip(files: { name: string; data: Uint8Array }[], date = new Date()): Blob {
  const encoder = new TextEncoder();
  const [time, day] = dosTime(date);
  const parts: BlobPart[] = [];
  const directory: Uint8Array[] = [];
  let offset = 0;
  for (const { name, data } of files) {
    const path = encoder.encode(name);
    const crc = crc32(data);
    // Local file header, then the data; flag 0x0800: the name is UTF-8.
    const local = new DataView(new ArrayBuffer(30));
    local.setUint32(0, 0x04034b50, true);
    local.setUint16(4, 20, true);
    local.setUint16(6, 0x0800, true);
    local.setUint16(8, 0, true); // stored
    local.setUint16(10, time, true);
    local.setUint16(12, day, true);
    local.setUint32(14, crc, true);
    local.setUint32(18, data.length, true);
    local.setUint32(22, data.length, true);
    local.setUint16(26, path.length, true);
    parts.push(local.buffer, path as BlobPart, data as BlobPart);

    const entry = new DataView(new ArrayBuffer(46));
    entry.setUint32(0, 0x02014b50, true);
    entry.setUint16(4, 20, true);
    entry.setUint16(6, 20, true);
    entry.setUint16(8, 0x0800, true);
    entry.setUint16(12, time, true);
    entry.setUint16(14, day, true);
    entry.setUint32(16, crc, true);
    entry.setUint32(20, data.length, true);
    entry.setUint32(24, data.length, true);
    entry.setUint16(28, path.length, true);
    entry.setUint32(42, offset, true);
    directory.push(new Uint8Array(entry.buffer), path);

    offset += 30 + path.length + data.length;
    if (offset > 0xffffffff || files.length > 0xffff) throw new Error('Too much for one ZIP file');
  }
  const directorySize = directory.reduce((sum, part) => sum + part.length, 0);
  const end = new DataView(new ArrayBuffer(22));
  end.setUint32(0, 0x06054b50, true);
  end.setUint16(8, files.length, true);
  end.setUint16(10, files.length, true);
  end.setUint32(12, directorySize, true);
  end.setUint32(16, offset, true);
  return new Blob([...parts, ...(directory as BlobPart[]), end.buffer], { type: 'application/zip' });
}
