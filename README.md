# Pixie Juice (pxjc)

High-performance media processing utility with Rust architecture and C performance hotspots for both native and WebAssembly targets.

## Features

### Performance Architecture
- **C Performance Hotspots**: Critical performance sections implemented in optimized C for maximum speed
- **Universal Compatibility**: C hotspots work for both native CLI and WebAssembly (WASM) builds
- **Embedded Backend**: All the processing takes place in client app, instant uploads and downloads
- **Target Optimization**: Native builds use full CPU optimization, WASM builds use size-optimized code
- **Graceful Fallback**: Automatic fallback to pure Rust implementations if C fails

### Image Processing **Under Development**
- **Universal optimization** for PNG, JPEG, WebP, GIF, BMP, and TIFF formats
- **Format auto-detection** and validation
- **Quality control** and compression settings  
- **Lossless and lossy optimization** with configurable parameters
- **Real-time optimization** using proven crates
- **C-accelerated hotspots** for critical image processing operations
- **Cross-platform**: Native builds use optimal libraries, WASM builds use image crate

### Mesh Processing **Under Development**
- **Complete 3D model optimization** with format conversion
- **Full format support**: OBJ, PLY, STL, DAE/Collada, FBX (ASCII), glTF, USDZ
- **Mesh decimation algorithms** with configurable reduction ratios
- **Vertex welding** to remove duplicate vertices within tolerance
- **Quality preservation** using simplified quadric error metrics
- **C-accelerated hotspots** for mesh decimation and vertex processing
- **Web-compatible**: All mesh processing works in browser via WebAssembly
- **Binary format support**: Binary PLY, STL, and glTF binary (GLB) fully supported in both backend and frontend
- **Advanced format support**: USDZ (Universal Scene Description) with ZIP parsing

### Video Processing **Planned for Future Release**
- **Format support** for MP4 and WebM  
- **Compression optimization** with quality/size balance
- **Frame extraction** and thumbnail generation
- **Batch processing** for video libraries
- **Metadata preservation** and optimization
- **Resolution scaling** and aspect ratio management
- **Audio track processing** and optimization

### Next.js Web Application **Under Development**
- **Modern React architecture** with Next.js 15, React 19, and React Three Fiber
- **WebAssembly integration** - Rust-powered optimization running in browser
- **Real-time 3D preview** - Interactive mesh display with Three.js/React Three Fiber
- **Drag-and-drop interface** - Intuitive file handling with modern UI
- **Batch processing** - Process multiple images and 3D models simultaneously
- **Quality controls** - Real-time optimization settings with instant feedback
- **Download management** - Individual and bulk download options
- **No server required** - All processing happens client-side
