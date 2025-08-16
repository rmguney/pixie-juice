# Pixie Juice

**Secure and high performance media processing engine in Rust and C, for WebAssembly targets.**

Pixie Juice is a client side WebAssembly application built with Rust and C that optimizes images and 3D models without uploading files to any server. Your data never leaves your device, ensuring complete information security while delivering fast media processing.

## Phase 1 - Planned Core Features (under active development, hot mess)

### Image Optimization

- **Formats**: PNG, JPEG, WebP, GIF, BMP, TIFF, SVG, ICO, AVIF, TGA
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

### Current Status of Individual Formats (Updated August 16, 2025)

| Format | Status | Compression | Notes |
|--------|--------|-------------|----------|
| **PNG** | ✅ Working | 30-68% | ✅ Complete - All sizes working |
| **JPEG** | ✅ Working | 15-69% | ✅ Complete - Quality control working |
| **WebP** | ✅ Working | Various | ⚠️ Animated file support |
| **BMP** | ✅ Working | 94-98% | ✅ Complete - Excellent compression |
| **TIFF** | ⚠️ Issues | 98-99% | ⚠️ Small files C hotspot error |
| **GIF** | ✅ Working | 0.6-9% | ⚠️ Animated file support |
| **ICO** | ✅ Working | 4-12% | ✅ Complete - All sizes working |
| **SVG** | ✅ Working | 7-13% | ✅ Complete - SIMD optimization |
| **TGA** | ⚠️ Issues | 95% | ⚠️ 16-bit depth not supported |
| **AVIF** | ❌ Broken | N/A | ❌ Format detection failed |
| **OBJ** | ✅ Working | 26-100% | ✅ Complete - High compression |
| **PLY** | ⚠️ Partial | 16.9% | ⚠️ Binary format parsing issues |
| **glTF** | ✅ Working | 50.5% | ✅ Complete - Works with fallback |
| **GLB** | ⚠️ Issues | 50.1% | ⚠️ Parsing errors but functional |
| **STL** | ❌ Broken | N/A | ❌ UI recognition issue |
| **FBX** | ❌ Broken | N/A | ❌ UI recognition issue |

## Phase 2 - Performance and Platform Enhancements (hopefully one day)

- **WebGPU and WebGL Integration**: GPU-accelerated processing and preview
- **Streaming Processing**: Large file handling with progressive optimization
- **Algorithm Improvements**: Better heuristics for compression and quality trade-offs
- **Dead Code Removal**: Cleanup of unused features and optimizations
- **Frontend Enhancements**: Improved UI/UX components

## Phase 3 - Future Enhancements (who knows)

- **Advanced Mesh Formats**: DAE, USD etc.
- **Advanced Image Formats**: APNG, KTX2 etc.
- **Audio and Video Support**: Browser-compatible formats like MP3, WAV, MP4, WebM etc.
- **Advanced 3D Features**: Animation and texture support, etc.

## Technical Overview

- **Hybrid Engine**: Rust `#![no_std]` core with custom WASM allocator, C SIMD hotspots, and manual FFI bindings
- **WASM-First**: Optimized for client-side browser execution
- **Real-time**: Instant preview and processing
- **Frontend**: Next.js 15 + React 19 + Three.js
- **Testing**: Python automation with Playwright and pytest
- **Test Fixtures**: Custom fixture generation script

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
