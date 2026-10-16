// Parses glTF/GLB and FBX for the 3D preview with three.js's loaders, off the
// main thread so a large model doesn't freeze the page. The scene goes back as
// plain data with its buffers and image bitmaps (see sceneTransfer.ts).

// The loaders are imported statically: a dynamic import() from a module worker
// made WebKit evaluate a second copy of three.js, whose objects then failed
// instanceof checks. The worker itself only starts when a model needs it.

import { ImageBitmapLoader, Texture, TextureLoader } from 'three';
import { FBXLoader } from 'three/addons/loaders/FBXLoader.js';
import { parseGltf } from './gltf';
import { serializeScene } from './sceneTransfer';
import type { ModelRequest, ModelResponse } from './modelProtocol';

// FBXLoader reads window.URL (textures embedded in binary files) and the window
// size (cameras), which a worker doesn't have.
(globalThis as { window?: unknown }).window ??= { URL, innerWidth: 1, innerHeight: 1 };

/** Texture loads of the file being parsed: FBXLoader returns its scene before they finish. */
let textureLoads: Promise<void>[] = [];

// FBXLoader loads its textures with TextureLoader, which needs the DOM. Here
// they are decoded to ImageBitmaps instead, flipped later like any texture
// whose flipY is set (see serializeScene). Only textures embedded in the file
// (blob: and data: URLs) are read: fetching the paths a file names would leak
// that it was opened, to this site or another, like the glTF check in gltf.ts.
TextureLoader.prototype.load = function (url, onLoad, onProgress, onError) {
  const texture = new Texture();
  if (!url.startsWith('blob:') && !url.startsWith('data:')) {
    onError?.(new Error(`not embedded in the file: ${url}`));
    return texture;
  }
  textureLoads.push(
    new Promise<void>((settle) => {
      const done = () => {
        if (url.startsWith('blob:')) URL.revokeObjectURL(url); // FBXLoader never revokes its own
        settle();
      };
      new ImageBitmapLoader(this.manager)
        .setOptions({ imageOrientation: 'none', premultiplyAlpha: 'none' })
        .setCrossOrigin(this.crossOrigin)
        .load(
          url,
          (bitmap) => {
            texture.image = bitmap;
            texture.needsUpdate = true;
            onLoad?.(texture);
            done();
          },
          onProgress,
          (error) => {
            onError?.(error);
            done();
          },
        );
    }),
  );
  return texture;
};

async function parse({ type, data }: ModelRequest) {
  textureLoads = [];
  // FBXLoader starts its texture loads while parsing (synchronously), so the list is this file's.
  const object = type === 'fbx' ? new FBXLoader().parse(data, '') : await parseGltf(data);
  await Promise.all(textureLoads);
  return object;
}

self.onmessage = async (event: MessageEvent<ModelRequest>) => {
  const { id } = event.data;
  let response: ModelResponse;
  let transfer: Transferable[] = [];
  try {
    const serialized = await serializeScene(await parse(event.data));
    response = { id, scene: serialized.scene };
    transfer = serialized.transfer;
  } catch (error) {
    response = { id, error: error instanceof Error ? error.message : String(error) };
  }
  self.postMessage(response, { transfer });
};
