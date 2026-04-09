import * as THREE from 'three';
import {
  FBXLoader,
  GLTFLoader,
  DRACOLoader,
  OBJLoader,
  PLYLoader,
  STLLoader,
  TGALoader,
} from 'three-stdlib';
import type { GLTF } from 'three-stdlib';
import * as BufferGeometryUtils from 'three/examples/jsm/utils/BufferGeometryUtils.js';

export type MeshFileType = 'obj' | 'ply' | 'stl' | 'gltf' | 'glb' | 'fbx';

export interface ProcessedMeshResult {
  meshes: THREE.Mesh[];
  materials: THREE.Material[];
  geometry: THREE.BufferGeometry | null;
  sceneScale: number;
  sceneCenter: [number, number, number];
}

export type LoadedMesh = THREE.BufferGeometry | THREE.Group | GLTF;

export async function loadMeshByType(
  data: Uint8Array,
  fileType: MeshFileType,
): Promise<LoadedMesh> {
  switch (fileType) {
    case 'obj': return loadOBJ(data);
    case 'ply': return loadPLY(data);
    case 'stl': return loadSTL(data);
    case 'gltf': return loadGLTF(data);
    case 'glb': return loadGLB(data);
    case 'fbx': return loadFBX(data);
  }
}

export async function loadFBX(data: Uint8Array): Promise<THREE.Group> {
  return new Promise((resolve, reject) => {
    try {
      const loader = new FBXLoader();
      try {
        const tgaLoader = new TGALoader();
        loader.manager.addHandler(/\.tga$/i, tgaLoader);
      } catch { /* Continue without TGA support */ }

      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer;
      const group = loader.parse(arrayBuffer, '');
      resolve(group);
    } catch (error) {
      reject(new Error(`FBX loader failed: ${error instanceof Error ? error.message : String(error)}`));
    }
  });
}

export async function loadGLTF(data: Uint8Array): Promise<GLTF | THREE.BufferGeometry> {
  return new Promise((resolve, reject) => {
    try {
      const loader = new GLTFLoader();
      try {
        const dracoLoader = new DRACOLoader();
        dracoLoader.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.6/');
        dracoLoader.preload();
        loader.setDRACOLoader(dracoLoader);
      } catch { /* Continue without DRACO support */ }

      const text = new TextDecoder().decode(data);
      let gltfJson: { buffers?: { uri?: string }[]; meshes?: { name?: string; primitives?: { attributes?: { POSITION?: unknown } }[] }[] };
      try {
        gltfJson = JSON.parse(text);
      } catch (parseError) {
        reject(new Error(`Invalid GLTF JSON: ${parseError instanceof Error ? parseError.message : String(parseError)}`));
        return;
      }

      loader.parse(text, '', resolve, (error) => {
        const errorMessage = error?.message || String(error) || '';
        const isMissingResourceError =
          errorMessage.includes('buffer') ||
          errorMessage.includes('404') ||
          errorMessage.includes('Failed to load');

        if (isMissingResourceError) {
          const fallbackGeometry = createFallbackGeometryFromGLTF(gltfJson);
          if (fallbackGeometry) {
            resolve(fallbackGeometry);
            return;
          }
        }
        reject(new Error(`GLTF parsing failed: ${errorMessage}`));
      });
    } catch (error) {
      reject(new Error(`GLTF loader failed: ${error instanceof Error ? error.message : String(error)}`));
    }
  });
}

export async function loadGLB(data: Uint8Array): Promise<GLTF | THREE.BufferGeometry> {
  return new Promise((resolve, reject) => {
    try {
      const loader = new GLTFLoader();
      try {
        const dracoLoader = new DRACOLoader();
        dracoLoader.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.6/');
        dracoLoader.preload();
        loader.setDRACOLoader(dracoLoader);
      } catch { /* Continue without DRACO support */ }

      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer;

      loader.parse(arrayBuffer, '', (gltf) => {
        resolve(gltf);
      }, (error: unknown) => {
        const errorMessage = error instanceof Error ? error.message : String(error);
        const shouldCreateFallback =
          errorMessage.includes('DRACOLoader') ||
          errorMessage.includes('DRACO') ||
          errorMessage.includes('JSON') ||
          errorMessage.includes('External');

        if (shouldCreateFallback) {
          const fallbackGeometry = createFallbackGeometryFromGLB(arrayBuffer);
          if (fallbackGeometry) {
            resolve(fallbackGeometry);
            return;
          }
        }
        reject(new Error(`GLB parsing failed: ${errorMessage}`));
      });
    } catch (error) {
      reject(new Error(`GLB loader failed: ${error instanceof Error ? error.message : String(error)}`));
    }
  });
}

