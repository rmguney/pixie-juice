export type ImageFormat = 'png' | 'jpeg' | 'webp' | 'gif' | 'bmp' | 'tga' | 'tiff' | 'ico' | 'svg';
export type MeshFormat = 'obj' | 'stl' | 'ply' | 'gltf' | 'glb' | 'fbx';
export type FormatName = ImageFormat | MeshFormat;

/** Raster formats the core can convert to. */
export type ConvertTarget = 'png' | 'jpeg' | 'webp' | 'gif' | 'bmp' | 'tga' | 'ico' | 'tiff';

export interface CoreOptions {
  /** 1-100: lossy encoder quality and the fidelity check's error budget. */
  quality?: number;
  /** Never change pixels or geometry. */
  lossless?: boolean;
  /** Keep EXIF/XMP/comments and other non-visual metadata. */
  preserveMetadata?: boolean;
  /** Share of mesh triangles to keep, (0, 1]. */
  meshRatio?: number;
}

export interface CoreResult {
  data: Uint8Array;
  format: FormatName;
  /** False when the input came back unchanged. */
  changed: boolean;
}

export interface DecodedImage {
  width: number;
  height: number;
  rgba: Uint8ClampedArray;
}

/** Triangles of an OBJ, STL or PLY for display. */
export interface DecodedMesh {
  /** xyz per vertex */
  positions: Float32Array;
  /** xyz per vertex, when the file has normals */
  normals?: Float32Array;
  /** rgb in 0..1 per vertex as stored (normally sRGB), when the file has colours */
  colors?: Float32Array;
  /** three per triangle */
  indices: Uint32Array;
}

export interface BuildInfo {
  version: string;
  commit: string;
  timestamp: string;
}

/** The Pixie Juice engine, running in a Web Worker. */
export interface PixieApi {
  info(): Promise<BuildInfo>;
  /** "image:png", "mesh:obj", ... or "unknown". */
  detect(data: Uint8Array): Promise<string>;
  optimize(data: Uint8Array, options?: CoreOptions): Promise<CoreResult>;
  convert(data: Uint8Array, target: ConvertTarget, options?: CoreOptions): Promise<CoreResult>;
  decodeImage(data: Uint8Array): Promise<DecodedImage>;
  decodeMesh(data: Uint8Array): Promise<DecodedMesh>;
}
