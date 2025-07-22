# Pixie Juice (pxjc)

High-performance media processing utility with Rust/C architecture with performance hotspots, for WebAssembly targets.

## Features

### Performance Architecture

- **C Performance Hotspots**: Critical performance sections implemented in optimized C for maximum speed
- **WebAssembly Optimized**: C hotspots compiled for WASM with SIMD support
- **Embedded Backend**: All processing takes place in client browser, instant uploads and downloads
- **Size-Optimized**: WASM builds use size-optimized code for web performance

### Image Processing **Phase 1 (Active Development)**

- **JPEG optimization** with excellent compressio
- **GIF optimization** with color quantization
- **BMP/TIFF conversion** to optimized formats
- **PNG optimization** with format-specific strategies
- **WebP optimization** with compression tuning
- **Format auto-detection** and validation
- **Quality control** and compression settings
- **Lossless and lossy optimization** with configurable parameters
- **Real-time optimization** using proven crates
- **C-accelerated hotspots** for critical image processing operations
- **Browser-native**: Optimized for WebAssembly execution in modern browsers

**Note**: PNG and WebP optimization in WASM is limited by available tooling. Native desktop builds will provide superior optimization for these formats.

### Mesh Processing **Phase 2**

- **Complete 3D model optimization** with format conversion
- **Full format support**: OBJ, PLY, STL, DAE/Collada, FBX (ASCII), glTF, USDZ
- **Mesh decimation algorithms** with configurable reduction ratios
- **Vertex welding** to remove duplicate vertices within tolerance
- **Quality preservation** using simplified quadric error metrics
- **C-accelerated hotspots** for mesh decimation and vertex processing
- **Web-compatible**: All mesh processing works in browser via WebAssembly
- **Binary format support**: Binary PLY, STL, and glTF binary (GLB) fully supported
- **Advanced format support**: USDZ (Universal Scene Description) with ZIP parsing

### Video Processing **Phase 3**

- **Format support** for MP4 and WebP
- **Compression optimization** with quality/size balance
- **Frame extraction** and thumbnail generation
- **Batch processing** for video libraries
- **Metadata preservation** and optimization
- **Resolution scaling** and aspect ratio management
- **Audio track processing** and optimization

### Next.js Web Application

- **Modern React architecture** with Next.js 15, React 19, and React Three Fiber
- **WebAssembly integration** - Rust + C powered optimization running in browser
- **Real-time 3D preview** - Interactive mesh display with Three.js/React Three Fiber
- **Drag-and-drop interface** - Intuitive file handling with modern UI
- **Batch processing** - Process multiple images and 3D models simultaneously
- **Quality controls** - Real-time optimization settings with instant feedback
- **Download management** - Individual and bulk download options
- **No server required** - All processing happens client-side