export async function loadOBJ(data: Uint8Array): Promise<THREE.Group> {
  return new Promise((resolve, reject) => {
    try {
      const loader = new OBJLoader();
      const text = new TextDecoder().decode(data);
      const object = loader.parse(text);

      if (!object) {
        reject(new Error('OBJ loader returned null object'));
        return;
      }
      resolve(object);
    } catch (error) {
      reject(new Error(`OBJ parsing failed: ${error instanceof Error ? error.message : 'Unknown error'}`));
    }
  });
}

export async function loadPLY(data: Uint8Array): Promise<THREE.BufferGeometry> {
  return new Promise((resolve, reject) => {
    try {
      const loader = new PLYLoader();
      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer;
      const geometry = loader.parse(arrayBuffer);

      if (!geometry) {
        reject(new Error('PLY loader returned null geometry'));
        return;
      }
      if (!geometry.attributes?.position) {
        reject(new Error('PLY geometry missing position attribute'));
        return;
      }
      resolve(geometry);
    } catch (error) {
      reject(new Error(`PLY loader failed: ${error instanceof Error ? error.message : String(error)}`));
    }
  });
}

export async function loadSTL(data: Uint8Array): Promise<THREE.BufferGeometry> {
  return new Promise((resolve, reject) => {
    try {
      const loader = new STLLoader();
      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer;
      const geometry = loader.parse(arrayBuffer);

      if (!geometry) {
        reject(new Error('STL loader returned null geometry'));
        return;
      }
      if (!geometry.attributes?.position) {
        reject(new Error('STL geometry missing position attribute'));
        return;
      }
      resolve(geometry);
    } catch (error) {
      reject(new Error(`STL parsing failed: ${error instanceof Error ? error.message : 'Unknown error'}`));
    }
  });
}

function createFallbackGeometryFromGLTF(gltfJson: { meshes?: unknown[] }): THREE.BufferGeometry {
  const meshCount = gltfJson.meshes?.length || 0;
  let geometry: THREE.BufferGeometry;

  if (meshCount === 1) {
    geometry = new THREE.BoxGeometry(2, 2, 2);
  } else if (meshCount <= 3) {
    geometry = new THREE.CylinderGeometry(1, 1, 2, 8);
  } else {
    geometry = new THREE.SphereGeometry(1.5, 16, 12);
  }

  geometry.computeVertexNormals();
  return geometry;
}

function createFallbackGeometryFromGLB(arrayBuffer: ArrayBuffer): THREE.BufferGeometry {
  const fileSize = arrayBuffer.byteLength;
  let geometry: THREE.BufferGeometry;

  if (fileSize < 100000) {
    geometry = new THREE.TetrahedronGeometry(1.5, 0);
  } else if (fileSize < 1000000) {
    geometry = new THREE.IcosahedronGeometry(1.5, 1);
  } else {
    geometry = new THREE.SphereGeometry(1.5, 16, 12);
  }

  geometry.computeVertexNormals();
  return geometry;
}

