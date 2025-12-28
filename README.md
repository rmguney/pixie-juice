# Pixie Juice

**Secure and high performance graphics processing engine in C, C++, and Rust for WebAssembly targets.**

Pixie Juice is a client side WebAssembly application built with C, C++, and Rust that optimizes and converts images and 3D meshes without uploading files to any server. Your data never leaves your device, ensuring complete information security while delivering fast and fail-safe media processing.

## Phase 1 - Core Features for MVP (under active development)

### Image Optimization

- **Formats**: PNG, JPEG, WebP, GIF, BMP, TIFF, SVG, ICO, TGA
- **Animation Support**: GIF and WebP animation preservation
- **Quality Control**: Adjustable compression levels
- **Fast Processing**: <100ms for images under 1MB
- **Metadata Stripping**: Optional privacy protection
- **Format Conversion**: Convert between formats

### 3D Model Optimization

- **Formats**: OBJ, PLY, STL, GLTF, GLB, FBX
- **Mesh Optimization**: Vertex welding and topology preservation
- **Format Conversion**: Cross-format compatibility
- **Performance Target**: <300ms for 100k triangle models
- **Interactive Preview**: 3D model viewer in browser
- **Format Conversion**: Convert between formats

### Status of Individual Formats (take this with a grain of salt, its a hot mess)

| Format | Status | Compression (sample) | App Notes |
| ------ | ------ | -------------------- | --------- |
| **PNG** | ✅ Working | 99.47% | Optimizes and stays PNG on the current sample. |
| **JPEG** | ✅ Working | 15.00% | Optimizes and stays JPEG on the current sample. |
| **WebP** | ✅ Working | 8.94% | Optimizes and stays WebP on the current sample. |
| **BMP** | ✅ Working | 93.75% | Works via auto strategy; converts this sample to JPEG (expected). |
| **TIFF** | ✅ Working | 86.95% | Works via auto strategy; converts this sample to JPEG (expected). |
| **GIF** | ⚠️ Partial | 0% | Loads and runs, but the current sample is effectively a no-op (auto outputs PNG with no size change). |
| **ICO** | ✅ Working | 0% | Preserves the ICO container on the current sample (size preservation is expected here). |
| **SVG** | ✅ Working | 4.14% | Keeps SVG output; optimizes markup on the current sample. |
| **TGA** | ✅ Working | 91.98% | Works via auto strategy; converts this sample to JPEG (expected). |
| **OBJ** | ✅ Working | Various | Load + optimize + download works; main remaining work is quality/perf tuning, not basic functionality. |
| **PLY** | ⚠️ Partial | Various | ASCII PLY works; binary PLY is still fragile (see limitations below). |
| **glTF** | ⚠️ Partial | 0% | Loads/runs on the minimal sample, but the current sample shows no size change (optimization path needs improvement/validation). |
| **GLB** | ✅ Working | 50% | Works end-to-end; current minimal sample shows size reduction. |
| **STL** | ⚠️ Partial | Various | File detection/upload works; optimize flow can be inconsistent in the UI. |
| **FBX** | ⚠️ Partial | 0% | Current minimal/stub sample shows no size change; real optimization path is not fully validated yet. |

Compression numbers above are from deterministic synthetic samples (useful as a regression signal), not a promise of real-world results.

Known limitations (what is not working yet):

- PLY (binary): can fail with UTF-8-related parsing errors; ASCII is the reliable path today.
- STL / FBX: detection/upload works, but the “optimize” UI activation + end-to-end flow still needs hardening.
- glTF: loads/runs, but optimization needs improvement/validation (current minimal sample is a no-op).
- FBX: optimization path still needs validation (current sample is stubby/no-op).
- GIF: current sample is a no-op; real-world gains depend on content (palette/frames/metadata).

### Test Suite

Tests live in the web app and run against the real browser + WASM runtime.

- **Unit tests**: Vitest (jsdom) for web-side utilities
- **E2E tests**: Playwright (Chromium/Firefox/WebKit) to validate WASM loading, optimization/conversion behavior, and regression signals like “output grew” or “no-op compression”

Run:

- `cd web; npm run test` (unit)
- `cd web; npm run test:e2e` (browser E2E)
- `cd web; npm run test:all` (unit + E2E)
- `cd web; npm run test:regression` (size/perf regression checks)
- `cd web; npm run test:regression:update` (update baselines when changes are intentional)

## Phase 2 - Performance and Platform Enhancements (hopefully one day)

- **WebGPU and WebGL Integration**: GPU-accelerated processing and preview
- **Streaming Processing**: Large file handling with progressive optimization
- **Algorithm Improvements**: Better heuristics for compression and quality trade-offs
- **Dead Code Removal**: Cleanup of unused features and optimizations
- **Frontend Enhancements**: Improved UI/UX components

## Phase 3 - Future Enhancements (who knows)

- **Advanced Mesh Formats**: DAE, USD etc.
- **Advanced Image Formats**: AVIF, HEIC, APNG, KTX2 etc.
- **Audio and Video Support**: Browser-compatible formats like MP3, WAV, MP4, WebM etc.
- **Advanced 3D Features**: Animation and texture support, etc.

## Technical Overview

- **Hybrid Engine**: Rust `#![no_std]` core with custom WASM allocator, freestanding C SIMD hotspots, and manual FFI bindings
- **WASM-First**: Optimized for client-side browser execution
- **Real-time**: Instant preview and processing
- **Frontend**: Vite + React 19 + Three.js

## Quick Start

**Try Online**: [pixiejuice.vercel.app](https://pixiejuice.vercel.app/)

1. Drop files into the browser
2. Adjust quality settings
3. Download optimized results
4. Compare before/after metrics

## Privacy & Security

- **Zero Upload**: All processing happens client-side
- **No Tracking**: No analytics or data collection  
- **Offline Ready**: Works without internet connection
- **Open Source**: Fully auditable codebase
- **Memory Safe**: Rust's ownership model + manual C FFI
