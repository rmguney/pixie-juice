import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
function wasmEnvPlugin() {
    return {
        name: 'wasm-env-shim',
        resolveId: function (id) {
            if (id === 'env') {
                return '\0virtual:env';
            }
        },
        load: function (id) {
            if (id === '\0virtual:env') {
                return "\n          export function emscripten_notify_memory_growth() {}\n          export function emscripten_memcpy_js() {}\n        ";
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
