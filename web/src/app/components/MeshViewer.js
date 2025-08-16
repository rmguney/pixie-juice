'use client';

import { Suspense, useEffect, useState } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera } from '@react-three/drei';
import * as THREE from 'three';

// Import professional Three.js loaders
import { FBXLoader } from 'three-stdlib';
import { GLTFLoader } from 'three-stdlib';
import { OBJLoader } from 'three-stdlib';
import { PLYLoader } from 'three-stdlib';
import { STLLoader } from 'three-stdlib';
import { ColladaLoader } from 'three-stdlib';

// OBJ/PLY loader components
function MeshModel({ meshData, fileType }) {
  const [geometry, setGeometry] = useState(null);
  const [error, setError] = useState(null);
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
        let geo;
        console.log(`Loading ${fileType} mesh with ${meshData.length} bytes`);

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
        } else if (fileType === 'dae') {
          geo = await loadDAE(meshData);
        } else if (fileType === 'fbx') {
          geo = await loadFBX(meshData);
        } else if (fileType === 'usdz') {
          geo = await parseUSDZ(meshData); // Keep custom USDZ parser for now
        } else {
          setError(`Unsupported mesh format: ${fileType}`);
          setIsLoading(false);
          return;
        }

        if (geo) {
          // Handle both geometry and full objects from loaders
          let geometry;
          if (geo.isBufferGeometry || geo.isGeometry) {
            geometry = geo;
          } else if (geo.scene) {
            // GLTF/GLB returns a scene, extract first mesh
            const mesh = geo.scene.children.find(child => child.isMesh);
            geometry = mesh ? mesh.geometry : null;
          } else if (geo.children && geo.children.length > 0) {
            // FBX and others might return groups, find first mesh
            const findMesh = (obj) => {
              if (obj.isMesh) return obj.geometry;
              for (const child of obj.children) {
                const result = findMesh(child);
                if (result) return result;
              }
              return null;
            };
            geometry = findMesh(geo);
          }

          if (geometry) {
            geometry.computeVertexNormals();
            geometry.center();
            setGeometry(geometry);
          } else {
            throw new Error('No geometry found in loaded model');
          }
        } else {
          throw new Error('Failed to create geometry');
        }
      } catch (err) {
        console.error('Error loading mesh:', err);
        setError(`Failed to load ${fileType}: ${err.message}`);
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
    console.warn('Mesh loading error:', error);
    return (
      <mesh>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial color="#ef4444" />
      </mesh>
    );
  }

  if (!geometry) {
    return (
      <mesh>
        <boxGeometry args={[0.5, 0.5, 0.5]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
    );
  }

  return (
    <mesh geometry={geometry}>
      <meshStandardMaterial 
        color="#6366f1" 
        metalness={0.2} 
        roughness={0.3}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

// Professional Three.js Loaders - Production Ready

// FBX Loader - Handles both binary and ASCII FBX files
async function loadFBX(data) {
  return new Promise((resolve, reject) => {
    const loader = new FBXLoader();
    
    // Convert Uint8Array to ArrayBuffer if needed
    const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
    
    loader.parse(arrayBuffer, '', (object) => {
      console.log('FBX loaded successfully:', object);
      resolve(object);
    }, (error) => {
      console.error('FBX loading error:', error);
      reject(new Error(`FBX parsing failed: ${error.message || 'Unknown error'}`));
    });
  });
}

// GLTF Loader - Handles GLTF JSON files
async function loadGLTF(data) {
  return new Promise((resolve, reject) => {
    const loader = new GLTFLoader();
    const text = new TextDecoder().decode(data);
    
    loader.parse(text, '', (gltf) => {
      console.log('GLTF loaded successfully:', gltf);
      resolve(gltf);
    }, (error) => {
      console.error('GLTF loading error:', error);
      reject(new Error(`GLTF parsing failed: ${error.message || 'Unknown error'}`));
    });
  });
}

// GLB Loader - Handles binary GLTF files
async function loadGLB(data) {
  return new Promise((resolve, reject) => {
    const loader = new GLTFLoader();
    
    // Convert Uint8Array to ArrayBuffer if needed
    const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
    
    loader.parse(arrayBuffer, '', (gltf) => {
      console.log('GLB loaded successfully:', gltf);
      resolve(gltf);
    }, (error) => {
      console.error('GLB loading error:', error);
      reject(new Error(`GLB parsing failed: ${error.message || 'Unknown error'}`));
    });
  });
}

// OBJ Loader
async function loadOBJ(data) {
  return new Promise((resolve, reject) => {
    try {
      const loader = new OBJLoader();
      const text = new TextDecoder().decode(data);
      
      const object = loader.parse(text);
      console.log('OBJ loaded successfully:', object);
      resolve(object);
    } catch (error) {
      console.error('OBJ loading error:', error);
      reject(new Error(`OBJ parsing failed: ${error.message || 'Unknown error'}`));
    }
  });
}

// PLY Loader - Handles both ASCII and binary PLY
async function loadPLY(data) {
  return new Promise((resolve, reject) => {
    const loader = new PLYLoader();
    
    // Convert Uint8Array to ArrayBuffer if needed
    const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
    
    loader.parse(arrayBuffer, (geometry) => {
      console.log('PLY loaded successfully:', geometry);
      resolve(geometry);
    }, (error) => {
      console.error('PLY loading error:', error);
      reject(new Error(`PLY parsing failed: ${error?.message || 'Unknown error'}`));
    });
  });
}

// STL Loader - Handles both ASCII and binary STL
async function loadSTL(data) {
  return new Promise((resolve, reject) => {
    try {
      const loader = new STLLoader();
      
      // Convert Uint8Array to ArrayBuffer if needed
      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
      
      const geometry = loader.parse(arrayBuffer);
      console.log('STL loaded successfully:', geometry);
      resolve(geometry);
    } catch (error) {
      console.error('STL loading error:', error);
      reject(new Error(`STL parsing failed: ${error.message || 'Unknown error'}`));
    }
  });
}

// DAE/Collada Loader
async function loadDAE(data) {
  return new Promise((resolve, reject) => {
    const loader = new ColladaLoader();
    const text = new TextDecoder().decode(data);
    
    loader.parse(text, (collada) => {
      console.log('DAE loaded successfully:', collada);
      resolve(collada.scene);
    }, (error) => {
      console.error('DAE loading error:', error);
      reject(new Error(`DAE parsing failed: ${error?.message || 'Unknown error'}`));
    });
  });
}

// Keep the custom USDZ parser for now since Three.js doesn't have a built-in USDZ loader

// Basic USDZ parser - extracts geometry from USD files in ZIP archive
async function parseUSDZ(data) {
  try {
    // USDZ is a ZIP file containing USD files
    // For a basic implementation, we'll try to extract and parse the main USD file
    
    // Check if we have JSZip available
    if (typeof JSZip === 'undefined') {
      throw new Error('JSZip library required for USDZ support');
    }
    
    const zip = new JSZip();
    const archive = await zip.loadAsync(data);
    
    // Look for USD files in the archive
    let usdContent = null;
    const usdFiles = Object.keys(archive.files).filter(name => 
      name.endsWith('.usda') || name.endsWith('.usd')
    );
    
    if (usdFiles.length === 0) {
      throw new Error('No USD files found in USDZ archive');
    }
    
    // Use the first USD file found
    const usdFile = archive.files[usdFiles[0]];
    usdContent = await usdFile.async('text');
    
    // Basic USD parsing - look for mesh data
    return parseUSDContent(usdContent);
    
  } catch (error) {
    console.error('USDZ parsing error:', error);
    throw new Error(`Failed to parse USDZ: ${error.message}`);
  }
}

// Basic USD content parser - extracts points and faces
function parseUSDContent(content) {
  const lines = content.split('\n');
  const vertices = [];
  const faces = [];
  
  let inMesh = false;
  let inPoints = false;
  let inFaces = false;
  
  for (const line of lines) {
    const trimmed = line.trim();
    
    if (trimmed.startsWith('def Mesh') || trimmed.includes('Mesh')) {
      inMesh = true;
    }
    
    if (inMesh) {
      if (trimmed.startsWith('float3[] points')) {
        inPoints = true;
        // Extract points from the same line if they exist
        const match = trimmed.match(/\[(.*?)\]/);
        if (match) {
          parseUSDPoints(match[1], vertices);
          inPoints = false;
        }
      } else if (trimmed.startsWith('int[] faceVertexIndices')) {
        inFaces = true;
        // Extract face indices from the same line if they exist
        const match = trimmed.match(/\[(.*?)\]/);
        if (match) {
          parseUSDFaces(match[1], faces);
          inFaces = false;
        }
      } else if (inPoints && trimmed.includes('[')) {
        const match = trimmed.match(/\[(.*?)\]/);
        if (match) {
          parseUSDPoints(match[1], vertices);
          inPoints = false;
        }
      } else if (inFaces && trimmed.includes('[')) {
        const match = trimmed.match(/\[(.*?)\]/);
        if (match) {
          parseUSDFaces(match[1], faces);
          inFaces = false;
        }
      }
    }
  }
  
  if (vertices.length === 0 || faces.length === 0) {
    throw new Error('No mesh data found in USD content');
  }
  
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
  geometry.setIndex(faces);
  
  return geometry;
}

// Parse USD point coordinates
function parseUSDPoints(pointsStr, vertices) {
  // Parse format like: (0, 0, 0), (1, 0, 0), (1, 1, 0)
  const pointMatches = pointsStr.match(/\([^)]+\)/g);
  if (pointMatches) {
    for (const point of pointMatches) {
      const coords = point.slice(1, -1).split(',').map(s => parseFloat(s.trim()));
      if (coords.length >= 3) {
        vertices.push(coords[0], coords[1], coords[2]);
      }
    }
  }
}

// Parse USD face indices
function parseUSDFaces(facesStr, faces) {
  // Parse format like: 0, 1, 2, 3, 4, 7, 6, 5
  const indices = facesStr.split(',').map(s => parseInt(s.trim())).filter(n => !isNaN(n));
  
  // Convert to triangles (assuming quads for now)
  for (let i = 0; i < indices.length; i += 4) {
    if (i + 3 < indices.length) {
      // Convert quad to two triangles
      faces.push(indices[i], indices[i + 1], indices[i + 2]);
      faces.push(indices[i], indices[i + 2], indices[i + 3]);
    }
  }
}

function Scene({ mesh }) {
  const [meshData, setMeshData] = useState(null);
  const [fileType, setFileType] = useState(null);
  const [loadError, setLoadError] = useState(null);

  useEffect(() => {
    if (!mesh) {
      setMeshData(null);
      setFileType(null);
      setLoadError(null);
      return;
    }

    const loadMeshData = async () => {
      try {
        setLoadError(null);
        console.log('Loading mesh file:', mesh.name, 'Size:', mesh.size, 'bytes');
        
        const arrayBuffer = await mesh.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);
        setMeshData(uint8Array);
        
        const extension = mesh.name.toLowerCase().split('.').pop();
        console.log('Detected file type:', extension);
        setFileType(extension);
      } catch (error) {
        console.error('Error reading mesh file:', error);
        setLoadError(`Failed to read file: ${error.message}`);
      }
    };

    loadMeshData();
  }, [mesh]);

  return (
    <>
      <PerspectiveCamera makeDefault position={[3, 3, 3]} />
      <OrbitControls enableDamping />
      
      {/* Dark theme lighting */}
      <ambientLight intensity={0.3} />
      <directionalLight position={[5, 5, 5]} intensity={1} />
      <directionalLight position={[-5, -5, -5]} intensity={0.4} />
      
      {/* Mesh or fallback */}
      {loadError ? (
        <mesh>
          <boxGeometry args={[1, 1, 1]} />
          <meshStandardMaterial color="#ef4444" />
        </mesh>
      ) : meshData && fileType ? (
        <MeshModel meshData={meshData} fileType={fileType} />
      ) : (
        <mesh>
          <boxGeometry args={[1, 1, 1]} />
          <meshStandardMaterial color="#374151" />
        </mesh>
      )}
      
      {/* Subtle grid */}
      <gridHelper args={[10, 10, '#374151', '#1F2937']} />
    </>
  );
}

export default function MeshViewer({ mesh }) {
  return (
    <div className="w-full h-60 bg-black rounded overflow-hidden border border-neutral-800">
      {mesh ? (
        <Canvas>
          <Suspense fallback={
            <mesh>
              <boxGeometry args={[1, 1, 1]} />
              <meshStandardMaterial color="#374151" />
            </mesh>
          }>
            <Scene mesh={mesh} />
          </Suspense>
        </Canvas>
      ) : (
        <div className="flex items-center justify-center h-full text-neutral-500">
          <div className="text-center">
            <div className="text-2xl mb-1">🧊</div>
            <p className="text-xs font-normal">Select a 3D model</p>
            <p className="text-xs text-neutral-600">OBJ, PLY, STL, GLTF, GLB, DAE, FBX, USDZ supported</p>
          </div>
        </div>
      )}
    </div>
  );
}
