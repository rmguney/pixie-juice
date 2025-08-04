'use client';

import { useState, useEffect } from 'react';
import { initWasm, getWasm, isWasmReady } from '../utils/wasmLoader';

export const useWasm = () => {
  const [wasm, setWasm] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    let isMounted = true;

    const loadWasm = async () => {
      try {
        setLoading(true);
        setError(null);
        
        console.log('🔄 Initializing WASM module...');
        
        // Check if already initialized
        if (isWasmReady()) {
          const wasmModule = getWasm();
          if (isMounted) {
            setWasm(wasmModule);
            setLoading(false);
            console.log('✅ WASM module ready (already initialized)');
          }
          return;
        }
        
        // Initialize WASM with proper environment handling
        const wasmModule = await initWasm();
        
        if (isMounted) {
          setWasm(wasmModule);
          setLoading(false);
          console.log('✅ WASM module loaded successfully');
          
          // Make available globally for debugging
          if (typeof window !== 'undefined') {
            window.pixieJuice = wasmModule;
            console.log('🌐 WASM module available at window.pixieJuice');
          }
        }
        
      } catch (err) {
        console.error('❌ WASM loading error:', err);
        if (isMounted) {
          setError(err.message);
          setLoading(false);
        }
      }
    };

    loadWasm();

    return () => {
      isMounted = false;
    };
  }, []);

  // Return the WASM functions with loading state
  return {
    // Basic optimization functions
    optimize_image: wasm?.optimize_image,
    optimize_mesh: wasm?.optimize_mesh,
    optimize_auto: wasm?.optimize_auto,
    
    // Format-specific optimization functions (Phase 1 - Available)
    optimize_png: wasm?.optimize_png,
    optimize_jpeg: wasm?.optimize_jpeg,
    optimize_webp: wasm?.optimize_webp,
    optimize_gif: wasm?.optimize_gif,
    optimize_ico: wasm?.optimize_ico,
    
    // Format detection functions (Phase 1 - Available)
    detect_format: wasm?.detect_format,
    is_webp: wasm?.is_webp,
    is_gif: wasm?.is_gif,
    is_ico: wasm?.is_ico,
    
    // Format conversion functions
    convert_to_webp: wasm?.convert_to_webp,
    convert_to_png: wasm?.convert_to_png,
    convert_to_jpeg: wasm?.convert_to_jpeg,
    convert_to_bmp: wasm?.convert_to_bmp,
    convert_to_gif: wasm?.convert_to_gif,
    convert_to_ico: wasm?.convert_to_ico,
    convert_to_tiff: wasm?.convert_to_tiff,
    convert_to_svg: wasm?.convert_to_svg,
    
    // Performance monitoring
    get_performance_metrics: wasm?.get_performance_metrics,
    reset_performance_stats: wasm?.reset_performance_stats,
    check_performance_compliance: wasm?.check_performance_compliance,
    
    // Advanced configuration
    set_lossless_mode: wasm?.set_lossless_mode,
    set_preserve_metadata: wasm?.set_preserve_metadata,
    
    // Library info
    version: wasm?.version,
    
    // State
    loading,
    error,
    available: !!wasm && !loading && !error
  };
};
