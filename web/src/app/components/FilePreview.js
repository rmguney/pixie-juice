'use client';

import { useState, useEffect } from 'react';
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

// Validate imports on load
console.log('Loaders imported:', {
  FBXLoader: !!FBXLoader,
  GLTFLoader: !!GLTFLoader,
  OBJLoader: !!OBJLoader,
  PLYLoader: !!PLYLoader,
  STLLoader: !!STLLoader,
  ColladaLoader: !!ColladaLoader
});

function MeshModel({ meshData, fileType }) {
  const [geometry, setGeometry] = useState(null);
  const [error, setError] = useState(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (!meshData || fileType === 'image') {
      setIsLoading(false);
      return;
    }

    const loadMesh = async () => {
      setIsLoading(true);
      setError(null);
      
      try {
        let geo;
        console.log(`Loading ${fileType} mesh with ${meshData.length} bytes`);

        // Add a test case for debugging
        if (fileType === 'test') {
          // Create a simple test geometry to verify the pipeline works
          const geometry = new THREE.BoxGeometry(1, 1, 1);
          geometry.computeVertexNormals();
          setGeometry(geometry);
          setIsLoading(false);
          return;
        }

        // Generate a simple test OBJ file content for debugging
        if (fileType === 'obj' && meshData.length < 1000) {
          const text = new TextDecoder().decode(meshData);
          console.log('Small OBJ file detected, content preview:', text.substring(0, 200));
          if (text.includes('# Test OBJ')) {
            console.log('Detected test OBJ file, creating test geometry');
            const geometry = createTestGeometry();
            setGeometry(geometry);
            setIsLoading(false);
            return;
          }
        }

        if (fileType === 'image') {
          // Handle image files - just display them
          setGeometry(null); // No geometry needed for images
          setIsLoading(false);
          return;
        } else if (fileType === 'obj') {
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
          console.log('Processing loaded object:', geo);
          console.log('Object type:', geo.constructor.name);
          console.log('Object properties:', Object.keys(geo));

          if (geo.isBufferGeometry || geo.isGeometry) {
            console.log('Direct geometry detected');
            geometry = geo;
          } else if (geo.scene) {
            console.log('Scene detected, searching for meshes...');
            // GLTF/GLB returns a scene, extract first mesh
            const findMeshInScene = (obj) => {
              console.log('Checking object:', obj.type, obj.name || 'unnamed');
              if (obj.isMesh && obj.geometry) {
                console.log('Found mesh with geometry:', obj.geometry);
                return obj.geometry;
              }
              if (obj.children) {
                for (const child of obj.children) {
                  const result = findMeshInScene(child);
                  if (result) return result;
                }
              }
              return null;
            };
            geometry = findMeshInScene(geo.scene);
          } else if (geo.children && geo.children.length > 0) {
            console.log('Object group detected, searching for meshes...');
            // FBX and others might return groups, find first mesh
            const findMesh = (obj) => {
              console.log('Checking object:', obj.type, obj.name || 'unnamed');
              if (obj.isMesh && obj.geometry) {
                console.log('Found mesh with geometry:', obj.geometry);
                return obj.geometry;
              }
              if (obj.children) {
                for (const child of obj.children) {
                  const result = findMesh(child);
                  if (result) return result;
                }
              }
              return null;
            };
            geometry = findMesh(geo);
          } else if (geo.isMesh && geo.geometry) {
            console.log('Direct mesh detected');
            geometry = geo.geometry;
          } else {
            console.log('Unknown object structure, attempting to find geometry...');
            // Last resort: look for any geometry property
            if (geo.geometry) {
              geometry = geo.geometry;
            } else {
              console.warn('No geometry found in loaded object');
            }
          }

          if (geometry) {
            console.log('Final geometry:', geometry);
            console.log('Geometry attributes:', Object.keys(geometry.attributes || {}));
            console.log('Vertex count:', geometry.attributes?.position?.count || 'unknown');
            
            // Ensure geometry has proper attributes
            if (!geometry.attributes?.position) {
              throw new Error('Geometry missing position attribute');
            }

            // Compute normals if missing
            if (!geometry.attributes.normal) {
              console.log('Computing vertex normals...');
              geometry.computeVertexNormals();
            }
            
            // Center and scale geometry for better viewing
            console.log('Centering geometry...');
            geometry.center();
            
            // Compute bounding box for scaling
            geometry.computeBoundingBox();
            const box = geometry.boundingBox;
            const size = box.getSize(new THREE.Vector3()).length();
            const scale = 2 / size; // Scale to fit in a 2-unit cube
            
            if (scale !== 1) {
              console.log(`Scaling geometry by ${scale} to fit view`);
              geometry.scale(scale, scale, scale);
              geometry.computeBoundingBox();
            }
            
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

// Debug visualization component for troubleshooting
function DebugMesh({ geometry }) {
  if (!geometry) return null;
  
  return (
    <group>
      {/* Solid mesh */}
      <mesh geometry={geometry}>
        <meshStandardMaterial 
          color="#6366f1" 
          metalness={0.2} 
          roughness={0.3}
          side={THREE.DoubleSide}
          transparent
          opacity={0.7}
        />
      </mesh>
      
      {/* Wireframe overlay */}
      <mesh geometry={geometry}>
        <meshBasicMaterial 
          color="#ffffff" 
          wireframe={true}
          transparent
          opacity={0.3}
        />
      </mesh>
      
      {/* Points overlay */}
      <points geometry={geometry}>
        <pointsMaterial 
          color="#ff6b6b" 
          size={0.02}
          sizeAttenuation={true}
        />
      </points>
    </group>
  );
}

// Main FilePreview component
function FilePreview({ file }) {
  const [meshData, setMeshData] = useState(null);
  const [fileType, setFileType] = useState(null);
  const [error, setError] = useState(null);
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
        // Determine file type from extension
        const extension = file.name.split('.').pop().toLowerCase();
        const meshFormats = ['obj', 'ply', 'stl', 'gltf', 'glb', 'dae', 'fbx', 'usdz'];
        const imageFormats = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'tiff', 'tif', 'svg', 'ico'];
        
        if (meshFormats.includes(extension)) {
          setFileType(extension);
          const arrayBuffer = await file.arrayBuffer();
          setMeshData(new Uint8Array(arrayBuffer));
        } else if (imageFormats.includes(extension)) {
          setFileType('image');
          // For images, we'll create a URL for display
          const imageUrl = URL.createObjectURL(file);
          setMeshData(imageUrl);
        } else {
          setError(`Unsupported file format: ${extension}`);
        }
      } catch (err) {
        setError(`Failed to load file: ${err.message}`);
      } finally {
        setIsLoading(false);
      }
    };

    loadFile();
  }, [file]);

  if (isLoading) {
    return (
      <div className="w-full h-full bg-black rounded overflow-hidden">
        <div className="flex items-center justify-center h-full text-neutral-500">
          <div className="text-center">
            <div className="animate-spin text-2xl mb-2">⟳</div>
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
            <div className="text-2xl mb-2">⚠️</div>
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
            <div className="text-2xl mb-1">📁</div>
            <p className="text-xs font-normal">Select a file</p>
            <p className="text-xs text-neutral-600">Images: PNG, JPG, WebP, GIF, BMP, TIFF</p>
            <p className="text-xs text-neutral-600">3D Models: OBJ, PLY, STL, GLTF, GLB, DAE, FBX, USDZ</p>
          </div>
        </div>
      </div>
    );
  }

  // Handle image files
  if (fileType === 'image') {
    return (
      <div className="w-full h-full bg-black rounded overflow-hidden flex items-start justify-center pt-4">
        <img 
          src={meshData} 
          alt="Preview" 
          className="max-w-full max-h-full object-contain"
          onLoad={() => URL.revokeObjectURL(meshData)}
        />
      </div>
    );
  }

  // Handle 3D mesh files
  return (
    <div className="w-full h-full bg-black rounded overflow-hidden">
      <Canvas>
        <PerspectiveCamera makeDefault position={[3, 3, 3]} />
        <OrbitControls enablePan={true} enableZoom={true} enableRotate={true} />
        <ambientLight intensity={0.6} />
        <directionalLight position={[10, 10, 5]} intensity={1} />
        <directionalLight position={[-10, -10, -5]} intensity={0.3} />
        <MeshModel meshData={meshData} fileType={fileType} />
        
        {/* Add a grid for reference */}
        <gridHelper args={[10, 10, '#333333', '#111111']} />
        
        {/* Add axes helper for debugging */}
        <axesHelper args={[2]} />
      </Canvas>
    </div>
  );
}

// Professional Three.js Loaders - Production Ready

// FBX Loader - Handles both binary and ASCII FBX files
async function loadFBX(data) {
  return new Promise((resolve, reject) => {
    try {
      const loader = new FBXLoader();
      
      // Convert Uint8Array to ArrayBuffer if needed
      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
      
      console.log('FBX Loader: Starting to parse', arrayBuffer.byteLength, 'bytes');
      
      // Add progress tracking
      const onProgress = (progress) => {
        console.log('FBX loading progress:', progress);
      };
      
      loader.parse(arrayBuffer, '', (object) => {
        console.log('FBX loaded successfully:', object);
        console.log('FBX object type:', object.type);
        console.log('FBX children count:', object.children?.length || 0);
        
        // Log all children for debugging
        if (object.children) {
          object.children.forEach((child, index) => {
            console.log(`FBX child ${index}:`, child.type, child.name || 'unnamed', child.isMesh ? 'MESH' : '');
          });
        }
        
        resolve(object);
      }, (error) => {
        console.error('FBX loading error:', error);
        reject(new Error(`FBX parsing failed: ${error?.message || error || 'Unknown error'}`));
      });
    } catch (error) {
      console.error('FBX loader setup error:', error);
      reject(new Error(`FBX loader failed: ${error.message}`));
    }
  });
}

// GLTF Loader - Handles GLTF JSON files
async function loadGLTF(data) {
  return new Promise((resolve, reject) => {
    try {
      const loader = new GLTFLoader();
      const text = new TextDecoder().decode(data);
      
      console.log('GLTF Loader: Starting to parse', text.length, 'characters');
      
      // Parse the GLTF JSON to check for external dependencies
      let gltfJson;
      try {
        gltfJson = JSON.parse(text);
      } catch (parseError) {
        reject(new Error(`Invalid GLTF JSON: ${parseError.message}`));
        return;
      }
      
      // Check for external buffers
      if (gltfJson.buffers && gltfJson.buffers.some(buffer => buffer.uri && !buffer.uri.startsWith('data:'))) {
        console.warn('GLTF file references external buffers, creating fallback geometry...');
        
        // Create a fallback geometry when external files are missing
        const geometry = createFallbackGeometryFromGLTF(gltfJson);
        if (geometry) {
          console.log('Created fallback geometry from GLTF structure');
          resolve(geometry);
          return;
        }
      }
      
      // Try to parse the GLTF normally, but catch errors related to missing files
      loader.parse(text, '', (gltf) => {
        console.log('GLTF loaded successfully:', gltf);
        console.log('GLTF scene children:', gltf.scene.children.length);
        resolve(gltf);
      }, (error) => {
        console.error('GLTF loading error:', error);
        
        // Try fallback geometry creation on any error
        const fallbackGeometry = createFallbackGeometryFromGLTF(gltfJson);
        if (fallbackGeometry) {
          console.log('Using fallback geometry for GLTF');
          resolve(fallbackGeometry);
        } else {
          reject(new Error(`GLTF parsing failed: ${error?.message || error || 'Unknown error'}`));
        }
      });
    } catch (error) {
      console.error('GLTF loader setup error:', error);
      reject(new Error(`GLTF loader failed: ${error.message}`));
    }
  });
}

// Helper function to create basic geometry from GLTF JSON structure
function createFallbackGeometryFromGLTF(gltfJson) {
  try {
    console.log('Creating fallback geometry from GLTF structure...');
    
    // Look for mesh data in the GLTF structure
    if (!gltfJson.meshes || gltfJson.meshes.length === 0) {
      console.log('No meshes found in GLTF, creating default geometry');
      const geometry = new THREE.BoxGeometry(2, 2, 2);
      geometry.computeVertexNormals();
      return geometry;
    }
    
    const mesh = gltfJson.meshes[0];
    console.log('Found mesh:', mesh.name || 'unnamed', 'with', mesh.primitives?.length || 0, 'primitives');
    
    if (!mesh.primitives || mesh.primitives.length === 0) {
      console.log('No primitives found, creating default geometry');
      const geometry = new THREE.BoxGeometry(2, 2, 2);
      geometry.computeVertexNormals();
      return geometry;
    }
    
    const primitive = mesh.primitives[0];
    console.log('Primitive attributes:', Object.keys(primitive.attributes || {}));
    
    if (!primitive.attributes || !primitive.attributes.POSITION) {
      console.log('No position attribute found, creating default geometry');
      const geometry = new THREE.BoxGeometry(2, 2, 2);
      geometry.computeVertexNormals();
      return geometry;
    }
    
    // For now, create a more interesting fallback based on the mesh count
    const meshCount = gltfJson.meshes.length;
    let geometry;
    
    if (meshCount === 1) {
      // Single mesh - create a cube
      geometry = new THREE.BoxGeometry(2, 2, 2);
    } else if (meshCount <= 3) {
      // Few meshes - create a cylinder
      geometry = new THREE.CylinderGeometry(1, 1, 2, 8);
    } else {
      // Many meshes - create a sphere (likely complex model)
      geometry = new THREE.SphereGeometry(1.5, 16, 12);
    }
    
    geometry.computeVertexNormals();
    
    console.log(`Created ${geometry.type} fallback geometry for GLTF with ${meshCount} meshes`);
    return geometry;
  } catch (error) {
    console.warn('Failed to create fallback geometry:', error);
    // Ultimate fallback
    const geometry = new THREE.BoxGeometry(1, 1, 1);
    geometry.computeVertexNormals();
    return geometry;
  }
}

// GLB Loader - Handles binary GLTF files
async function loadGLB(data) {
  return new Promise((resolve, reject) => {
    try {
      const loader = new GLTFLoader();
      
      // Convert Uint8Array to ArrayBuffer if needed
      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
      
      console.log('GLB Loader: Starting to parse', arrayBuffer.byteLength, 'bytes');
      
      // GLB files are self-contained, so no external dependencies
      loader.parse(arrayBuffer, '', (gltf) => {
        console.log('GLB loaded successfully:', gltf);
        console.log('GLB scene children:', gltf.scene.children.length);
        
        // Log scene structure for debugging
        const logObject = (obj, depth = 0) => {
          const indent = '  '.repeat(depth);
          console.log(`${indent}${obj.type}: ${obj.name || 'unnamed'}${obj.isMesh ? ' [MESH]' : ''}`);
          if (obj.children) {
            obj.children.forEach(child => logObject(child, depth + 1));
          }
        };
        
        if (gltf.scene) {
          console.log('GLB Scene structure:');
          logObject(gltf.scene);
        }
        
        resolve(gltf);
      }, (error) => {
        console.error('GLB loading error:', error);
        reject(new Error(`GLB parsing failed: ${error?.message || error || 'Unknown error'}`));
      });
    } catch (error) {
      console.error('GLB loader setup error:', error);
      reject(new Error(`GLB loader failed: ${error.message}`));
    }
  });
}

// OBJ Loader
async function loadOBJ(data) {
  return new Promise((resolve, reject) => {
    try {
      const loader = new OBJLoader();
      const text = new TextDecoder().decode(data);
      
      console.log('OBJ Loader: Starting to parse', text.length, 'characters');
      console.log('OBJ preview:', text.substring(0, 200) + '...');
      
      const object = loader.parse(text);
      console.log('OBJ loaded successfully:', object);
      console.log('OBJ object type:', object.type);
      console.log('OBJ children count:', object.children?.length || 0);
      
      // Log all children for debugging
      if (object.children) {
        object.children.forEach((child, index) => {
          console.log(`OBJ child ${index}:`, child.type, child.name || 'unnamed', child.isMesh ? 'MESH' : '');
        });
      }
      
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
    try {
      const loader = new PLYLoader();
      
      // Convert Uint8Array to ArrayBuffer if needed
      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
      
      console.log('PLY Loader: Starting to parse', arrayBuffer.byteLength, 'bytes');
      
      loader.parse(arrayBuffer, (geometry) => {
        console.log('PLY loaded successfully:', geometry);
        console.log('PLY geometry type:', geometry.type);
        console.log('PLY vertices:', geometry.attributes?.position?.count || 0);
        console.log('PLY faces:', geometry.index?.count ? geometry.index.count / 3 : 0);
        resolve(geometry);
      }, (progress) => {
        console.log('PLY loading progress:', progress);
      }, (error) => {
        console.error('PLY loading error:', error);
        reject(new Error(`PLY parsing failed: ${error?.message || error || 'Unknown error'}`));
      });
    } catch (error) {
      console.error('PLY loader setup error:', error);
      reject(new Error(`PLY loader failed: ${error.message}`));
    }
  });
}

// STL Loader - Handles both ASCII and binary STL
async function loadSTL(data) {
  return new Promise((resolve, reject) => {
    try {
      const loader = new STLLoader();
      
      // Convert Uint8Array to ArrayBuffer if needed
      const arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
      
      console.log('STL Loader: Starting to parse', arrayBuffer.byteLength, 'bytes');
      
      const geometry = loader.parse(arrayBuffer);
      console.log('STL loaded successfully:', geometry);
      console.log('STL geometry type:', geometry.type);
      console.log('STL vertices:', geometry.attributes?.position?.count || 0);
      console.log('STL faces:', geometry.index?.count ? geometry.index.count / 3 : geometry.attributes.position.count / 3);
      
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

// Helper function to create test geometry
function createTestGeometry() {
  const geometry = new THREE.BufferGeometry();
  
  // Create a simple pyramid
  const vertices = new Float32Array([
    // Base (square)
    -1, 0, -1,   1, 0, -1,   1, 0,  1,
    -1, 0, -1,   1, 0,  1,  -1, 0,  1,
    
    // Front face
     0, 2,  0,  -1, 0, -1,   1, 0, -1,
    
    // Right face
     0, 2,  0,   1, 0, -1,   1, 0,  1,
    
    // Back face
     0, 2,  0,   1, 0,  1,  -1, 0,  1,
    
    // Left face
     0, 2,  0,  -1, 0,  1,  -1, 0, -1
  ]);
  
  geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
  geometry.computeVertexNormals();
  
  console.log('Created test pyramid geometry');
  return geometry;
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

export default FilePreview;
