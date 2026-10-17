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
    const cloud = await api.decodeMesh(text('v 0 0 0\nv 1 0 0\n'));
    const wire = await api.decodeMesh(text('v 0 0 0\nv 1 0 0\nv 1 1 0\nl 1 2 3\n'));
    return {
      positions: Array.from(obj.positions),
      normals: obj.normals?.length,
      colors: obj.colors?.length,
      indices: Array.from(obj.indices),
      types: [obj.positions.constructor.name, obj.indices.constructor.name],
      cloud: { positions: cloud.positions.length, indices: cloud.indices.length, normals: cloud.normals },
      wire: { indices: wire.indices.length, lines: wire.lines?.length },
    };
  });
  expect(r.positions).toEqual([0, 0, 0, 1, 0, 0, 0, 1, 0]);
  expect(r.normals).toBe(9);
  expect(r.colors).toBe(9);
  expect(r.indices).toEqual([0, 1, 2]);
  expect(r.types).toEqual(['Float32Array', 'Uint32Array']);
  expect(r.cloud).toEqual({ positions: 6, indices: 0, normals: undefined });
  expect(r.wire).toEqual({ indices: 0, lines: 4 });
});

test('previews a PLY that optimizing passes through', async ({ page }) => {
  const ply =
    'ply\nformat ascii 1.0\nelement vertex 4\nproperty double x\nproperty double y\nproperty double z\n' +
    'property float confidence\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n' +
    '0 0 0 1\n1 0 0 1\n1 1 0 1\n0 1 0 1\n4 0 1 2 3\n';
  await preview(page, 'scan.ply', Buffer.from(ply));
  await expectModelShown(page);
});

test('shows point clouds and wireframes, and reports a mesh with nothing to show', async ({ page }) => {
  await preview(page, 'cloud.obj', Buffer.from('v 0 0 0\nv 1 0 0\nv 0 1 0\n'));
  await expectModelShown(page);
  await preview(page, 'wire.obj', Buffer.from('v 0 0 0\nv 1 0 0\nv 1 1 0\nl 1 2 3 1\n'));
  await expectModelShown(page);
  await preview(page, 'empty.obj', Buffer.from('# nothing\no empty\ng none\n'));
  await expect(page.getByText('Cannot preview this OBJ: The mesh has no geometry')).toBeVisible();
});

test('a glTF that names external files is not previewed, and nothing is fetched', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  const gltf = JSON.stringify({ asset: { version: '2.0' }, buffers: [{ byteLength: 4, uri: 'https://example.com/track.bin' }] });
  await preview(page, 'remote.gltf', Buffer.from(gltf));
  await expect(page.getByText(/Cannot preview this GLTF: the glTF needs track\.bin/)).toBeVisible();
  expect(requests.filter((url) => url.includes('example.com'))).toEqual([]);
});

test('previews a .gltf whose buffer is embedded as a data: URI', async ({ page }) => {
  const positions = Buffer.from(new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]).buffer);
  const gltf = JSON.stringify({
    asset: { version: '2.0' },
    buffers: [{ byteLength: positions.length, uri: `data:application/octet-stream;base64,${positions.toString('base64')}` }],
    bufferViews: [{ buffer: 0, byteLength: positions.length }],
    accessors: [{ bufferView: 0, componentType: 5126, count: 3, type: 'VEC3', min: [0, 0, 0], max: [1, 1, 0] }],
    meshes: [{ primitives: [{ attributes: { POSITION: 0 } }] }],
    nodes: [{ mesh: 0 }],
    scenes: [{ nodes: [0] }],
    scene: 0,
  });
  await preview(page, 'triangle.gltf', Buffer.from(gltf));
  await expectModelShown(page);
});

