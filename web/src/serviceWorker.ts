// The service worker, which keeps the app usable offline. On the first visit
// it stores what the page loads anyway (the app and the engine); the parts
// loaded on demand (3D preview, SVGO, Draco, Basis, the HEIC module) are stored
// when first used, so nobody downloads what they never use. Built separately
// (see the offline plugin in vite.config.ts), which fills in the file list and
// a version for it.

declare const self: ServiceWorkerGlobalScope;
/** The files every visit loads, the page itself ("/") first. */
declare const PRECACHE: string[];
/** Changes with any file of the build. */
declare const VERSION: string;

const PREFIX = 'pixie-juice-';
const CACHE = PREFIX + VERSION;

self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(PRECACHE)));
});

// A new version takes over once no tab runs the old one, then drops the old files.
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((key) => key.startsWith(PREFIX) && key !== CACHE).map((key) => caches.delete(key))))
      .then(() => self.clients.claim()),
  );
});

/** A built file from the network, stored for next time (its name has a content hash, so it never goes stale). */
async function fetchAndStore(request: Request): Promise<Response> {
  const response = await fetch(request);
  if (response.ok && new URL(request.url).pathname.startsWith('/assets/')) {
    const copy = response.clone();
    void caches.open(CACHE).then((cache) => cache.put(request, copy));
  }
  return response;
}

self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET' || new URL(request.url).origin !== self.location.origin) return;
  if (request.mode === 'navigate') {
    // The page comes from the network while there is one, so a new version shows at once.
    event.respondWith(fetch(request).catch(async () => (await caches.match('/', { cacheName: CACHE })) ?? Response.error()));
    return;
  }
  // Any stored copy is current, including one a newer version (waiting to take over) stored.
  event.respondWith(caches.match(request).then((stored) => stored ?? fetchAndStore(request)));
});

export {};