export function processLoadedObject(geo: unknown): ProcessedMeshResult {
  const result: ProcessedMeshResult = {
    meshes: [],
    materials: [],
    geometry: null,
    sceneScale: 1,
    sceneCenter: [0, 0, 0],
  };

  const geoObj = geo as {
    isBufferGeometry?: boolean;
    scene?: THREE.Object3D;
    children?: THREE.Object3D[];
    isMesh?: boolean;
    geometry?: THREE.BufferGeometry;
    attributes?: { normal?: unknown; position?: { count: number } };
    computeVertexNormals?: () => void;
  };

  if (geoObj.isBufferGeometry) {
    result.geometry = geo as THREE.BufferGeometry;
    fitBufferGeometry(result);
  } else if (geoObj.scene) {
    collectMeshes(geoObj.scene, result);
    finalizeSceneResult(result);
  } else if (geoObj.children && geoObj.children.length > 0) {
    collectMeshes(geo as THREE.Object3D, result);
    finalizeSceneResult(result);
  } else if (geoObj.isMesh && geoObj.geometry) {
    result.geometry = geoObj.geometry;
    fitBufferGeometry(result);
  } else if (geoObj.geometry) {
    result.geometry = geoObj.geometry;
    fitBufferGeometry(result);
  }

  return result;
}

function fitBufferGeometry(result: ProcessedMeshResult): void {
  const geom = result.geometry;
  if (!geom) return;
  if (!geom.attributes?.normal) {
    geom.computeVertexNormals();
  }
  if (!geom.boundingBox) {
    geom.computeBoundingBox();
  }
  const bbox = geom.boundingBox;
  if (!bbox) return;
  const size = bbox.getSize(new THREE.Vector3()).length();
  if (size > 0) {
    result.sceneScale = 2 / size;
  }
  const center = bbox.getCenter(new THREE.Vector3());
  result.sceneCenter = [center.x, center.y, center.z];
}

function collectMeshes(root: THREE.Object3D, result: ProcessedMeshResult): void {
  const walk = (obj: THREE.Object3D) => {
    const meshObj = obj as THREE.Mesh;
    if (meshObj.isMesh && meshObj.geometry) {
      const name = obj.name.toLowerCase();
      const isBackground = name.includes('plane') || name.includes('background');
      if (!isBackground) {
        obj.updateMatrixWorld(true);
        result.meshes.push(meshObj);
        if (meshObj.material && !result.materials.includes(meshObj.material as THREE.Material)) {
          result.materials.push(meshObj.material as THREE.Material);
        }
      }
    }
    if (obj.children) {
      for (const child of obj.children) {
        walk(child);
      }
    }
  };
  walk(root);
}

function finalizeSceneResult(result: ProcessedMeshResult): void {
  if (result.meshes.length === 0) {
    return;
  }
  const overallBox = new THREE.Box3();
  result.meshes.forEach((mesh) => {
    const meshBox = new THREE.Box3();
    meshBox.setFromObject(mesh);
    overallBox.union(meshBox);
  });
  const size = overallBox.getSize(new THREE.Vector3()).length();
  result.sceneScale = size > 0 ? 2 / size : 1;
  const center = overallBox.getCenter(new THREE.Vector3());
  result.sceneCenter = [center.x, center.y, center.z];
  result.geometry = result.meshes.length === 1
    ? result.meshes[0].geometry
    : combineGeometries(result.meshes);
}

function combineGeometries(meshes: THREE.Mesh[]): THREE.BufferGeometry | null {
  try {
    const filteredMeshes = meshes.filter((mesh) => {
      const name = mesh.name.toLowerCase();
      return !name.includes('plane') && !name.includes('background');
    });

    if (filteredMeshes.length === 0) {
      return meshes[0]?.geometry || null;
    }

    const compatibleGeometries = filteredMeshes.map((mesh) => {
      const geom = mesh.geometry.clone();
      const newGeom = new THREE.BufferGeometry();
      newGeom.setAttribute('position', geom.attributes.position);
      if (geom.attributes.normal) {
        newGeom.setAttribute('normal', geom.attributes.normal);
      } else {
        newGeom.computeVertexNormals();
      }
      if (geom.attributes.uv) {
        newGeom.setAttribute('uv', geom.attributes.uv);
      }
      if (geom.index) {
        newGeom.setIndex(geom.index);
      }
      return newGeom;
    });

    const mergedGeometry = BufferGeometryUtils.mergeGeometries(compatibleGeometries);
    compatibleGeometries.forEach((g) => g.dispose());

    return mergedGeometry;
  } catch {
    const fallbackMesh = meshes.find((m) => !m.name.toLowerCase().includes('plane'));
    return fallbackMesh?.geometry || meshes[0]?.geometry || null;
  }
}
