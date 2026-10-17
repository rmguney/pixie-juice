// glTF/GLB for the preview, loaded on first use: three.js's loader with the
// decoders for compressed geometry and KTX2 textures. Draco's decoder and the
// Basis transcoder ship with three.js and are served by this app; each is
// fetched only when a file uses it.

import { type AnimationClip, LoadingManager, type Object3D } from 'three';
import { DRACOLoader } from 'three/addons/loaders/DRACOLoader.js';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { KTX2Loader } from 'three/addons/loaders/KTX2Loader.js';
import { MeshoptDecoder } from 'three/addons/libs/meshopt_decoder.module.js';
import basisWasmUrl from 'three/examples/jsm/libs/basis/basis_transcoder.wasm?url';
import basisWrapperUrl from 'three/examples/jsm/libs/basis/basis_transcoder.js?url';
import dracoWasmUrl from 'three/examples/jsm/libs/draco/gltf/draco_decoder.wasm?url';
import dracoWrapperUrl from 'three/examples/jsm/libs/draco/gltf/draco_wasm_wrapper.js?url';
import { externalUris, gltfJson, uriFileName } from './companions';

/** The decoders' own files, by the names the loaders ask for. */
const DECODER_FILES: Record<string, string> = {
  'draco_wasm_wrapper.js': dracoWrapperUrl,
  'draco_decoder.wasm': dracoWasmUrl,
  'basis_transcoder.js': basisWrapperUrl,
  'basis_transcoder.wasm': basisWasmUrl,
};

let decoders: { draco: DRACOLoader; ktx2: KTX2Loader } | null = null;

function sharedDecoders() {
  if (!decoders) {
    const manager = new LoadingManager();
    manager.setURLModifier((url) => DECODER_FILES[url] ?? url);
    const ktx2 = new KTX2Loader(manager).setTranscoderPath('');
    // No renderer here to ask which GPU formats it takes: KTX2 is transcoded to RGBA,
    // which every GPU takes (see sceneTransfer.ts).
    (ktx2 as unknown as { workerConfig: Record<string, boolean> }).workerConfig = {
      astcSupported: false,
      astcHDRSupported: false,
      etc1Supported: false,
      etc2Supported: false,
      dxtSupported: false,
      bptcSupported: false,
      pvrtcSupported: false,
    };
    decoders = { draco: new DRACOLoader(manager).setDecoderPath(''), ktx2 };
  }
  return decoders;
}

/**
 * Parses a glTF or GLB. The files it refers to come from `files` (the file
 * list), never from where its URIs point: fetching those would leak that the
 * file was opened, and they aren't part of the upload.
 */
export async function parseGltf(buffer: ArrayBuffer, files: File[] = []): Promise<{ scene: Object3D; animations: AnimationClip[] }> {
  const { glb, json } = gltfJson(new Uint8Array(buffer));
  const urls = new Map<string, string>();
  try {
    for (const uri of externalUris(json)) {
      const name = uriFileName(uri);
      const file = files.find((candidate) => candidate.name === name);
      if (!file) throw new Error(`the glTF needs ${name}; add it to the list with the .gltf`);
      urls.set(uri, URL.createObjectURL(file));
    }
    const manager = new LoadingManager();
    manager.setURLModifier((url) => urls.get(url) ?? url);
    const { draco, ktx2 } = sharedDecoders();
    const loader = new GLTFLoader(manager).setDRACOLoader(draco).setKTX2Loader(ktx2).setMeshoptDecoder(MeshoptDecoder);
    // A .gltf goes to the loader already parsed, so a large one isn't decoded twice
    // (parse() takes a JSON object too, though its type declaration leaves that out).
    const gltf = await loader.parseAsync(glb ? buffer : (json as unknown as string), '');
    return { scene: gltf.scene, animations: gltf.animations };
  } finally {
    for (const url of urls.values()) URL.revokeObjectURL(url);
  }
}
