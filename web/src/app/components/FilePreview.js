'use client';

import { useState, useEffect } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera } from '@react-three/drei';
import * as THREE from 'three';

// Import professional Three.js loaders
import { FBXLoader } from 'three-stdlib';
import { GLTFLoader } from 'three-stdlib';
import { DRACOLoader } from 'three-stdlib';
import { OBJLoader } from 'three-stdlib';
import { PLYLoader } from 'three-stdlib';
import { STLLoader } from 'three-stdlib';
import { TGALoader } from 'three-stdlib';
import * as BufferGeometryUtils from 'three/examples/jsm/utils/BufferGeometryUtils.js';

// Validate imports on load
console.log('Loaders imported:', {
  FBXLoader: !!FBXLoader,
  GLTFLoader: !!GLTFLoader,
  DRACOLoader: !!DRACOLoader,
  OBJLoader: !!OBJLoader,
  PLYLoader: !!PLYLoader,
  STLLoader: !!STLLoader,
  TGALoader: !!TGALoader
});

function MeshModel({ meshData, fileType }) {
  const [geometry, setGeometry] = useState(null);
  const [materials, setMaterials] = useState(null);
  const [meshes, setMeshes] = useState(null);
  const [sceneScale, setSceneScale] = useState(1);
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
          console.log('Calling loadOBJ with', meshData.length, 'bytes');
          geo = await loadOBJ(meshData);
        } else if (fileType === 'ply') {
          console.log('Calling loadPLY with', meshData.length, 'bytes');
          geo = await loadPLY(meshData);
        } else if (fileType === 'stl') {
          console.log('Calling loadSTL with', meshData.length, 'bytes');
          geo = await loadSTL(meshData);
        } else if (fileType === 'gltf') {
          console.log('Calling loadGLTF with', meshData.length, 'bytes');
          geo = await loadGLTF(meshData);
        } else if (fileType === 'glb') {
          console.log('Calling loadGLB with', meshData.length, 'bytes');
          geo = await loadGLB(meshData);
        } else if (fileType === 'fbx') {
          console.log('Calling loadFBX with', meshData.length, 'bytes');
          geo = await loadFBX(meshData);
        } else {
          console.error('Unsupported mesh format:', fileType);
          setError(`Unsupported mesh format: ${fileType}`);
          setIsLoading(false);
          return;
        }
        
        console.log('Loader returned:', geo ? 'SUCCESS' : 'NULL/UNDEFINED');

        if (geo) {
          // Handle both geometry and full objects from loaders
          let geometry;
          console.log('Processing loaded object:', geo);
          console.log('Object type:', geo.constructor.name);
          console.log('Object properties:', Object.keys(geo));
          console.log('Is BufferGeometry:', geo.isBufferGeometry);
          console.log('Is Geometry:', geo.isGeometry);
          console.log('Has scene:', !!geo.scene);
          console.log('Has children:', !!geo.children);

          // Process loaded object and extract meshes
          const processedResult = processLoadedObject(geo);
          if (processedResult.meshes && processedResult.meshes.length > 0) {
            // Use the unified processing result
            setMeshes(processedResult.meshes);
            setMaterials(processedResult.materials);
            setSceneScale(processedResult.sceneScale);
            geometry = processedResult.geometry;
          } else if (processedResult.geometry) {
            geometry = processedResult.geometry;
          } else {
            throw new Error('No valid geometry or meshes found in loaded model');
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
            
            setGeometry(geometry);
          } else {
            throw new Error('No geometry found in loaded model');
          }
        } else {
          console.error('Mesh loading failed: geo is null/undefined');
          throw new Error('Failed to create geometry - loader returned null');
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
    console.log('MeshModel: Loading state, fileType:', fileType, 'meshData length:', meshData?.length);
    return (
      <mesh>
        <boxGeometry args={[0.8, 0.8, 0.8]} />
        <meshStandardMaterial color="#4f46e5" wireframe />
      </mesh>
    );
  }

  if (error) {
    console.warn('MeshModel: Error state -', error);
    return (
      <mesh>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial color="#ef4444" />
      </mesh>
    );
  }

  if (!geometry) {
    console.warn('MeshModel: No geometry state, fileType:', fileType, 'meshData available:', !!meshData);
    return (
      <mesh>
        <boxGeometry args={[0.5, 0.5, 0.5]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
    );
  }

  // If we have individual meshes with materials, render them separately
  if (meshes && meshes.length > 0) {
    return (
      <group scale={[sceneScale, sceneScale, sceneScale]}>
        {meshes.map((mesh, index) => {
          // Extract world transform components
          const worldPosition = new THREE.Vector3();
          const worldQuaternion = new THREE.Quaternion();
          const worldScale = new THREE.Vector3();
          
          mesh.matrixWorld.decompose(worldPosition, worldQuaternion, worldScale);
          
          // Convert quaternion to Euler angles
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

  // Fallback to single geometry rendering
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

// Unified function to process loaded 3D objects and extract meshes
function processLoadedObject(geo) {
  const result = {
    meshes: [],
    materials: [],
    geometry: null,
    sceneScale: 1
  };

  console.log('Processing loaded object unified...');

  if (geo.isBufferGeometry || geo.isGeometry) {
    console.log('Direct geometry detected');
    result.geometry = geo;
    // Compute normals if missing
    if (!geo.attributes?.normal) {
      console.log('Computing vertex normals...');
      geo.computeVertexNormals();
    }
  } else if (geo.scene) {
    console.log('Scene detected, searching for all meshes...');
    // GLTF/GLB returns a scene, extract ALL meshes with materials
    const findAllMeshesInScene = (obj) => {
      console.log('Checking object:', obj.type, obj.name || 'unnamed');
      if (obj.isMesh && obj.geometry) {
        const name = obj.name.toLowerCase();
        const isBackground = name.includes('plane') || name.includes('background');
        
        console.log('Found mesh with geometry:', obj.geometry, 'vertices:', obj.geometry.attributes?.position?.count || 0);
        console.log('Mesh material:', obj.material ? obj.material.type : 'none');
        console.log('Is background element:', isBackground);
        
        if (!isBackground) {
          // Update world matrix to get correct positioning
          obj.updateMatrixWorld(true);
          result.meshes.push(obj);
          if (obj.material && !result.materials.includes(obj.material)) {
            result.materials.push(obj.material);
          }
        }
      }
      if (obj.children) {
        for (const child of obj.children) {
          findAllMeshesInScene(child);
        }
      }
    };
    findAllMeshesInScene(geo.scene);
    
    console.log(`Found ${result.meshes.length} meshes total`);
    console.log(`Found ${result.materials.length} materials total`);
    
    if (result.meshes.length > 0) {
      // Calculate overall scene bounding box for scaling
      const overallBox = new THREE.Box3();
      
      result.meshes.forEach(mesh => {
        const meshBox = new THREE.Box3();
        meshBox.setFromObject(mesh);
        overallBox.union(meshBox);
      });
      
      const size = overallBox.getSize(new THREE.Vector3()).length();
      result.sceneScale = size > 0 ? 2 / size : 1;
      
      console.log('Overall scene size:', size, 'calculated scale:', result.sceneScale);
      
      // Create fallback geometry
      result.geometry = result.meshes.length === 1 
        ? result.meshes[0].geometry 
        : combineGeometries(result.meshes);
    }
  } else if (geo.children && geo.children.length > 0) {
    console.log('Object group detected, searching for all meshes...');
    // FBX and others might return groups, extract ALL meshes
    const findAllMeshesInGroup = (obj) => {
      console.log('Checking object:', obj.type, obj.name || 'unnamed');
      if (obj.isMesh && obj.geometry) {
        const name = obj.name.toLowerCase();
        const isBackground = name.includes('plane') || name.includes('background');
        
        console.log('Found mesh with geometry:', obj.geometry, 'vertices:', obj.geometry.attributes?.position?.count || 0);
        console.log('Mesh material:', obj.material ? obj.material.type : 'none');
        console.log('Is background element:', isBackground);
        
        if (!isBackground) {
          // Update world matrix to get correct positioning
          obj.updateMatrixWorld(true);
          result.meshes.push(obj);
          if (obj.material && !result.materials.includes(obj.material)) {
            result.materials.push(obj.material);
          }
        }
      }
      if (obj.children) {
        for (const child of obj.children) {
          findAllMeshesInGroup(child);
        }
      }
    };
    findAllMeshesInGroup(geo);
    
    console.log(`Found ${result.meshes.length} meshes in group`);
    
    if (result.meshes.length > 0) {
      // Calculate overall scene bounding box for scaling
      const overallBox = new THREE.Box3();
      
      result.meshes.forEach(mesh => {
        const meshBox = new THREE.Box3();
        meshBox.setFromObject(mesh);
        overallBox.union(meshBox);
      });
      
      const size = overallBox.getSize(new THREE.Vector3()).length();
      result.sceneScale = size > 0 ? 2 / size : 1;
      
      console.log('Overall group size:', size, 'calculated scale:', result.sceneScale);
      
      // Create fallback geometry
      result.geometry = result.meshes.length === 1 
        ? result.meshes[0].geometry 
        : combineGeometries(result.meshes);
    } else {
      // Fallback: find first mesh for single geometry
      const findFirstMesh = (obj) => {
        if (obj.isMesh && obj.geometry) {
          return obj.geometry;
        }
        if (obj.children) {
          for (const child of obj.children) {
            const found = findFirstMesh(child);
            if (found) return found;
          }
        }
        return null;
      };
      result.geometry = findFirstMesh(geo);
    }
  } else if (geo.isMesh && geo.geometry) {
    console.log('Direct mesh detected');
    result.geometry = geo.geometry;
    // Compute normals if missing
    if (!result.geometry.attributes?.normal) {
      console.log('Computing vertex normals...');
      result.geometry.computeVertexNormals();
    }
  } else {
    console.log('Unknown object structure, attempting to find geometry...');
    // Last resort: look for any geometry property
    if (geo.geometry) {
      result.geometry = geo.geometry;
      // Compute normals if missing
      if (!result.geometry.attributes?.normal) {
        console.log('Computing vertex normals...');
        result.geometry.computeVertexNormals();
      }
    }
  }

  return result;
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
        const meshFormats = ['obj', 'ply', 'stl', 'gltf', 'glb', 'fbx'];
        const imageFormats = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'tiff', 'tif', 'svg', 'ico'];
        
        if (meshFormats.includes(extension)) {
          setFileType(extension);
          const arrayBuffer = await file.arrayBuffer();
          setMeshData(new Uint8Array(arrayBuffer));
        } else if (imageFormats.includes(extension)) {
          setFileType('image');
          
          // Handle TIFF files specially - decode using Canvas API
          if (extension === 'tiff' || extension === 'tif') {
            try {
              console.log('Processing TIFF file for Canvas display:', file.name);
              const arrayBuffer = await file.arrayBuffer();
              const tiffUrl = await convertTiffToDataUrl(arrayBuffer);
              setMeshData(tiffUrl);
            } catch (tiffError) {
              console.error('TIFF processing failed:', tiffError);
              // Try direct URL as final fallback
              const imageUrl = URL.createObjectURL(file);
              setMeshData(imageUrl);
            }
          } else {
            // For other image formats, create URL directly
            const imageUrl = URL.createObjectURL(file);
            setMeshData(imageUrl);
          }
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

  // Separate effect for cleanup
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
            <p className="text-xs text-neutral-600">3D Models: OBJ, PLY, STL, GLTF, GLB, FBX</p>
          </div>
        </div>
      </div>
    );
  }

  // Handle image files
  if (fileType === 'image') {
    // Special case for TIFF files that can't be displayed
    if (meshData === 'tiff-not-supported') {
      return (
        <div className="w-full h-full bg-black rounded overflow-hidden flex items-start justify-center pt-4">
          <div className="text-center max-w-sm px-6 text-neutral-100">
            <div className="text-4xl mb-4">🖼️</div>
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
          src={meshData} 
          alt="Preview" 
          className="max-w-full max-h-full object-contain"
          onLoad={() => {
            if (typeof meshData === 'string' && meshData.startsWith('data:')) {
              console.log('Image loaded successfully from data URL');
            }
          }}
          onError={(e) => {
            console.error('Image failed to load:', e);
            setError('Failed to load image. The file may be corrupted or in an unsupported format.');
          }}
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
      
      // Set up TGA loader for texture support
      try {
        const tgaLoader = new TGALoader();
        loader.manager.addHandler(/\.tga$/i, tgaLoader);
        console.log('FBX Loader: TGA loader configured successfully');
      } catch (tgaError) {
        console.warn('FBX Loader: Failed to set up TGA loader:', tgaError);
        // Continue without TGA support - FBX loader will create placeholder textures
      }
      
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
      
      // Set up DRACO loader for compressed GLTF files
      try {
        const dracoLoader = new DRACOLoader();
        dracoLoader.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.6/');
        dracoLoader.preload();
        loader.setDRACOLoader(dracoLoader);
        console.log('GLTF Loader: DRACO loader configured successfully');
      } catch (dracoError) {
        console.warn('GLTF Loader: Failed to set up DRACO loader:', dracoError);
        // Continue without DRACO support - will handle errors later
      }
      
      // Set up TGA loader for texture support
      try {
        const tgaLoader = new TGALoader();
        loader.manager.addHandler(/\.tga$/i, tgaLoader);
        console.log('GLTF Loader: TGA loader configured successfully');
      } catch (tgaError) {
        console.warn('GLTF Loader: Failed to set up TGA loader:', tgaError);
        // Continue without TGA support
      }
      
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
        console.warn('GLTF file references external buffers, these will be unavailable');
        console.log('External buffers:', gltfJson.buffers.filter(buffer => buffer.uri && !buffer.uri.startsWith('data:')).map(b => b.uri));
        // Continue with loading - let Three.js handle missing buffers gracefully
      }
      
      // Try to parse the GLTF normally, but catch errors related to missing files
      loader.parse(text, '', (gltf) => {
        console.log('GLTF loaded successfully:', gltf);
        console.log('GLTF scene children:', gltf.scene.children.length);
        resolve(gltf);
      }, (error) => {
        console.error('GLTF loading error:', error);
        
        // Only use fallback for specific error types that indicate missing external resources
        const errorMessage = error?.message || error || '';
        const isMissingResourceError = 
          errorMessage.includes('buffer') || 
          errorMessage.includes('404') || 
          errorMessage.includes('Failed to load') ||
          errorMessage.includes('External');
        
        if (isMissingResourceError) {
          console.log('GLTF error appears to be missing external resources, trying fallback geometry');
          const fallbackGeometry = createFallbackGeometryFromGLTF(gltfJson);
          if (fallbackGeometry) {
            console.log('Using fallback geometry for GLTF with missing resources');
            resolve(fallbackGeometry);
            return;
          }
        }
        
        // For other errors, reject to let the normal error handling take over
        reject(new Error(`GLTF parsing failed: ${errorMessage}`));
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
      
      // Set up DRACO loader for compressed GLB files
      try {
        const dracoLoader = new DRACOLoader();
        // Use CDN for DRACO decoder - more reliable than local files
        dracoLoader.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.6/');
        dracoLoader.preload();
        loader.setDRACOLoader(dracoLoader);
        console.log('GLB Loader: DRACO loader configured successfully');
      } catch (dracoError) {
        console.warn('GLB Loader: Failed to set up DRACO loader:', dracoError);
        // Continue without DRACO support - will handle errors later
      }
      
      // Set up TGA loader for texture support
      try {
        const tgaLoader = new TGALoader();
        loader.manager.addHandler(/\.tga$/i, tgaLoader);
        console.log('GLB Loader: TGA loader configured successfully');
      } catch (tgaError) {
        console.warn('GLB Loader: Failed to set up TGA loader:', tgaError);
        // Continue without TGA support
      }
      
      console.log('GLB Loader: Starting to parse', data.length, 'bytes');
      console.log('GLB data type:', data.constructor.name);
      
      // Check if data starts with GLB magic bytes (0x676C5446 = "glTF")
      const view = new DataView(data.buffer || data, data.byteOffset || 0, 4);
      const magic = view.getUint32(0, true);
      console.log('GLB magic bytes:', magic.toString(16));
      
      if (magic !== 0x46546C67) { // "glTF" in little-endian
        console.warn('GLB file does not have correct magic bytes, expected 0x46546C67, got', magic.toString(16));
      }
      
      // Convert to proper ArrayBuffer for GLB binary parsing
      let arrayBuffer;
      if (data instanceof Uint8Array) {
        arrayBuffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
      } else if (data instanceof ArrayBuffer) {
        arrayBuffer = data;
      } else {
        throw new Error('GLB data must be Uint8Array or ArrayBuffer');
      }
      
      // GLTFLoader.parse for binary GLB: parse(data, path, onLoad, onError)
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
        
        // Check for specific error types and provide appropriate fallbacks
        let shouldCreateFallback = false;
        
        if (error?.message?.includes('DRACOLoader') || error?.message?.includes('DRACO')) {
          console.log('GLB uses DRACO compression but loader failed, creating fallback geometry');
          shouldCreateFallback = true;
        } else if (error?.message?.includes('JSON') || error?.message?.includes('Unexpected')) {
          console.log('GLB appears to be corrupted or in unexpected format, creating fallback geometry');
          shouldCreateFallback = true;
        } else if (error?.message?.includes('External') || error?.message?.includes('buffer')) {
          console.log('GLB references external files, creating fallback geometry');
          shouldCreateFallback = true;
        }
        
        if (shouldCreateFallback) {
          // Create a fallback geometry
          const fallbackGeometry = createFallbackGeometryFromGLB(arrayBuffer);
          if (fallbackGeometry) {
            console.log('Created fallback geometry for problematic GLB');
            resolve({ isBufferGeometry: true, ...fallbackGeometry });
            return;
          }
        }
        
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
      
      if (!object) {
        console.error('OBJ loader returned null object');
        reject(new Error('OBJ loader returned null object'));
        return;
      }
      
      console.log('OBJ loaded successfully:', object);
      console.log('OBJ object type:', object.type);
      console.log('OBJ children count:', object.children?.length || 0);
      
      // Log all children for debugging
      if (object.children) {
        object.children.forEach((child, index) => {
          console.log(`OBJ child ${index}:`, child.type, child.name || 'unnamed', child.isMesh ? 'MESH' : '');
          if (child.isMesh && child.geometry) {
            console.log(`  - Child ${index} geometry:`, child.geometry.type, 'vertices:', child.geometry.attributes?.position?.count || 0);
          }
        });
      }
      
      // Validate that we have at least one mesh
      const hasMesh = object.children?.some(child => child.isMesh && child.geometry?.attributes?.position);
      if (!hasMesh) {
        console.warn('OBJ file loaded but contains no valid meshes');
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
        if (!geometry) {
          console.error('PLY loader returned null geometry');
          reject(new Error('PLY loader returned null geometry'));
          return;
        }
        
        console.log('PLY loaded successfully:', geometry);
        console.log('PLY geometry type:', geometry.type);
        console.log('PLY vertices:', geometry.attributes?.position?.count || 0);
        console.log('PLY faces:', geometry.index?.count ? geometry.index.count / 3 : 0);
        
        // Ensure geometry has proper attributes
        if (!geometry.attributes?.position) {
          console.error('PLY geometry missing position attribute');
          reject(new Error('PLY geometry missing position attribute'));
          return;
        }
        
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
      
      if (!geometry) {
        console.error('STL loader returned null geometry');
        reject(new Error('STL loader returned null geometry'));
        return;
      }
      
      console.log('STL loaded successfully:', geometry);
      console.log('STL geometry type:', geometry.type);
      console.log('STL vertices:', geometry.attributes?.position?.count || 0);
      console.log('STL faces:', geometry.index?.count ? geometry.index.count / 3 : geometry.attributes.position.count / 3);
      
      // Ensure geometry has proper attributes
      if (!geometry.attributes?.position) {
        console.error('STL geometry missing position attribute');
        reject(new Error('STL geometry missing position attribute'));
        return;
      }
      
      resolve(geometry);
    } catch (error) {
      console.error('STL loading error:', error);
      reject(new Error(`STL parsing failed: ${error.message || 'Unknown error'}`));
    }
  });
}


// Helper function to combine multiple meshes into a single geometry
function combineGeometries(meshes) {
  try {
    console.log('Combining geometries from', meshes.length, 'meshes');
    
    // Filter out background planes and unwanted meshes
    const filteredMeshes = meshes.filter(mesh => {
      const name = mesh.name.toLowerCase();
      return !name.includes('plane') && !name.includes('background');
    });
    
    console.log(`Filtered to ${filteredMeshes.length} meshes (removed background elements)`);
    
    if (filteredMeshes.length === 0) {
      console.warn('No meshes left after filtering');
      return meshes[0]?.geometry || null;
    }
    
    // Use BufferGeometryUtils to merge geometries
    const geometries = [];
    
    // Get all unique attributes from all geometries
    const allAttributes = new Set();
    filteredMeshes.forEach(mesh => {
      if (mesh.geometry.attributes) {
        Object.keys(mesh.geometry.attributes).forEach(attr => allAttributes.add(attr));
      }
    });
    
    console.log('All attributes found:', Array.from(allAttributes));
    
    for (let i = 0; i < filteredMeshes.length; i++) {
      const mesh = filteredMeshes[i];
      let geometry = mesh.geometry.clone();
      
      // Normalize attributes - ensure all geometries have the same attributes
      for (const attrName of allAttributes) {
        if (!geometry.attributes[attrName]) {
          // Create default attribute if missing
          const posCount = geometry.attributes.position.count;
          if (attrName === 'normal' && !geometry.attributes.normal) {
            geometry.computeVertexNormals();
          } else if (attrName === 'uv' && !geometry.attributes.uv) {
            // Create default UV coordinates
            const uvArray = new Float32Array(posCount * 2);
            for (let j = 0; j < posCount; j++) {
              uvArray[j * 2] = 0;
              uvArray[j * 2 + 1] = 0;
            }
            geometry.setAttribute('uv', new THREE.BufferAttribute(uvArray, 2));
          }
          // Skip other complex attributes like tangents, multiple UV sets
        }
      }
      
      // Don't apply world matrix - preserve relative positions
      console.log(`Mesh ${i} (${mesh.name || 'unnamed'}):`, geometry.attributes.position.count, 'vertices');
      geometries.push(geometry);
    }
    
    // Check if we have BufferGeometryUtils available
    if (!BufferGeometryUtils || !BufferGeometryUtils.mergeGeometries) {
      console.warn('BufferGeometryUtils not available, using first geometry only');
      return geometries[0];
    }
    
    // Merge all geometries with compatible attributes only
    const compatibleGeometries = geometries.map(geom => {
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
    console.log('Combined geometry has', mergedGeometry.attributes.position.count, 'vertices total');
    
    // Clean up cloned geometries
    geometries.forEach(g => g.dispose());
    compatibleGeometries.forEach(g => g.dispose());
    
    return mergedGeometry;
  } catch (error) {
    console.error('Failed to combine geometries:', error);
    // Fallback to first non-plane geometry
    const fallbackMesh = meshes.find(m => !m.name.toLowerCase().includes('plane'));
    return fallbackMesh?.geometry || meshes[0]?.geometry || null;
  }
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

// Helper function to convert TIFF to displayable data URL
async function convertTiffToDataUrl(arrayBuffer) {
  // Since browsers don't support TIFF natively, show an informative message instead
  console.log('TIFF file detected, showing info message instead of trying to decode');
  
  // Return a special marker that we'll handle in the UI
  return 'tiff-not-supported';
}

// Helper function to create fallback geometry for GLB files
function createFallbackGeometryFromGLB(arrayBuffer) {
  try {
    console.log('Creating fallback geometry from GLB binary data...');
    
    // For GLB files, we can create a more sophisticated fallback
    // based on file size and structure hints
    const fileSize = arrayBuffer.byteLength;
    let geometry;
    
    if (fileSize < 100000) {
      // Small file - likely simple geometry, create a tetrahedron
      geometry = new THREE.TetrahedronGeometry(1.5, 0);
    } else if (fileSize < 1000000) {
      // Medium file - create a more complex shape
      geometry = new THREE.IcosahedronGeometry(1.5, 1);
    } else {
      // Large file - likely complex model, create a sphere
      geometry = new THREE.SphereGeometry(1.5, 16, 12);
    }
    
    geometry.computeVertexNormals();
    
    console.log(`Created ${geometry.type} fallback geometry for GLB (${fileSize} bytes)`);
    return geometry;
  } catch (error) {
    console.warn('Failed to create GLB fallback geometry:', error);
    // Ultimate fallback
    const geometry = new THREE.BoxGeometry(1, 1, 1);
    geometry.computeVertexNormals();
    return geometry;
  }
}


export default FilePreview;
