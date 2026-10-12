import type { MeshFormat } from '../core/types';

type FileKind = 'image' | 'mesh';

export const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'tga', 'tif', 'tiff', 'ico', 'svg'] as const;
export const MESH_EXTENSIONS = ['obj', 'stl', 'ply', 'gltf', 'glb', 'fbx'] as const satisfies readonly MeshFormat[];

export function extensionOf(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot < 0 ? '' : name.slice(dot + 1).toLowerCase();
}

/** Mesh extensions are the engine's mesh format names. */
export function isMeshExtension(extension: string): extension is MeshFormat {
  return (MESH_EXTENSIONS as readonly string[]).includes(extension);
}

/** Image or mesh by extension; null for anything Pixie Juice doesn't handle. */
export function fileKind(name: string): FileKind | null {
  const extension = extensionOf(name);
  if ((IMAGE_EXTENSIONS as readonly string[]).includes(extension)) return 'image';
  if (isMeshExtension(extension)) return 'mesh';
  return null;
}

/** "photo.jpg" -> "photo_juice.webp" */
export function outputName(original: string, extension: string): string {
  const dot = original.lastIndexOf('.');
  const base = dot < 0 ? original : original.slice(0, dot);
  return `${base}_juice.${extension}`;
}
