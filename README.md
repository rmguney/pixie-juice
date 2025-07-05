# Pixie Juice (pxjc)

High-performance media processing utility with Rust-first architecture and C performance hotspots

## Features

### Image Processing (Completed)
- **Universal optimization** for PNG, JPEG, WebP, GIF, BMP, and TIFF formats
- **Format auto-detection** and validation
- **Quality control** and compression settings
- **Pure Rust implementation** using proven crates (`oxipng`, `jpeg-encoder`, `webp`)

### Mesh Processing (In Development)
- 3D model optimization and format conversion
- Support for OBJ, PLY, DAE, and glTF formats
- Mesh decimation algorithms

### Video Processing  (Planned)
- **Format support** for MP4 and WebM
- **Compression optimization** with quality/size balance
- **Frame extraction** and thumbnail generation
- **Batch processing** for video libraries
- **Metadata preservation** and optimization
- **Resolution scaling** and aspect ratio management
- **Audio track processing** and optimization

### Next.js Web Application (Production Ready)

- **Modern React architecture** with Next.js 15, React 19, and React Three Fiber
- **WebAssembly integration** - Rust-powered optimization running in browser
- **Real-time 3D preview** - Interactive mesh display with Three.js/React Three Fiber
- **Drag-and-drop interface** - Intuitive file handling with modern UI
- **Batch processing** - Process multiple images and 3D models simultaneously
- **Quality controls** - Real-time optimization settings with instant feedback
- **Download management** - Individual and bulk download options
- **No server required** - All processing happens client-side

## Installation

### Prerequisites
- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- C compiler (for performance hotspots)

### Build from Source
```bash
git clone <your-repo-url>
cd pixie-juice
cargo build --release
```

## Usage

### Image Optimization
```bash
# Optimize a single image
pxjc optimize input.jpg

# Optimize multiple images
pxjc optimize *.png

# Batch optimize with custom settings
pxjc optimize --quality 85 --format webp images/
```

### Available Commands
- `pxjc optimize` - Optimize images with auto-format detection
- `pxjc help` - Show available commands and options

## Project Structure

```
pixie-juice/
├── rust_core/          # Core Rust processing logic
├── frontend/           # CLI application
├── hotspots/           # C performance hotspots
├── web/                # Next.js WebAssembly application
│   ├── src/app/        # React components and pages
│   ├── pkg/            # Generated WASM bindings
│   └── public/         # Static assets
└── tests/              # Test suite
```

## Web Application

The modern React-based web application provides a powerful, user-friendly interface for media optimization.

### Development

```bash
cd web
npm install
npm run dev
```

### Features

- **Drag & Drop Interface**: Simply drag files into the browser window
- **Real-time 3D Preview**: Interactive mesh visualization with React Three Fiber
- **Batch Processing**: Optimize multiple files simultaneously
- **Quality Controls**: Adjust compression settings with real-time feedback
- **Format Support**: Images (PNG, JPEG, WebP, GIF, BMP, TIFF) and 3D models (OBJ, PLY, STL)
- **Client-side Processing**: All optimization happens in your browser via WebAssembly

### Deployment

```bash
cd web
npm run build
npm run start
```

The web app can be deployed to any static hosting service (Vercel, Netlify, GitHub Pages).
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --package rust_core
```
