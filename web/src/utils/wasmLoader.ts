import type { WasmModule } from '../types';

let wasmModule: WasmModule | null = null;
let wasmInitialized = false;

export async function initWasm(): Promise<WasmModule> {
  if (wasmInitialized && wasmModule) {
    return wasmModule;
  }
  
  try {
    const wasmPkg = await import('../../pkg/pixie_juice.js');

    wasmModule = {
      optimize_image: wasmPkg.optimize_image,
      optimize_mesh: wasmPkg.optimize_mesh,
      optimize_auto: wasmPkg.optimize_auto,
      detect_format: wasmPkg.detect_format,
      version: wasmPkg.version,
      optimize_png: wasmPkg.optimize_png,
      optimize_jpeg: wasmPkg.optimize_jpeg,
      optimize_webp: wasmPkg.optimize_webp,
      optimize_gif: wasmPkg.optimize_gif,
      optimize_ico: wasmPkg.optimize_ico,
      optimize_tga: wasmPkg.optimize_tga,
      is_webp: wasmPkg.is_webp,
      is_gif: wasmPkg.is_gif,
      is_ico: wasmPkg.is_ico,
      is_tga: wasmPkg.is_tga,
      is_obj: wasmPkg.is_obj,
      is_gltf: wasmPkg.is_gltf,
      is_stl: wasmPkg.is_stl,
      is_fbx: wasmPkg.is_fbx,
      is_ply: wasmPkg.is_ply,
      convert_to_webp: wasmPkg.convert_to_webp,
      convert_to_png: wasmPkg.convert_to_png,
      convert_to_jpeg: wasmPkg.convert_to_jpeg,
      convert_to_bmp: wasmPkg.convert_to_bmp,
      convert_to_gif: wasmPkg.convert_to_gif,
      convert_to_ico: wasmPkg.convert_to_ico,
      convert_to_tiff: wasmPkg.convert_to_tiff,
      convert_to_svg: wasmPkg.convert_to_svg,
      convert_to_tga: wasmPkg.convert_to_tga,
      strip_tiff_metadata_simd: wasmPkg.strip_tiff_metadata_simd,
      get_performance_metrics: wasmPkg.get_performance_metrics,
      reset_performance_stats: wasmPkg.reset_performance_stats,
      check_performance_compliance: wasmPkg.check_performance_compliance,
      set_lossless_mode: wasmPkg.set_lossless_mode,
      set_preserve_metadata: wasmPkg.set_preserve_metadata,
      build_timestamp: wasmPkg.build_timestamp,
      pixie_get_memory_target_mb: wasmPkg.pixie_get_memory_target_mb,
    };
    
    wasmInitialized = true;
    return wasmModule;
    
  } catch (error) {
    console.error('Failed to initialize WASM module:', error);
    throw new Error(`WASM initialization failed: ${error instanceof Error ? error.message : String(error)}`);
  }
}

export function getWasm(): WasmModule {
  if (!wasmInitialized || !wasmModule) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }
  return wasmModule;
}

export function isWasmReady(): boolean {
  return wasmInitialized && wasmModule !== null;
}