/** A 64×64 PNG, red on top and blue below, drawn in the page. */
async function redBluePng(page: Page): Promise<Buffer> {
  const bytes = await page.evaluate(async () => {
    const canvas = Object.assign(document.createElement('canvas'), { width: 64, height: 64 });
    const context = canvas.getContext('2d')!;
    context.fillStyle = '#ff0000';
    context.fillRect(0, 0, 64, 32);
    context.fillStyle = '#0000ff';
    context.fillRect(0, 32, 64, 32);
    const blob = await new Promise<Blob>((resolve) => canvas.toBlob((b) => resolve(b!), 'image/png'));
    return Array.from(new Uint8Array(await blob.arrayBuffer()));
  });
  return Buffer.from(bytes);
}

/** How many pixels of the preview are clearly red and clearly blue: the smaller count. */
async function redAndBluePixels(page: Page): Promise<number> {
  const shot = await page.getByTestId('mesh-preview').screenshot();
  return page.evaluate(async (bytes) => {
    const bitmap = await createImageBitmap(new Blob([new Uint8Array(bytes)], { type: 'image/png' }));
    const canvas = Object.assign(document.createElement('canvas'), { width: bitmap.width, height: bitmap.height });
    const context = canvas.getContext('2d')!;
    context.drawImage(bitmap, 0, 0);
    const { data } = context.getImageData(0, 0, bitmap.width, bitmap.height);
    let red = 0;
    let blue = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (data[i] > 200 && data[i + 1] < 60 && data[i + 2] < 60) red++;
      if (data[i + 2] > 200 && data[i] < 60 && data[i + 1] < 60) blue++;
    }
    return Math.min(red, blue);
  }, Array.from(shot));
}

/** A GLB from its JSON and binary chunk, each padded to four bytes. */
function glb(json: object, bin: Buffer): Buffer {
  const text = Buffer.from(JSON.stringify(json));
  const jsonChunk = Buffer.concat([text, Buffer.alloc((4 - (text.length % 4)) % 4, 0x20)]);
  const binChunk = Buffer.concat([bin, Buffer.alloc((4 - (bin.length % 4)) % 4)]);
  const header = Buffer.alloc(12);
  header.writeUInt32LE(0x46546c67, 0); // "glTF"
  header.writeUInt32LE(2, 4);
  header.writeUInt32LE(12 + 8 + jsonChunk.length + 8 + binChunk.length, 8);
  const chunk = (type: number, data: Buffer) => {
    const head = Buffer.alloc(8);
    head.writeUInt32LE(data.length, 0);
    head.writeUInt32LE(type, 4);
    return Buffer.concat([head, data]);
  };
  return Buffer.concat([header, chunk(0x4e4f534a, jsonChunk), chunk(0x004e4942, binChunk)]); // "JSON", "BIN"
}

test('previews a GLB with its texture', async ({ page }) => {
  // A square on the ground, unlit, textured red on one half and blue on the other.
  const png = await redBluePng(page);
  const shape = square();
  const bin = Buffer.concat([shape.bin, png]);
  const model = glb(
    {
      asset: { version: '2.0' },
      extensionsUsed: ['KHR_materials_unlit'],
      buffers: [{ byteLength: bin.length }],
      bufferViews: [...shape.views, { buffer: 0, byteOffset: 92, byteLength: png.length }],
      accessors: shape.accessors,
      images: [{ bufferView: 3, mimeType: 'image/png' }],
      textures: [{ source: 0 }],
      materials: [
        { pbrMetallicRoughness: { baseColorTexture: { index: 0 } }, doubleSided: true, extensions: { KHR_materials_unlit: {} } },
      ],
      meshes: [{ primitives: [{ attributes: { POSITION: 0, TEXCOORD_0: 1 }, indices: 2, material: 0 }] }],
      nodes: [{ mesh: 0 }],
      scenes: [{ nodes: [0] }],
      scene: 0,
    },
    bin,
  );
  await preview(page, 'textured.glb', model);
  await expectModelShown(page);

  // Both texture colours reach the screen.
  await expect.poll(() => redAndBluePixels(page)).toBeGreaterThan(500);
});

