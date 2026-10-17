type ImageFormat = 'png' | 'jpeg' | 'webp' | 'gif' | 'bmp' | 'tga' | 'tiff' | 'ico' | 'svg' | 'heic' | 'avif';
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
  /** Scale images down to fit this many pixels on their longest side (0 or absent: keep the size). */
  maxSize?: number;
  /**
   * For a .gltf: files it may refer to (buffers, textures), by file name, packed
   * into the GLB. Blobs (Files) are read only if the glTF names them.
   */
  resources?: { name: string; data: Uint8Array | Blob }[];
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

/** An OBJ, STL or PLY ready to display: triangles, lines, or with neither, its vertices as points. */
export interface DecodedMesh {
  /** xyz per vertex */
  positions: Float32Array;
  /** xyz per vertex: the file's, or smooth ones computed from the triangles (absent for points and lines) */
  normals?: Float32Array;
  /** linear rgb in 0..1 per vertex (files store sRGB), when the file has colours */
  colors?: Float32Array;
  /** three per triangle */
  indices: Uint32Array;
  /** two per line segment (OBJ "l") */
  lines?: Uint32Array;
}

export interface BuildInfo {
  version: string;
  timestamp: string;
  /** SHA-256 of the core sources the module was built from (core/cmake/SourceHash.cmake) */
  source: string;
}

/**
 * The Pixie Juice engine, running in Web Workers. With `transfer` set, `data`'s
 * buffer moves to the worker instead of being copied, and is no longer usable.
 * Aborting `signal` stops the request mid-task.
 */
export interface PixieApi {
  /** How many files it processes at once. */
  readonly concurrency: number;
  info(): Promise<BuildInfo>;
  /** "image:png", "mesh:obj", ... or "unknown". */
  detect(data: Uint8Array): Promise<string>;
  optimize(data: Uint8Array, options?: CoreOptions, transfer?: boolean, signal?: AbortSignal): Promise<CoreResult>;
  convert(
    data: Uint8Array,
    target: ConvertTarget,
    options?: CoreOptions,
    transfer?: boolean,
    signal?: AbortSignal,
  ): Promise<CoreResult>;
  decodeImage(data: Uint8Array, transfer?: boolean): Promise<DecodedImage>;
  decodeMesh(data: Uint8Array, transfer?: boolean): Promise<DecodedMesh>;
}
