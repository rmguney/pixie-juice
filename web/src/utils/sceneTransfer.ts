// A three.js scene as plain data that can cross to another thread: the model
// worker parses a file and serializes the result; the page rebuilds it. Buffers
// are transferred, not copied; images cross as the encoded files they were (a
// Blob shares its bytes) and are decoded on the page, and KTX2 textures the
// worker transcoded cross as pixels. Meshes (skinned and morphing ones too),
// points and lines keep their geometry, materials and hierarchy; cameras,
// lights and animation clips come along.

import {
  AmbientLight,
  AnimationClip,
  Bone,
  BooleanKeyframeTrack,
  BufferAttribute,
  BufferGeometry,
  type Camera,
  type Color,
  ColorKeyframeTrack,
  DataTexture,
  DirectionalLight,
  Group,
  HemisphereLight,
  InstancedBufferAttribute,
  InstancedMesh,
  InterpolateLinear,
  type InterpolationModes,
  type KeyframeTrack,
  type Light,
  Line,
  LineBasicMaterial,
  LineDashedMaterial,
  LineLoop,
  LineSegments,
  LinearFilter,
  LinearMipmapLinearFilter,
  type Material,
  Matrix4,
  Mesh,
  MeshBasicMaterial,
  MeshLambertMaterial,
  MeshPhongMaterial,
  MeshPhysicalMaterial,
  MeshStandardMaterial,
  NumberKeyframeTrack,
  type Object3D,
  OrthographicCamera,
  PerspectiveCamera,
  PointLight,
  Points,
  PointsMaterial,
  QuaternionKeyframeTrack,
  RGBAFormat,
  Skeleton,
  SkinnedMesh,
  Source,
  SpotLight,
  StringKeyframeTrack,
  Texture,
  type TypedArray,
  UnsignedByteType,
  Vector3,
  VectorKeyframeTrack,
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
  /** Morph targets: per attribute, one set of values per target. */
  morph?: { relative: boolean; attributes: Record<string, AttributeData[]> };
}

interface MaterialData {
  type: string;
  values: Record<string, Value>;
}

/** An encoded image, or the pixels of one (RGBA, top row first) the worker transcoded. */
type ImageData = Blob | { width: number; height: number; pixels: Uint8Array };

interface TextureData {
  /** Into SceneData.images: textures that shared an image still share it. */
  image: number;
  values: Record<string, Value>;
}

type NodeKind =
  | 'group' | 'mesh' | 'skinnedMesh' | 'instancedMesh' | 'points' | 'line' | 'lineSegments' | 'lineLoop' | 'bone'
  | 'perspectiveCamera' | 'orthographicCamera'
  | 'directionalLight' | 'pointLight' | 'spotLight' | 'hemisphereLight' | 'ambientLight';

interface NodeData {
  kind: NodeKind;
  name: string;
  /** Kept, as animation tracks may name nodes by it. */
  uuid: string;
  /** The local transform, column-major. */
  matrix: number[];
  geometry?: number;
  material?: number | number[];
  instances?: { count: number; matrices: Float32Array; colors?: AttributeData };
  morph?: { influences: number[]; dictionary: Record<string, number> };
  /** Bones by node index (nodes are numbered depth first, the root 0). */
  skin?: { bones: number[]; boneInverses: number[][]; bindMatrix: number[]; bindMode: string };
  camera?: Record<string, number>;
  light?: Record<string, number | number[]>;
  children: NodeData[];
}

interface TrackData {
  name: string;
  type: string;
  times: Float32Array;
  values: Float32Array | unknown[];
  interpolation: number;
}

