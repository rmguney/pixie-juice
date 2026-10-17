// Models for the 3D preview, all read off the main thread so large files never
// block the page: OBJ, STL and PLY by the engine in its worker (normals,
// colours, lines and point clouds included); glTF/GLB (materials, textures,
// scene graph, cameras, lights, animation) and FBX by three.js's loaders in the
// model worker, which starts on first use and stops when idle.

import {
  type AnimationClip,
  Box3,
  BufferAttribute,
  BufferGeometry,
  DoubleSide,
  Group,
  InstancedMesh,
  Line,
  LineBasicMaterial,
  LineSegments,
  type Material,
  Mesh,
  MeshStandardMaterial,
  type Object3D,
  Points,
  PointsMaterial,
  SkinnedMesh,
  Texture,
} from 'three';
import { getPixie } from '../core/client';
import type { DecodedMesh, MeshFormat } from '../core/types';
import type { ModelRequest, ModelResponse } from './modelProtocol';
import { deserializeScene } from './sceneTransfer';

interface LoadedModel {
  object: Object3D;
  box: Box3;
  animations: AnimationClip[];
}

/** What the engine read: triangles and lines, or with neither, the vertices as points. */
function display({ positions, normals, colors, indices, lines }: DecodedMesh, color: string): Object3D {
  const group = new Group();
  // Vertex colours are multiplied by the material colour, so it stays white for them.
  const tint = colors ? '#ffffff' : color;
  // The surface, the lines and the points share the vertex arrays.
  const position = new BufferAttribute(positions, 3);
  const color3 = colors && new BufferAttribute(colors, 3);
  const geometry = (index?: Uint32Array) => {
    const built = new BufferGeometry();
    built.setAttribute('position', position);
    if (color3) built.setAttribute('color', color3);
    if (index) built.setIndex(new BufferAttribute(index, 1));
    return built;
  };
  const add = (object: Mesh | LineSegments | Points) => {
    object.userData.surface = !colors; // drawn in the theme's colour, which follows the colour scheme
    group.add(object);
  };
  if (indices.length > 0) {
    const surface = geometry(indices);
    if (normals) surface.setAttribute('normal', new BufferAttribute(normals, 3));
    add(
      new Mesh(
        surface,
        new MeshStandardMaterial({ color: tint, metalness: 0.05, roughness: 0.55, side: DoubleSide, vertexColors: Boolean(colors) }),
      ),
    );
  }
  if (lines && lines.length > 0) {
    add(new LineSegments(geometry(lines), new LineBasicMaterial({ color: tint, vertexColors: Boolean(colors) })));
  }
  if (group.children.length === 0) {
    // Point size in screen pixels: a cloud stays visible at any scale.
    add(new Points(geometry(), new PointsMaterial({ color: tint, size: 2, sizeAttenuation: false, vertexColors: Boolean(colors) })));
  }
  return group;
}

/** The model worker stops after this long without work; Draco's and meshopt's memory goes with it. */
const MODEL_IDLE_MS = 30_000;

let modelWorker: Worker | null = null;
let modelIdle: ReturnType<typeof setTimeout> | undefined;
type Parsed = { object: Object3D; animations: AnimationClip[] };
const pendingModels = new Map<number, { resolve: (model: Parsed) => void; reject: (error: Error) => void }>();
let nextModel = 0;

function modelSettled() {
  if (pendingModels.size > 0) return;
  modelIdle = setTimeout(() => {
    modelWorker?.terminate();
    modelWorker = null;
  }, MODEL_IDLE_MS);
}

/** Parses a glTF, GLB or FBX in the model worker; the buffer moves to it. */
function parseInWorker(buffer: ArrayBuffer, type: ModelRequest['type'], files: File[]): Promise<Parsed> {
  clearTimeout(modelIdle);
  if (!modelWorker) {
    const worker = new Worker(new URL('./modelWorker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = ({ data: response }: MessageEvent<ModelResponse>) => {
      const pending = pendingModels.get(response.id);
      pendingModels.delete(response.id);
      if ('error' in response) pending?.reject(new Error(response.error));
      else if (pending) deserializeScene(response.scene).then(pending.resolve, pending.reject);
      modelSettled();
    };
    worker.onerror = (event) => {
      // Fail what was asked; the next model gets a new worker.
      for (const { reject } of pendingModels.values()) reject(new Error(event.message || 'The model worker failed'));
      pendingModels.clear();
      worker.terminate();
      modelWorker = null;
    };
    modelWorker = worker;
  }
  const id = nextModel++;
  return new Promise((resolve, reject) => {
    pendingModels.set(id, { resolve, reject });
    try {
      modelWorker!.postMessage({ id, type, data: buffer, files } satisfies ModelRequest, [buffer]);
    } catch (error) {
      pendingModels.delete(id);
      modelSettled();
      reject(error instanceof Error ? error : new Error(String(error)));
    }
  });
}

async function parse(buffer: ArrayBuffer, type: MeshFormat, color: string, files: File[]): Promise<Parsed> {
  switch (type) {
    case 'obj':
    case 'stl':
    case 'ply':
      return { object: display(await getPixie().decodeMesh(new Uint8Array(buffer), true), color), animations: [] };
    case 'gltf':
    case 'glb':
    case 'fbx':
      return parseInWorker(buffer, type, files);
  }
}

/** Frees the GPU resources (geometries, materials, textures, instance and bone data) under a node. */
export function disposeModel(object: Object3D) {
  object.traverse((node) => {
    if (!(node instanceof Mesh || node instanceof Line || node instanceof Points)) return;
    if (node instanceof InstancedMesh) node.dispose();
    if (node instanceof SkinnedMesh) node.skeleton.dispose();
    node.geometry.dispose();
    for (const material of [node.material as Material | Material[]].flat()) {
      for (const value of Object.values(material)) {
        if (!(value instanceof Texture)) continue;
        value.dispose();
        if (typeof ImageBitmap !== 'undefined' && value.image instanceof ImageBitmap) value.image.close();
      }
      material.dispose();
    }
  });
}

/**
 * Loads a model and measures it for framing; `color` is for meshes without
 * materials of their own, `files` those a .gltf may refer to. The buffer moves
 * to a worker: don't use it afterwards.
 */
export async function loadModel(buffer: ArrayBuffer, type: MeshFormat, color: string, files: File[] = []): Promise<LoadedModel> {
  const { object, animations } = await parse(buffer, type, color, files);
  const box = new Box3().setFromObject(object);
  if (box.isEmpty()) {
    disposeModel(object);
    throw new Error('The model has no geometry');
  }
  return { object, box, animations };
}
