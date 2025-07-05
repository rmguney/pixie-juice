'use client';

import { useState, useEffect } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera } from '@react-three/drei';
import * as THREE from 'three';

function MeshModel({ meshData, fileType }) {
  const [geometry, setGeometry] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (!meshData) return;

    const loadMesh = async () => {
      try {
        let geo;

        if (fileType === 'obj') {
          // Simple OBJ parser
          const text = new TextDecoder().decode(meshData);
          geo = parseOBJ(text);
        } else if (fileType === 'ply') {
          // Simple PLY parser
          const text = new TextDecoder().decode(meshData);
          geo = parsePLY(text);
        } else if (fileType === 'stl') {
          // STL would need binary parsing
          setError('STL preview not yet implemented');
          return;
        }

        if (geo) {
          geo.computeVertexNormals();
          geo.center();
          setGeometry(geo);
        }
      } catch (err) {
        console.error('Error loading mesh:', err);
        setError('Failed to load mesh');
      }
    };

    loadMesh();
  }, [meshData, fileType]);

  if (error) {
    return (
      <mesh>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial color="red" />
      </mesh>
    );
  }

  if (!geometry) {
    return (
      <mesh>
        <boxGeometry args={[0.5, 0.5, 0.5]} />
        <meshStandardMaterial color="neutral" />
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

// Simple OBJ parser
function parseOBJ(text) {
  const vertices = [];
  const faces = [];

  const lines = text.split('\n');
  
  for (const line of lines) {
    const parts = line.trim().split(/\s+/);
    
    if (parts[0] === 'v') {
      vertices.push(
        parseFloat(parts[1]),
        parseFloat(parts[2]),
        parseFloat(parts[3])
      );
    } else if (parts[0] === 'f') {
      // Simple triangulation (assumes triangular faces or converts quads)
      const faceVertices = parts.slice(1).map(p => parseInt(p.split('/')[0]) - 1);
      
      if (faceVertices.length === 3) {
        faces.push(...faceVertices);
      } else if (faceVertices.length === 4) {
        // Convert quad to two triangles
        faces.push(faceVertices[0], faceVertices[1], faceVertices[2]);
        faces.push(faceVertices[0], faceVertices[2], faceVertices[3]);
      }
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
  geometry.setIndex(faces);
  
  return geometry;
}

// Simple PLY parser (ASCII only)
function parsePLY(text) {
  const lines = text.split('\n');
  let vertexCount = 0;
  let faceCount = 0;
  let headerEnd = 0;

  // Parse header
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    
    if (line.startsWith('element vertex')) {
      vertexCount = parseInt(line.split(' ')[2]);
    } else if (line.startsWith('element face')) {
      faceCount = parseInt(line.split(' ')[2]);
    } else if (line === 'end_header') {
      headerEnd = i + 1;
      break;
    }
  }

  const vertices = [];
  const faces = [];

  // Parse vertices
  for (let i = headerEnd; i < headerEnd + vertexCount; i++) {
    const parts = lines[i].trim().split(/\s+/);
    vertices.push(
      parseFloat(parts[0]),
      parseFloat(parts[1]),
      parseFloat(parts[2])
    );
  }

  // Parse faces
  for (let i = headerEnd + vertexCount; i < headerEnd + vertexCount + faceCount; i++) {
    const parts = lines[i].trim().split(/\s+/);
    const faceVertexCount = parseInt(parts[0]);
    
    if (faceVertexCount === 3) {
      faces.push(
        parseInt(parts[1]),
        parseInt(parts[2]),
        parseInt(parts[3])
      );
    } else if (faceVertexCount === 4) {
      // Convert quad to triangles
      faces.push(
        parseInt(parts[1]),
        parseInt(parts[2]),
        parseInt(parts[3])
      );
      faces.push(
        parseInt(parts[1]),
        parseInt(parts[3]),
        parseInt(parts[4])
      );
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
  geometry.setIndex(faces);
  
  return geometry;
}

function Scene({ mesh }) {
  const [meshData, setMeshData] = useState(null);
  const [fileType, setFileType] = useState(null);

  useEffect(() => {
    if (!mesh) {
      setMeshData(null);
      setFileType(null);
      return;
    }

    const loadMeshData = async () => {
      try {
        const arrayBuffer = await mesh.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);
        setMeshData(uint8Array);
        
        const extension = mesh.name.toLowerCase().split('.').pop();
        setFileType(extension);
      } catch (error) {
        console.error('Error reading mesh file:', error);
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
      {meshData && fileType ? (
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

function ImagePreview({ file }) {
  const [imageUrl, setImageUrl] = useState(null);
  
  useEffect(() => {
    if (!file) {
      setImageUrl(null);
      return;
    }
    
    // Create object URL for image preview
    const url = URL.createObjectURL(file);
    setImageUrl(url);
    
    // Clean up URL when component unmounts or file changes
    return () => {
      URL.revokeObjectURL(url);
    };
  }, [file]);
  
  if (!imageUrl) {
    return null;
  }
  
  return (
    <div className="w-full h-full flex items-center justify-center">
      <img 
        src={imageUrl} 
        alt="Files" 
        className="max-w-full max-h-full object-contain" 
      />
    </div>
  );
}

export default function FilePreview({ file }) {
  if (!file) {
    return (
      <div className="w-full h-72 flex items-center justify-center bg-black rounded overflow-hidden border-t border-neutral-800">
        <div className="text-center">
          <p className="text-xs font-normal text-neutral-400">Select a file to preview</p>
          <p className="text-xs text-neutral-600">Images and 3D models supported</p>
        </div>
      </div>
    );
  }

  const isImage = file.type.startsWith('image/');
  const isMesh = !isImage && /\.(obj|ply|stl|gltf|glb|fbx|dae)$/i.test(file.name);
  
  return (
    <div className="w-full h-72 bg-black rounded overflow-hidden border border-neutral-800">
      <div className="absolute inset-x-0 top-0 px-4 py-1 bg-neutral-900/80 backdrop-blur-sm z-10">
        <div className="flex items-center space-x-2">
          <span className="text-sm">{isImage ? '🖼️' : '🧊'}</span>
          <span className="text-xs text-white truncate max-w-xs">{file.name}</span>
        </div>
      </div>
      
      {isImage ? (
        <ImagePreview file={file} />
      ) : isMesh ? (
        <Canvas>
          <Scene mesh={file} />
        </Canvas>
      ) : (
        <div className="flex items-center justify-center h-full text-neutral-500">
          <div className="text-center">
            <div className="text-2xl mb-1">📄</div>
            <p className="text-xs font-normal">Unsupported file format</p>
          </div>
        </div>
      )}
    </div>
  );
}
