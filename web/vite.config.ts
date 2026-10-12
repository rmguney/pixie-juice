import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  build: {
    target: 'es2022',
    // The largest chunks, three.js (3D preview) and SVGO, are already loaded only when needed.
    chunkSizeWarningLimit: 600,
  },
  worker: {
    // The engine worker imports the Emscripten ES module.
    format: 'es',
  },
  optimizeDeps: {
    // Imported only from the worker, which the dev server's dependency scan doesn't crawl;
    // without this, its first use triggers a re-optimization and a page reload.
    include: ['svgo/browser'],
  },
  server: {
    port: 3000,
  },
});