test('previews an ASCII FBX with an embedded texture and a camera', async ({ page }) => {
  const png = await redBluePng(page);
  const fbx = [
    '; FBX 7.3.0 project file',
    'FBXHeaderExtension:  {',
    '\tFBXHeaderVersion: 1003',
    '\tFBXVersion: 7300',
    '}',
    'Objects:  {',
    '\tGeometry: 1000, "Geometry::quad", "Mesh" {',
    '\t\tVertices: *12 {',
    '\t\t\ta: -1,0,-1,1,0,-1,1,0,1,-1,0,1',
    '\t\t}',
    // One quad facing up (counter-clockwise seen from above), with normals for the lighting.
    '\t\tPolygonVertexIndex: *4 {',
    '\t\t\ta: 0,3,2,-2',
    '\t\t}',
    '\t\tLayerElementNormal: 0 {',
    '\t\t\tMappingInformationType: "ByPolygonVertex"',
    '\t\t\tReferenceInformationType: "Direct"',
    '\t\t\tNormals: *12 {',
    '\t\t\t\ta: 0,1,0,0,1,0,0,1,0,0,1,0',
    '\t\t\t}',
    '\t\t}',
    '\t\tLayerElementUV: 0 {',
    '\t\t\tMappingInformationType: "ByPolygonVertex"',
    '\t\t\tReferenceInformationType: "IndexToDirect"',
    '\t\t\tUV: *8 {',
    '\t\t\t\ta: 0,0,1,0,1,1,0,1',
    '\t\t\t}',
    '\t\t\tUVIndex: *4 {',
    '\t\t\t\ta: 0,3,2,1',
    '\t\t\t}',
    '\t\t}',
    '\t}',
    '\tModel: 2000, "Model::quad", "Mesh" {',
    '\t\tVersion: 232',
    '\t}',
    '\tMaterial: 3000, "Material::painted", "" {',
    '\t\tShadingModel: "lambert"',
    '\t}',
    '\tTexture: 4000, "Texture::paint", "" {',
    '\t\tFileName: "paint.png"',
    '\t}',
    '\tVideo: 5000, "Video::paint", "Clip" {',
    '\t\tRelativeFilename: "paint.png"',
    '\t\tContent: ,',
    `\t\t\t"${png.toString('base64')}"`,
    '\t}',
    '\tNodeAttribute: 6000, "NodeAttribute::camera", "Camera" {',
    '\t\tTypeFlags: "Camera"',
    '\t}',
    '\tModel: 7000, "Model::camera", "Camera" {',
    '\t\tVersion: 232',
    '\t}',
    '}',
    'Connections:  {',
    '\tC: "OO",1000,2000',
    '\tC: "OO",2000,0',
    '\tC: "OO",3000,2000',
    '\tC: "OP",4000,3000, "DiffuseColor"',
    '\tC: "OO",5000,4000',
    '\tC: "OO",6000,7000',
    '\tC: "OO",7000,0',
    '}',
    '',
  ].join('\n');
  await preview(page, 'painted.fbx', Buffer.from(fbx));
  await expectModelShown(page);
  await expect.poll(() => redAndBluePixels(page)).toBeGreaterThan(500);
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
  // Encoded on a blank page: the app's Content Security Policy refuses the injected encoder script.
  const encoder = await page.context().newPage();
  await encoder.addScriptTag({
    path: fileURLToPath(new URL('../../node_modules/three/examples/jsm/libs/draco/draco_encoder.js', import.meta.url)),
  });
  const glb = await encoder.evaluate(() => {
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
  await encoder.close();
  await preview(page, 'square-draco.glb', Buffer.from(glb));
  await expectModelShown(page);
});

/** A unit square on the ground, as glTF JSON pieces and its buffer (positions, UVs, indices). */
function square(): { bin: Buffer; views: object[]; accessors: object[] } {
  const positions = Buffer.from(new Float32Array([-1, 0, -1, 1, 0, -1, 1, 0, 1, -1, 0, 1]).buffer);
  const uvs = Buffer.from(new Float32Array([0, 0, 1, 0, 1, 1, 0, 1]).buffer);
  const indices = Buffer.from(new Uint16Array([0, 2, 1, 0, 3, 2]).buffer);
  return {
    bin: Buffer.concat([positions, uvs, indices]),
    views: [
      { buffer: 0, byteOffset: 0, byteLength: 48 },
      { buffer: 0, byteOffset: 48, byteLength: 32 },
      { buffer: 0, byteOffset: 80, byteLength: 12 },
    ],
    accessors: [
      { bufferView: 0, componentType: 5126, count: 4, type: 'VEC3', min: [-1, 0, -1], max: [1, 0, 1] },
      { bufferView: 1, componentType: 5126, count: 4, type: 'VEC2' },
      { bufferView: 2, componentType: 5123, count: 6, type: 'SCALAR' },
    ],
  };
}

test('previews a .gltf with its .bin and texture from the list, and names a missing one', async ({ page }) => {
  const png = await redBluePng(page);
  const { bin, views, accessors } = square();
  const gltf = {
    asset: { version: '2.0' },
    extensionsUsed: ['KHR_materials_unlit'],
    buffers: [{ byteLength: bin.length, uri: 'scene%20data.bin' }],
    bufferViews: views,
    accessors,
    images: [{ uri: 'textures/paint.png' }],
    textures: [{ source: 0 }],
    materials: [{ pbrMetallicRoughness: { baseColorTexture: { index: 0 } }, doubleSided: true, extensions: { KHR_materials_unlit: {} } }],
    meshes: [{ primitives: [{ attributes: { POSITION: 0, TEXCOORD_0: 1 }, indices: 2, material: 0 }] }],
    nodes: [{ mesh: 0 }],
    scenes: [{ nodes: [0] }],
    scene: 0,
  };
  await page.locator('#file-input').setInputFiles([
    { name: 'scene.gltf', mimeType: 'model/gltf+json', buffer: Buffer.from(JSON.stringify(gltf)) },
  ]);
  await page.getByRole('button', { name: 'Preview scene.gltf' }).click();
  await expect(page.getByText(/needs scene data\.bin/)).toBeVisible();

  // Adding the files loads it again, with them.
  await page.locator('#file-input').setInputFiles([
    { name: 'scene data.bin', mimeType: 'application/octet-stream', buffer: bin },
    { name: 'paint.png', mimeType: 'image/png', buffer: png },
  ]);
  await page.getByRole('button', { name: 'Preview scene.gltf' }).click();
  await expectModelShown(page);
  await expect.poll(() => redAndBluePixels(page)).toBeGreaterThan(500);

  // The engine takes them too (packing them into a GLB when that comes out smaller, which a
  // file this small doesn't), and names what's missing without them.
  const packed = await page.evaluate(
    async ({ json, buffer, image }) => {
      const api = window.pixieJuice!;
      const input = new TextEncoder().encode(json);
      const out = await api.optimize(input, {
        resources: [
          { name: 'scene data.bin', data: new Uint8Array(buffer) },
          { name: 'paint.png', data: new Uint8Array(image) },
        ],
      });
      const missing = await api.optimize(input).then(
        () => '',
        (error: Error) => error.message,
      );
      return { format: out.format, missing };
    },
    { json: JSON.stringify(gltf), buffer: Array.from(bin), image: Array.from(png) },
  );
  expect(['glb', 'gltf']).toContain(packed.format);
  expect(packed.missing).toMatch(/needs scene data\.bin/);
});

test('previews a KTX2 texture (KHR_texture_basisu)', async ({ page }) => {
  // A 64 x 64 KTX2, red over blue, written by three.js's exporter.
  const { DataTexture, RGBAFormat, SRGBColorSpace, UnsignedByteType } = await import('three');
  const { KTX2Exporter } = await import('three/addons/exporters/KTX2Exporter.js');
  const pixels = new Uint8Array(64 * 64 * 4);
  for (let i = 0; i < 64 * 64; i++) {
    const red = i < 64 * 32;
    pixels.set(red ? [255, 0, 0, 255] : [0, 0, 255, 255], i * 4);
  }
  const texture = new DataTexture(pixels, 64, 64, RGBAFormat, UnsignedByteType);
  texture.colorSpace = SRGBColorSpace;
  const ktx2 = Buffer.from(await new KTX2Exporter().parse(texture));
  const { bin, views, accessors } = square();
  const model = glb(
    {
      asset: { version: '2.0' },
      extensionsUsed: ['KHR_materials_unlit', 'KHR_texture_basisu'],
      extensionsRequired: ['KHR_texture_basisu'],
      buffers: [{ byteLength: bin.length + ktx2.length }],
      bufferViews: [...views, { buffer: 0, byteOffset: 92, byteLength: ktx2.length }],
      accessors,
      images: [{ bufferView: 3, mimeType: 'image/ktx2' }],
      textures: [{ extensions: { KHR_texture_basisu: { source: 0 } } }],
      materials: [{ pbrMetallicRoughness: { baseColorTexture: { index: 0 } }, doubleSided: true, extensions: { KHR_materials_unlit: {} } }],
      meshes: [{ primitives: [{ attributes: { POSITION: 0, TEXCOORD_0: 1 }, indices: 2, material: 0 }] }],
      nodes: [{ mesh: 0 }],
      scenes: [{ nodes: [0] }],
      scene: 0,
    },
    Buffer.concat([bin, ktx2]),
  );
  await preview(page, 'compressed.glb', model);
  await expectModelShown(page);
  await expect.poll(() => redAndBluePixels(page)).toBeGreaterThan(500);
});

test('plays a glTF animation, with a control to pause it', async ({ page }) => {
  const { bin, views, accessors } = square();
  // Two keyframes moving the square up and back.
  const times = Buffer.from(new Float32Array([0, 1, 2]).buffer);
  const moves = Buffer.from(new Float32Array([0, 0, 0, 0, 0.5, 0, 0, 0, 0]).buffer);
  const data = Buffer.concat([bin, times, moves]);
  const model = glb(
    {
      asset: { version: '2.0' },
      buffers: [{ byteLength: data.length }],
      bufferViews: [...views, { buffer: 0, byteOffset: 92, byteLength: 12 }, { buffer: 0, byteOffset: 104, byteLength: 36 }],
      accessors: [
        ...accessors,
        { bufferView: 3, componentType: 5126, count: 3, type: 'SCALAR', min: [0], max: [2] },
        { bufferView: 4, componentType: 5126, count: 3, type: 'VEC3' },
      ],
      meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 2 }] }],
      nodes: [{ mesh: 0, name: 'square' }],
      animations: [{ channels: [{ sampler: 0, target: { node: 0, path: 'translation' } }], samplers: [{ input: 3, output: 4 }] }],
      scenes: [{ nodes: [0] }],
      scene: 0,
    },
    data,
  );
  await preview(page, 'moving.glb', model);
  await expectModelShown(page);
  await page.getByRole('button', { name: 'Pause animation' }).click();
  await expect(page.getByRole('button', { name: 'Play animation' })).toBeVisible();
});

test('previews a glTF with its own lights and camera', async ({ page }) => {
  const { bin, views, accessors } = square();
  const model = glb(
    {
      asset: { version: '2.0' },
      extensionsUsed: ['KHR_lights_punctual'],
      extensions: { KHR_lights_punctual: { lights: [{ type: 'point', intensity: 20, color: [1, 0.8, 0.6] }, { type: 'directional' }] } },
      buffers: [{ byteLength: bin.length }],
      bufferViews: views,
      accessors,
      cameras: [{ type: 'perspective', perspective: { yfov: 0.8, znear: 0.1 } }],
      meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 2 }] }],
      nodes: [
        { mesh: 0 },
        { translation: [0, 2, 0], extensions: { KHR_lights_punctual: { light: 0 } } },
        { rotation: [-0.7071, 0, 0, 0.7071], extensions: { KHR_lights_punctual: { light: 1 } } },
        { translation: [0, 3, 4], rotation: [-0.2, 0, 0, 0.98], camera: 0 },
      ],
      scenes: [{ nodes: [0, 1, 2, 3] }],
      scene: 0,
    },
    bin,
  );
  await preview(page, 'lit.glb', model);
  await expectModelShown(page);
});
