import { useState, useEffect } from 'react';
import { initWasm, getWasm, isWasmReady } from '../utils/wasmLoader';
import type { WasmModule, WasmHook } from '../types';

export const useWasm = (): WasmHook => {
  const [wasm, setWasm] = useState<WasmModule | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    const loadWasm = async () => {
      try {
        setLoading(true);
        setError(null);
        
        if (isWasmReady()) {
          const wasmModule = getWasm();
          if (isMounted) {
            setWasm(wasmModule);
            setLoading(false);
          }
          return;
        }
        
        const wasmModule = await initWasm();
        
        if (isMounted) {
          setWasm(wasmModule);
          setLoading(false);
          
          if (typeof window !== 'undefined') {
            (window as unknown as { pixieJuice: WasmModule }).pixieJuice = wasmModule;
          }
        }
        
      } catch (err) {
        console.error('WASM loading error:', err);
        if (isMounted) {
          setError(err instanceof Error ? err.message : String(err));
          setLoading(false);
        }
      }
    };

    loadWasm();

    return () => {
      isMounted = false;
    };
  }, []);

  return {
    optimize_image: wasm?.optimize_image,
    optimize_mesh: wasm?.optimize_mesh,
    optimize_auto: wasm?.optimize_auto,
    optimize_png: wasm?.optimize_png,
    optimize_jpeg: wasm?.optimize_jpeg,
    optimize_webp: wasm?.optimize_webp,
    optimize_gif: wasm?.optimize_gif,
    optimize_ico: wasm?.optimize_ico,
    optimize_tga: wasm?.optimize_tga,
    optimize_obj: wasm?.optimize_obj,
    optimize_stl: wasm?.optimize_stl,
    optimize_fbx: wasm?.optimize_fbx,
    optimize_gltf: wasm?.optimize_gltf,
    optimize_ply: wasm?.optimize_ply,
    detect_format: wasm?.detect_format,
    is_webp: wasm?.is_webp,
    is_gif: wasm?.is_gif,
    is_ico: wasm?.is_ico,
    is_tga: wasm?.is_tga,
    is_obj: wasm?.is_obj,
    is_gltf: wasm?.is_gltf,
    is_stl: wasm?.is_stl,
    is_fbx: wasm?.is_fbx,
    is_ply: wasm?.is_ply,
    convert_to_webp: wasm?.convert_to_webp,
    convert_to_png: wasm?.convert_to_png,
    convert_to_jpeg: wasm?.convert_to_jpeg,
    convert_to_bmp: wasm?.convert_to_bmp,
    convert_to_gif: wasm?.convert_to_gif,
    convert_to_ico: wasm?.convert_to_ico,
    convert_to_tiff: wasm?.convert_to_tiff,
    convert_to_svg: wasm?.convert_to_svg,
    convert_to_tga: wasm?.convert_to_tga,
    strip_tiff_metadata_simd: wasm?.strip_tiff_metadata_simd,
    get_performance_metrics: wasm?.get_performance_metrics,
    reset_performance_stats: wasm?.reset_performance_stats,
    check_performance_compliance: wasm?.check_performance_compliance,
    set_lossless_mode: wasm?.set_lossless_mode,
    set_preserve_metadata: wasm?.set_preserve_metadata,
    version: wasm?.version,
    build_timestamp: wasm?.build_timestamp,
    pixie_get_memory_target_mb: wasm?.pixie_get_memory_target_mb,
    loading,
    error,
    available: !!wasm && !loading && !error
  };
};
