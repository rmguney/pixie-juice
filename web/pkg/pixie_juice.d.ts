/* tslint:disable */
/* eslint-disable */
/**
 * WASM-compatible wrapper for auto optimization
 */
export function pixie_optimize_auto(data: Uint8Array, quality: number): Uint8Array;
/**
 * WASM-compatible wrapper for image optimization
 */
export function pixie_optimize_image(data: Uint8Array, quality: number): Uint8Array;
/**
 * WASM-compatible wrapper for mesh optimization
 */
export function pixie_optimize_mesh(data: Uint8Array): Uint8Array;
/**
 * WASM-compatible function to get memory target for verification
 */
export function pixie_get_memory_target_mb(): number;
/**
 * WASM-compatible performance statistics getter
 */
export function pixie_get_performance_stats(): any;
/**
 * WASM-compatible performance reset
 */
export function pixie_reset_performance_stats(): void;
/**
 * WASM-compatible performance compliance checker
 */
export function pixie_check_performance_compliance(): boolean;
export function run_wasm_benchmarks(): any;
export function init(): void;
export function optimize_image(data: Uint8Array, quality: number): Uint8Array;
export function optimize_mesh(data: Uint8Array, target_ratio?: number | null): Uint8Array;
export function optimize_auto(data: Uint8Array, quality: number): Uint8Array;
export function version(): string;
export function build_timestamp(): string;
export function detect_format(data: Uint8Array): string;
export function get_performance_metrics(): any;
export function reset_performance_stats(): void;
export function check_performance_compliance(): boolean;
export function optimize_png(data: Uint8Array, quality: number): Uint8Array;
export function optimize_jpeg(data: Uint8Array, quality: number): Uint8Array;
export function optimize_webp(data: Uint8Array, quality: number): Uint8Array;
export function optimize_gif(data: Uint8Array, quality: number): Uint8Array;
export function optimize_ico(data: Uint8Array, quality: number): Uint8Array;
export function optimize_tga(data: Uint8Array, quality: number): Uint8Array;
export function is_webp(data: Uint8Array): boolean;
export function is_gif(data: Uint8Array): boolean;
export function is_ico(data: Uint8Array): boolean;
export function is_tga(data: Uint8Array): boolean;
export function convert_to_webp(data: Uint8Array, quality: number): Uint8Array;
export function convert_to_png(data: Uint8Array): Uint8Array;
export function convert_to_jpeg(data: Uint8Array, quality: number): Uint8Array;
export function convert_to_bmp(data: Uint8Array): Uint8Array;
export function convert_to_gif(data: Uint8Array, quality: number): Uint8Array;
export function convert_to_ico(data: Uint8Array, quality: number): Uint8Array;
export function convert_to_tiff(data: Uint8Array, quality: number): Uint8Array;
export function strip_tiff_metadata_simd(data: Uint8Array, preserve_icc: boolean): Uint8Array;
export function convert_to_svg(data: Uint8Array, quality: number): Uint8Array;
export function convert_to_tga(data: Uint8Array, quality: number): Uint8Array;
export function set_lossless_mode(enabled: boolean): any;
export function set_preserve_metadata(enabled: boolean): any;
export function optimize_obj(data: Uint8Array, reduction_ratio: number): Uint8Array;
export function optimize_gltf(data: Uint8Array, reduction_ratio: number): Uint8Array;
export function optimize_stl(data: Uint8Array, reduction_ratio: number): Uint8Array;
export function optimize_fbx(data: Uint8Array, reduction_ratio: number): Uint8Array;
export function optimize_ply(data: Uint8Array, reduction_ratio: number): Uint8Array;
export function is_obj(data: Uint8Array): boolean;
export function is_gltf(data: Uint8Array): boolean;
export function is_stl(data: Uint8Array): boolean;
export function is_fbx(data: Uint8Array): boolean;
export function is_ply(data: Uint8Array): boolean;
/**
 * Color space enumeration
 */
export enum ColorSpace {
  RGB = 0,
  RGBA = 1,
  Grayscale = 2,
  GrayscaleAlpha = 3,
  CMYK = 4,
  YUV = 5,
  HSV = 6,
  LAB = 7,
}
/**
 * Mesh simplification algorithms
 */
export enum SimplificationAlgorithm {
  /**
   * Quadric Error Metrics - highest quality
   */
  QuadricErrorMetrics = 0,
  /**
   * Edge collapse - good balance of speed and quality
   */
  EdgeCollapse = 1,
  /**
   * Vertex clustering - fastest but lower quality
   */
  VertexClustering = 2,
}
/**
 * Configuration for image optimization operations
 */
export class ImageOptConfig {
  private constructor();
  free(): void;
  quality: number;
  lossless: boolean;
  preserve_metadata: boolean;
  optimize_colors: boolean;
  get max_colors(): number | undefined;
  set max_colors(value: number | null | undefined);
  use_c_hotspots: boolean;
  enable_simd: boolean;
  get compression_level(): number | undefined;
  set compression_level(value: number | null | undefined);
  fast_mode: boolean;
  preserve_alpha: boolean;
  get max_width(): number | undefined;
  set max_width(value: number | null | undefined);
  get max_height(): number | undefined;
  set max_height(value: number | null | undefined);
  get target_reduction(): number | undefined;
  set target_reduction(value: number | null | undefined);
}
/**
 * Configuration for mesh optimization operations
 */
export class MeshOptConfig {
  private constructor();
  free(): void;
  target_ratio: number;
  preserve_topology: boolean;
  weld_vertices: boolean;
  vertex_tolerance: number;
  simplification_algorithm: SimplificationAlgorithm;
  use_c_hotspots: boolean;
  generate_normals: boolean;
  optimize_vertex_cache: boolean;
  preserve_uv_seams: boolean;
  preserve_boundaries: boolean;
}
/**
 * Main configuration structure for Pixie Juice WASM
 */
export class PixieConfig {
  free(): void;
  constructor();
  /**
   * Convert to internal ImageOptConfig
   */
  to_image_config(): ImageOptConfig;
  /**
   * Convert to internal MeshOptConfig
   */
  to_mesh_config(): MeshOptConfig;
  use_c_hotspots: boolean;
  quality: number;
  enable_threading: boolean;
}
