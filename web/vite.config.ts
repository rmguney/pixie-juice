import { defineConfig, Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

function wasmEnvPlugin(): Plugin {
  return {
    name: 'wasm-env-shim',
    resolveId(id) {
      if (id === 'env') {
        return '\0virtual:env';
      }
    },
    load(id) {
      if (id === '\0virtual:env') {
        return `
          export function emscripten_notify_memory_growth() {}
          export function emscripten_memcpy_js() {}
        `;
      }
    },
  };
}

export default defineConfig({
  plugins: [
    react(),
    wasmEnvPlugin(),
    wasm(),
    topLevelAwait(),
  ],
  build: {
    target: 'esnext',
  },
  optimizeDeps: {
    exclude: ['pixie-juice'],
  },
  server: {
    port: 3000,
  },
});
