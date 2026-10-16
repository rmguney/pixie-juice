// A three.js scene as plain data that can cross to another thread: the model
// worker parses a file and serializes the result; the page rebuilds it. Buffers
// and image bitmaps are transferred, not copied. Meshes, points and lines keep
// their geometry, materials (with textures) and the node hierarchy; cameras,
// lights, skeletons and animations are left out of the preview.

import {
  BufferAttribute,
  BufferGeometry,
  Group,
  InstancedBufferAttribute,
  InstancedMesh,
  Line,
  LineBasicMaterial,
  LineDashedMaterial,
  LineLoop,
  LineSegments,
  type Material,
  Matrix4,
  Mesh,
  MeshBasicMaterial,
  MeshLambertMaterial,
  MeshPhongMaterial,
  MeshPhysicalMaterial,
  MeshStandardMaterial,
  type Object3D,
  Points,
  PointsMaterial,
  Source,
  Texture,
  type TypedArray,
} from 'three';

type Value = number | boolean | string | null | { color: number[] } | { vector2: number[] } | { texture: number };

interface AttributeData {
  array: TypedArray;
  itemSize: number;
  normalized: boolean;
}

interface GeometryData {
  attributes: Record<string, AttributeData>;
  index: TypedArray | null;
  groups: { start: number; count: number; materialIndex?: number }[];
}

interface MaterialData {
  type: string;
  values: Record<string, Value>;
}

interface TextureData {
  /** Into SceneData.images: textures that shared an image still share it. */
  image: number;
  values: Record<string, Value>;
}

type NodeKind = 'group' | 'mesh' | 'instancedMesh' | 'points' | 'line' | 'lineSegments' | 'lineLoop';

interface NodeData {
  kind: NodeKind;
  name: string;
  /** The local transform, column-major. */
  matrix: number[];
  geometry?: number;
  material?: number | number[];
  instances?: { count: number; matrices: Float32Array; colors?: AttributeData };
  children: NodeData[];
}

export interface SceneData {
  root: NodeData;
  geometries: GeometryData[];
  materials: MaterialData[];
  textures: TextureData[];
  images: ImageBitmap[];
}

const MATERIALS: Record<string, new () => Material> = {
  MeshStandardMaterial,
  MeshPhysicalMaterial,
  MeshBasicMaterial,
  MeshPhongMaterial,
  MeshLambertMaterial,
  PointsMaterial,
  LineBasicMaterial,
  LineDashedMaterial,
};

/** Texture settings worth carrying; the image itself travels separately. */
const TEXTURE_KEYS = [
  'name', 'mapping', 'channel', 'wrapS', 'wrapT', 'magFilter', 'minFilter', 'anisotropy', 'offset', 'repeat',
  'center', 'rotation', 'generateMipmaps', 'premultiplyAlpha', 'colorSpace',
] as const;

/** Material fields that describe the object, not its appearance. */
const SKIPPED_MATERIAL_KEYS = new Set(['uuid', 'type', 'userData', 'defines', 'version']);

function kindOf(node: Object3D): NodeKind {
  if ((node as InstancedMesh).isInstancedMesh) return 'instancedMesh';
  if ((node as Mesh).isMesh) return 'mesh';
  if ((node as Points).isPoints) return 'points';
  if ((node as LineSegments).isLineSegments) return 'lineSegments';
  if ((node as LineLoop).isLineLoop) return 'lineLoop';
  if ((node as Line).isLine) return 'line';
  return 'group';
}

