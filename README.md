# Pixie Juice

An image and 3D mesh optimizer that runs entirely in the browser: a C++23 engine built on established codec and geometry libraries, compiled to WebAssembly, behind a React and Three.js UI. Files never leave your device.

[Live demo](https://pixiejuice.vercel.app) · [![CI](https://github.com/rmguney/pixie-juice/actions/workflows/ci.yml/badge.svg)](https://github.com/rmguney/pixie-juice/actions/workflows/ci.yml) · MIT licence

Drop in images or meshes, pick a quality, download the results. In Auto mode files keep their format, except BMP and TGA, which come back as PNG (or JPEG, see below), and glTF, which comes back as GLB. Whenever nothing smaller passes verification, the original comes back unchanged.

## Formats

| Format | Library | What optimize does |
| --- | --- | --- |
| JPEG | [mozjpeg](https://github.com/mozilla/mozjpeg) | Lossless transcode (optimized progressive Huffman tables) or a trellis-quantized re-encode at the chosen quality, whichever is smaller. Keeps the ICC profile and EXIF orientation; drops GPS, camera data, thumbnails, XMP and comments. Multi-picture JPEGs (an `MPF` segment, as in gain-map HDR photos) pass through. |
| PNG | [zopflipng](https://github.com/google/zopfli) (lodepng + zopfli) | Lossless: smallest colour type including palette, best filter strategy, zopfli deflate for images up to 256K pixels. Keeps the colour chunks (`iCCP`, `sRGB`, `gAMA`, `cHRM`, `cICP`, `mDCV`, `cLLI`) and orientation; an RGB profile on an image that becomes greyscale is swapped for the equivalent grey profile, or the image stays RGB. Animated PNGs pass through. |
| WebP | [libwebp](https://chromium.googlesource.com/webm/libwebp) | Metadata stripped with libwebpmux, keeping the ICC profile and orientation. Still images are also re-encoded: lossy at the chosen quality (not in Lossless mode) and, for lossless sources, losslessly. |
| GIF | [giflib](https://giflib.sourceforge.net) | Every frame re-encoded from its own palette with timing, disposal and loop count kept, then every frame of the result checked against the original; comments and application metadata dropped. |
| TIFF | [libtiff](https://libtiff.gitlab.io/libtiff/) + zlib | 8-bit single-page images re-compressed with Deflate or LZW plus predictor, keeping ICC profile, orientation and resolution. Other layouts pass through, as do float samples and compressions this build doesn't decode (JPEG, LZMA, Zstandard, WebP, JBIG, LERC). With Preserve metadata on, TIFFs pass through. |
| ICO | lodepng, stb_image | Each entry (PNG or BMP) re-encoded as an optimized PNG when smaller. Files with more than 64 entries, or with entries that overlap, are refused. |
| BMP, TGA | stb_image | Re-encoded as PNG (lossless) or, for opaque images below quality 85 outside Lossless mode, JPEG. |
| SVG | [SVGO](https://github.com/svg/svgo) | Default preset, multipass, in the worker, whatever the output format chosen. Lossless mode removes only comments, metadata, editor data and the XML declaration. Legacy encodings named in the XML declaration are read correctly; output is UTF-8. |
| OBJ, STL, PLY | [meshoptimizer](https://github.com/zeux/meshoptimizer), fast_obj, miniply | Exact vertex welding, optional simplification, vertex cache, overdraw and fetch ordering; written back in the same format with normals, UVs, vertex colours and material groups. Polygons are triangulated. Files the writer can't reproduce exactly pass through: point clouds, lines, curves, OBJs with several objects or groups, extra PLY properties, binary STL colour attributes, non-finite coordinates. |
| glTF, GLB | gltfpack (meshoptimizer) | Deduplication, GPU ordering, `KHR_mesh_quantization` (off in lossless mode), optional simplification. Output is GLB. Files using extensions gltfpack doesn't carry through pass through. |
| FBX | — | Detected and passed through. |

Conversions to PNG, JPEG, WebP, GIF, BMP, TGA, ICO and TIFF are available from the Output format menu. Colour management uses [Little CMS](https://www.littlecms.com).

## How it works

```
web/ (React UI) ──postMessage──▶ worker ──Embind──▶ pixie_core.wasm
                                    └─ SVGO            core/  (C++23)
```

- **Engine (`core/`).** Every codec and geometry algorithm comes from a library; the engine's own code is the pipeline around them: detect the format, ask the format's library for candidate encodings, verify each, keep the smallest.
- **Verification.** An image candidate is accepted only if it is smaller, decodes at the same size, keeps transparency, colour, ICC profile and orientation, and stays within a pixel-error budget set by the quality slider, on average and in its worst 8×8 block (exact in lossless mode). OBJ, STL and PLY output must hold exactly the same triangles, with the same normals, UVs, colours and groups, unless simplification was requested. glTF output is accepted on size: gltfpack's result isn't compared triangle by triangle. Anything else returns the original.
- **Colour and orientation.** Optimizing keeps the ICC profile and EXIF/TIFF orientation, so the result displays exactly like the original. Converting applies the orientation to the pixels and embeds an RGB profile where the target can carry one (PNG, JPEG, WebP, TIFF). Otherwise Little CMS converts the pixels to sRGB (RGB and grey profiles); a CMYK profile no longer describes pixels the decoder has already turned into RGB, so it is dropped. With Lossless on, a conversion fails rather than lose pixels (to JPEG always, to GIF above 256 colours, to ICO above 256 pixels across).
- **Hostile input.** Image dimensions are checked against a 64-megapixel cap before any decoder allocates pixels (for GIF, every frame, and 512 megapixels across frames). PNG image data may not inflate past the size its header implies (profile and text chunks past 16 MiB), PLY element counts must fit in the file, and JPEG decoding stops after 256 scans. Tests feed truncated and corrupted files of every format through the engine under AddressSanitizer, LeakSanitizer and UndefinedBehaviorSanitizer, with the libraries instrumented too; any report fails the build.
- **Web app (`web/`).** The engine runs in Web Workers, so optimizing never blocks the page, and a batch is processed several files at a time: one worker per spare CPU core, up to four (two on devices with 4 GB of memory or less), with large files kept from decoding all at once. Workers start when work needs them; since WebAssembly memory never shrinks, extra ones stop after 30 idle seconds and the last after five idle minutes. Three.js and SVGO load on demand. If the engine itself crashes, its worker restarts it. After a run, each image is shown against its result on a split view with a draggable divider. Downloads are named after the format the engine reports for the output, and kept unique within a batch. The production page sets a Content Security Policy: scripts and WebAssembly come only from the app's origin, inline scripts are refused, and other sites can't frame the page.
- **3D preview.** three.js draws every model, and every model is read off the main thread, so large files don't block the page. OBJ, STL and PLY are read by the engine, with the same readers that optimize them. glTF, GLB and FBX are parsed by three.js's loaders in a separate worker (stopped after 30 idle seconds), including meshopt and Draco compression (the Draco decoder is served by the app and fetched only when a file needs it); the scene comes back as transferred buffers and image bitmaps, with its materials, textures and hierarchy, and is rebuilt on the page.

## Build and test

Requirements: CMake 3.25+, Ninja, a C++23 compiler, [Emscripten](https://emscripten.org) 6.0.12 for the WebAssembly build, and Node 22. CMake fetches the libraries as release tarballs (commit archives for stb, miniply and lodepng), each checked against its SHA-256, and applies a few small fixes from [`core/cmake/patches/`](core/cmake/patches). CI builds and tests the engine on Linux with GCC 13; it is also developed with MSVC 19.4x on Windows. Native Clang and macOS builds aren't tested.

```bash
# Native engine and its tests (on Windows, from a Visual Studio developer prompt)
cd core
cmake --preset native && cmake --build --preset native && ctest --preset native
cmake --preset native-asan && cmake --build --preset native-asan && ctest --preset native-asan

# WebAssembly engine (writes web/src/core/generated/)
cmake --preset wasm && cmake --build --preset wasm

# Web app
cd ../web
npm ci
npx tsc -b && npm run lint && npm test
npx playwright install && npm run test:e2e   # Chromium, Firefox and WebKit
npm run dev                                   # http://localhost:3000
```

The browser tests drive the real engine: they generate images on a canvas, optimize and convert them, decode the results and compare pixels, and walk through the UI from upload to download. CI runs them against the production build, under its Content Security Policy.

The live site serves the committed `web/src/core/generated/` module, so engine changes reach it after `npm run build:wasm` and a commit. The module embeds a hash of the `core/` sources it was built from ([`SourceHash.cmake`](core/cmake/SourceHash.cmake)); CI fails when it doesn't match the committed sources, and the browser tests run against the committed module. `await pixieJuice.info()` in the browser console reports the engine version, build time and that source hash; the page header shows the site version, `v<major>.<minor>+<commit>`, with major and minor from `web/package.json`.

## C API

[`core/include/pixie/pixie.h`](core/include/pixie/pixie.h) exposes the engine to C and to any language with a C FFI (Python, C#, Go). The native build produces the `pixie` shared library (`pixie.dll`, `libpixie.so`, `libpixie.dylib`), which exports only these functions.

```c
#include <pixie/pixie.h>

pixie_options options = pixie_default_options();
options.quality = 80;
pixie_result result = pixie_optimize(data, size, &options);
if (result.error) {
  fprintf(stderr, "%s\n", result.error);
} else if (result.changed) {
  fwrite(result.data, 1, result.size, out);  /* result.format names the output format */
}                                            /* unchanged: the input is already optimal */
pixie_result_free(&result);
```

`pixie_convert`, `pixie_decode` (pixels) and `pixie_decode_mesh` (triangles) work the same way. Calls share no state and can run on several threads at once.

## Known limitations

- **FBX** is detected but not optimized.
- **Animated WebP** gets metadata stripping only; frames are not re-encoded.
- **GIF** palettes are not reduced. Converting to GIF keeps the exact colours when there are at most 256 (255 with transparency) and otherwise quantizes with median cut; partial transparency becomes on or off.
- **TIFF** optimization covers 8-bit single-page greyscale and RGB(A); multi-page, 16-bit, float and CMYK files pass through.
- **glTF** optimization isn't verified geometrically (see Verification).
- **3D preview** draws OBJ, STL and PLY triangles only (point clouds and lines aren't shown), doesn't decode KTX2/Basis textures in glTF, and shows glTF and FBX scenes without their cameras, lights or animation.
- **glTF** files must be self-contained (GLB, or embedded `data:` URIs). Files that reference external buffers or images are refused, and the preview fetches nothing, since those files aren't part of the input.

## Roadmap

1. FBX import through ufbx, written out as GLB.
2. Animated WebP and GIF re-encoding with palette reduction.
3. AVIF and JPEG XL output.
4. Meshopt compression (`EXT_meshopt_compression`) as an option for glTF.
5. Benchmarks against other optimizers on a public image and model set.
