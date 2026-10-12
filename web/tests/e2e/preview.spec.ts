import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { test, expect, type Page } from '@playwright/test';
import { openApp } from './helpers';

test.beforeEach(async ({ page }) => openApp(page));

async function preview(page: Page, name: string, buffer: Buffer) {
  await page.locator('#file-input').setInputFiles({ name, mimeType: 'application/octet-stream', buffer });
  await expect(page.getByRole('button', { name: `Preview ${name}` })).toBeVisible();
}

/** The model finished loading, without an error message. */
async function expectModelShown(page: Page) {
  await expect(page.getByTestId('mesh-preview')).toHaveAttribute('aria-busy', 'false');
  await expect(page.getByText(/Cannot preview/)).toHaveCount(0);
}

test('the engine decodes OBJ, STL and PLY triangles for display', async ({ page }) => {
  const r = await page.evaluate(async () => {
    const api = window.pixieJuice!;
    const text = (s: string) => new TextEncoder().encode(s);
    const obj = await api.decodeMesh(text('v 0 0 0 1 0 0\nv 1 0 0 0 1 0\nv 0 1 0 0 0 1\nvn 0 0 1\nf 1//1 2//1 3//1\n'));
    let error = '';
    try {
      await api.decodeMesh(text('v 0 0 0\nv 1 0 0\n'));
    } catch (e) {
      error = (e as Error).message;
    }
    return {
      positions: Array.from(obj.positions),
      normals: obj.normals?.length,
      colors: obj.colors?.length,
      indices: Array.from(obj.indices),
      types: [obj.positions.constructor.name, obj.indices.constructor.name],
      error,
    };
  });
  expect(r.positions).toEqual([0, 0, 0, 1, 0, 0, 0, 1, 0]);
  expect(r.normals).toBe(9);
  expect(r.colors).toBe(9);
  expect(r.indices).toEqual([0, 1, 2]);
  expect(r.types).toEqual(['Float32Array', 'Uint32Array']);
  expect(r.error).toBe('The mesh has no triangles');
});

test('previews a PLY that optimizing passes through', async ({ page }) => {
  const ply =
    'ply\nformat ascii 1.0\nelement vertex 4\nproperty double x\nproperty double y\nproperty double z\n' +
    'property float confidence\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n' +
    '0 0 0 1\n1 0 0 1\n1 1 0 1\n0 1 0 1\n4 0 1 2 3\n';
  await preview(page, 'scan.ply', Buffer.from(ply));
  await expectModelShown(page);
});

test('reports a mesh without triangles instead of showing an empty scene', async ({ page }) => {
  await preview(page, 'cloud.obj', Buffer.from('v 0 0 0\nv 1 0 0\nv 0 1 0\n'));
  await expect(page.getByText('Cannot preview this OBJ: The mesh has no triangles')).toBeVisible();
});

test('previews a GLB compressed with EXT_meshopt_compression', async ({ page }) => {
  const glb = readFileSync(fileURLToPath(new URL('./fixtures/grid-meshopt.glb', import.meta.url)));
  await preview(page, 'grid-meshopt.glb', glb);
  await expectModelShown(page);
});

/** The parts of Draco's encoder module (shipped with three.js) used below. */
interface DracoEncoderModule {
  Encoder: new () => { EncodeMeshToDracoBuffer(mesh: object, out: object): number };
  MeshBuilder: new () => {
    AddFacesToMesh(mesh: object, faces: number, indices: Uint32Array): boolean;
    AddFloatAttributeToMesh(mesh: object, type: number, count: number, size: number, values: Float32Array): number;
  };
  Mesh: new () => object;
  DracoInt8Array: new () => { GetValue(index: number): number };
  POSITION: number;
}

test('previews a GLB compressed with KHR_draco_mesh_compression', async ({ page }) => {
  await page.addScriptTag({
    path: fileURLToPath(new URL('../../node_modules/three/examples/jsm/libs/draco/draco_encoder.js', import.meta.url)),
  });
  const glb = await page.evaluate(() => {
    // A square: four corners, two triangles, Draco-encoded.
    const draco = (window as unknown as { DracoEncoderModule: () => DracoEncoderModule }).DracoEncoderModule();
    const mesh = new draco.Mesh();
    const builder = new draco.MeshBuilder();
    builder.AddFacesToMesh(mesh, 2, new Uint32Array([0, 1, 2, 0, 2, 3]));
    const position = builder.AddFloatAttributeToMesh(
      mesh, draco.POSITION, 4, 3, new Float32Array([0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0]));
    const encoded = new draco.DracoInt8Array();
    const length = new draco.Encoder().EncodeMeshToDracoBuffer(mesh, encoded);
    const bin = new Uint8Array((length + 3) & ~3);
    for (let i = 0; i < length; i++) bin[i] = encoded.GetValue(i) & 0xff;

    const json = new TextEncoder().encode(JSON.stringify({
      asset: { version: '2.0' },
      extensionsUsed: ['KHR_draco_mesh_compression'],
      extensionsRequired: ['KHR_draco_mesh_compression'],
      buffers: [{ byteLength: bin.length }],
      bufferViews: [{ buffer: 0, byteLength: length }],
      accessors: [
        { componentType: 5126, count: 4, type: 'VEC3', min: [0, 0, 0], max: [1, 1, 0] },
        { componentType: 5125, count: 6, type: 'SCALAR' },
      ],
      meshes: [{
        primitives: [{
          attributes: { POSITION: 0 },
          indices: 1,
          extensions: { KHR_draco_mesh_compression: { bufferView: 0, attributes: { POSITION: position } } },
        }],
      }],
      nodes: [{ mesh: 0 }],
      scenes: [{ nodes: [0] }],
      scene: 0,
    }));
    const jsonLength = (json.length + 3) & ~3;
    const glb = new Uint8Array(12 + 8 + jsonLength + 8 + bin.length);
    const view = new DataView(glb.buffer);
    view.setUint32(0, 0x46546c67, true);  // "glTF"
    view.setUint32(4, 2, true);
    view.setUint32(8, glb.length, true);
    view.setUint32(12, jsonLength, true);
    view.setUint32(16, 0x4e4f534a, true);  // "JSON"
    glb.fill(0x20, 20, 20 + jsonLength);  // JSON is padded with spaces
    glb.set(json, 20);
    view.setUint32(20 + jsonLength, bin.length, true);
    view.setUint32(24 + jsonLength, 0x004e4942, true);  // "BIN"
    glb.set(bin, 28 + jsonLength);
    return Array.from(glb);
  });
  await preview(page, 'square-draco.glb', Buffer.from(glb));
  await expectModelShown(page);
});
