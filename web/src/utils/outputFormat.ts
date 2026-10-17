import type { FormatName } from '../core/types';

interface OutputFormat {
  /** File extension without the dot, e.g. "png" */
  extension: string;
  mimeType: string;
}

const FORMATS: Record<FormatName, OutputFormat> = {
  png: { extension: 'png', mimeType: 'image/png' },
  jpeg: { extension: 'jpg', mimeType: 'image/jpeg' },
  webp: { extension: 'webp', mimeType: 'image/webp' },
  gif: { extension: 'gif', mimeType: 'image/gif' },
  bmp: { extension: 'bmp', mimeType: 'image/bmp' },
  tga: { extension: 'tga', mimeType: 'image/x-tga' },
  tiff: { extension: 'tiff', mimeType: 'image/tiff' },
  ico: { extension: 'ico', mimeType: 'image/x-icon' },
  svg: { extension: 'svg', mimeType: 'image/svg+xml' },
  heic: { extension: 'heic', mimeType: 'image/heic' },
  avif: { extension: 'avif', mimeType: 'image/avif' },
  obj: { extension: 'obj', mimeType: 'model/obj' },
  stl: { extension: 'stl', mimeType: 'model/stl' },
  ply: { extension: 'ply', mimeType: 'application/octet-stream' },
  gltf: { extension: 'gltf', mimeType: 'model/gltf+json' },
  glb: { extension: 'glb', mimeType: 'model/gltf-binary' },
  fbx: { extension: 'fbx', mimeType: 'application/octet-stream' },
};

/** Extension and MIME type for a format the engine reported for its output. */
export function outputFormat(format: FormatName): OutputFormat {
  return FORMATS[format];
}
