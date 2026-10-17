# Pixie Juice

An image and 3D model optimizer that runs entirely in the browser: a C++23 engine built on established codec and geometry libraries, compiled to WebAssembly, behind a React and three.js UI. Files never leave your device, and once visited the site works offline.

[Live demo](https://pixiejuice.vercel.app) · [![CI](https://github.com/rmguney/pixie-juice/actions/workflows/ci.yml/badge.svg)](https://github.com/rmguney/pixie-juice/actions/workflows/ci.yml) · MIT licence

Drop, browse or paste files, pick a quality, download the results one by one or as a ZIP. Auto keeps each file's format (BMP and TGA become PNG or JPEG, glTF becomes GLB); when nothing smaller passes verification, the original comes back unchanged. Any image can also be converted to PNG, JPEG, WebP, GIF, BMP, TGA, ICO or TIFF, and scaled down to a maximum size.

## Formats

| Format | Library | Optimize |
| --- | --- | --- |
| JPEG | [mozjpeg](https://github.com/mozilla/mozjpeg) | Lossless transcode or trellis re-encode at the chosen quality, whichever is smaller |
| PNG | [zopflipng](https://github.com/google/zopfli) | Lossless: best colour type, filters and deflate |
| WebP | [libwebp](https://github.com/webmproject/libwebp) | Re-encode (lossy or lossless, as the source), metadata stripped |
| GIF | [giflib](https://giflib.sourceforge.net) | Every frame re-encoded, timing and looping kept |
| TIFF | [libtiff](https://libtiff.gitlab.io/libtiff/) + zlib, zstd, xz, mozjpeg, libwebp | Deflate or LZW with predictor, or the file's own Zstandard, LZMA, WebP or JPEG |
| ICO | lodepng, stb_image | Each entry as an optimized PNG |
| BMP, TGA | stb_image | PNG, or JPEG for opaque images below quality 85 |
| SVG | [SVGO](https://github.com/svg/svgo) | Default preset, multipass |
| HEIC | [libheif](https://github.com/strukturag/libheif) + [libde265](https://github.com/strukturag/libde265) | Kept (no encoder); converts to any format |
| AVIF | the browser | Kept (no encoder); converts to any format |
| OBJ, STL, PLY | [meshoptimizer](https://github.com/zeux/meshoptimizer), fast_obj, miniply | Exact welding, optional simplification, cache/overdraw/fetch ordering |
| glTF, GLB | gltfpack | Deduplication, quantization, optional simplification; output GLB, with a .gltf's .bin and textures packed in |
| FBX | — | Detected, passed through |

Colour profiles and orientation are always kept; conversions apply orientation and use [Little CMS](https://www.littlecms.com) where a target can't carry the profile.

## How it works

```
web/ (React UI) ──postMessage──▶ workers ──Embind──▶ pixie_core.wasm, pixie_heif.wasm
                                    └─ SVGO              core/ (C++23)
```

- **Engine (`core/`).** The libraries do the encoding; the engine asks each for candidates, verifies them and keeps the smallest. An image candidate must decode at the same size with the same transparency, colour and orientation, within a pixel-error budget set by quality (exact in Lossless mode). OBJ, STL and PLY must keep exactly the same triangles unless simplifying; glTF is accepted on size.
- **Hostile input.** Decoders check dimensions against a 64-megapixel cap, and files that couldn't hold the image they claim are refused. Tests run every format's truncated and corrupted files under ASan, LSan and UBSan, and [libFuzzer](core/fuzz/README.md) runs the whole API nightly.
- **Web app (`web/`).** Up to four engine workers process a batch in parallel, stop when idle, and stop at once when a run is cancelled. three.js, SVGO and the HEIC module load on demand. A service worker keeps the app and the engine for offline use, and each on-demand part once used. The page runs under a strict Content Security Policy.
- **3D preview.** Every model is parsed off the main thread: OBJ, STL and PLY by the engine (triangles, lines or point clouds), glTF and FBX by three.js's loaders in a worker (meshopt, Draco and KTX2 included). Cameras, lights and animation come along; the first animation plays.

## Build and test

Requirements: CMake 3.25+, Ninja, a C++23 compiler, [Emscripten](https://emscripten.org) 6.0.12 and Node 22. CMake fetches each library as a SHA-256-checked release tarball and applies the fixes in [`core/cmake/patches/`](core/cmake/patches). CI builds on Linux with GCC; development also uses MSVC.

```bash
cd core
cmake --preset native && cmake --build --preset native && ctest --preset native
cmake --preset native-asan && cmake --build --preset native-asan && ctest --preset native-asan
cmake --preset wasm && cmake --build --preset wasm      # writes web/src/core/generated/

cd ../web
npm ci
npx tsc -b && npm run lint && npm test
npx playwright install && npm run test:e2e               # Chromium, Firefox, WebKit
npm run dev                                              # http://localhost:3000
```

The site ships the committed `web/src/core/generated/` modules: after engine changes, run `npm run build:wasm` and commit. Each module embeds a hash of the `core/` sources it was built from, and CI fails if it doesn't match.

## C API

[`pixie.h`](core/include/pixie/pixie.h) exposes the engine to C and any language with a C FFI; the native build produces the `pixie` shared library.

```c
pixie_options options = pixie_default_options();
options.quality = 80;
pixie_result result = pixie_optimize(data, size, &options);
if (!result.error && result.changed) fwrite(result.data, 1, result.size, out);
pixie_result_free(&result);
```

`pixie_convert`, `pixie_decode` and `pixie_decode_mesh` work the same way, and calls can run on several threads at once.

## Limitations

- FBX is not optimized; animated WebP is only stripped of metadata; GIF palettes are not reduced.
- TIFF optimization covers 8-bit single-page greyscale and RGB(A); other layouts pass through.
- HEIC and AVIF are read-only. HEIC uses the LGPL libraries libheif and libde265 (in a module of their own) and HEVC, which is patent-encumbered in some countries.
- glTF output isn't verified geometrically; a .gltf needs its .bin and texture files added with it.
- The 3D preview plays only a model's first animation.

## Roadmap

1. FBX import through ufbx, written out as GLB.
2. Animated WebP and GIF re-encoding with palette reduction.
3. AVIF and JPEG XL output.
4. Meshopt compression as a glTF option.
5. Benchmarks against other optimizers.
