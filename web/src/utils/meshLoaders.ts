// Models for the 3D preview. OBJ, STL and PLY are read by the engine in its
// worker, so large files never block the page; glTF/GLB (materials, textures,
// scene graph) and FBX use three.js's loaders, each fetched only when needed.

import {
  Box3,
  BufferAttribute,
  BufferGeometry,
  Color,
  DoubleSide,
  Line,
  type Material,
  Mesh,
  MeshStandardMaterial,
  type Object3D,
  Points,
  SRGBColorSpace,
  Texture,
} from 'three';
import { getPixie } from '../core/client';
import type { DecodedMesh, MeshFormat } from '../core/types';

interface LoadedModel {
  object: Object3D;
  box: Box3;
}

function surface(geometry: BufferGeometry, color: string): Mesh {
  if (!geometry.attributes.normal) geometry.computeVertexNormals();
  const vertexColors = Boolean(geometry.attributes.color);
  const material = new MeshStandardMaterial({
    // Vertex colours are multiplied by the material colour, so it stays white for them.
    color: vertexColors ? '#ffffff' : color,
    metalness: 0.05,
    roughness: 0.55,
    side: DoubleSide,
    vertexColors,
  });
  return new Mesh(geometry, material);
}

function geometryOf({ positions, normals, colors, indices }: DecodedMesh): BufferGeometry {
  const geometry = new BufferGeometry();
  geometry.setAttribute('position', new BufferAttribute(positions, 3));
  if (normals) geometry.setAttribute('normal', new BufferAttribute(normals, 3));
  if (colors) {
    // Files store sRGB; three.js shades in linear light.
    const color = new Color();
    for (let i = 0; i < colors.length; i += 3) {
      color.setRGB(colors[i], colors[i + 1], colors[i + 2], SRGBColorSpace).toArray(colors, i);
    }
    geometry.setAttribute('color', new BufferAttribute(colors, 3));
  }
  geometry.setIndex(new BufferAttribute(indices, 1));
  return geometry;
}

/** The bytes as an ArrayBuffer, copied only when the view doesn't cover a whole one. */
function arrayBuffer(data: Uint8Array): ArrayBuffer {
  const { buffer } = data;
  return buffer instanceof ArrayBuffer && data.byteOffset === 0 && data.byteLength === buffer.byteLength
    ? buffer
    : data.slice().buffer;
}

async function parse(data: Uint8Array, type: MeshFormat, color: string): Promise<Object3D> {
  switch (type) {
    case 'obj':
    case 'stl':
    case 'ply':
      return surface(geometryOf(await getPixie().decodeMesh(data)), color);
    case 'gltf':
    case 'glb': {
      const { parseGltf } = await import('./gltf');
      return parseGltf(arrayBuffer(data));
    }
    case 'fbx': {
      const { FBXLoader } = await import('three/addons/loaders/FBXLoader.js');
      return new FBXLoader().parse(arrayBuffer(data), '');
    }
  }
}

/** Frees the GPU resources (geometries, materials, textures) under a node. */
export function disposeModel(object: Object3D) {
  object.traverse((node) => {
    if (!(node instanceof Mesh || node instanceof Line || node instanceof Points)) return;
    node.geometry.dispose();
    for (const material of [node.material as Material | Material[]].flat()) {
      for (const value of Object.values(material)) {
        if (value instanceof Texture) value.dispose();
      }
      material.dispose();
    }
  });
}

/** Loads a model and measures it for framing; `color` is for meshes without materials of their own. */
export async function loadModel(data: Uint8Array, type: MeshFormat, color: string): Promise<LoadedModel> {
  const object = await parse(data, type, color);
  const box = new Box3().setFromObject(object);
  if (box.isEmpty()) {
    disposeModel(object);
    throw new Error('The model has no geometry');
  }
  return { object, box };
}
