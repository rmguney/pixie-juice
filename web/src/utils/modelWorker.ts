// Parses glTF/GLB and FBX for the 3D preview with three.js's loaders, off the
// main thread so a large model doesn't freeze the page. The scene goes back as
// plain data with its buffers and encoded images (see sceneTransfer.ts).

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

/** The image reads an FBX starts while it is parsed, and the object URLs they read. */
interface Reads {
  loads: Promise<void>[];
  urls: string[];
}
/** Set only while FBXLoader parses: it starts its image reads synchronously and doesn't wait for them. */
let reads: Reads | null = null;

// Texture images are read here as the encoded files they are and decoded on
// the page (see deserializeScene): ImageBitmaps decoded in a worker and
// transferred came out garbled in WebKit on Linux. Only images embedded in the
// file (blob: and data: URLs) are read: fetching the paths a file names would
// leak that it was opened, to this site or another, like the glTF check in gltf.ts.
function readImage(url: string, onLoad: (image: Blob) => void, onError?: (error: unknown) => void) {
  if (!url.startsWith('blob:') && !url.startsWith('data:')) {
    onError?.(new Error(`not embedded in the file: ${url}`));
    return;
  }
  const load = fetch(url)
    .then((response) => response.blob())
    .then(onLoad, (error: unknown) => onError?.(error));
  reads?.loads.push(load);
  if (url.startsWith('blob:')) reads?.urls.push(url);
}

// GLTFLoader reads images with ImageBitmapLoader (with TextureLoader where it
// distrusts ImageBitmap), FBXLoader with TextureLoader. Both need decoding the
// worker leaves to the page, so either gets the encoded image instead.
ImageBitmapLoader.prototype.load = function (url, onLoad, _onProgress, onError) {
  readImage(url, (image) => onLoad?.(image as unknown as ImageBitmap), onError);
};

TextureLoader.prototype.load = function (url, onLoad, _onProgress, onError) {
  const texture = new Texture();
  readImage(
    url,
    (image) => {
      texture.image = image;
      onLoad?.(texture);
    },
    onError,
  );
  return texture;
};

async function parse({ type, data }: ModelRequest) {
  if (type !== 'fbx') return parseGltf(data); // GLTFLoader waits for its images and revokes its URLs
  const own: Reads = { loads: [], urls: [] };
  reads = own;
  try {
    const object = new FBXLoader().parse(data, '');
    reads = null;
    await Promise.all(own.loads);
    return object;
  } finally {
    reads = null;
    for (const url of own.urls) URL.revokeObjectURL(url); // FBXLoader never revokes its own
  }
}

self.onmessage = async (event: MessageEvent<ModelRequest>) => {
  const { id } = event.data;
  let response: ModelResponse;
  let transfer: Transferable[] = [];
  try {
    const serialized = serializeScene(await parse(event.data));
    response = { id, scene: serialized.scene };
    transfer = serialized.transfer;
  } catch (error) {
    response = { id, error: error instanceof Error ? error.message : String(error) };
  }
  self.postMessage(response, { transfer });
};
