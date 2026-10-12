import { test, expect } from '@playwright/test';
import { openApp } from './helpers';

test.beforeEach(async ({ page }) => openApp(page));

/** A 20 x 20 grid: 800 triangles, every triangle with its own corner copies. */
function gridObj(n = 20): string {
  let text = 'o grid\n';
  let count = 0;
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) {
      for (const [px, py] of [[x, y], [x + 1, y], [x + 1, y + 1], [x, y], [x + 1, y + 1], [x, y + 1]]) {
        text += `v ${px.toFixed(6)} ${py.toFixed(6)} 0.000000\n`;
      }
      text += `f ${count + 1} ${count + 2} ${count + 3}\nf ${count + 4} ${count + 5} ${count + 6}\n`;
      count += 6;
    }
  }
  return text;
}

test('OBJ: welded, smaller, every triangle kept; simplification on request', async ({ page }) => {
  const r = await page.evaluate(async (obj) => {
    const api = window.pixieJuice!;
    const input = new TextEncoder().encode(obj);
    const faces = (data: Uint8Array) => new TextDecoder().decode(data).split('\n').filter((l) => l.startsWith('f ')).length;
    const full = await api.optimize(input);
    const reduced = await api.optimize(input, { meshRatio: 0.25 });
    return { inputSize: input.length, fullSize: full.data.length, format: full.format, full: faces(full.data), reduced: faces(reduced.data) };
  }, gridObj());
  expect(r.format).toBe('obj');
  expect(r.fullSize).toBeLessThan(r.inputSize);
  expect(r.full).toBe(800);
  expect(r.reduced).toBeGreaterThan(0);
  expect(r.reduced).toBeLessThan(800);
});

test('STL: binary passes through untouched; ASCII is compacted', async ({ page }) => {
  const r = await page.evaluate(async () => {
    const api = window.pixieJuice!;
    const triangles = 64;
    const binary = new Uint8Array(84 + triangles * 50);
    const view = new DataView(binary.buffer);
    view.setUint32(80, triangles, true);
    for (let t = 0; t < triangles; t++) {
      for (let k = 0; k < 9; k++) view.setFloat32(84 + t * 50 + 12 + k * 4, Math.sin(t * 9 + k), true);
    }
    const binaryOut = await api.optimize(binary);

    let ascii = 'solid demo\n';
    for (let t = 0; t < 20; t++) {
      ascii += '  facet normal 0.000000e+00 0.000000e+00 1.000000e+00\n    outer loop\n';
      ascii += `      vertex ${t}.000000 0.000000 0.000000\n      vertex ${t + 1}.000000 0.000000 0.000000\n      vertex ${t}.000000 1.000000 0.000000\n`;
      ascii += '    endloop\n  endfacet\n';
    }
    ascii += 'endsolid demo\n';
    const asciiIn = new TextEncoder().encode(ascii);
    const asciiOut = await api.optimize(asciiIn);
    const text = new TextDecoder().decode(asciiOut.data);
    return {
      binaryUnchanged: !binaryOut.changed && binaryOut.data.length === binary.length,
      asciiSmaller: asciiOut.data.length < asciiIn.length,
      facets: (text.match(/facet normal/g) ?? []).length,
    };
  });
  expect(r.binaryUnchanged).toBe(true);
  expect(r.asciiSmaller).toBe(true);
  expect(r.facets).toBe(20);
});