/** Collects a scene into SceneData and the buffers and bitmaps to transfer with it. */
export async function serializeScene(root: Object3D): Promise<{ scene: SceneData; transfer: Transferable[] }> {
  const geometries: GeometryData[] = [];
  const materials: MaterialData[] = [];
  const textures: TextureData[] = [];
  const images: ImageBitmap[] = [];
  const geometryIds = new Map<BufferGeometry, number>();
  const materialIds = new Map<Material, number>();
  const textureIds = new Map<Texture, number>();
  // One image per three.js Source (and orientation): clones that share one stay one upload.
  const imageIds = new Map<Source, { straight?: number; flipped?: number }>();
  const flippedOnly = new Set<ImageBitmap>();
  const transfer = new Set<Transferable>();

  function attribute(source: BufferGeometry['attributes'][string]): AttributeData {
    if ('isInterleavedBufferAttribute' in source && source.isInterleavedBufferAttribute) {
      // Stored with other attributes in one buffer: copied out to its own.
      const { data, offset, itemSize, count } = source;
      const array = new (source.array.constructor as new (length: number) => TypedArray)(count * itemSize);
      for (let i = 0; i < count; i++) {
        for (let k = 0; k < itemSize; k++) array[i * itemSize + k] = data.array[i * data.stride + offset + k];
      }
      transfer.add(array.buffer as ArrayBuffer);
      return { array, itemSize, normalized: source.normalized };
    }
    transfer.add(source.array.buffer as ArrayBuffer);
    return { array: source.array, itemSize: source.itemSize, normalized: source.normalized };
  }

  function geometry(source: BufferGeometry): number {
    let id = geometryIds.get(source);
    if (id === undefined) {
      const attributes: Record<string, AttributeData> = {};
      for (const [name, value] of Object.entries(source.attributes)) attributes[name] = attribute(value);
      if (source.index) transfer.add(source.index.array.buffer as ArrayBuffer);
      id = geometries.push({ attributes, index: source.index?.array ?? null, groups: source.groups.map((g) => ({ ...g })) }) - 1;
      geometryIds.set(source, id);
    }
    return id;
  }

  /** The index of the texture's image, flipped if the texture asks for it. */
  async function image(source: Texture): Promise<number | undefined> {
    const original: unknown = source.image;
    if (!(typeof ImageBitmap !== 'undefined' && original instanceof ImageBitmap)) return undefined; // compressed or failed
    const ids = imageIds.get(source.source) ?? {};
    imageIds.set(source.source, ids);
    const variant = source.flipY ? 'flipped' : 'straight';
    if (ids[variant] === undefined) {
      // WebGL can't flip an ImageBitmap on upload, so a texture that expects it gets flipped pixels.
      const bitmap = source.flipY ? await createImageBitmap(original, { imageOrientation: 'flipY' }) : original;
      if (source.flipY && ids.straight === undefined) flippedOnly.add(original);
      if (!source.flipY) flippedOnly.delete(original);
      transfer.add(bitmap);
      ids[variant] = images.push(bitmap) - 1;
    }
    return ids[variant];
  }

  async function texture(source: Texture): Promise<number | undefined> {
    let id = textureIds.get(source);
    if (id !== undefined) return id;
    const index = await image(source);
    if (index === undefined) return undefined;
    const values: Record<string, Value> = {};
    for (const key of TEXTURE_KEYS) values[key] = plain(source[key]) ?? null;
    id = textures.push({ image: index, values }) - 1;
    textureIds.set(source, id);
    return id;
  }

  async function material(source: Material): Promise<number> {
    let id = materialIds.get(source);
    if (id === undefined) {
      const values: Record<string, Value> = {};
      for (const field of Object.keys(source)) {
        // Some properties (alphaTest, clearcoat, sheen, ...) are getters over a "_" field.
        const key = field.startsWith('_') ? field.slice(1) : field;
        if (SKIPPED_MATERIAL_KEYS.has(key) || !(key in source)) continue;
        const value: unknown = (source as unknown as Record<string, unknown>)[key];
        if ((value as Texture | null)?.isTexture === true) {
          const index = await texture(value as Texture);
          if (index !== undefined) values[key] = { texture: index };
        } else {
          const simple = plain(value);
          if (simple !== undefined) values[key] = simple;
        }
      }
      id = materials.push({ type: source.type, values }) - 1;
      materialIds.set(source, id);
    }
    return id;
  }

  async function node(source: Object3D): Promise<NodeData> {
    source.updateMatrix();
    const data: NodeData = { kind: kindOf(source), name: source.name, matrix: source.matrix.toArray(), children: [] };
    if (data.kind !== 'group') {
      const drawable = source as Mesh;
      data.geometry = geometry(drawable.geometry);
      // One material at a time, so ones sharing a texture find it already serialized.
      if (Array.isArray(drawable.material)) {
        data.material = [];
        for (const each of drawable.material) data.material.push(await material(each));
      } else {
        data.material = await material(drawable.material);
      }
    }
    if (data.kind === 'instancedMesh') {
      const { count, instanceMatrix, instanceColor } = source as InstancedMesh;
      const matrices = (instanceMatrix.array as Float32Array).subarray(0, count * 16);
      transfer.add(matrices.buffer as ArrayBuffer);
      data.instances = { count, matrices, colors: instanceColor ? attribute(instanceColor) : undefined };
    }
    for (const child of source.children) data.children.push(await node(child));
    return data;
  }

  const scene: SceneData = { root: await node(root), geometries, materials, textures, images };
  // Originals only ever used flipped aren't sent: free them now rather than at collection.
  for (const original of flippedOnly) original.close();
  return { scene, transfer: [...transfer] };
}

