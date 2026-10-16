import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// The commit this site is built from: Vercel names the commit it deploys;
// elsewhere git does.
function siteCommit(): string {
  const vercel = process.env.VERCEL_GIT_COMMIT_SHA;
  if (vercel) return vercel.slice(0, 7);
  try {
    return execSync('git rev-parse --short=7 HEAD', { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim();
  } catch {
    return '';
  }
}

// The header's version, v<major>.<minor>+<commit>: major and minor from
// package.json (its patch number is unused), then the commit as build metadata.
function siteVersion(): string {
  const { version } = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8')) as { version: string };
  const [major, minor] = version.split('.');
  const commit = siteCommit();
  return `v${major}.${minor}${commit ? `+${commit}` : ''}`;
}

// The production page's Content Security Policy, as a <meta> tag (the dev
// server needs inline scripts). Scripts, workers and WebAssembly come from this
// origin; the Draco decoder runs in a blob: worker; previews read user files
// through blob: and data: URLs. Inline style attributes set slider fills.
// vercel.json adds the headers a <meta> tag can't set, such as frame-ancestors.
const contentSecurityPolicy = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "worker-src 'self' blob:",
  "connect-src 'self' data: blob:",
  "img-src 'self' data: blob:",
  "style-src 'self' 'unsafe-inline'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
].join('; ');

export default defineConfig({
  plugins: [
    react(),
    {
      name: 'content-security-policy',
      apply: 'build',
      transformIndexHtml: () => [
        {
          tag: 'meta',
          attrs: { 'http-equiv': 'Content-Security-Policy', content: contentSecurityPolicy },
          injectTo: 'head-prepend',
        },
      ],
    },
  ],
  define: {
    __SITE_VERSION__: JSON.stringify(siteVersion()),
  },
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