test('PLY: optimized in its own format with every face', async ({ page }) => {
  const r = await page.evaluate(async () => {
    let body = '';
    let vertices = 0;
    const faces: string[] = [];
    for (let y = 0; y < 8; y++) {
      for (let x = 0; x < 8; x++) {
        const base = vertices;
        for (const [px, py] of [[x, y], [x + 1, y], [x + 1, y + 1], [x, y + 1]]) {
          body += `${px}.000000 ${py}.000000 0.000000\n`;
          vertices++;
        }
        faces.push(`3 ${base} ${base + 1} ${base + 2}`, `3 ${base} ${base + 2} ${base + 3}`);
      }
    }
    const ply =
      `ply\nformat ascii 1.0\ncomment made by a test\nelement vertex ${vertices}\nproperty float x\nproperty float y\nproperty float z\n` +
      `element face ${faces.length}\nproperty list uchar int vertex_indices\nend_header\n${body}${faces.join('\n')}\n`;
    const input = new TextEncoder().encode(ply);
    const out = await window.pixieJuice!.optimize(input);
    const text = new TextDecoder().decode(out.data);
    return {
      format: out.format,
      smaller: out.data.length < input.length,
      faceCount: Number(/element face (\d+)/.exec(text)?.[1]),
      vertexCount: Number(/element vertex (\d+)/.exec(text)?.[1]),
    };
  });
  expect(r.format).toBe('ply');
  expect(r.smaller).toBe(true);
  expect(r.faceCount).toBe(128);
  expect(r.vertexCount).toBe(81);
});

test('glTF: gltfpack produces a valid GLB with every triangle', async ({ page }) => {
  const r = await page.evaluate(async () => {
    const n = 16;
    const positions: number[] = [];
    const indices: number[] = [];
    for (let y = 0; y <= n; y++) for (let x = 0; x <= n; x++) positions.push(x, y, 0);
    for (let y = 0; y < n; y++) {
      for (let x = 0; x < n; x++) {
        const i = y * (n + 1) + x;
        indices.push(i, i + 1, i + n + 2, i, i + n + 2, i + n + 1);
      }
    }
    const buffer = new Uint8Array(positions.length * 4 + indices.length * 4);
    new Float32Array(buffer.buffer, 0, positions.length).set(positions);
    new Uint32Array(buffer.buffer, positions.length * 4, indices.length).set(indices);
    let binary = '';
    buffer.forEach((b) => (binary += String.fromCharCode(b)));
    const gltf = {
      asset: { version: '2.0' },
      scene: 0,
      scenes: [{ nodes: [0] }],
      nodes: [{ name: 'grid', mesh: 0 }],
      meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 1 }] }],
      accessors: [
        { bufferView: 0, componentType: 5126, count: positions.length / 3, type: 'VEC3', min: [0, 0, 0], max: [n, n, 0] },
        { bufferView: 1, componentType: 5125, count: indices.length, type: 'SCALAR' },
      ],
      bufferViews: [
        { buffer: 0, byteLength: positions.length * 4 },
        { buffer: 0, byteOffset: positions.length * 4, byteLength: indices.length * 4 },
      ],
      buffers: [{ byteLength: buffer.length, uri: `data:application/octet-stream;base64,${btoa(binary)}` }],
    };
    const input = new TextEncoder().encode(JSON.stringify(gltf));
    const out = await window.pixieJuice!.optimize(input);

    // GLB: 12-byte header, then a JSON chunk.
    const view = new DataView(out.data.buffer, out.data.byteOffset);
    const jsonLength = view.getUint32(12, true);
    const json = JSON.parse(new TextDecoder().decode(out.data.subarray(20, 20 + jsonLength)));
    const indexAccessor = json.accessors[json.meshes[0].primitives[0].indices];
    return {
      format: out.format,
      magic: view.getUint32(0, true) === 0x46546c67,
      totalLength: view.getUint32(8, true) === out.data.length,
      triangles: indexAccessor.count / 3,
      nodeName: json.nodes.some((node: { name?: string }) => node.name === 'grid'),
    };
  });
  expect(r.format).toBe('glb');
  expect(r.magic).toBe(true);
  expect(r.totalLength).toBe(true);
  expect(r.triangles).toBe(2 * 16 * 16);
  expect(r.nodeName).toBe(true);
});

test('FBX is detected and passed through unchanged', async ({ page }) => {
  const r = await page.evaluate(async () => {
    const input = new TextEncoder().encode('Kaydara FBX Binary  \0\x1a\0' + ' '.repeat(64));
    const out = await window.pixieJuice!.optimize(input);
    return { format: out.format, changed: out.changed, same: out.data.length === input.length };
  });
  expect(r).toEqual({ format: 'fbx', changed: false, same: true });
});
