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

export async function parseGltf(buffer: ArrayBuffer): Promise<Object3D> {
  return (await gltfLoader().parseAsync(buffer, '')).scene;
}
