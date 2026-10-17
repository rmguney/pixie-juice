import { execSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { defineConfig, transformWithEsbuild, type Plugin } from 'vite';
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
// server needs inline scripts). Scripts and workers come from this origin; the
// WebAssembly engine and the model decoders run in those workers, which have a
// policy of their own. Image previews read user files through blob: URLs (data:
// for the select arrow). Inline style attributes set slider fills. vercel.json
// adds the headers a <meta> tag can't set, such as frame-ancestors.
const contentSecurityPolicy = [
  "default-src 'self'",
  "script-src 'self'",
  "worker-src 'self'",
  "connect-src 'self'",
  "img-src 'self' data: blob:",
  "style-src 'self' 'unsafe-inline'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
].join('; ');

/**
 * Emits sw.js, the service worker (src/serviceWorker.ts), with the list of
 * files to store when it installs, and a version that changes with any file
 * of the build, so each deploy installs a new worker.
 */
function offline(): Plugin {
  return {
    name: 'offline',
    apply: 'build',
    enforce: 'post',
    async generateBundle(_, bundle) {
      const page = bundle['index.html'];
      if (page?.type !== 'asset') this.error('index.html is missing from the bundle');
      const files = Object.keys(bundle)
        .filter((name) => name !== 'index.html')
        .map((name) => `/${name}`)
        .sort();
      const version = createHash('sha256').update(files.join('\n')).update(page.source).digest('hex').slice(0, 16);
      // Stored on install: the page's own scripts, styles and icons, and the engine
      // (its worker and module), which every visit loads; the rest when first used.
      const startup = new Set<string>();
      for (const [name, item] of Object.entries(bundle)) {
        if (item.type !== 'chunk' || !item.isEntry) continue;
        startup.add(name);
        for (const imported of item.imports) startup.add(imported);
        item.viteMetadata?.importedCss.forEach((css) => startup.add(css));
      }
      for (const name of Object.keys(bundle)) {
        if (/^assets\/(worker|pixie_core|favicon)-[^/]+$/.test(name)) startup.add(name);
      }
      const source = readFileSync(new URL('./src/serviceWorker.ts', import.meta.url), 'utf8');
      const { code } = await transformWithEsbuild(source, 'serviceWorker.ts', {
        format: 'iife',
        minify: true,
        define: {
          PRECACHE: JSON.stringify(['/', ...Array.from(startup).sort().map((name) => `/${name}`)]),
          VERSION: JSON.stringify(version),
        },
      });
      this.emitFile({ type: 'asset', fileName: 'sw.js', source: code });
    },
  };
}

export default defineConfig({
  plugins: [
    react(),
    offline(),
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