export interface SceneData {
  root: NodeData;
  geometries: GeometryData[];
  materials: MaterialData[];
  textures: TextureData[];
  images: ImageData[];
  animations: { name: string; duration: number; tracks: TrackData[] }[];
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

const TRACKS: Record<string, typeof KeyframeTrack> = {
  vector: VectorKeyframeTrack,
  quaternion: QuaternionKeyframeTrack,
  number: NumberKeyframeTrack,
  color: ColorKeyframeTrack,
  bool: BooleanKeyframeTrack,
  string: StringKeyframeTrack,
};

/** Texture settings worth carrying; the image itself travels separately. */
const TEXTURE_KEYS = [
  'name', 'mapping', 'channel', 'wrapS', 'wrapT', 'magFilter', 'minFilter', 'anisotropy', 'offset', 'repeat',
  'center', 'rotation', 'generateMipmaps', 'flipY', 'premultiplyAlpha', 'colorSpace',
] as const;

/** Material fields that describe the object, not its appearance. */
const SKIPPED_MATERIAL_KEYS = new Set(['uuid', 'type', 'userData', 'defines', 'version']);

type Flags = Partial<Record<`is${string}`, boolean>>;

function kindOf(node: Object3D): NodeKind {
  const is = node as unknown as Flags;
  if (is.isSkinnedMesh) return 'skinnedMesh';
  if (is.isInstancedMesh) return 'instancedMesh';
  if (is.isMesh) return 'mesh';
  if (is.isPoints) return 'points';
  if (is.isLineSegments) return 'lineSegments';
  if (is.isLineLoop) return 'lineLoop';
  if (is.isLine) return 'line';
  if (is.isBone) return 'bone';
  if (is.isPerspectiveCamera) return 'perspectiveCamera';
  if (is.isOrthographicCamera) return 'orthographicCamera';
  if (is.isDirectionalLight) return 'directionalLight';
  if (is.isSpotLight) return 'spotLight';
  if (is.isPointLight) return 'pointLight';
  if (is.isHemisphereLight) return 'hemisphereLight';
  if (is.isAmbientLight) return 'ambientLight';
  return 'group';
}

const drawable = (kind: NodeKind) =>
  ['mesh', 'skinnedMesh', 'instancedMesh', 'points', 'line', 'lineSegments', 'lineLoop'].includes(kind);

/** Collects a scene and its animation clips into SceneData and the buffers to transfer with it. */
export function serializeScene(root: Object3D, clips: AnimationClip[] = []): { scene: SceneData; transfer: Transferable[] } {
  const geometries: GeometryData[] = [];
  const materials: MaterialData[] = [];
  const textures: TextureData[] = [];
  const images: ImageData[] = [];
  const geometryIds = new Map<BufferGeometry, number>();
  const materialIds = new Map<Material, number>();
  const textureIds = new Map<Texture, number>();
  // One image per three.js Source: clones that share one stay one image.
  const imageIds = new Map<Source, number>();
  const transfer = new Set<Transferable>();

  // Nodes are numbered depth first, as they are rebuilt; light targets aren't nodes of their own.
  root.updateMatrixWorld(true);
  const targets = new Set<Object3D>();
  root.traverse((node) => {
    const target = (node as DirectionalLight).target as Object3D | undefined;
    if (target && (node as unknown as Flags).isLight) targets.add(target);
  });
  const nodeIds = new Map<Object3D, number>();
  const number = (node: Object3D) => {
    nodeIds.set(node, nodeIds.size);
    for (const child of node.children) if (!targets.has(child)) number(child);
  };
  number(root);

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
      const data: GeometryData = { attributes, index: source.index?.array ?? null, groups: source.groups.map((g) => ({ ...g })) };
      const morphs = Object.entries(source.morphAttributes);
      if (morphs.length > 0) {
        data.morph = {
          relative: source.morphTargetsRelative,
          attributes: Object.fromEntries(morphs.map(([name, list]) => [name, list.map(attribute)])),
        };
      }
      id = geometries.push(data) - 1;
      geometryIds.set(source, id);
    }
    return id;
  }

  /** The index of the texture's image: encoded (see modelWorker.ts), or transcoded pixels. */
  function image(source: Texture): number | undefined {
    let id = imageIds.get(source.source);
    if (id !== undefined) return id;
    let data: ImageData | undefined;
    if (source.image instanceof Blob) {
      data = source.image;
    } else if (source.format === RGBAFormat && source.type === UnsignedByteType) {
      // A KTX2 texture, transcoded to RGBA: its full-size level.
      const level = ((source as Texture & { mipmaps?: { data: unknown; width: number; height: number }[] }).mipmaps?.[0] ??
        source.image) as { data?: unknown; width: number; height: number } | undefined;
      if (level?.data instanceof Uint8Array && level.data.length === level.width * level.height * 4) {
        // The transcoder gives each level a buffer of its own: it moves rather than being copied.
        const whole = level.data.byteOffset === 0 && level.data.byteLength === level.data.buffer.byteLength;
        const pixels = whole ? level.data : level.data.slice();
        transfer.add(pixels.buffer as ArrayBuffer);
        data = { width: level.width, height: level.height, pixels };
      }
    }
    if (!data) return undefined; // GPU-compressed or not read
    id = images.push(data) - 1;
    imageIds.set(source.source, id);
    return id;
  }

  function texture(source: Texture): number | undefined {
    let id = textureIds.get(source);
    if (id !== undefined) return id;
    const index = image(source);
    if (index === undefined) return undefined;
    const values: Record<string, Value> = {};
    for (const key of TEXTURE_KEYS) values[key] = plain(source[key]) ?? null;
    id = textures.push({ image: index, values }) - 1;
    textureIds.set(source, id);
    return id;
  }

  function material(source: Material): number {
    let id = materialIds.get(source);
    if (id === undefined) {
      const values: Record<string, Value> = {};
      for (const field of Object.keys(source)) {
        // Some properties (alphaTest, clearcoat, sheen, ...) are getters over a "_" field.
        const key = field.startsWith('_') ? field.slice(1) : field;
        if (SKIPPED_MATERIAL_KEYS.has(key) || !(key in source)) continue;
        const value: unknown = (source as unknown as Record<string, unknown>)[key];
        if ((value as Texture | null)?.isTexture === true) {
          const index = texture(value as Texture);
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

  function node(source: Object3D): NodeData {
    source.updateMatrix();
    const kind = kindOf(source);
    const data: NodeData = { kind, name: source.name, uuid: source.uuid, matrix: source.matrix.toArray(), children: [] };
    if (drawable(kind)) {
      const shape = source as Mesh;
      data.geometry = geometry(shape.geometry);
      data.material = Array.isArray(shape.material) ? shape.material.map(material) : material(shape.material);
      if (shape.morphTargetInfluences && shape.morphTargetDictionary) {
        data.morph = { influences: [...shape.morphTargetInfluences], dictionary: { ...shape.morphTargetDictionary } };
      }
    }
    if (kind === 'instancedMesh') {
      const { count, instanceMatrix, instanceColor } = source as InstancedMesh;
      const matrices = (instanceMatrix.array as Float32Array).subarray(0, count * 16);
      transfer.add(matrices.buffer as ArrayBuffer);
      data.instances = { count, matrices, colors: instanceColor ? attribute(instanceColor) : undefined };
    }
    if (kind === 'skinnedMesh') {
      const { skeleton, bindMatrix, bindMode } = source as SkinnedMesh;
      const bones = skeleton.bones.map((bone) => nodeIds.get(bone));
      // Bones outside the scene can't be rebuilt: the mesh is shown in its bind pose.
      if (bones.every((id) => id !== undefined)) {
        data.skin = {
          bones: bones as number[],
          boneInverses: skeleton.boneInverses.map((m) => m.toArray()),
          bindMatrix: bindMatrix.toArray(),
          bindMode,
        };
      } else {
        data.kind = 'mesh';
      }
    }
    if (kind === 'perspectiveCamera') {
      const { fov, near, far, zoom } = source as PerspectiveCamera;
      data.camera = { fov, near, far, zoom };
    } else if (kind === 'orthographicCamera') {
      const { left, right, top, bottom, near, far, zoom } = source as OrthographicCamera;
      data.camera = { left, right, top, bottom, near, far, zoom };
    }
    if (kind.endsWith('Light')) {
      const light = source as Light & {
        distance?: number;
        decay?: number;
        angle?: number;
        penumbra?: number;
        groundColor?: Color;
        target?: Object3D;
      };
      data.light = { color: light.color.toArray(), intensity: light.intensity };
      if (light.distance !== undefined) data.light.distance = light.distance;
      if (light.decay !== undefined) data.light.decay = light.decay;
      if (light.angle !== undefined) data.light.angle = light.angle;
      if (light.penumbra !== undefined) data.light.penumbra = light.penumbra;
      if (light.groundColor) data.light.groundColor = light.groundColor.toArray();
      // Where it points, relative to the light.
      if (light.target) data.light.target = source.worldToLocal(light.target.getWorldPosition(new Vector3())).toArray();
    }
    data.children = source.children.filter((child) => !targets.has(child)).map(node);
    return data;
  }

  const animations = clips.map((clip) => ({
    name: clip.name,
    duration: clip.duration,
    tracks: clip.tracks.map((track) => {
      const type = (track as KeyframeTrack & { ValueTypeName: string }).ValueTypeName;
      let values: Float32Array | unknown[] = track.values as Float32Array;
      // glTF's cubic spline keys hold in-tangent, value, out-tangent: the values are kept, played linearly.
      const factory = (track as KeyframeTrack & { createInterpolant: { isInterpolantFactoryMethodGLTFCubicSpline?: boolean } })
        .createInterpolant;
      if (factory?.isInterpolantFactoryMethodGLTFCubicSpline) {
        const size = track.getValueSize() / 3;
        const linear = new Float32Array(track.times.length * size);
        for (let k = 0; k < track.times.length; k++) {
          for (let i = 0; i < size; i++) linear[k * size + i] = (values as Float32Array)[k * size * 3 + size + i];
        }
        values = linear;
      }
      if (values instanceof Float32Array) transfer.add(values.buffer as ArrayBuffer);
      transfer.add(track.times.buffer as ArrayBuffer);
      return {
        name: track.name,
        type,
        times: track.times as Float32Array,
        values: values instanceof Float32Array ? values : Array.from(values as ArrayLike<unknown>),
        interpolation: factory?.isInterpolantFactoryMethodGLTFCubicSpline ? InterpolateLinear : track.getInterpolation(),
      };
    }),
  }));

  const scene: SceneData = { root: node(root), geometries, materials, textures, images, animations };
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
function assign(target: object, values: Record<string, Value>, textures: (Texture | null)[]) {
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

const toAttribute = ({ array, itemSize, normalized }: AttributeData) => new BufferAttribute(array, itemSize, normalized);

function light(kind: NodeKind, data: Record<string, number | number[]>): Light {
  const num = (key: string, fallback: number) => (typeof data[key] === 'number' ? (data[key] as number) : fallback);
  let built: Light;
  switch (kind) {
    case 'directionalLight': built = new DirectionalLight(); break;
    case 'pointLight': built = new PointLight(undefined, undefined, num('distance', 0), num('decay', 2)); break;
    case 'spotLight':
      built = new SpotLight(undefined, undefined, num('distance', 0), num('angle', Math.PI / 3), num('penumbra', 0), num('decay', 2));
      break;
    case 'hemisphereLight':
      built = new HemisphereLight();
      (built as HemisphereLight).groundColor.fromArray(data.groundColor as number[]);
      break;
    default: built = new AmbientLight();
  }
  built.color.fromArray(data.color as number[]);
  built.intensity = num('intensity', 1);
  const target = (built as DirectionalLight).target;
  if (target && Array.isArray(data.target)) {
    target.position.fromArray(data.target);
    built.add(target);
  }
  return built;
}

function camera(kind: NodeKind, data: Record<string, number>): Camera {
  const built =
    kind === 'perspectiveCamera'
      ? new PerspectiveCamera(data.fov, 1, data.near, data.far)
      : new OrthographicCamera(data.left, data.right, data.top, data.bottom, data.near, data.far);
  built.zoom = data.zoom ?? 1;
  return built;
}

/** Rebuilds a scene and its animation clips from SceneData, decoding its images. */
export async function deserializeScene(scene: SceneData): Promise<{ object: Object3D; animations: AnimationClip[] }> {
  const { root, geometries, materials, textures, images, animations } = scene;
  // WebGL neither flips an ImageBitmap nor changes its alpha on upload, so an
  // image is decoded the way its textures ask; textures asking alike share one.
  const sources = new Map<string, Promise<Source | null>>();
  function decoded(image: number, flipY: boolean, premultiply: boolean): Promise<Source | null> {
    const key = `${image} ${flipY} ${premultiply}`;
    let source = sources.get(key);
    if (!source) {
      const data = images[image];
      source =
        data instanceof Blob
          ? createImageBitmap(data, {
              imageOrientation: flipY ? 'flipY' : 'from-image',
              premultiplyAlpha: premultiply ? 'premultiply' : 'none',
              colorSpaceConversion: 'none',
            }).then(
              (bitmap) => new Source(bitmap),
              () => null, // an image the browser can't decode leaves its textures out
            )
          : Promise.resolve(new Source({ data: data.pixels, width: data.width, height: data.height }));
      sources.set(key, source);
    }
    return source;
  }
  const builtTextures = await Promise.all(
    textures.map(async ({ image, values }) => {
      const source = await decoded(image, values.flipY === true, values.premultiplyAlpha === true);
      if (!source) return null;
      const raw = !(images[image] instanceof Blob);
      const texture = raw ? new DataTexture(null, 1, 1, RGBAFormat, UnsignedByteType) : new Texture();
      texture.source = source;
      assign(texture, values, []);
      texture.flipY = false; // decoded the right way up
      if (raw) {
        // Pixels at full size only: the GPU makes the smaller levels.
        texture.generateMipmaps = true;
        texture.minFilter = LinearMipmapLinearFilter;
        texture.magFilter = LinearFilter;
      }
      texture.needsUpdate = true;
      return texture;
    }),
  );
  const builtMaterials = materials.map(({ type, values }) => {
    const material = new (MATERIALS[type] ?? MeshStandardMaterial)();
    assign(material, values, builtTextures);
    material.needsUpdate = true;
    return material;
  });
  const builtGeometries = geometries.map(({ attributes, index, groups, morph }) => {
    const geometry = new BufferGeometry();
    for (const [name, data] of Object.entries(attributes)) geometry.setAttribute(name, toAttribute(data));
    if (index) geometry.setIndex(new BufferAttribute(index, 1));
    for (const { start, count, materialIndex } of groups) geometry.addGroup(start, count, materialIndex);
    if (morph) {
      geometry.morphTargetsRelative = morph.relative;
      const targets = geometry.morphAttributes as Record<string, BufferAttribute[]>;
      for (const [name, list] of Object.entries(morph.attributes)) targets[name] = list.map(toAttribute);
    }
    return geometry;
  });

  const nodes: Object3D[] = [];
  const skins: [SkinnedMesh, NonNullable<NodeData['skin']>][] = [];

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
      case 'skinnedMesh': object = new SkinnedMesh(geometry, material); break;
      case 'points': object = new Points(geometry, material); break;
      case 'line': object = new Line(geometry, material); break;
      case 'lineSegments': object = new LineSegments(geometry, material); break;
      case 'lineLoop': object = new LineLoop(geometry, material); break;
      case 'bone': object = new Bone(); break;
      case 'perspectiveCamera':
      case 'orthographicCamera':
        object = camera(data.kind, data.camera ?? {});
        break;
      case 'directionalLight':
      case 'pointLight':
      case 'spotLight':
      case 'hemisphereLight':
      case 'ambientLight':
        object = light(data.kind, data.light ?? {});
        break;
      case 'instancedMesh': {
        const { count, matrices, colors } = data.instances!;
        const mesh = new InstancedMesh(geometry, material, count);
        mesh.instanceMatrix = new InstancedBufferAttribute(matrices, 16);
        if (colors) mesh.instanceColor = new InstancedBufferAttribute(colors.array, colors.itemSize, colors.normalized);
        object = mesh;
        break;
      }
      default: object = new Group();
    }
    nodes.push(object);
    object.name = data.name;
    object.uuid = data.uuid;
    new Matrix4().fromArray(data.matrix).decompose(object.position, object.quaternion, object.scale);
    if (data.morph && (object as Mesh).isMesh) {
      const mesh = object as Mesh;
      mesh.updateMorphTargets();
      mesh.morphTargetInfluences = [...data.morph.influences];
      mesh.morphTargetDictionary = { ...data.morph.dictionary };
    }
    if (data.kind === 'skinnedMesh' && data.skin) skins.push([object as SkinnedMesh, data.skin]);
    for (const child of data.children) object.add(build(child));
    return object;
  }

  const object = build(root);
  for (const [mesh, { bones, boneInverses, bindMatrix, bindMode }] of skins) {
    const skeleton = new Skeleton(
      bones.map((id) => nodes[id] as Bone),
      boneInverses.map((m) => new Matrix4().fromArray(m)),
    );
    mesh.bindMode = bindMode as SkinnedMesh['bindMode'];
    mesh.bind(skeleton, new Matrix4().fromArray(bindMatrix));
  }
  const clips = animations.map(
    ({ name, duration, tracks }) =>
      new AnimationClip(
        name,
        duration,
        tracks.flatMap(({ name: track, type, times, values, interpolation }) => {
          const Track = TRACKS[type];
          return Track ? [new Track(track, times, values as ArrayLike<number>, interpolation as InterpolationModes)] : [];
        }),
      ),
  );
  return { object, animations: clips };
}