/** A value as plain data: primitives, colours and 2D vectors; undefined for anything else. */
function plain(value: unknown): Value | undefined {
  if (value === null || ['number', 'boolean', 'string'].includes(typeof value)) return value as Value;
  if (typeof value !== 'object') return undefined;
  if ((value as { isColor?: boolean }).isColor) return { color: (value as { toArray(): number[] }).toArray() };
  if ((value as { isVector2?: boolean }).isVector2) return { vector2: (value as { toArray(): number[] }).toArray() };
  return undefined;
}

/** Applies serialized values to a three.js object, skipping any it doesn't have. */
function assign(target: object, values: Record<string, Value>, textures: Texture[]) {
  const fields = target as Record<string, unknown>;
  for (const [key, value] of Object.entries(values)) {
    if (!(key in target)) continue;
    const current = fields[key];
    if (value !== null && typeof value === 'object') {
      if ('texture' in value) fields[key] = textures[value.texture];
      else if ('color' in value) (current as { fromArray?(a: number[]): void })?.fromArray?.(value.color);
      else (current as { fromArray?(a: number[]): void })?.fromArray?.(value.vector2);
    } else if (current === null || current === undefined || typeof current === typeof value) {
      fields[key] = value;
    }
  }
}

/** Rebuilds a scene from SceneData. */
export function deserializeScene({ root, geometries, materials, textures, images }: SceneData): Object3D {
  const sources = images.map((image) => new Source(image));
  const builtTextures = textures.map(({ image, values }) => {
    const texture = new Texture();
    texture.source = sources[image];
    assign(texture, values, []);
    texture.flipY = false; // already oriented (see serializeScene)
    texture.needsUpdate = true;
    return texture;
  });
  const builtMaterials = materials.map(({ type, values }) => {
    const material = new (MATERIALS[type] ?? MeshStandardMaterial)();
    assign(material, values, builtTextures);
    material.needsUpdate = true;
    return material;
  });
  const builtGeometries = geometries.map(({ attributes, index, groups }) => {
    const geometry = new BufferGeometry();
    for (const [name, { array, itemSize, normalized }] of Object.entries(attributes)) {
      geometry.setAttribute(name, new BufferAttribute(array, itemSize, normalized));
    }
    if (index) geometry.setIndex(new BufferAttribute(index, 1));
    for (const { start, count, materialIndex } of groups) geometry.addGroup(start, count, materialIndex);
    return geometry;
  });

  function build(data: NodeData): Object3D {
    const geometry = data.geometry === undefined ? undefined : builtGeometries[data.geometry];
    const material = Array.isArray(data.material)
      ? data.material.map((i) => builtMaterials[i])
      : data.material === undefined
        ? undefined
        : builtMaterials[data.material];
    let object: Object3D;
    switch (data.kind) {
      case 'mesh': object = new Mesh(geometry, material); break;
      case 'points': object = new Points(geometry, material); break;
      case 'line': object = new Line(geometry, material); break;
      case 'lineSegments': object = new LineSegments(geometry, material); break;
      case 'lineLoop': object = new LineLoop(geometry, material); break;
      case 'instancedMesh': {
        const { count, matrices, colors } = data.instances!;
        const mesh = new InstancedMesh(geometry, material, count);
        mesh.instanceMatrix.array.set(matrices);
        if (colors) mesh.instanceColor = new InstancedBufferAttribute(colors.array, colors.itemSize, colors.normalized);
        object = mesh;
        break;
      }
      default: object = new Group();
    }
    object.name = data.name;
    new Matrix4().fromArray(data.matrix).decompose(object.position, object.quaternion, object.scale);
    for (const child of data.children) object.add(build(child));
    return object;
  }

  return build(root);
}
