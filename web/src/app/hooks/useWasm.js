'use client';

import { useState, useEffect } from 'react';

export function useWasm() {
  const [wasm, setWasm] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function initWasm() {
      try {
        // Dynamic import to avoid SSR issues
        const wasmModule = await import('../../../pkg/pixie_juice_web.js');
        
        // Initialize WASM
        await wasmModule.default();
        
        // Create optimizers
        const imageOptimizer = new wasmModule.ImageOptimizer();
        const meshOptimizer = new wasmModule.MeshOptimizer();
        
        setWasm({
          ImageOptimizer: wasmModule.ImageOptimizer,
          MeshOptimizer: wasmModule.MeshOptimizer,
          WasmOptConfig: wasmModule.WasmOptConfig,
          FileHandler: wasmModule.FileHandler,
          imageOptimizer,
          meshOptimizer,
        });
        
        setLoading(false);
      } catch (err) {
        console.error('Failed to initialize WASM:', err);
        setError(err);
        setLoading(false);
      }
    }

    initWasm();
  }, []);

  return { wasm, loading, error };
}
