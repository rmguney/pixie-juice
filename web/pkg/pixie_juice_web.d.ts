/* tslint:disable */
/* eslint-disable */
export function main(): void;
export function supported_image_formats(): any[];
export function get_format_support_info(format: string): any;
export function supported_mesh_formats(): any[];
export class CanvasRenderer {
  free(): void;
  constructor(canvas: HTMLCanvasElement);
  /**
   * Load and render an image from a blob URL
   */
  load_and_render_image(blob_url: string): Promise<void>;
  /**
   * Clear the canvas
   */
  clear(): void;
}
export class FileHandler {
  private constructor();
  free(): void;
  /**
   * Read a File object from the browser into bytes
   */
  static read_file_as_bytes(file: File): Promise<Uint8Array>;
  /**
   * Create a download URL for processed file data
   */
  static create_download_url(data: Uint8Array, mime_type: string): string;
}
export class ImageOptimizer {
  free(): void;
  constructor();
  /**
   * Process an image file from browser File API using real optimization
   */
  process_image_file(file_data: Uint8Array, input_format: string, output_format: string, config: WasmOptConfig): Promise<Uint8Array>;
  /**
   * Get image information from file data using core
   */
  get_image_info(file_data: Uint8Array, format: string): any;
}
export class MeshOptimizer {
  free(): void;
  constructor();
  /**
   * Process a mesh file from browser File API using real optimization
   */
  process_mesh_file(file_data: Uint8Array, input_format: string, config: WasmOptConfig): Promise<Uint8Array>;
}
export class ProgressCallback {
  free(): void;
  constructor(callback: Function);
  update(progress: number, message: string): void;
}
export class WasmOptConfig {
  free(): void;
  constructor();
  quality: number;
  preserve_metadata: boolean;
  lossless: boolean;
  reduce_colors: boolean;
  target_reduction: number;
  preserve_alpha: boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmoptconfig_free: (a: number, b: number) => void;
  readonly wasmoptconfig_new: () => number;
  readonly wasmoptconfig_quality: (a: number) => number;
  readonly wasmoptconfig_set_quality: (a: number, b: number) => void;
  readonly wasmoptconfig_preserve_metadata: (a: number) => number;
  readonly wasmoptconfig_set_preserve_metadata: (a: number, b: number) => void;
  readonly wasmoptconfig_lossless: (a: number) => number;
  readonly wasmoptconfig_set_lossless: (a: number, b: number) => void;
  readonly wasmoptconfig_reduce_colors: (a: number) => number;
  readonly wasmoptconfig_set_reduce_colors: (a: number, b: number) => void;
  readonly wasmoptconfig_target_reduction: (a: number) => number;
  readonly wasmoptconfig_set_target_reduction: (a: number, b: number) => void;
  readonly wasmoptconfig_preserve_alpha: (a: number) => number;
  readonly wasmoptconfig_set_preserve_alpha: (a: number, b: number) => void;
  readonly imageoptimizer_new: () => number;
  readonly imageoptimizer_process_image_file: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => any;
  readonly imageoptimizer_get_image_info: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
  readonly meshoptimizer_new: () => number;
  readonly meshoptimizer_process_mesh_file: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
  readonly __wbg_filehandler_free: (a: number, b: number) => void;
  readonly filehandler_read_file_as_bytes: (a: any) => any;
  readonly filehandler_create_download_url: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly __wbg_canvasrenderer_free: (a: number, b: number) => void;
  readonly canvasrenderer_new: (a: any) => [number, number, number];
  readonly canvasrenderer_load_and_render_image: (a: number, b: number, c: number) => any;
  readonly canvasrenderer_clear: (a: number) => void;
  readonly __wbg_progresscallback_free: (a: number, b: number) => void;
  readonly progresscallback_new: (a: any) => number;
  readonly progresscallback_update: (a: number, b: number, c: number, d: number) => [number, number];
  readonly supported_image_formats: () => [number, number];
  readonly get_format_support_info: (a: number, b: number) => any;
  readonly supported_mesh_formats: () => [number, number];
  readonly main: () => void;
  readonly __wbg_meshoptimizer_free: (a: number, b: number) => void;
  readonly __wbg_imageoptimizer_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_6: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __externref_drop_slice: (a: number, b: number) => void;
  readonly closure4_externref_shim: (a: number, b: number, c: any) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha4d2336f14cfaaf8: (a: number, b: number) => void;
  readonly closure100_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure1015_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
