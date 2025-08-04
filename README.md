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

### Current Status of Individual Formats

| Format | Status | Compression | Hotspots | Performance |
|--------|--------|-------------|----------|-------------|
| **PNG** | ✅ Working | 10-63% | ⚠️ Partial | 82-1870ms |
| **JPEG** | ✅ Working | 15-69% | ✅ Complete | 17-720ms |
| **WebP** | ✅ Working | 5-30% | ✅ Complete | 11-761ms |
| **BMP** | ✅ Working | 94-98% | ✅ Complete | 11-86ms |
| **TIFF** | ✅ Working | 98-99% | ✅ Complete | 20-416ms |
| **GIF** | ✅ Working | 0.6-94% | ✅ Complete | 1-19ms |
| **ICO** | ✅ Working | 10-25% | ✅ Complete | <50ms |
| **SVG** | ⚠️ Issues | 5-15% | ⚠️ Memory Error | Variable |
| **TGA** | ❌ Broken | N/A | ❌ Not Detected | N/A |
| **AVIF** | ❌ Broken | N/A | ❌ Not Detected | N/A |
| **OBJ** | ⚠️ Partial | 26-100% | ✅ Complete | <3ms |
| **PLY** | ⚠️ Partial | 16.9% | ✅ Complete | <1ms |
| **glTF** | ✅ Working | 50.5% | ✅ Complete | <1ms |
| **GLB** | ⚠️ Issues | 50.1% | ✅ Complete | <1ms |
| **STL** | ❌ Broken | N/A | ❌ Not Supported | N/A |
| **FBX** | ❌ Broken | N/A | ❌ Not Supported | N/A |

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
