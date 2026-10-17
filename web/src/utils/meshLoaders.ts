// Models for the 3D preview, all read off the main thread so large files never
// block the page: OBJ, STL and PLY by the engine in its worker (normals and
// colours included); glTF/GLB (materials, textures, scene graph) and FBX by
// three.js's loaders in the model worker, which starts on first use and stops
// when idle.

import {
  Box3,
  BufferAttribute,
  BufferGeometry,
  DoubleSide,
  Line,
  type Material,
  Mesh,
  MeshStandardMaterial,
  type Object3D,
  Points,
  Texture,
} from 'three';
import { getPixie } from '../core/client';
import type { DecodedMesh, MeshFormat } from '../core/types';
import type { ModelRequest, ModelResponse } from './modelProtocol';
import { deserializeScene } from './sceneTransfer';

interface LoadedModel {
  object: Object3D;
  box: Box3;
}

function surface({ positions, normals, colors, indices }: DecodedMesh, color: string): Mesh {
  const geometry = new BufferGeometry();
  geometry.setAttribute('position', new BufferAttribute(positions, 3));
  geometry.setAttribute('normal', new BufferAttribute(normals, 3));
  if (colors) geometry.setAttribute('color', new BufferAttribute(colors, 3));
  geometry.setIndex(new BufferAttribute(indices, 1));
  const material = new MeshStandardMaterial({
    // Vertex colours are multiplied by the material colour, so it stays white for them.
    color: colors ? '#ffffff' : color,
    metalness: 0.05,
    roughness: 0.55,
    side: DoubleSide,
    vertexColors: Boolean(colors),
  });
  const mesh = new Mesh(geometry, material);
  mesh.userData.surface = !colors; // drawn in the theme's colour, which follows the colour scheme
  return mesh;
}

/** The model worker stops after this long without work; Draco's and meshopt's memory goes with it. */
const MODEL_IDLE_MS = 30_000;

let modelWorker: Worker | null = null;
let modelIdle: ReturnType<typeof setTimeout> | undefined;
const pendingModels = new Map<number, { resolve: (object: Object3D) => void; reject: (error: Error) => void }>();
let nextModel = 0;

function modelSettled() {
  if (pendingModels.size > 0) return;
  modelIdle = setTimeout(() => {
    modelWorker?.terminate();
    modelWorker = null;
  }, MODEL_IDLE_MS);
}

/** Parses a glTF, GLB or FBX in the model worker; the buffer moves to it. */
function parseInWorker(buffer: ArrayBuffer, type: ModelRequest['type']): Promise<Object3D> {
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
    modelWorker!.postMessage({ id, type, data: buffer } satisfies ModelRequest, [buffer]);
  });
}

async function parse(buffer: ArrayBuffer, type: MeshFormat, color: string): Promise<Object3D> {
  switch (type) {
    case 'obj':
    case 'stl':
    case 'ply':
      return surface(await getPixie().decodeMesh(new Uint8Array(buffer), true), color);
    case 'gltf':
    case 'glb':
    case 'fbx':
      return parseInWorker(buffer, type);
  }
}

/** Frees the GPU resources (geometries, materials, textures) under a node. */
export function disposeModel(object: Object3D) {
  object.traverse((node) => {
    if (!(node instanceof Mesh || node instanceof Line || node instanceof Points)) return;
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
 * materials of their own. The buffer moves to a worker: don't use it afterwards.
 */
export async function loadModel(buffer: ArrayBuffer, type: MeshFormat, color: string): Promise<LoadedModel> {
  const object = await parse(buffer, type, color);
  const box = new Box3().setFromObject(object);
  if (box.isEmpty()) {
    disposeModel(object);
    throw new Error('The model has no geometry');
  }
  return { object, box };
}
