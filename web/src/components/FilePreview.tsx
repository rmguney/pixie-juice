import { useState, useEffect } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera } from '@react-three/drei';
import * as THREE from 'three';
import { FBXLoader } from 'three-stdlib';
import { GLTFLoader } from 'three-stdlib';
import { DRACOLoader } from 'three-stdlib';
import { OBJLoader } from 'three-stdlib';
import { PLYLoader } from 'three-stdlib';
import { STLLoader } from 'three-stdlib';
import { TGALoader } from 'three-stdlib';
import * as BufferGeometryUtils from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import type { FilePreviewProps } from '../types';
import type { GLTF } from 'three-stdlib';

type MeshFileType = 'obj' | 'ply' | 'stl' | 'gltf' | 'glb' | 'fbx';
type FileType = MeshFileType | 'image';

interface ProcessedMeshResult {
  meshes: THREE.Mesh[];
  materials: THREE.Material[];
  geometry: THREE.BufferGeometry | null;
  sceneScale: number;
}

interface MeshModelProps {
  meshData: Uint8Array;
  fileType: MeshFileType;
}

function MeshModel({ meshData, fileType }: MeshModelProps) {
  const [geometry, setGeometry] = useState<THREE.BufferGeometry | null>(null);
  const [meshes, setMeshes] = useState<THREE.Mesh[] | null>(null);
  const [sceneScale, setSceneScale] = useState(1);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (!meshData) {
      setIsLoading(false);
      return;
    }

    const loadMesh = async () => {
      setIsLoading(true);
      setError(null);
      
      try {
        let geo: THREE.BufferGeometry | THREE.Group | GLTF | null = null;

        if (fileType === 'obj') {
          geo = await loadOBJ(meshData);
        } else if (fileType === 'ply') {
          geo = await loadPLY(meshData);
        } else if (fileType === 'stl') {
          geo = await loadSTL(meshData);
        } else if (fileType === 'gltf') {
          geo = await loadGLTF(meshData);
        } else if (fileType === 'glb') {
          geo = await loadGLB(meshData);
        } else if (fileType === 'fbx') {
          geo = await loadFBX(meshData);
        } else {
          setError(`Unsupported mesh format: ${fileType}`);
          setIsLoading(false);
          return;
        }

        if (geo) {
          const processedResult = processLoadedObject(geo);
          if (processedResult.meshes && processedResult.meshes.length > 0) {
            setMeshes(processedResult.meshes);
            setSceneScale(processedResult.sceneScale);
            if (processedResult.geometry) {
              setGeometry(processedResult.geometry);
            }
          } else if (processedResult.geometry) {
            setGeometry(processedResult.geometry);
          } else {
            throw new Error('No valid geometry or meshes found in loaded model');
          }
        } else {
          throw new Error('Failed to create geometry - loader returned null');
        }
      } catch (err) {
        setError(`Failed to load ${fileType}: ${err instanceof Error ? err.message : String(err)}`);
      } finally {
        setIsLoading(false);
      }
    };

    loadMesh();
  }, [meshData, fileType]);

  if (isLoading) {
    return (
      <mesh>
        <boxGeometry args={[0.8, 0.8, 0.8]} />
        <meshStandardMaterial color="#4f46e5" wireframe />
      </mesh>
    );
  }

  if (error) {
    return (
      <mesh>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial color="#ef4444" />
      </mesh>
    );
  }

  if (!geometry && !meshes) {
    return (
      <mesh>
        <boxGeometry args={[0.5, 0.5, 0.5]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
    );
  }

  if (meshes && meshes.length > 0) {
    return (
      <group scale={[sceneScale, sceneScale, sceneScale]}>
        {meshes.map((mesh, index) => {
          const worldPosition = new THREE.Vector3();
          const worldQuaternion = new THREE.Quaternion();
          const worldScale = new THREE.Vector3();
          mesh.matrixWorld.decompose(worldPosition, worldQuaternion, worldScale);
          const worldRotation = new THREE.Euler();
          worldRotation.setFromQuaternion(worldQuaternion);
          
          return (
            <mesh 
              key={index} 
              geometry={mesh.geometry} 
              material={mesh.material || undefined}
              position={[worldPosition.x, worldPosition.y, worldPosition.z]}
              rotation={[worldRotation.x, worldRotation.y, worldRotation.z]}
              scale={[worldScale.x, worldScale.y, worldScale.z]}
            >
              {!mesh.material && (
                <meshStandardMaterial 
                  color="#6366f1" 
                  metalness={0.2} 
                  roughness={0.3}
                  side={THREE.DoubleSide}
                />
              )}
            </mesh>
          );
        })}
      </group>
    );
  }

  return (
    <mesh geometry={geometry!}>
      <meshStandardMaterial 
        color="#6366f1" 
        metalness={0.2} 
        roughness={0.3}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

function processLoadedObject(geo: unknown): ProcessedMeshResult {
  const result: ProcessedMeshResult = {
    meshes: [],
    materials: [],
    geometry: null,
    sceneScale: 1
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
    if (!result.geometry.attributes?.normal) {
      result.geometry.computeVertexNormals();
    }
  } else if (geoObj.scene) {
    const findAllMeshesInScene = (obj: THREE.Object3D) => {
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
          findAllMeshesInScene(child);
        }
      }
    };
    findAllMeshesInScene(geoObj.scene);
    
    if (result.meshes.length > 0) {
      const overallBox = new THREE.Box3();
      result.meshes.forEach(mesh => {
        const meshBox = new THREE.Box3();
        meshBox.setFromObject(mesh);
        overallBox.union(meshBox);
      });
      const size = overallBox.getSize(new THREE.Vector3()).length();
      result.sceneScale = size > 0 ? 2 / size : 1;
      result.geometry = result.meshes.length === 1 
        ? result.meshes[0].geometry 
        : combineGeometries(result.meshes);
    }
  } else if (geoObj.children && geoObj.children.length > 0) {
    const findAllMeshesInGroup = (obj: THREE.Object3D) => {
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
          findAllMeshesInGroup(child);
        }
      }
    };
    findAllMeshesInGroup(geo as THREE.Object3D);
    
    if (result.meshes.length > 0) {
      const overallBox = new THREE.Box3();
      result.meshes.forEach(mesh => {
        const meshBox = new THREE.Box3();
        meshBox.setFromObject(mesh);
        overallBox.union(meshBox);
      });
      const size = overallBox.getSize(new THREE.Vector3()).length();
      result.sceneScale = size > 0 ? 2 / size : 1;
      result.geometry = result.meshes.length === 1 
        ? result.meshes[0].geometry 
        : combineGeometries(result.meshes);
    }
  } else if (geoObj.isMesh && geoObj.geometry) {
    result.geometry = geoObj.geometry;
    if (!result.geometry.attributes?.normal) {
      result.geometry.computeVertexNormals();
    }
  } else if (geoObj.geometry) {
    result.geometry = geoObj.geometry;
    if (!result.geometry.attributes?.normal) {
      result.geometry.computeVertexNormals();
    }
  }

  return result;
}

