# Pixie Juice

> **Work in progress.** An image and 3D mesh optimizer that runs entirely in the browser: a C++23 engine built
> on established codec and geometry libraries, compiled to WebAssembly, behind a React and Three.js UI.
> Files never leave your device.

[Live demo](https://pixiejuice.vercel.app) · [![CI](https://github.com/rmguney/pixie-juice/actions/workflows/ci.yml/badge.svg)](https://github.com/rmguney/pixie-juice/actions/workflows/ci.yml) · MIT licence

Drop in images or meshes, pick a quality, download the results. In Auto mode every file stays in its own format, and the original comes back unchanged whenever nothing smaller passes verification.

## Formats

| Format | Library | What optimize does |
| --- | --- | --- |
| JPEG | [mozjpeg](https://github.com/mozilla/mozjpeg) | Lossless transcode (optimized progressive Huffman tables) or a trellis-quantized re-encode at the chosen quality, whichever is smaller. Keeps the ICC profile and EXIF orientation; drops GPS, camera data, thumbnails, XMP and comments. |
| PNG | [zopflipng](https://github.com/google/zopfli) (lodepng + zopfli) | Lossless: smallest colour type including palette, best filter strategy, zopfli deflate for images up to 256K pixels. Keeps the colour chunks (`iCCP`, `sRGB`, `gAMA`, `cHRM`) and orientation. Animated PNGs pass through. |
| WebP | [libwebp](https://chromium.googlesource.com/webm/libwebp) | Metadata stripped with libwebpmux, keeping the ICC profile and orientation; still images re-encoded lossy at the chosen quality, and losslessly when the source is lossless. |
| GIF | [giflib](https://giflib.sourceforge.net) | Every frame re-encoded from its own palette with timing, disposal and loop count kept; comments and application metadata dropped. |
| TIFF | [libtiff](https://libtiff.gitlab.io/libtiff/) + zlib | 8-bit single-page images re-compressed with Deflate or LZW plus predictor, keeping ICC profile, orientation and resolution. Other layouts pass through. |
| ICO | lodepng, stb_image | Each entry (PNG or BMP) re-encoded as an optimized PNG when smaller. |
| BMP, TGA | stb_image | Re-encoded as PNG (lossless) or, for opaque images below quality 85, JPEG. |
| SVG | [SVGO](https://github.com/svg/svgo) | Default preset, multipass, in the worker. Lossless mode removes only comments, metadata, editor data and the XML declaration. Legacy encodings named in the XML declaration are read correctly; output is UTF-8. |
| OBJ, STL, PLY | [meshoptimizer](https://github.com/zeux/meshoptimizer), fast_obj, miniply | Exact vertex welding, optional simplification, vertex cache, overdraw and fetch ordering; written back in the same format with normals, UVs, vertex colours and material groups. Polygons are triangulated. Files the writer can't reproduce exactly pass through: point clouds, lines, curves, OBJs with several objects or groups, extra PLY properties, binary STL colour attributes, non-finite coordinates. |
| glTF, GLB | gltfpack (meshoptimizer) | Deduplication, GPU ordering, `KHR_mesh_quantization` (off in lossless mode), optional simplification. Output is GLB. |
| FBX | — | Detected and passed through. |

Conversions to PNG, JPEG, WebP, GIF, BMP, TGA, ICO and TIFF are available from the Output format menu. Colour management uses [Little CMS](https://www.littlecms.com).

## How it works

```
web/ (React UI) ──postMessage──▶ worker ──Embind──▶ pixie_core.wasm
                                    └─ SVGO            core/  (C++23)
```

- **Engine (`core/`).** Every codec and geometry algorithm comes from a library; the engine's own code is the pipeline around them: detect the format, ask the format's library for candidate encodings, verify each, keep the smallest.
- **Verification.** A candidate is accepted only if it is smaller, decodes at the same size, keeps transparency and colour, and stays within a pixel-error budget set by the quality slider (exact in lossless mode). Meshes must hold exactly the same triangles unless simplification was requested. Anything else returns the original.
- **Colour and orientation.** Optimizing keeps the ICC profile and EXIF/TIFF orientation, so the result displays exactly like the original. Converting applies the orientation to the pixels and embeds the profile where the target can carry one (PNG, JPEG, WebP, TIFF); for GIF, BMP, TGA and ICO the pixels are converted to sRGB with Little CMS.
- **Hostile input.** Every header field is bounds-checked before allocation; images over 64 megapixels are refused. Tests feed truncated and corrupted files of every format through the engine under AddressSanitizer.
- **Web app (`web/`).** The engine runs in a Web Worker so the page never freezes; Three.js and SVGO load on demand. Downloads are named after the format the engine reports for the output.
- **3D preview.** three.js draws every model. OBJ, STL and PLY are read by the engine in the worker, with the same readers that optimize them, so large files don't block the page. glTF and GLB use three.js's loader, including meshopt and Draco compression (the Draco decoder is served by the app and fetched only when a file needs it); FBX uses three.js's FBX loader.

## Build and test

Requirements: CMake 3.25+, Ninja, a C++23 compiler (MSVC 19.4x, GCC 13+ or Clang 17+), [Emscripten](https://emscripten.org) 6.0.12 for the WebAssembly build, and Node 22. Library sources are fetched by CMake.

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

The browser tests drive the real engine: they generate images on a canvas, optimize and convert them, decode the results and compare pixels, and walk through the UI from upload to download.

The live site serves the committed `web/src/core/generated/` module, so engine changes reach it after `npm run build:wasm` and a commit. `await pixieJuice.info()` in the browser console reports the engine version and the commit it was built from.

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
- **3D preview** draws triangles only (point clouds and lines aren't shown) and doesn't decode KTX2/Basis textures in glTF.
- **glTF** files must be self-contained (GLB, or embedded `data:` URIs). Files that reference external buffers or images are refused, since those files aren't part of the input.

## Roadmap

1. FBX import through ufbx, written out as GLB.
2. Animated WebP and GIF re-encoding with palette reduction.
3. AVIF and JPEG XL output.
4. Meshopt compression (`EXT_meshopt_compression`) as an option for glTF.
