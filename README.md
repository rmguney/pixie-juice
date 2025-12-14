# Pixie Juice

**Secure and high performance graphics processing engine in Rust and C, for WebAssembly targets.**

Pixie Juice is a client side WebAssembly application built with Rust and C that optimizes images and 3D meshes without uploading files to any server. Your data never leaves your device, ensuring complete information security while delivering fast media processing.

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

| Format | Status | Compression | Notes |
|--------|--------|-------------|----------|
| **PNG** | ✅ Working | 30-68% | ✅ Complete |
| **JPEG** | ✅ Working | 15-69% | ✅ Complete |
| **WebP** | ✅ Working | Various | ⚠️ Animated file support |
| **BMP** | ✅ Working | 94-98% | ✅ Complete |
| **TIFF** | ⚠️ Issues | 98-99% | ⚠️ Small files C hotspot error |
| **GIF** | ✅ Working | 0.6-9% | ⚠️ Animated file support |
| **ICO** | ✅ Working | 4-12% | ✅ Complete |
| **SVG** | ✅ Working | 7-13% | ✅ Complete |
| **TGA** | ✅ Working | 95% | ✅ Complete - Up to 16-bit depth |
| **OBJ** | ✅ Working | 26-100% | ✅ Complete |
| **PLY** | ⚠️ Partial | 16.9% | ⚠️ Binary format parsing issues |
| **glTF** | ✅ Working | 50.5% | ✅ Complete - Works with fallback |
| **GLB** | ⚠️ Issues | 50.1% | ⚠️ Parsing errors |
| **STL** | ⚠️ Issues | Various | ⚠️ Parsing errors |
| **FBX** | ⚠️ Issues | Various | ⚠️ Parsing errors |

### Test Suite (awaiting overhaul)

- Fixture generation for testing formats
- Comprehensive tests for all supported formats
- Automated browser tests with Playwright (if necessary for WASM validation
- Performance benchmarks to ensure speed targets are met)
- Memory leak checks on WASM heap

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
