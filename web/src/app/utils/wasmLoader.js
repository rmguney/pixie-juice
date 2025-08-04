/**
 * Custom WASM loader to handle environment-specific imports
 * Fixes WASM module loading issues in Next.js
 */

let wasmModule = null;
let wasmInitialized = false;

/**
 * Initialize WASM module with proper environment handling
 * @returns {Promise<Object>} The initialized WASM module
 */
export async function initWasm() {
  if (wasmInitialized && wasmModule) {
    return wasmModule;
  }
  
  try {
    console.log('🔄 Loading WASM module...');
    
    // Dynamic import - bundler target handles initialization automatically
    const wasmPkg = await import('../../../pkg/pixie_juice.js');
    
    // Helper function to safely extract functions
    const safeExtract = (fn, name) => {
      if (typeof fn === 'function') {
        return fn;
      } else {
        console.warn(`WASM function ${name} not available`);
        return undefined;
      }
    };
    
    // Extract all exported functions safely
    wasmModule = {
      // Core optimization functions
      optimize_image: safeExtract(wasmPkg.optimize_image, 'optimize_image'),
      optimize_mesh: safeExtract(wasmPkg.optimize_mesh, 'optimize_mesh'),
      optimize_auto: safeExtract(wasmPkg.optimize_auto, 'optimize_auto'),
      
      // Format detection
      detect_format: safeExtract(wasmPkg.detect_format, 'detect_format'),
      version: safeExtract(wasmPkg.version, 'version'),
      
      // Format-specific optimizations (Phase 1 - Core formats)
      optimize_png: safeExtract(wasmPkg.optimize_png, 'optimize_png'),
      optimize_jpeg: safeExtract(wasmPkg.optimize_jpeg, 'optimize_jpeg'),
      optimize_webp: safeExtract(wasmPkg.optimize_webp, 'optimize_webp'),
      optimize_gif: safeExtract(wasmPkg.optimize_gif, 'optimize_gif'),
      optimize_ico: safeExtract(wasmPkg.optimize_ico, 'optimize_ico'),
      
      // Format detection functions (Phase 1)
      is_webp: safeExtract(wasmPkg.is_webp, 'is_webp'),
      is_gif: safeExtract(wasmPkg.is_gif, 'is_gif'),
      is_ico: safeExtract(wasmPkg.is_ico, 'is_ico'),
      
      // Format conversion functions
      convert_to_webp: safeExtract(wasmPkg.convert_to_webp, 'convert_to_webp'),
      convert_to_png: safeExtract(wasmPkg.convert_to_png, 'convert_to_png'),
      convert_to_jpeg: safeExtract(wasmPkg.convert_to_jpeg, 'convert_to_jpeg'),
      convert_to_bmp: safeExtract(wasmPkg.convert_to_bmp, 'convert_to_bmp'),
      convert_to_gif: safeExtract(wasmPkg.convert_to_gif, 'convert_to_gif'),
      convert_to_ico: safeExtract(wasmPkg.convert_to_ico, 'convert_to_ico'),
      convert_to_tiff: safeExtract(wasmPkg.convert_to_tiff, 'convert_to_tiff'),
      convert_to_svg: safeExtract(wasmPkg.convert_to_svg, 'convert_to_svg'),
      
      // Performance monitoring
      get_performance_metrics: safeExtract(wasmPkg.get_performance_metrics, 'get_performance_metrics'),
      reset_performance_stats: safeExtract(wasmPkg.reset_performance_stats, 'reset_performance_stats'),
      check_performance_compliance: safeExtract(wasmPkg.check_performance_compliance, 'check_performance_compliance'),
      
      // Configuration functions
      set_lossless_mode: safeExtract(wasmPkg.set_lossless_mode, 'set_lossless_mode'),
      set_preserve_metadata: safeExtract(wasmPkg.set_preserve_metadata, 'set_preserve_metadata'),
    };
    
    wasmInitialized = true;
    console.log('WASM module initialized successfully');
    console.log('Available functions:', Object.keys(wasmModule).length);
    
    return wasmModule;
    
  } catch (error) {
    console.error('Failed to initialize WASM module:', error);
    throw new Error(`WASM initialization failed: ${error.message}`);
  }
}

/**
 * Get the initialized WASM module (throws if not initialized)
 * @returns {Object} The WASM module
 */
export function getWasm() {
  if (!wasmInitialized || !wasmModule) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }
  return wasmModule;
}

/**
 * Check if WASM is initialized
 * @returns {boolean} True if WASM is ready
 */
export function isWasmReady() {
  return wasmInitialized && wasmModule !== null;
}
