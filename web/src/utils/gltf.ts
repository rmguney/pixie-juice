// glTF/GLB for the preview, loaded on first use: three.js's loader with the
// decoders for compressed geometry. Draco's decoder ships with three.js and is
// served by this app; it is fetched only when a file uses it.

import { LoadingManager, type Object3D } from 'three';
import { DRACOLoader } from 'three/addons/loaders/DRACOLoader.js';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { MeshoptDecoder } from 'three/addons/libs/meshopt_decoder.module.js';
import dracoWasmUrl from 'three/examples/jsm/libs/draco/gltf/draco_decoder.wasm?url';
import dracoWrapperUrl from 'three/examples/jsm/libs/draco/gltf/draco_wasm_wrapper.js?url';

let loader: GLTFLoader | null = null;

function gltfLoader(): GLTFLoader {
  if (!loader) {
    const files: Record<string, string> = {
      'draco_wasm_wrapper.js': dracoWrapperUrl,
      'draco_decoder.wasm': dracoWasmUrl,
    };
    const manager = new LoadingManager();
    manager.setURLModifier((url) => files[url] ?? url);
    loader = new GLTFLoader()
      .setDRACOLoader(new DRACOLoader(manager).setDecoderPath(''))
      .setMeshoptDecoder(MeshoptDecoder);
  }
  return loader;
}

interface GltfJson {
  buffers?: { uri?: string }[];
  images?: { uri?: string }[];
}

/** The glTF JSON of a .gltf file, or of a GLB's JSON chunk. */
function gltfJson(buffer: ArrayBuffer): { glb: boolean; json: GltfJson } {
  const bytes = new Uint8Array(buffer);
  const view = new DataView(buffer);
  const glb = bytes.length >= 20 && view.getUint32(0, true) === 0x46546c67; // "glTF"
  const text = glb ? bytes.subarray(20, 20 + view.getUint32(12, true)) : bytes;
  return { glb, json: JSON.parse(new TextDecoder().decode(text)) };
}

export async function parseGltf(buffer: ArrayBuffer): Promise<Object3D> {
  // Only embedded resources: fetching the URIs a file names would leak that it was opened.
  const { glb, json } = gltfJson(buffer);
  const external = [...(json.buffers ?? []), ...(json.images ?? [])].some((item) => item.uri && !item.uri.startsWith('data:'));
  if (external) throw new Error('the glTF references external files, which are not part of the upload');
  // A .gltf goes to the loader already parsed, so a large one isn't decoded twice
  // (parse() takes a JSON object too, though its type declaration leaves that out).
  return (await gltfLoader().parseAsync(glb ? buffer : (json as unknown as string), '')).scene;
}
