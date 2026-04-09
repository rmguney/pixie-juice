import { useState, useEffect } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera } from '@react-three/drei';
import * as THREE from 'three';
import type { FilePreviewProps } from '../types';
import {
  loadMeshByType,
  processLoadedObject,
  type MeshFileType,
} from '../utils/meshLoaders';

type FileType = MeshFileType | 'image';

interface MeshModelProps {
  meshData: Uint8Array;
  fileType: MeshFileType;
}

function MeshModel({ meshData, fileType }: MeshModelProps) {
  const [geometry, setGeometry] = useState<THREE.BufferGeometry | null>(null);
  const [meshes, setMeshes] = useState<THREE.Mesh[] | null>(null);
  const [sceneScale, setSceneScale] = useState(1);
  const [sceneCenter, setSceneCenter] = useState<[number, number, number]>([0, 0, 0]);
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
        const geo = await loadMeshByType(meshData, fileType);

        if (geo) {
          const processedResult = processLoadedObject(geo);
          setSceneCenter(processedResult.sceneCenter);
          if (processedResult.meshes && processedResult.meshes.length > 0) {
            setMeshes(processedResult.meshes);
            setSceneScale(processedResult.sceneScale);
            if (processedResult.geometry) {
              setGeometry(processedResult.geometry);
            }
          } else if (processedResult.geometry) {
            setGeometry(processedResult.geometry);
            setSceneScale(processedResult.sceneScale);
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

  const offset: [number, number, number] = [
    -sceneCenter[0] * sceneScale,
    -sceneCenter[1] * sceneScale,
    -sceneCenter[2] * sceneScale,
  ];

  if (meshes && meshes.length > 0) {
    return (
      <group scale={sceneScale} position={offset}>
        {meshes.map((mesh, idx) => (
          <primitive key={idx} object={mesh} />
        ))}
      </group>
    );
  }

  if (!geometry) {
    return null;
  }

  return (
    <group scale={sceneScale} position={offset}>
      <mesh geometry={geometry}>
        <meshStandardMaterial
          color="#6366f1"
          metalness={0.2}
          roughness={0.3}
          side={THREE.DoubleSide}
        />
      </mesh>
    </group>
  );
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