async function loadFBX(data: Uint8Array): Promise<THREE.Group> {
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

async function loadGLTF(data: Uint8Array): Promise<GLTF | THREE.BufferGeometry> {
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

async function loadGLB(data: Uint8Array): Promise<GLTF | THREE.BufferGeometry> {
  return new Promise((resolve, reject) => {
    try {
      const loader = new GLTFLoader();
      try {
        const dracoLoader = new DRACOLoader();
        dracoLoader.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.6/');
        dracoLoader.preload();
        loader.setDRACOLoader(dracoLoader);
      } catch { /* Continue without DRACO support */ }
      
      let arrayBuffer: ArrayBuffer;
      if (data instanceof Uint8Array) {
        arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer;
      } else {
        arrayBuffer = data as ArrayBuffer;
      }
      
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

async function loadOBJ(data: Uint8Array): Promise<THREE.Group> {
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

async function loadPLY(data: Uint8Array): Promise<THREE.BufferGeometry> {
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

async function loadSTL(data: Uint8Array): Promise<THREE.BufferGeometry> {
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

function combineGeometries(meshes: THREE.Mesh[]): THREE.BufferGeometry | null {
  try {
    const filteredMeshes = meshes.filter(mesh => {
      const name = mesh.name.toLowerCase();
      return !name.includes('plane') && !name.includes('background');
    });
    
    if (filteredMeshes.length === 0) {
      return meshes[0]?.geometry || null;
    }
    
    const compatibleGeometries = filteredMeshes.map(mesh => {
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
    compatibleGeometries.forEach(g => g.dispose());
    
    return mergedGeometry;
  } catch {
    const fallbackMesh = meshes.find(m => !m.name.toLowerCase().includes('plane'));
    return fallbackMesh?.geometry || meshes[0]?.geometry || null;
  }
}

export default function FilePreview({ file }: FilePreviewProps) {
  const [meshData, setMeshData] = useState<Uint8Array | string | null>(null);
  const [fileType, setFileType] = useState<FileType | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!file) {
      setMeshData(null);
      setFileType(null);
      setError(null);
      return;
    }

    const loadFile = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const extension = file.name.split('.').pop()?.toLowerCase() || '';
        const meshFormats: MeshFileType[] = ['obj', 'ply', 'stl', 'gltf', 'glb', 'fbx'];
        const imageFormats = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'tiff', 'tif', 'svg', 'ico'];
        
        if (meshFormats.includes(extension as MeshFileType)) {
          setFileType(extension as MeshFileType);
          const arrayBuffer = await file.arrayBuffer();
          setMeshData(new Uint8Array(arrayBuffer));
        } else if (imageFormats.includes(extension)) {
          setFileType('image');
          
          if (extension === 'tiff' || extension === 'tif') {
            setMeshData('tiff-not-supported');
          } else {
            const imageUrl = URL.createObjectURL(file);
            setMeshData(imageUrl);
          }
        } else {
          setError(`Unsupported file format: ${extension}`);
        }
      } catch (err) {
        setError(`Failed to load file: ${err instanceof Error ? err.message : String(err)}`);
      } finally {
        setIsLoading(false);
      }
    };

    loadFile();
  }, [file]);

  useEffect(() => {
    return () => {
      if (meshData && typeof meshData === 'string' && meshData.startsWith('blob:')) {
        URL.revokeObjectURL(meshData);
      }
    };
  }, [meshData]);

  if (isLoading) {
    return (
      <div className="w-full h-full bg-black rounded overflow-hidden">
        <div className="flex items-center justify-center h-full text-neutral-500">
          <div className="text-center">
            <div className="animate-spin text-2xl mb-2">&#8635;</div>
            <p className="text-xs">Loading file...</p>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full h-full bg-black rounded overflow-hidden">
        <div className="flex items-center justify-center h-full text-red-500">
          <div className="text-center">
            <div className="text-2xl mb-2">&#9888;&#65039;</div>
            <p className="text-xs">{error}</p>
          </div>
        </div>
      </div>
    );
  }

  if (!meshData || !fileType) {
    return (
      <div className="w-full h-full bg-black rounded overflow-hidden">
        <div className="flex items-center justify-center h-full text-neutral-500">
          <div className="text-center">
            <div className="text-2xl mb-1">&#128193;</div>
            <p className="text-xs font-normal">Select a file</p>
            <p className="text-xs text-neutral-600">Images: PNG, JPG, WebP, GIF, BMP, TIFF</p>
            <p className="text-xs text-neutral-600">3D Models: OBJ, PLY, STL, GLTF, GLB, FBX</p>
          </div>
        </div>
      </div>
    );
  }

  if (fileType === 'image') {
    if (meshData === 'tiff-not-supported') {
      return (
        <div className="w-full h-full bg-black rounded overflow-hidden flex items-start justify-center pt-4">
          <div className="text-center max-w-sm px-6 text-neutral-100">
            <div className="text-4xl mb-4">&#128444;&#65039;</div>
            <p className="text-xs text-neutral-500 mb-3">
              TIFF files cannot be displayed directly in browsers. Use the processing panel to convert to PNG, JPEG, or WebP for web viewing.
            </p>
            <div className="text-xs text-neutral-500">
              <p className="mb-1">File: {file?.name}</p>
              <p>Size: {file?.size ? Math.round(file.size / 1024) + ' KB' : 'Unknown'}</p>
            </div>
          </div>
        </div>
      );
    }

    return (
      <div className="w-full h-full bg-black rounded overflow-hidden flex items-start justify-center pt-4">
        <img 
          src={meshData as string} 
          alt="Preview" 
          className="max-w-full max-h-full object-contain"
          onError={() => {
            setError('Failed to load image. The file may be corrupted or in an unsupported format.');
          }}
        />
      </div>
    );
  }

  return (
    <div className="w-full h-full bg-black rounded overflow-hidden">
      <Canvas>
        <PerspectiveCamera makeDefault position={[3, 3, 3]} />
        <OrbitControls enablePan={true} enableZoom={true} enableRotate={true} />
        <ambientLight intensity={0.6} />
        <directionalLight position={[10, 10, 5]} intensity={1} />
        <directionalLight position={[-10, -10, -5]} intensity={0.3} />
        <MeshModel meshData={meshData as Uint8Array} fileType={fileType as MeshFileType} />
        <gridHelper args={[10, 10, '#333333', '#111111']} />
        <axesHelper args={[2]} />
      </Canvas>
    </div>
  );
}
